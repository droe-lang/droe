use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};
use futures::Stream;

// Include the generated protobuf code
pub mod llm_service {
    tonic::include_proto!("droevm.llm");
}

use llm_service::{
    droe_vmllm_service_client::DroeVmllmServiceClient,
    *,
};

use crate::service::{LLMRequest, LLMResponse, LLMStreamEvent};
use crate::intelligence::InferenceMode;
use crate::{LLMError, Result};

pub struct GrpcClient {
    client: Option<DroeVmllmServiceClient<Channel>>,
    port: u16,
    server_process: Option<std::process::Child>,
}

impl GrpcClient {
    pub fn new(port: u16) -> Self {
        Self {
            client: None,
            port,
            server_process: None,
        }
    }

    /// Check if the gRPC server is healthy by attempting a TCP connection
    async fn is_server_healthy(&self) -> bool {
        match timeout(Duration::from_millis(1000), TcpStream::connect(format!("127.0.0.1:{}", self.port))).await {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }

    /// Spawn the droe-llm server process
    fn spawn_server(&mut self) -> Result<()> {
        tracing::info!("🚀 Starting droe-llm gRPC server on port {}", self.port);
        
        // Try different methods to start the server
        let child = match Command::new("droe-llm")
            .args(&["serve", "--port", &self.port.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn() {
                Ok(child) => {
                    tracing::info!("✅ Started droe-llm binary directly");
                    child
                },
                Err(_) => {
                    // Fallback to cargo run from the droe-llm directory
                    tracing::info!("📦 Binary not found, trying cargo run...");
                    
                    // Find the droe-llm directory relative to current working directory
                    let potential_dirs = [
                        "../droe/droe-llm",
                        "droe/droe-llm", 
                        "../../droe/droe-llm",
                        "../../../droe/droe-llm",
                        "./droe-llm",
                    ];
                    
                    let mut found_dir = None;
                    for dir in &potential_dirs {
                        if std::path::Path::new(dir).join("Cargo.toml").exists() {
                            found_dir = Some(*dir);
                            break;
                        }
                    }
                    
                    let droe_llm_dir = found_dir.ok_or_else(|| {
                        LLMError::ConfigError("Could not find droe-llm directory".to_string())
                    })?;
                    
                    tracing::info!("📍 Found droe-llm directory at: {}", droe_llm_dir);
                    
                    Command::new("cargo")
                        .args(&[
                            "run", 
                            "--features", "grpc", 
                            "--", 
                            "serve", 
                            "--port", &self.port.to_string()
                        ])
                        .current_dir(droe_llm_dir)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .map_err(|e| LLMError::ConfigError(format!("Failed to spawn droe-llm server via cargo: {}", e)))?
                }
            };

        self.server_process = Some(child);
        tracing::info!("✅ droe-llm server spawned with PID: {:?}", self.server_process.as_ref().map(|p| p.id()));
        
        Ok(())
    }

    /// Wait for the server to become ready
    async fn wait_for_server(&self, max_attempts: u32) -> Result<()> {
        for attempt in 1..=max_attempts {
            if self.is_server_healthy().await {
                tracing::info!("✅ Server is healthy after {} attempts", attempt);
                return Ok(());
            }
            
            tracing::debug!("⏳ Waiting for server... attempt {}/{}", attempt, max_attempts);
            sleep(Duration::from_millis(500)).await;
        }
        
        Err(LLMError::ConfigError("Server failed to start within timeout".to_string()))
    }

    /// Ensure the client is connected (auto-start server if needed)
    async fn ensure_connected(&mut self) -> Result<()> {
        // If we already have a connected client, check if it's still healthy
        if let Some(ref mut client) = self.client {
            // Quick health check using service info call
            if let Ok(_) = client.get_service_info(Request::new(Empty {})).await {
                return Ok(());
            }
            // If health check failed, clear the client to reconnect
            self.client = None;
        }

        // Check if server is running
        if !self.is_server_healthy().await {
            // Server is not running, start it
            self.spawn_server()?;
            self.wait_for_server(20).await?; // Wait up to 10 seconds
        }

        // Connect to the server
        let channel = Channel::from_shared(format!("http://127.0.0.1:{}", self.port))
            .map_err(|e| LLMError::ConfigError(format!("Invalid gRPC endpoint: {}", e)))?
            .connect()
            .await
            .map_err(|e| LLMError::ConfigError(format!("Failed to connect to gRPC server: {}", e)))?;

        self.client = Some(DroeVmllmServiceClient::new(channel));
        tracing::info!("🔗 Connected to droe-llm gRPC server");
        
        Ok(())
    }

    /// Generate DSL code
    pub async fn generate_dsl(&mut self, request: LLMRequest) -> Result<LLMResponse> {
        self.ensure_connected().await?;
        
        let client = self.client.as_mut().ok_or_else(|| LLMError::ConfigError("Client not connected".to_string()))?;
        
        let grpc_request = convert_to_grpc_request(request);
        let response = client.generate_dsl(Request::new(grpc_request))
            .await
            .map_err(|e| LLMError::ConfigError(format!("gRPC call failed: {}", e)))?;
            
        convert_from_grpc_response(response.into_inner())
    }

    /// Generate DSL code with streaming
    pub async fn generate_dsl_stream(&mut self, request: LLMRequest) -> Result<impl futures::Stream<Item = LLMStreamEvent>> {
        self.ensure_connected().await?;
        
        let client = self.client.as_mut().ok_or_else(|| LLMError::ConfigError("Client not connected".to_string()))?;
        
        let grpc_request = convert_to_grpc_request(request);
        let response = client.generate_dsl_stream(Request::new(grpc_request))
            .await
            .map_err(|e| LLMError::ConfigError(format!("gRPC stream call failed: {}", e)))?;
            
        let stream = response.into_inner();
        Ok(futures::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(msg)) => {
                    let event = convert_stream_response(msg);
                    Some((event, stream))
                }
                Ok(None) => None, // Stream ended
                Err(_) => None, // Stream error, end stream
            }
        }))
    }

    /// Validate code
    pub async fn validate_code(&mut self, code: &str, mode: InferenceMode) -> Result<crate::intelligence::ValidationResult> {
        self.ensure_connected().await?;
        
        let client = self.client.as_mut().ok_or_else(|| LLMError::ConfigError("Client not connected".to_string()))?;
        
        let request = ValidationRequest {
            code: code.to_string(),
            mode: mode_to_string(mode),
            file_path: None,
            workspace_bounds: None,
        };
        
        let response = client.validate_code(Request::new(request))
            .await
            .map_err(|e| LLMError::ConfigError(format!("gRPC validation call failed: {}", e)))?;
            
        convert_validation_result(response.into_inner().result.unwrap_or_default())
    }

    /// Get service information
    pub async fn get_service_info(&mut self) -> Result<crate::service::ServiceInfo> {
        self.ensure_connected().await?;
        
        let client = self.client.as_mut().ok_or_else(|| LLMError::ConfigError("Client not connected".to_string()))?;
        
        let response = client.get_service_info(Request::new(Empty {}))
            .await
            .map_err(|e| LLMError::ConfigError(format!("gRPC service info call failed: {}", e)))?;
            
        let info = response.into_inner();
        Ok(crate::service::ServiceInfo {
            version: info.version,
            model_name: "ollama".to_string(), // Default since not in proto
            ollama_url: "http://localhost:11434".to_string(), // Default
            streaming_enabled: true,
            supported_modes: info.supported_modes.iter().map(|m| string_to_mode(m)).collect::<Result<Vec<_>>>()?,
            server_info: serde_json::json!({}),
            session_stats: Default::default(),
        })
    }
}

impl Drop for GrpcClient {
    fn drop(&mut self) {
        if let Some(mut child) = self.server_process.take() {
            tracing::info!("🛑 Stopping droe-llm server process");
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

// Helper conversion functions
fn convert_to_grpc_request(req: LLMRequest) -> LlmRequest {
    LlmRequest {
        prompt: req.prompt,
        context: req.context,
        file_path: req.file_path,
        existing_content: req.existing_content,
        mode: mode_to_string(req.mode_hint.unwrap_or(InferenceMode::Regular)),
        temperature: None,
        top_p: None,
        top_k: None,
        max_tokens: None,
        stop_sequences: vec![],
        session_id: Some(req.session_id),
        client_id: Some(req.client_id),
        streaming: req.streaming,
    }
}

fn convert_from_grpc_response(resp: LlmResponse) -> Result<LLMResponse> {
    Ok(LLMResponse {
        generated_code: resp.generated_code,
        detected_mode: string_to_mode(&resp.mode_detected)?,
        confidence: resp.confidence,
        validation_result: convert_validation_result(resp.validation.unwrap_or_default())?,
        tokens_used: resp.token_count as u32,
        inference_time_ms: resp.generation_time_ms as u32,
        session_id: resp.session_id,
        file_updates: None, // Would need to convert partial update
    })
}

fn convert_stream_response(resp: LlmStreamResponse) -> LLMStreamEvent {
    use llm_stream_response::Event;
    
    match resp.event {
        Some(Event::Token(token)) => LLMStreamEvent::TokenGenerated {
            token: token.text.clone(),
            accumulated: token.text, // For simplicity, using the same value
        },
        Some(Event::Status(status)) => {
            match status.status.as_str() {
                "mode_detected" => LLMStreamEvent::ModeDetected {
                    mode: string_to_mode(&status.mode_detected).unwrap_or(InferenceMode::Regular),
                    confidence: status.confidence,
                },
                _ => LLMStreamEvent::TokenGenerated {
                    token: format!("Status: {}", status.status),
                    accumulated: "".to_string(),
                }
            }
        },
        Some(Event::Validation(_)) => LLMStreamEvent::ValidationUpdate {
            is_valid: true,
            errors: vec![],
        },
        Some(Event::Error(error)) => LLMStreamEvent::Error {
            error: error.error_message,
        },
        Some(Event::Complete(complete)) => LLMStreamEvent::Completed {
            final_response: LLMResponse {
                generated_code: complete.final_code,
                detected_mode: InferenceMode::Regular, // Default
                confidence: 1.0,
                validation_result: convert_validation_result(complete.final_validation.unwrap_or_default()).unwrap_or_default(),
                tokens_used: complete.total_tokens as u32,
                inference_time_ms: complete.generation_time_ms as u32,
                session_id: resp.session_id,
                file_updates: None,
            },
        },
        None => LLMStreamEvent::Error {
            error: "Empty stream event".to_string(),
        },
    }
}

fn convert_validation_result(result: ValidationResult) -> Result<crate::intelligence::ValidationResult> {
    Ok(crate::intelligence::ValidationResult {
        is_valid: result.is_valid,
        is_safe: true, // Default to safe
        errors: vec![], // Would need proper conversion
        warnings: vec![], // Would need proper conversion
        safety: None, // Would need proper conversion
        ros2: None, // Would need proper conversion
    })
}

fn mode_to_string(mode: InferenceMode) -> String {
    match mode {
        InferenceMode::Regular => "regular".to_string(),
        InferenceMode::Robotics => "robotics".to_string(),
    }
}

fn string_to_mode(mode_str: &str) -> Result<InferenceMode> {
    match mode_str {
        "regular" => Ok(InferenceMode::Regular),
        "robotics" => Ok(InferenceMode::Robotics),
        _ => Err(LLMError::ConfigError(format!("Invalid mode: {}", mode_str))),
    }
}