use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use crate::{LLMError, intelligence::InferenceMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub client_id: String,
    pub mode: InferenceMode,
    pub created_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
    pub is_active: bool,
    pub request_count: u32,
    pub total_tokens: u32,
}

#[derive(Debug)]
struct ActiveSession {
    info: SessionInfo,
    cancellation_token: CancellationToken,
    last_request: Instant,
}

pub struct SessionManager {
    active_sessions: Arc<RwLock<HashMap<String, ActiveSession>>>,
    session_timeout: Duration,
    cleanup_interval: Duration,
    max_sessions_per_client: usize,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(60),  // 1 minute
            max_sessions_per_client: 10,
        }
    }

    pub fn with_config(
        session_timeout: Duration,
        cleanup_interval: Duration,
        max_sessions_per_client: usize,
    ) -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout,
            cleanup_interval,
            max_sessions_per_client,
        }
    }

    pub async fn create_session(
        &self,
        client_id: String,
        mode: InferenceMode,
    ) -> Result<String, LLMError> {
        let session_id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now();
        
        // Check if client has too many active sessions
        let client_session_count = self.count_client_sessions(&client_id).await;
        if client_session_count >= self.max_sessions_per_client {
            // Clean up oldest session for this client
            self.cleanup_oldest_client_session(&client_id).await?;
        }

        let session_info = SessionInfo {
            session_id: session_id.clone(),
            client_id,
            mode,
            created_at: now,
            last_activity: now,
            is_active: true,
            request_count: 0,
            total_tokens: 0,
        };

        let active_session = ActiveSession {
            info: session_info,
            cancellation_token: CancellationToken::new(),
            last_request: Instant::now(),
        };

        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), active_session);
        }

        Ok(session_id)
    }

    pub async fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        let sessions = self.active_sessions.read().await;
        sessions.get(session_id).map(|s| s.info.clone())
    }

    pub async fn update_session_activity(
        &self,
        session_id: &str,
        tokens_used: u32,
    ) -> Result<(), LLMError> {
        let mut sessions = self.active_sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.info.last_activity = std::time::SystemTime::now();
            session.info.request_count += 1;
            session.info.total_tokens += tokens_used;
            session.last_request = Instant::now();
            Ok(())
        } else {
            Err(LLMError::SessionNotFound(session_id.to_string()))
        }
    }

    pub async fn get_cancellation_token(&self, session_id: &str) -> Option<CancellationToken> {
        let sessions = self.active_sessions.read().await;
        sessions.get(session_id).map(|s| s.cancellation_token.clone())
    }

    pub async fn cancel_session(&self, session_id: &str) -> Result<(), LLMError> {
        let sessions = self.active_sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            session.cancellation_token.cancel();
            Ok(())
        } else {
            Err(LLMError::SessionNotFound(session_id.to_string()))
        }
    }

    pub async fn remove_session(&self, session_id: &str) -> Result<SessionInfo, LLMError> {
        let mut sessions = self.active_sessions.write().await;
        
        if let Some(mut session) = sessions.remove(session_id) {
            session.info.is_active = false;
            session.cancellation_token.cancel();
            Ok(session.info)
        } else {
            Err(LLMError::SessionNotFound(session_id.to_string()))
        }
    }

    pub async fn list_sessions(&self, client_id: Option<String>) -> Vec<SessionInfo> {
        let sessions = self.active_sessions.read().await;
        
        sessions
            .values()
            .filter(|session| {
                client_id.as_ref().map_or(true, |id| &session.info.client_id == id)
            })
            .map(|session| session.info.clone())
            .collect()
    }

    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.active_sessions.write().await;
        let now = Instant::now();
        let mut expired_sessions = Vec::new();

        for (session_id, session) in sessions.iter() {
            if now.duration_since(session.last_request) > self.session_timeout {
                expired_sessions.push(session_id.clone());
            }
        }

        for session_id in &expired_sessions {
            if let Some(session) = sessions.remove(session_id) {
                session.cancellation_token.cancel();
            }
        }

        expired_sessions.len()
    }

    pub async fn get_session_stats(&self) -> SessionStats {
        let sessions = self.active_sessions.read().await;
        
        let total_sessions = sessions.len();
        let mut client_counts = HashMap::new();
        let mut mode_counts = HashMap::new();
        let mut total_requests = 0;
        let mut total_tokens = 0;

        for session in sessions.values() {
            *client_counts.entry(session.info.client_id.clone()).or_insert(0) += 1;
            *mode_counts.entry(session.info.mode).or_insert(0) += 1;
            total_requests += session.info.request_count;
            total_tokens += session.info.total_tokens;
        }

        SessionStats {
            total_sessions,
            client_counts,
            mode_counts,
            total_requests,
            total_tokens,
        }
    }

    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let session_manager = Arc::new(self.clone());
        let cleanup_interval = self.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                let cleaned_count = session_manager.cleanup_expired_sessions().await;
                
                if cleaned_count > 0 {
                    tracing::debug!("Cleaned up {} expired sessions", cleaned_count);
                }
            }
        })
    }

    async fn count_client_sessions(&self, client_id: &str) -> usize {
        let sessions = self.active_sessions.read().await;
        sessions
            .values()
            .filter(|session| session.info.client_id == client_id)
            .count()
    }

    async fn cleanup_oldest_client_session(&self, client_id: &str) -> Result<(), LLMError> {
        let mut sessions = self.active_sessions.write().await;
        
        // Find the oldest session for this client
        let oldest_session_id = sessions
            .iter()
            .filter(|(_, session)| session.info.client_id == client_id)
            .min_by_key(|(_, session)| session.last_request)
            .map(|(id, _)| id.clone());

        if let Some(session_id) = oldest_session_id {
            if let Some(session) = sessions.remove(&session_id) {
                session.cancellation_token.cancel();
                tracing::info!("Cleaned up oldest session {} for client {}", session_id, client_id);
            }
        }

        Ok(())
    }
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            active_sessions: Arc::clone(&self.active_sessions),
            session_timeout: self.session_timeout,
            cleanup_interval: self.cleanup_interval,
            max_sessions_per_client: self.max_sessions_per_client,
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub client_counts: HashMap<String, usize>,
    pub mode_counts: HashMap<InferenceMode, usize>,
    pub total_requests: u32,
    pub total_tokens: u32,
}

// Helper trait for session validation
pub trait SessionValidator {
    fn validate_session_request(&self, session_info: &SessionInfo, request: &str) -> bool;
}

pub struct DefaultSessionValidator;

impl SessionValidator for DefaultSessionValidator {
    fn validate_session_request(&self, session_info: &SessionInfo, _request: &str) -> bool {
        // Basic validation - session must be active and not too old
        if !session_info.is_active {
            return false;
        }

        // Check if session is too old (1 hour)
        let session_age = std::time::SystemTime::now()
            .duration_since(session_info.created_at)
            .unwrap_or_default();

        session_age < Duration::from_secs(3600)
    }
}