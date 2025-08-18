use async_stream::stream;
use futures::stream::Stream;
use std::pin::Pin;
use std::time::Instant;
use tokio_stream::StreamExt;

use crate::{
    config::{LLMConfig, WorkspaceBounds},
    intelligence::{InferenceMode, ModeDetector, ModeResult, PartialUpdateEngine, PartialUpdateResult, DiffEngine, ValidationEngine, ValidationResult},
    providers::{InferenceParams, OllamaClient},
    session::SessionManager,
    LLMError,
};

#[derive(Debug, Clone)]
pub struct LLMRequest {
    pub prompt: String,
    pub context: Option<String>,
    pub mode_hint: Option<InferenceMode>,
    pub workspace_bounds: Option<WorkspaceBounds>,
    pub client_id: String,
    pub session_id: String,
    pub streaming: bool,
    pub file_path: Option<String>,
    pub existing_content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub generated_code: String,
    pub detected_mode: InferenceMode,
    pub confidence: f32,
    pub validation_result: ValidationResult,
    pub tokens_used: u32,
    pub inference_time_ms: u32,
    pub session_id: String,
    pub file_updates: Option<PartialUpdateResult>,
}

#[derive(Debug, Clone)]
pub enum LLMStreamEvent {
    ModeDetected {
        mode: InferenceMode,
        confidence: f32,
    },
    TokenGenerated {
        token: String,
        accumulated: String,
    },
    ValidationUpdate {
        is_valid: bool,
        errors: Vec<String>,
    },
    PartialUpdate {
        file_path: String,
        preview: String,
    },
    Completed {
        final_response: LLMResponse,
    },
    Error {
        error: String,
    },
}

pub struct LLMService {
    ollama_client: OllamaClient,
    mode_detector: ModeDetector,
    validator: ValidationEngine,
    config: LLMConfig,
    session_manager: SessionManager,
    partial_update_engine: PartialUpdateEngine,
}

impl LLMService {
    pub fn new(config: LLMConfig) -> Self {
        let ollama_client = OllamaClient::new(
            config.llm.ollama_url.clone(),
            std::time::Duration::from_millis(config.llm.timeout_ms),
        );

        Self {
            ollama_client,
            mode_detector: ModeDetector::new(),
            validator: ValidationEngine::new(),
            config,
            session_manager: SessionManager::new(),
            partial_update_engine: PartialUpdateEngine::new(),
        }
    }

    pub async fn generate_dsl_stream(
        &self,
        request: LLMRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = LLMStreamEvent> + Send>>, LLMError> {
        let session_id = request.session_id.clone();
        
        // Ensure session exists or create it
        let actual_session_id = if session_id.is_empty() || session_id == "auto" || self.session_manager.get_session(&session_id).await.is_none() {
            let mode_hint = request.mode_hint.unwrap_or(InferenceMode::Regular);
            self.session_manager
                .create_session(request.client_id.clone(), mode_hint)
                .await?
        } else {
            session_id
        };

        let cancellation_token = self
            .session_manager
            .get_cancellation_token(&actual_session_id)
            .await
            .ok_or_else(|| LLMError::SessionNotFound(actual_session_id.clone()))?;

        let service = self.clone();
        let stream = stream! {
            // 1. Mode detection with immediate feedback
            let mode_result = service.detect_mode(&request.prompt, &request.context, &request);
            yield LLMStreamEvent::ModeDetected {
                mode: mode_result.mode,
                confidence: mode_result.confidence,
            };

            // 2. Parameter optimization
            let base_params = InferenceParams {
                model: service.config.llm.model.clone(),
                temperature: 0.7,
                top_p: 0.9,
                top_k: 40,
                max_tokens: 800,
                stop_sequences: vec![],
            };
            let params = service.ollama_client.create_optimized_params(mode_result.mode, &base_params);
            let enhanced_prompt = service.prepare_prompt(&request.prompt, mode_result.mode, &request);

            // 3. Streaming Ollama inference
            let mut accumulated_text = String::new();
            let mut token_count = 0;
            let start_time = Instant::now();

            match service.ollama_client.generate_stream(enhanced_prompt, params).await {
                Ok(mut ollama_stream) => {
                    while let Some(chunk) = ollama_stream.next().await {
                        // Check for cancellation
                        if cancellation_token.is_cancelled() {
                            yield LLMStreamEvent::Error {
                                error: "Generation cancelled by user".to_string(),
                            };
                            break;
                        }

                        match chunk {
                            Ok(token) => {
                                accumulated_text.push_str(&token);
                                token_count += 1;

                                yield LLMStreamEvent::TokenGenerated {
                                    token: token.clone(),
                                    accumulated: accumulated_text.clone(),
                                };

                                // Real-time validation for robotics mode
                                if mode_result.mode == InferenceMode::Robotics && accumulated_text.contains('\n') {
                                    let validation = service.validator.validate_partial(
                                        &accumulated_text,
                                        mode_result.mode,
                                        &request.workspace_bounds,
                                    );
                                    yield LLMStreamEvent::ValidationUpdate {
                                        is_valid: validation.is_valid,
                                        errors: validation.errors,
                                    };
                                }

                                // Partial file updates for streaming
                                if let (Some(file_path), Some(existing_content)) = (&request.file_path, &request.existing_content) {
                                    if accumulated_text.len() > service.config.performance.partial_update_threshold {
                                        match service.partial_update_engine.apply_llm_update(
                                            existing_content,
                                            &accumulated_text,
                                            &request.prompt,
                                        ) {
                                            Ok(update_result) => {
                                                yield LLMStreamEvent::PartialUpdate {
                                                    file_path: file_path.clone(),
                                                    preview: update_result.preview,
                                                };
                                            }
                                            Err(_) => {
                                                // Continue without partial updates on error
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                yield LLMStreamEvent::Error {
                                    error: format!("Ollama error: {}", e),
                                };
                                break;
                            }
                        }
                    }

                    // 4. Final validation and response
                    let final_validation = match service.validator.validate(
                        &accumulated_text,
                        mode_result.mode,
                        &request.workspace_bounds,
                    ) {
                        Ok(validation) => validation,
                        Err(e) => {
                            yield LLMStreamEvent::Error {
                                error: format!("Validation error: {}", e),
                            };
                            return;
                        }
                    };

                    let inference_time = start_time.elapsed().as_millis() as u32;

                    // Handle partial file updates
                    let file_updates = if let (Some(_file_path), Some(existing_content)) = (&request.file_path, &request.existing_content) {
                        match service.partial_update_engine.apply_llm_update(
                            existing_content,
                            &accumulated_text,
                            &request.prompt,
                        ) {
                            Ok(update_result) => Some(update_result),
                            Err(_) => None,
                        }
                    } else {
                        None
                    };

                    let final_response = LLMResponse {
                        generated_code: accumulated_text,
                        detected_mode: mode_result.mode,
                        confidence: mode_result.confidence,
                        validation_result: final_validation,
                        tokens_used: token_count,
                        inference_time_ms: inference_time,
                        session_id: actual_session_id.clone(),
                        file_updates,
                    };

                    // Update session activity
                    if let Err(e) = service.session_manager.update_session_activity(&actual_session_id, token_count).await {
                        tracing::warn!("Failed to update session activity: {}", e);
                    }

                    yield LLMStreamEvent::Completed { final_response };
                }
                Err(e) => {
                    yield LLMStreamEvent::Error {
                        error: format!("Failed to start generation: {}", e),
                    };
                }
            }
        };

        Ok(Box::pin(stream))
    }

    pub async fn generate_dsl(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        // Non-streaming implementation for edge/robotics
        
        // Handle session creation like streaming version
        let session_id = request.session_id.clone();
        let actual_session_id = if session_id.is_empty() || session_id == "auto" || self.session_manager.get_session(&session_id).await.is_none() {
            let mode_hint = request.mode_hint.unwrap_or(InferenceMode::Regular);
            self.session_manager
                .create_session(request.client_id.clone(), mode_hint)
                .await?
        } else {
            session_id
        };
        
        let mode_result = self.detect_mode(&request.prompt, &request.context, &request);
        
        let base_params = InferenceParams {
            model: self.config.llm.model.clone(),
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 800,
            stop_sequences: vec![],
        };
        let params = self.ollama_client.create_optimized_params(mode_result.mode, &base_params);
        let enhanced_prompt = self.prepare_prompt(&request.prompt, mode_result.mode, &request);

        let start_time = Instant::now();
        let raw_response = self
            .ollama_client
            .generate_complete(enhanced_prompt, params)
            .await?;
        let inference_time = start_time.elapsed().as_millis() as u32;

        let validation = self.validator.validate(
            &raw_response,
            mode_result.mode,
            &request.workspace_bounds,
        )?;

        // Handle partial file updates
        let file_updates = if let (Some(_file_path), Some(existing_content)) = (&request.file_path, &request.existing_content) {
            match self.partial_update_engine.apply_llm_update(
                existing_content,
                &raw_response,
                &request.prompt,
            ) {
                Ok(update_result) => Some(update_result),
                Err(_) => None,
            }
        } else {
            None
        };

        // Update session activity 
        let tokens_used = raw_response.split_whitespace().count() as u32; // Rough token estimate
        let _ = self.session_manager.update_session_activity(&actual_session_id, tokens_used).await;

        Ok(LLMResponse {
            generated_code: raw_response,
            detected_mode: mode_result.mode,
            confidence: mode_result.confidence,
            validation_result: validation,
            tokens_used: 0, // Would need actual token count from Ollama
            inference_time_ms: inference_time,
            session_id: actual_session_id,
            file_updates,
        })
    }

    pub async fn cancel_generation(&self, session_id: &str) -> Result<(), LLMError> {
        self.session_manager.cancel_session(session_id).await?;
        Ok(())
    }

    pub async fn validate_code(
        &self,
        code: &str,
        mode: InferenceMode,
        workspace_bounds: &Option<WorkspaceBounds>,
    ) -> Result<ValidationResult, LLMError> {
        self.validator.validate(code, mode, workspace_bounds)
    }

    pub async fn apply_partial_update(
        &self,
        file_path: &str,
        original_content: &str,
        prompt: &str,
        mode_hint: Option<InferenceMode>,
        session_id: String,
        streaming: bool,
    ) -> Result<PartialUpdateResult, LLMError> {
        // Create a request for generating the update
        let request = LLMRequest {
            prompt: prompt.to_string(),
            context: Some(format!("File: {}", file_path)),
            mode_hint,
            workspace_bounds: None,
            client_id: "partial-update".to_string(),
            session_id,
            streaming,
            file_path: Some(file_path.to_string()),
            existing_content: Some(original_content.to_string()),
        };

        // Generate the updated code
        let response = self.generate_dsl(request).await?;

        // Apply the partial update
        self.partial_update_engine.apply_llm_update(
            original_content,
            &response.generated_code,
            prompt,
        )
    }

    pub fn get_file_diff(
        &self,
        file_path: &str,
        original_content: &str,
        updated_content: &str,
    ) -> Result<String, LLMError> {
        let diff_engine = DiffEngine::new();
        Ok(diff_engine.generate_unified_diff(original_content, updated_content, file_path))
    }

    pub async fn get_service_info(&self) -> Result<ServiceInfo, LLMError> {
        let server_info = self.ollama_client.get_server_info().await?;
        let session_stats = self.session_manager.get_session_stats().await;

        Ok(ServiceInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            model_name: self.config.llm.model.clone(),
            ollama_url: self.config.llm.ollama_url.clone(),
            streaming_enabled: self.config.performance.enable_streaming,
            supported_modes: vec![InferenceMode::Regular, InferenceMode::Robotics],
            server_info,
            session_stats,
        })
    }

    pub async fn create_session(&self, client_id: String, mode: InferenceMode) -> Result<String, LLMError> {
        self.session_manager.create_session(client_id, mode).await
    }

    fn detect_mode(&self, prompt: &str, context: &Option<String>, request: &LLMRequest) -> ModeResult {
        // Use hint if provided
        if let Some(mode_hint) = request.mode_hint {
            let confidence = match mode_hint {
                InferenceMode::Robotics => 0.9,
                InferenceMode::Regular => 0.8,
            };
            return ModeResult {
                mode: mode_hint,
                confidence,
                detected_keywords: vec![],
            };
        }

        // Check file-based hints
        if let Some(hint) = self.mode_detector.get_mode_hint(&request.file_path, &request.existing_content) {
            let confidence = 0.7;
            return ModeResult {
                mode: hint,
                confidence,
                detected_keywords: vec![],
            };
        }

        // Use full mode detection
        self.mode_detector.detect_mode(prompt, context)
    }

    fn prepare_prompt(&self, prompt: &str, mode: InferenceMode, request: &LLMRequest) -> String {
        let mut enhanced_prompt = self.config.create_system_prompt(mode, prompt);

        // Add context if available
        if let Some(context) = &request.context {
            enhanced_prompt.push_str(&format!("\n\nContext: {}", context));
        }

        // Add file context for partial updates
        if let Some(existing_content) = &request.existing_content {
            enhanced_prompt.push_str(&format!(
                "\n\nExisting file content:\n```\n{}\n```\n\nUpdate the file based on the prompt.",
                existing_content
            ));
        }

        enhanced_prompt
    }
}

impl Clone for LLMService {
    fn clone(&self) -> Self {
        Self {
            ollama_client: self.ollama_client.clone(),
            mode_detector: ModeDetector::new(),
            validator: ValidationEngine::new(),
            config: self.config.clone(),
            session_manager: self.session_manager.clone(),
            partial_update_engine: PartialUpdateEngine::new(),
        }
    }
}

impl LLMService {
    pub fn config(&self) -> &LLMConfig {
        &self.config
    }

    pub fn ollama_client(&self) -> &OllamaClient {
        &self.ollama_client
    }
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub version: String,
    pub model_name: String,
    pub ollama_url: String,
    pub streaming_enabled: bool,
    pub supported_modes: Vec<InferenceMode>,
    pub server_info: serde_json::Value,
    pub session_stats: crate::session::SessionStats,
}