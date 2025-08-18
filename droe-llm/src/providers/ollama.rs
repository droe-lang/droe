use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_stream::{Stream, StreamExt};
use std::pin::Pin;
use std::time::Duration;
use bytes::Bytes;
use thiserror::Error;
use crate::intelligence::InferenceMode;

#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Stream error: {0}")]
    StreamError(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Server error: {status} - {message}")]
    ServerError { status: u16, message: String },
    
    #[error("Timeout error")]
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParams {
    pub model: String,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub max_tokens: u32,
    pub stop_sequences: Vec<String>,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    top_k: u32,
    num_predict: u32,
    stop: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: Option<String>,
    done: Option<bool>,
    error: Option<String>,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new(base_url: String, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
        }
    }

    pub async fn generate_stream(
        &self,
        prompt: String,
        params: InferenceParams,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, OllamaError>> + Send + 'static>>, OllamaError> {
        let request = OllamaGenerateRequest {
            model: params.model.clone(),
            prompt,
            stream: true,
            options: OllamaOptions {
                temperature: params.temperature,
                top_p: params.top_p,
                top_k: params.top_k,
                num_predict: params.max_tokens,
                stop: params.stop_sequences,
            },
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let stream = response
            .bytes_stream()
            .map(|chunk_result| {
                match chunk_result {
                    Ok(chunk) => parse_streaming_chunk_static(chunk),
                    Err(e) => Err(OllamaError::StreamError(e.to_string())),
                }
            })
            .filter_map(|result| {
                match result {
                    Ok(Some(text)) => Some(Ok(text)),
                    Ok(None) => None, // Skip empty chunks
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(Box::pin(stream))
    }

    pub async fn generate_complete(
        &self,
        prompt: String,
        params: InferenceParams,
    ) -> Result<String, OllamaError> {
        let request = OllamaGenerateRequest {
            model: params.model.clone(),
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: params.temperature,
                top_p: params.top_p,
                top_k: params.top_k,
                num_predict: params.max_tokens,
                stop: params.stop_sequences,
            },
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let response_body: OllamaResponse = response.json().await?;
        
        if let Some(error) = response_body.error {
            return Err(OllamaError::StreamError(error));
        }

        Ok(response_body.response.unwrap_or_default())
    }

    pub async fn check_model_exists(&self, model_name: &str) -> Result<bool, OllamaError> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: "Failed to list models".to_string(),
            });
        }

        let tags_response: Value = response.json().await?;
        
        if let Some(models) = tags_response.get("models").and_then(|m| m.as_array()) {
            for model in models {
                if let Some(name) = model.get("name").and_then(|n| n.as_str()) {
                    if name == model_name || name.starts_with(&format!("{}:", model_name)) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    pub async fn pull_model(&self, model_name: &str) -> Result<(), OllamaError> {
        let request = serde_json::json!({
            "name": model_name
        });

        let response = self
            .client
            .post(format!("{}/api/pull", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: "Failed to pull model".to_string(),
            });
        }

        Ok(())
    }

    pub async fn get_server_info(&self) -> Result<Value, OllamaError> {
        let response = self
            .client
            .get(format!("{}/api/version", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: "Failed to get server info".to_string(),
            });
        }

        Ok(response.json().await?)
    }

    #[allow(dead_code)]
    fn parse_streaming_chunk(&self, chunk: Bytes) -> Result<Option<String>, OllamaError> {
        let chunk_str = String::from_utf8_lossy(&chunk);
        
        // Ollama sends JSON lines, we need to parse each line
        for line in chunk_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let response: OllamaResponse = serde_json::from_str(line)?;
            
            if let Some(error) = response.error {
                return Err(OllamaError::StreamError(error));
            }

            if let Some(text) = response.response {
                if !text.is_empty() {
                    return Ok(Some(text));
                }
            }

            // Check if done
            if response.done == Some(true) {
                return Ok(None); // Signal end of stream
            }
        }

        Ok(None)
    }

    pub fn create_optimized_params(&self, mode: InferenceMode, base_params: &InferenceParams) -> InferenceParams {
        match mode {
            InferenceMode::Robotics => InferenceParams {
                model: base_params.model.clone(),
                temperature: 0.05, // Very low for consistency
                top_p: 0.8,
                top_k: 20,
                max_tokens: 200, // Limited for robotics
                stop_sequences: vec![
                    "Human:".to_string(),
                    "User:".to_string(),
                    "\n\n".to_string(), // Stop at double newline for robotics
                ],
            },
            InferenceMode::Regular => InferenceParams {
                model: base_params.model.clone(),
                temperature: 0.7,
                top_p: 0.9,
                top_k: 40,
                max_tokens: 800,
                stop_sequences: vec![
                    "Human:".to_string(),
                    "User:".to_string(),
                ],
            },
        }
    }
}

impl Clone for OllamaClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
        }
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new(
            "http://localhost:11434".to_string(),
            Duration::from_secs(30),
        )
    }
}

fn parse_streaming_chunk_static(chunk: Bytes) -> Result<Option<String>, OllamaError> {
    let chunk_str = String::from_utf8_lossy(&chunk);
    
    // Ollama sends JSON lines, we need to parse each line
    for line in chunk_str.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let response: OllamaResponse = serde_json::from_str(line)?;
        
        if let Some(error) = response.error {
            return Err(OllamaError::StreamError(error));
        }

        if let Some(text) = response.response {
            if !text.is_empty() {
                return Ok(Some(text));
            }
        }

        // Check if done
        if response.done == Some(true) {
            return Ok(None); // Signal end of stream
        }
    }

    Ok(None)
}