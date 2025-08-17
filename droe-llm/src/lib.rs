// DroeLLM - Centralized LLM Service for DROELANG
// Adaptive AI for robotics and development

pub mod service;
pub mod providers;
pub mod intelligence;
pub mod session;
pub mod config;

// Re-export main types
pub use service::{LLMService, LLMRequest, LLMResponse, LLMStreamEvent, LLMServiceImpl};
pub use providers::{OllamaClient, OllamaError, InferenceParams};
pub use intelligence::{ModeDetector, InferenceMode, ModeResult, ValidationEngine, ValidationResult, PartialUpdateEngine, PartialUpdateResult};
pub use session::{SessionManager, SessionInfo};
pub use config::{LLMConfig, WorkspaceBounds};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Mode detection error: {0}")]
    ModeDetectionError(String),
    
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Partial update error: {0}")]
    PartialUpdateError(String),
    
    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),
    
    #[error("Ollama error: {0}")]
    OllamaError(#[from] crate::providers::OllamaError),
}

pub type Result<T> = std::result::Result<T, LLMError>;