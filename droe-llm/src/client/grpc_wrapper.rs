use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};
use futures::Stream;
use tracing::{info, debug, warn, error};

use crate::client::grpc_client::GrpcClient;
use crate::service::{LLMRequest, LLMResponse, LLMStreamEvent, ServiceInfo};
use crate::intelligence::{InferenceMode, ValidationResult};
use crate::{LLMError, Result};

/// Configuration for the gRPC client wrapper
#[derive(Debug, Clone)]
pub struct GrpcWrapperConfig {
    /// Default port for new server instances
    pub default_port: u16,
    /// Maximum number of connection attempts
    pub max_connection_attempts: u32,
    /// Timeout for server startup
    pub startup_timeout_ms: u64,
    /// Health check interval
    pub health_check_interval_ms: u64,
    /// Path to droe-llm binary (None = use PATH)
    pub binary_path: Option<String>,
    /// Additional arguments for droe-llm server
    pub server_args: Vec<String>,
}

impl Default for GrpcWrapperConfig {
    fn default() -> Self {
        Self {
            default_port: 50051,
            max_connection_attempts: 20,
            startup_timeout_ms: 10000,
            health_check_interval_ms: 500,
            binary_path: None,
            server_args: vec![],
        }
    }
}

/// Wrapper around the gRPC client that handles auto-spawning of droe-llm servers
/// Provides a higher-level interface for managing multiple server instances
pub struct GrpcClientWrapper {
    config: GrpcWrapperConfig,
    clients: Arc<Mutex<HashMap<u16, GrpcClient>>>,
    active_ports: Arc<Mutex<Vec<u16>>>,
}

impl GrpcClientWrapper {
    /// Create a new gRPC client wrapper
    pub fn new(config: GrpcWrapperConfig) -> Self {
        Self {
            config,
            clients: Arc::new(Mutex::new(HashMap::new())),
            active_ports: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a wrapper with default configuration
    pub fn with_defaults() -> Self {
        Self::new(GrpcWrapperConfig::default())
    }

    /// Create a wrapper with custom port
    pub fn with_port(port: u16) -> Self {
        let mut config = GrpcWrapperConfig::default();
        config.default_port = port;
        Self::new(config)
    }

    /// Get or create a client for the specified port
    pub async fn get_client(&self, port: Option<u16>) -> Result<Arc<Mutex<GrpcClient>>> {
        let target_port = port.unwrap_or(self.config.default_port);
        
        let mut clients = self.clients.lock().await;
        
        if !clients.contains_key(&target_port) {
            info!("🔄 Creating new gRPC client for port {}", target_port);
            let client = GrpcClient::new(target_port);
            clients.insert(target_port, client);
            
            // Track active port
            let mut active_ports = self.active_ports.lock().await;
            if !active_ports.contains(&target_port) {
                active_ports.push(target_port);
            }
        }
        
        // Return wrapped client
        Ok(Arc::new(Mutex::new(clients.remove(&target_port).unwrap())))
    }

    /// Get a client for the default port
    pub async fn get_default_client(&self) -> Result<Arc<Mutex<GrpcClient>>> {
        self.get_client(None).await
    }

    /// Check if a server is running on the specified port
    pub async fn is_server_running(&self, port: u16) -> bool {
        self.check_port_health(port).await
    }

    /// Find the next available port starting from the default port
    pub async fn find_available_port(&self) -> Result<u16> {
        let mut port = self.config.default_port;
        let max_attempts = 100;
        
        for _ in 0..max_attempts {
            if !self.is_server_running(port).await {
                return Ok(port);
            }
            port += 1;
        }
        
        Err(LLMError::ConfigError("No available ports found".to_string()))
    }

    /// Start a new server instance on an available port
    pub async fn start_new_server(&self) -> Result<u16> {
        let port = self.find_available_port().await?;
        
        info!("🚀 Starting new droe-llm server on port {}", port);
        
        // Build command
        let binary = self.config.binary_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("droe-llm");
            
        let mut cmd = Command::new(binary);
        cmd.args(&["serve", "--port", &port.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            
        // Add custom args
        for arg in &self.config.server_args {
            cmd.arg(arg);
        }
        
        let child = cmd.spawn()
            .map_err(|e| LLMError::ConfigError(format!("Failed to spawn droe-llm server: {}", e)))?;

        info!("✅ droe-llm server spawned with PID: {}", child.id());
        
        // Wait for server to become ready
        self.wait_for_server_ready(port).await?;
        
        // Track the port
        let mut active_ports = self.active_ports.lock().await;
        if !active_ports.contains(&port) {
            active_ports.push(port);
        }
        
        Ok(port)
    }

    /// Stop server on the specified port
    pub async fn stop_server(&self, port: u16) -> Result<()> {
        info!("🛑 Stopping server on port {}", port);
        
        // Remove from active clients
        {
            let mut clients = self.clients.lock().await;
            clients.remove(&port);
        }
        
        // Remove from active ports
        {
            let mut active_ports = self.active_ports.lock().await;
            active_ports.retain(|&p| p != port);
        }
        
        // Note: The actual process termination is handled by the Drop trait on GrpcClient
        // when it's removed from the HashMap
        
        Ok(())
    }

    /// Stop all servers managed by this wrapper
    pub async fn stop_all_servers(&self) -> Result<()> {
        let ports: Vec<u16> = {
            let active_ports = self.active_ports.lock().await;
            active_ports.clone()
        };
        
        for port in ports {
            if let Err(e) = self.stop_server(port).await {
                warn!("Failed to stop server on port {}: {}", port, e);
            }
        }
        
        Ok(())
    }

    /// Get list of active server ports
    pub async fn get_active_ports(&self) -> Vec<u16> {
        let active_ports = self.active_ports.lock().await;
        active_ports.clone()
    }

    /// Generate DSL using the default client
    pub async fn generate_dsl(&self, request: LLMRequest) -> Result<LLMResponse> {
        let client = self.get_default_client().await?;
        let mut client_guard = client.lock().await;
        client_guard.generate_dsl(request).await
    }

    /// Generate DSL with streaming using the default client
    pub async fn generate_dsl_stream(&self, request: LLMRequest) -> Result<impl futures::Stream<Item = LLMStreamEvent>> {
        let client = self.get_default_client().await?;
        let mut client_guard = client.lock().await;
        client_guard.generate_dsl_stream(request).await
    }

    /// Validate code using the default client
    pub async fn validate_code(&self, code: &str, mode: InferenceMode) -> Result<ValidationResult> {
        let client = self.get_default_client().await?;
        let mut client_guard = client.lock().await;
        client_guard.validate_code(code, mode).await
    }

    /// Get service information from the default client
    pub async fn get_service_info(&self) -> Result<ServiceInfo> {
        let client = self.get_default_client().await?;
        let mut client_guard = client.lock().await;
        client_guard.get_service_info().await
    }

    /// Load balance requests across multiple servers
    pub async fn generate_dsl_load_balanced(&self, request: LLMRequest) -> Result<LLMResponse> {
        let active_ports = self.get_active_ports().await;
        
        if active_ports.is_empty() {
            // Start a new server if none are active
            let port = self.start_new_server().await?;
            let client = self.get_client(Some(port)).await?;
            let mut client_guard = client.lock().await;
            return client_guard.generate_dsl(request).await;
        }
        
        // Use the first available server (could implement more sophisticated load balancing)
        let port = active_ports[0];
        let client = self.get_client(Some(port)).await?;
        let mut client_guard = client.lock().await;
        client_guard.generate_dsl(request).await
    }

    /// Health check for a specific port
    async fn check_port_health(&self, port: u16) -> bool {
        match timeout(
            Duration::from_millis(1000),
            TcpStream::connect(format!("127.0.0.1:{}", port))
        ).await {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }

    /// Wait for server to become ready on the specified port
    async fn wait_for_server_ready(&self, port: u16) -> Result<()> {
        let max_attempts = self.config.startup_timeout_ms / self.config.health_check_interval_ms;
        
        for attempt in 1..=max_attempts {
            if self.check_port_health(port).await {
                info!("✅ Server on port {} is ready after {} attempts", port, attempt);
                return Ok(());
            }
            
            debug!("⏳ Waiting for server on port {}... attempt {}/{}", port, attempt, max_attempts);
            sleep(Duration::from_millis(self.config.health_check_interval_ms)).await;
        }
        
        Err(LLMError::ConfigError(format!("Server on port {} failed to start within timeout", port)))
    }
}

impl Drop for GrpcClientWrapper {
    fn drop(&mut self) {
        // Cleanup will be handled by individual GrpcClient Drop implementations
        info!("🧹 Cleaning up gRPC client wrapper");
    }
}

/// Builder for GrpcWrapperConfig
pub struct GrpcWrapperConfigBuilder {
    config: GrpcWrapperConfig,
}

impl GrpcWrapperConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: GrpcWrapperConfig::default(),
        }
    }

    pub fn default_port(mut self, port: u16) -> Self {
        self.config.default_port = port;
        self
    }

    pub fn max_connection_attempts(mut self, attempts: u32) -> Self {
        self.config.max_connection_attempts = attempts;
        self
    }

    pub fn startup_timeout_ms(mut self, timeout: u64) -> Self {
        self.config.startup_timeout_ms = timeout;
        self
    }

    pub fn health_check_interval_ms(mut self, interval: u64) -> Self {
        self.config.health_check_interval_ms = interval;
        self
    }

    pub fn binary_path<S: Into<String>>(mut self, path: S) -> Self {
        self.config.binary_path = Some(path.into());
        self
    }

    pub fn server_args(mut self, args: Vec<String>) -> Self {
        self.config.server_args = args;
        self
    }

    pub fn build(self) -> GrpcWrapperConfig {
        self.config
    }
}

impl Default for GrpcWrapperConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wrapper_creation() {
        let wrapper = GrpcClientWrapper::with_defaults();
        assert_eq!(wrapper.config.default_port, 50051);
    }

    #[tokio::test]
    async fn test_find_available_port() {
        let wrapper = GrpcClientWrapper::with_port(50060); // Use a different port for testing
        let port = wrapper.find_available_port().await.unwrap();
        assert!(port >= 50060);
    }

    #[tokio::test]
    async fn test_config_builder() {
        let config = GrpcWrapperConfigBuilder::new()
            .default_port(8080)
            .max_connection_attempts(10)
            .startup_timeout_ms(5000)
            .binary_path("/usr/local/bin/droe-llm")
            .build();
            
        assert_eq!(config.default_port, 8080);
        assert_eq!(config.max_connection_attempts, 10);
        assert_eq!(config.startup_timeout_ms, 5000);
        assert_eq!(config.binary_path, Some("/usr/local/bin/droe-llm".to_string()));
    }
}