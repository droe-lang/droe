use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{LLMError, Result};
use crate::intelligence::InferenceMode;
use crate::config::WorkspaceBounds;

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct ValidationRequest {
    pub code: String,
    pub mode: String,
    pub file_path: Option<String>,
    pub workspace_bounds: Option<WorkspaceBounds>,
}

#[derive(Debug, Serialize)]
pub struct PartialUpdateRequest {
    pub file_path: String,
    pub original_content: String,
    pub prompt: String,
    pub mode_hint: Option<String>,
    pub session_id: Option<String>,
    pub streaming: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CancelRequest {
    pub session_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct ValidationResultResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub safety_warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PartialUpdateResultResponse {
    pub updated_content: String,
    pub preview: String,
    pub lines_added: i32,
    pub lines_removed: i32,
    pub lines_modified: i32,
    pub similarity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub event_type: String,
    pub data: Value,
    pub session_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct ServiceInfoResponse {
    pub version: String,
    pub model_name: String,
    pub ollama_url: String,
    pub streaming_enabled: bool,
    pub supported_modes: Vec<String>,
    pub server_info: Value,
    pub session_stats: Value,
}

pub struct JsonRpcClient {
    server_address: String,
    timeout: std::time::Duration,
}

impl JsonRpcClient {
    pub fn new(server_address: String, timeout: std::time::Duration) -> Self {
        Self {
            server_address,
            timeout,
        }
    }

    pub async fn generate_dsl(
        &self,
        prompt: String,
        context: Option<String>,
        mode: Option<InferenceMode>,
        file_path: Option<String>,
        existing_content: Option<String>,
        session_id: Option<String>,
        client_id: Option<String>,
    ) -> Result<GenerateResponse> {
        let request = GenerateRequest {
            prompt,
            context,
            file_path,
            existing_content,
            mode: mode.map(|m| mode_to_string(m)),
            temperature: None,
            top_p: None,
            top_k: None,
            max_tokens: None,
            stop_sequences: None,
            session_id,
            client_id,
            streaming: Some(false),
        };

        let response = self.send_request("generate_dsl", serde_json::to_value(request)?).await?;
        let generate_response: GenerateResponse = serde_json::from_value(response)?;
        Ok(generate_response)
    }

    pub async fn generate_dsl_stream(
        &self,
        prompt: String,
        context: Option<String>,
        mode: Option<InferenceMode>,
        file_path: Option<String>,
        existing_content: Option<String>,
        session_id: Option<String>,
        client_id: Option<String>,
    ) -> Result<Vec<StreamEvent>> {
        let request = GenerateRequest {
            prompt,
            context,
            file_path,
            existing_content,
            mode: mode.map(|m| mode_to_string(m)),
            temperature: None,
            top_p: None,
            top_k: None,
            max_tokens: None,
            stop_sequences: None,
            session_id,
            client_id,
            streaming: Some(true),
        };

        let responses = self.send_streaming_request("generate_dsl_stream", serde_json::to_value(request)?).await?;
        let mut stream_events = Vec::new();
        
        for response in responses {
            let stream_event: StreamEvent = serde_json::from_value(response)?;
            stream_events.push(stream_event);
        }
        
        Ok(stream_events)
    }

    pub async fn generate_dsl_stream_with_callback<F>(
        &self,
        prompt: String,
        context: Option<String>,
        mode: Option<InferenceMode>,
        file_path: Option<String>,
        existing_content: Option<String>,
        session_id: Option<String>,
        client_id: Option<String>,
        mut callback: F,
    ) -> Result<GenerateResponse>
    where
        F: FnMut(StreamEvent) -> Result<()> + Send,
    {
        let request = GenerateRequest {
            prompt,
            context,
            file_path,
            existing_content,
            mode: mode.map(|m| mode_to_string(m)),
            temperature: None,
            top_p: None,
            top_k: None,
            max_tokens: None,
            stop_sequences: None,
            session_id,
            client_id,
            streaming: Some(true),
        };

        // Send the streaming request with real-time callback
        let final_response = self.send_streaming_request_with_callback(
            "generate_dsl_stream", 
            serde_json::to_value(request)?,
            &mut callback
        ).await?;

        // Parse the final completed event
        let stream_event: StreamEvent = serde_json::from_value(final_response)?;
        if let Some(data) = stream_event.data.as_object() {
            if let (Some(generated_code), Some(detected_mode), Some(confidence), Some(tokens_used), Some(inference_time_ms), Some(session_id)) = (
                data.get("generated_code").and_then(|v| v.as_str()),
                data.get("detected_mode").and_then(|v| v.as_str()),
                data.get("confidence").and_then(|v| v.as_f64()),
                data.get("tokens_used").and_then(|v| v.as_u64()),
                data.get("inference_time_ms").and_then(|v| v.as_u64()),
                data.get("session_id").and_then(|v| v.as_str()),
            ) {
                return Ok(GenerateResponse {
                    generated_code: generated_code.to_string(),
                    detected_mode: detected_mode.to_string(),
                    confidence: confidence as f32,
                    validation_result: ValidationResultResponse {
                        is_valid: true,
                        errors: vec![],
                        warnings: vec![],
                        safety_warnings: vec![],
                    },
                    tokens_used: tokens_used as u32,
                    inference_time_ms: inference_time_ms as u32,
                    session_id: session_id.to_string(),
                    file_updates: None,
                });
            }
        }

        Err(LLMError::ConfigError("Failed to parse completed event".to_string()))
    }

    pub async fn validate_code(
        &self,
        code: String,
        mode: InferenceMode,
        file_path: Option<String>,
        workspace_bounds: Option<WorkspaceBounds>,
    ) -> Result<ValidationResultResponse> {
        let request = ValidationRequest {
            code,
            mode: mode_to_string(mode),
            file_path,
            workspace_bounds,
        };

        let response = self.send_request("validate_code", serde_json::to_value(request)?).await?;
        let validation_response: ValidationResultResponse = serde_json::from_value(response)?;
        Ok(validation_response)
    }

    pub async fn apply_partial_update(
        &self,
        file_path: String,
        original_content: String,
        prompt: String,
        mode_hint: Option<InferenceMode>,
        session_id: Option<String>,
        streaming: Option<bool>,
    ) -> Result<PartialUpdateResultResponse> {
        let request = PartialUpdateRequest {
            file_path,
            original_content,
            prompt,
            mode_hint: mode_hint.map(|m| mode_to_string(m)),
            session_id,
            streaming,
        };

        let response = self.send_request("apply_partial_update", serde_json::to_value(request)?).await?;
        let update_response: PartialUpdateResultResponse = serde_json::from_value(response)?;
        Ok(update_response)
    }

    pub async fn cancel_generation(&self, session_id: String, reason: Option<String>) -> Result<Value> {
        let request = CancelRequest {
            session_id,
            reason,
        };

        self.send_request("cancel_generation", serde_json::to_value(request)?).await
    }

    pub async fn get_service_info(&self) -> Result<ServiceInfoResponse> {
        let response = self.send_request("get_service_info", json!({})).await?;
        let service_info: ServiceInfoResponse = serde_json::from_value(response)?;
        Ok(service_info)
    }

    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: json!(Uuid::new_v4().to_string()),
        };

        let mut stream = TcpStream::connect(&self.server_address).await
            .map_err(|e| LLMError::ConfigError(format!("Failed to connect to server: {}", e)))?;

        let request_line = format!("{}\n", serde_json::to_string(&request)?);
        stream.write_all(request_line.as_bytes()).await
            .map_err(|e| LLMError::ConfigError(format!("Failed to send request: {}", e)))?;

        let (reader, _) = stream.split();
        let mut lines = BufReader::new(reader).lines();

        if let Some(line) = lines.next_line().await
            .map_err(|e| LLMError::ConfigError(format!("Failed to read response: {}", e)))? {
            let response: JsonRpcResponse = serde_json::from_str(&line)?;
            
            if let Some(error) = response.error {
                return Err(LLMError::ConfigError(format!("JSON-RPC error {}: {}", error.code, error.message)));
            }
            
            response.result.ok_or_else(|| LLMError::ConfigError("No result in response".to_string()))
        } else {
            Err(LLMError::ConfigError("No response received".to_string()))
        }
    }

    async fn send_streaming_request(&self, method: &str, params: Value) -> Result<Vec<Value>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: json!(Uuid::new_v4().to_string()),
        };

        let mut stream = TcpStream::connect(&self.server_address).await
            .map_err(|e| LLMError::ConfigError(format!("Failed to connect to server: {}", e)))?;

        let request_line = format!("{}\n", serde_json::to_string(&request)?);
        stream.write_all(request_line.as_bytes()).await
            .map_err(|e| LLMError::ConfigError(format!("Failed to send request: {}", e)))?;

        let (reader, _) = stream.split();
        let mut lines = BufReader::new(reader).lines();
        let mut responses = Vec::new();

        while let Some(line) = lines.next_line().await
            .map_err(|e| LLMError::ConfigError(format!("Failed to read response: {}", e)))? {
            let response: JsonRpcResponse = serde_json::from_str(&line)?;
            
            if let Some(error) = response.error {
                return Err(LLMError::ConfigError(format!("JSON-RPC error {}: {}", error.code, error.message)));
            }
            
            if let Some(result) = response.result {
                // Check if this is a completion event before moving the result
                let should_break = if let Ok(stream_event) = serde_json::from_value::<StreamEvent>(result.clone()) {
                    stream_event.event_type == "completed"
                } else {
                    false
                };

                responses.push(result);
                
                if should_break {
                    break;
                }
            }
        }

        Ok(responses)
    }

    async fn send_streaming_request_with_callback<F>(
        &self, 
        method: &str, 
        params: Value,
        callback: &mut F
    ) -> Result<Value>
    where
        F: FnMut(StreamEvent) -> Result<()> + Send,
    {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: json!(Uuid::new_v4().to_string()),
        };

        let mut stream = TcpStream::connect(&self.server_address).await
            .map_err(|e| LLMError::ConfigError(format!("Failed to connect to server: {}", e)))?;

        let request_line = format!("{}\n", serde_json::to_string(&request)?);
        stream.write_all(request_line.as_bytes()).await
            .map_err(|e| LLMError::ConfigError(format!("Failed to send request: {}", e)))?;

        let (reader, _) = stream.split();
        let mut lines = BufReader::new(reader).lines();

        while let Some(line) = lines.next_line().await
            .map_err(|e| LLMError::ConfigError(format!("Failed to read response: {}", e)))? {
            let response: JsonRpcResponse = serde_json::from_str(&line)?;
            
            if let Some(error) = response.error {
                return Err(LLMError::ConfigError(format!("JSON-RPC error {}: {}", error.code, error.message)));
            }
            
            if let Some(result) = response.result {
                // Parse the stream event and call the callback immediately
                if let Ok(stream_event) = serde_json::from_value::<StreamEvent>(result.clone()) {
                    // Call the callback for each event as it arrives
                    callback(stream_event.clone())?;
                    
                    // If this is a completion event, return it as the final result
                    if stream_event.event_type == "completed" {
                        return Ok(result);
                    }
                }
            }
        }

        Err(LLMError::ConfigError("Stream ended without completion".to_string()))
    }

    pub async fn health_check(&self) -> Result<bool> {
        match self.get_service_info().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

fn mode_to_string(mode: InferenceMode) -> String {
    match mode {
        InferenceMode::Regular => "regular".to_string(),
        InferenceMode::Robotics => "robotics".to_string(),
    }
}

impl Clone for JsonRpcClient {
    fn clone(&self) -> Self {
        Self {
            server_address: self.server_address.clone(),
            timeout: self.timeout,
        }
    }
}