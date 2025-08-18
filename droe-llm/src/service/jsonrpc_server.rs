use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::service::{LLMService, LLMStreamEvent};
use crate::intelligence::InferenceMode;
use crate::config::WorkspaceBounds;

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub context: Option<String>,
    pub file_path: Option<String>,
    pub existing_content: Option<String>,
    pub mode: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub max_tokens: Option<i32>,
    pub stop_sequences: Option<Vec<String>>,
    pub session_id: Option<String>,
    pub client_id: Option<String>,
    pub streaming: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ValidationRequest {
    pub code: String,
    pub mode: String,
    pub file_path: Option<String>,
    pub workspace_bounds: Option<WorkspaceBounds>,
}

#[derive(Debug, Deserialize)]
pub struct PartialUpdateRequest {
    pub file_path: String,
    pub original_content: String,
    pub prompt: String,
    pub mode_hint: Option<String>,
    pub session_id: Option<String>,
    pub streaming: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub session_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub generated_code: String,
    pub detected_mode: String,
    pub confidence: f32,
    pub validation_result: ValidationResultResponse,
    pub tokens_used: u32,
    pub inference_time_ms: u32,
    pub session_id: String,
    pub file_updates: Option<PartialUpdateResultResponse>,
}

#[derive(Debug, Serialize)]
pub struct ValidationResultResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub safety_warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PartialUpdateResultResponse {
    pub updated_content: String,
    pub preview: String,
    pub lines_added: i32,
    pub lines_removed: i32,
    pub lines_modified: i32,
    pub similarity_score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamEvent {
    pub event_type: String,
    pub data: Value,
    pub session_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct ServiceInfoResponse {
    pub version: String,
    pub model_name: String,
    pub ollama_url: String,
    pub streaming_enabled: bool,
    pub supported_modes: Vec<String>,
    pub server_info: Value,
    pub session_stats: Value,
}

pub struct JsonRpcServer {
    llm_service: Arc<LLMService>,
    port: u16,
}

impl JsonRpcServer {
    pub fn new(llm_service: LLMService, port: u16) -> Self {
        Self {
            llm_service: Arc::new(llm_service),
            port,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        tracing::info!("🚀 JSON-RPC Server listening on port {}", self.port);

        loop {
            let (stream, addr) = listener.accept().await?;
            let service = Arc::clone(&self.llm_service);
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, service).await {
                    tracing::error!("Connection error from {}: {}", addr, e);
                }
            });
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        service: Arc<LLMService>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (reader, mut writer) = stream.split();
        let mut lines = BufReader::new(reader).lines();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                        id: None,
                    };
                    let response_line = format!("{}\n", serde_json::to_string(&error_response)?);
                    writer.write_all(response_line.as_bytes()).await?;
                    continue;
                }
            };

            let response = Self::handle_request(request, Arc::clone(&service)).await;
            
            match response {
                Ok(responses) => {
                    for resp in responses {
                        let response_line = format!("{}\n", serde_json::to_string(&resp)?);
                        writer.write_all(response_line.as_bytes()).await?;
                    }
                }
                Err(e) => {
                    tracing::error!("Request handling error: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn handle_request(
        request: JsonRpcRequest,
        service: Arc<LLMService>,
    ) -> Result<Vec<JsonRpcResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let method = request.method.as_str();
        
        match method {
            "generate_dsl" => {
                let params: GenerateRequest = serde_json::from_value(request.params)?;
                let llm_request = Self::convert_to_llm_request(params)?;
                
                match service.generate_dsl(llm_request).await {
                    Ok(response) => {
                        let result = GenerateResponse {
                            generated_code: response.generated_code,
                            detected_mode: Self::mode_to_string(response.detected_mode),
                            confidence: response.confidence,
                            validation_result: Self::convert_validation_result(response.validation_result),
                            tokens_used: response.tokens_used,
                            inference_time_ms: response.inference_time_ms,
                            session_id: response.session_id,
                            file_updates: response.file_updates.map(Self::convert_partial_update_result),
                        };
                        
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(serde_json::to_value(result)?),
                            error: None,
                            id: request.id,
                        }])
                    }
                    Err(e) => {
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32603,
                                message: format!("Internal error: {}", e),
                                data: None,
                            }),
                            id: request.id,
                        }])
                    }
                }
            }
            
            "generate_dsl_stream" => {
                let params: GenerateRequest = serde_json::from_value(request.params)?;
                let mut llm_request = Self::convert_to_llm_request(params)?;
                
                // Create a new session for streaming if one wasn't provided
                if llm_request.session_id.is_empty() || llm_request.session_id == "auto" {
                    let mode_hint = llm_request.mode_hint.unwrap_or(crate::intelligence::InferenceMode::Regular);
                    llm_request.session_id = service.create_session(llm_request.client_id.clone(), mode_hint).await?;
                }
                
                match service.generate_dsl_stream(llm_request.clone()).await {
                    Ok(mut stream) => {
                        let mut responses = Vec::new();
                        
                        while let Some(event) = stream.next().await {
                            let stream_event = Self::convert_stream_event(event, &llm_request.session_id);
                            responses.push(JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(serde_json::to_value(stream_event)?),
                                error: None,
                                id: request.id.clone(),
                            });
                        }
                        
                        Ok(responses)
                    }
                    Err(e) => {
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32603,
                                message: format!("Stream error: {}", e),
                                data: None,
                            }),
                            id: request.id,
                        }])
                    }
                }
            }
            
            "validate_code" => {
                let params: ValidationRequest = serde_json::from_value(request.params)?;
                let mode = Self::string_to_mode(&params.mode)?;
                
                match service.validate_code(&params.code, mode, &params.workspace_bounds).await {
                    Ok(validation) => {
                        let result = Self::convert_validation_result(validation);
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(serde_json::to_value(result)?),
                            error: None,
                            id: request.id,
                        }])
                    }
                    Err(e) => {
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32603,
                                message: format!("Validation error: {}", e),
                                data: None,
                            }),
                            id: request.id,
                        }])
                    }
                }
            }
            
            "apply_partial_update" => {
                let params: PartialUpdateRequest = serde_json::from_value(request.params)?;
                let mode_hint = params.mode_hint.map(|m| Self::string_to_mode(&m)).transpose()?;
                let session_id = params.session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
                let streaming = params.streaming.unwrap_or(false);
                
                match service.apply_partial_update(
                    &params.file_path,
                    &params.original_content,
                    &params.prompt,
                    mode_hint,
                    session_id,
                    streaming,
                ).await {
                    Ok(result) => {
                        let converted_result = Self::convert_partial_update_result(result);
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(serde_json::to_value(converted_result)?),
                            error: None,
                            id: request.id,
                        }])
                    }
                    Err(e) => {
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32603,
                                message: format!("Partial update error: {}", e),
                                data: None,
                            }),
                            id: request.id,
                        }])
                    }
                }
            }
            
            "cancel_generation" => {
                let params: CancelRequest = serde_json::from_value(request.params)?;
                
                match service.cancel_generation(&params.session_id).await {
                    Ok(_) => {
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(json!({
                                "cancelled": true,
                                "message": "Generation cancelled successfully"
                            })),
                            error: None,
                            id: request.id,
                        }])
                    }
                    Err(e) => {
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(json!({
                                "cancelled": false,
                                "message": format!("Failed to cancel: {}", e)
                            })),
                            error: None,
                            id: request.id,
                        }])
                    }
                }
            }
            
            "get_service_info" => {
                match service.get_service_info().await {
                    Ok(info) => {
                        let result = ServiceInfoResponse {
                            version: info.version,
                            model_name: info.model_name,
                            ollama_url: info.ollama_url,
                            streaming_enabled: info.streaming_enabled,
                            supported_modes: info.supported_modes.iter().map(|m| Self::mode_to_string(*m)).collect(),
                            server_info: info.server_info,
                            session_stats: serde_json::to_value(info.session_stats)?,
                        };
                        
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(serde_json::to_value(result)?),
                            error: None,
                            id: request.id,
                        }])
                    }
                    Err(e) => {
                        Ok(vec![JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32603,
                                message: format!("Service info error: {}", e),
                                data: None,
                            }),
                            id: request.id,
                        }])
                    }
                }
            }
            
            _ => {
                Ok(vec![JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Method not found: {}", method),
                        data: None,
                    }),
                    id: request.id,
                }])
            }
        }
    }

    fn convert_to_llm_request(params: GenerateRequest) -> Result<crate::service::LLMRequest, Box<dyn std::error::Error + Send + Sync>> {
        let mode_hint = params.mode.map(|m| Self::string_to_mode(&m)).transpose()?;
        
        Ok(crate::service::LLMRequest {
            prompt: params.prompt,
            context: params.context,
            mode_hint,
            workspace_bounds: None, // Could be added to GenerateRequest if needed
            client_id: params.client_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            session_id: params.session_id.unwrap_or_else(|| "auto".to_string()), // Use "auto" to signal session creation needed
            streaming: params.streaming.unwrap_or(false),
            file_path: params.file_path,
            existing_content: params.existing_content,
        })
    }

    fn convert_stream_event(event: LLMStreamEvent, session_id: &str) -> StreamEvent {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        match event {
            LLMStreamEvent::ModeDetected { mode, confidence } => {
                StreamEvent {
                    event_type: "mode_detected".to_string(),
                    data: json!({
                        "mode": Self::mode_to_string(mode),
                        "confidence": confidence
                    }),
                    session_id: session_id.to_string(),
                    timestamp,
                }
            }
            LLMStreamEvent::TokenGenerated { token, accumulated } => {
                StreamEvent {
                    event_type: "token_generated".to_string(),
                    data: json!({
                        "token": token,
                        "accumulated": accumulated
                    }),
                    session_id: session_id.to_string(),
                    timestamp,
                }
            }
            LLMStreamEvent::ValidationUpdate { is_valid, errors } => {
                StreamEvent {
                    event_type: "validation_update".to_string(),
                    data: json!({
                        "is_valid": is_valid,
                        "errors": errors
                    }),
                    session_id: session_id.to_string(),
                    timestamp,
                }
            }
            LLMStreamEvent::PartialUpdate { file_path, preview } => {
                StreamEvent {
                    event_type: "partial_update".to_string(),
                    data: json!({
                        "file_path": file_path,
                        "preview": preview
                    }),
                    session_id: session_id.to_string(),
                    timestamp,
                }
            }
            LLMStreamEvent::Completed { final_response } => {
                StreamEvent {
                    event_type: "completed".to_string(),
                    data: json!({
                        "generated_code": final_response.generated_code,
                        "detected_mode": Self::mode_to_string(final_response.detected_mode),
                        "confidence": final_response.confidence,
                        "validation_result": Self::convert_validation_result(final_response.validation_result),
                        "tokens_used": final_response.tokens_used,
                        "inference_time_ms": final_response.inference_time_ms,
                        "session_id": final_response.session_id,
                        "file_updates": final_response.file_updates.map(Self::convert_partial_update_result)
                    }),
                    session_id: final_response.session_id,
                    timestamp,
                }
            }
            LLMStreamEvent::Error { error } => {
                StreamEvent {
                    event_type: "error".to_string(),
                    data: json!({
                        "error": error
                    }),
                    session_id: session_id.to_string(),
                    timestamp,
                }
            }
        }
    }

    fn convert_validation_result(result: crate::intelligence::ValidationResult) -> ValidationResultResponse {
        let safety_warnings = if let Some(safety) = result.safety {
            safety.safety_warnings
        } else {
            Vec::new()
        };

        ValidationResultResponse {
            is_valid: result.is_valid,
            errors: result.errors,
            warnings: result.warnings,
            safety_warnings,
        }
    }

    fn convert_partial_update_result(result: crate::intelligence::PartialUpdateResult) -> PartialUpdateResultResponse {
        PartialUpdateResultResponse {
            updated_content: result.updated_content,
            preview: result.preview,
            lines_added: result.stats.lines_added as i32,
            lines_removed: result.stats.lines_removed as i32,
            lines_modified: result.stats.lines_modified as i32,
            similarity_score: 0.95, // Default similarity score since it's not in DiffStats
        }
    }

    fn string_to_mode(mode_str: &str) -> Result<InferenceMode, Box<dyn std::error::Error + Send + Sync>> {
        match mode_str {
            "regular" => Ok(InferenceMode::Regular),
            "robotics" => Ok(InferenceMode::Robotics),
            _ => Err(format!("Invalid mode: {}", mode_str).into()),
        }
    }

    fn mode_to_string(mode: InferenceMode) -> String {
        match mode {
            InferenceMode::Regular => "regular".to_string(),
            InferenceMode::Robotics => "robotics".to_string(),
        }
    }
}