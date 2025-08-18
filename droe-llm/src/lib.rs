// DroeLLM - Centralized LLM Service for DROELANG
// Adaptive AI for robotics and development

pub mod service;
pub mod providers;
pub mod intelligence;
pub mod session;
pub mod config;
pub mod client;

// Re-export main types
pub use service::{LLMService, LLMRequest, LLMResponse, LLMStreamEvent, ServiceInfo, JsonRpcServer};
#[cfg(feature = "grpc")]
pub use service::LLMServiceImpl;
pub use providers::{OllamaClient, OllamaError, InferenceParams};
pub use intelligence::{ModeDetector, InferenceMode, ModeResult, ValidationEngine, ValidationResult, PartialUpdateEngine, PartialUpdateResult};
pub use session::{SessionManager, SessionInfo};
pub use config::{LLMConfig, WorkspaceBounds};
pub use client::{JsonRpcClient, GenerateResponse, ValidationResultResponse, PartialUpdateResultResponse, StreamEvent, ServiceInfoResponse};
#[cfg(feature = "grpc")]
pub use client::{GrpcClient, GrpcClientWrapper, GrpcWrapperConfig, GrpcWrapperConfigBuilder};

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
    
    #[cfg(feature = "grpc")]
    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),
    
    #[error("Ollama error: {0}")]
    OllamaError(#[from] crate::providers::OllamaError),
}

pub type Result<T> = std::result::Result<T, LLMError>;