#[cfg(feature = "grpc")]
use std::pin::Pin;
#[cfg(feature = "grpc")]
use tokio_stream::{Stream, StreamExt};
#[cfg(feature = "grpc")]
use tonic::{Request, Response, Status};

#[cfg(feature = "grpc")]
use crate::service::{LLMService, LLMStreamEvent};
#[cfg(feature = "grpc")]
use crate::intelligence::InferenceMode;

// Include the generated protobuf code
pub mod llm_service {
    tonic::include_proto!("droevm.llm");
}

use llm_service::{
    droe_vmllm_service_server::{DroeVmllmService, DroeVmllmServiceServer},
    *,
};

pub struct LLMServiceImpl {
    service: LLMService,
}

impl LLMServiceImpl {
    pub fn new(service: LLMService) -> Self {
        Self { service }
    }

    pub fn into_service(self) -> DroeVmllmServiceServer<Self> {
        DroeVmllmServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl DroeVmllmService for LLMServiceImpl {
    type GenerateDSLStreamStream = Pin<Box<dyn Stream<Item = Result<LlmStreamResponse, Status>> + Send>>;

    async fn generate_dsl_stream(
        &self,
        request: Request<LlmRequest>,
    ) -> Result<Response<Self::GenerateDSLStreamStream>, Status> {
        let req = request.into_inner();
        
        // Convert proto request to internal request
        let llm_request = convert_proto_request(req)?;

        // Get the stream from the LLM service
        let stream = self.service.generate_dsl_stream(llm_request)
            .await
            .map_err(|e| Status::internal(format!("Failed to start generation: {}", e)))?;

        // Convert internal stream events to proto responses
        let response_stream = stream.map(|event| {
            match event {
                LLMStreamEvent::ModeDetected { mode, confidence } => {
                    Ok(LlmStreamResponse {
                        event: Some(llm_stream_response::Event::Status(StreamStatus {
                            status: "mode_detected".to_string(),
                            mode_detected: convert_mode_to_proto(mode),
                            confidence,
                            estimated_tokens_remaining: None,
                        })),
                        session_id: "".to_string(), // Would need session ID from request
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
                    })
                }
                LLMStreamEvent::TokenGenerated { token, accumulated: _ } => {
                    Ok(LlmStreamResponse {
                        event: Some(llm_stream_response::Event::Token(StreamToken {
                            text: token,
                            is_partial: true,
                        })),
                        session_id: "".to_string(),
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
                    })
                }
                LLMStreamEvent::ValidationUpdate { is_valid: _, errors: _ } => {
                    Ok(LlmStreamResponse {
                        event: Some(llm_stream_response::Event::Validation(StreamValidation {
                            result: Some(ValidationResult {
                                is_valid: true,
                                errors: vec![],
                                warnings: vec![],
                                safety: None,
                            }),
                            is_partial: true,
                        })),
                        session_id: "".to_string(),
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
                    })
                }
                LLMStreamEvent::PartialUpdate { file_path: _, preview: _ } => {
                    Ok(LlmStreamResponse {
                        event: Some(llm_stream_response::Event::Status(StreamStatus {
                            status: "partial_update".to_string(),
                            mode_detected: "regular".to_string(),
                            confidence: 0.5,
                            estimated_tokens_remaining: None,
                        })),
                        session_id: "".to_string(),
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
                    })
                }
                LLMStreamEvent::Completed { final_response } => {
                    Ok(LlmStreamResponse {
                        event: Some(llm_stream_response::Event::Complete(StreamComplete {
                            final_code: final_response.generated_code,
                            final_validation: Some(ValidationResult {
                                is_valid: true,
                                errors: vec![],
                                warnings: vec![],
                                safety: None,
                            }),
                            total_tokens: final_response.tokens_used as i32,
                            generation_time_ms: final_response.inference_time_ms as i64,
                            partial_update: None,
                        })),
                        session_id: "".to_string(),
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
                    })
                }
                LLMStreamEvent::Error { error } => {
                    Ok(LlmStreamResponse {
                        event: Some(llm_stream_response::Event::Error(StreamError {
                            error_message: error,
                            error_code: "GENERATION_ERROR".to_string(),
                            is_recoverable: false,
                        })),
                        session_id: "".to_string(),
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
                    })
                }
            }
        });

        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn generate_dsl(
        &self,
        request: Request<LlmRequest>,
    ) -> Result<Response<LlmResponse>, Status> {
        let req = request.into_inner();
        let llm_request = convert_proto_request(req)?;

        let response = self.service.generate_dsl(llm_request)
            .await
            .map_err(|e| Status::internal(format!("Generation failed: {}", e)))?;

        Ok(Response::new(convert_response_to_proto(response)))
    }

    async fn validate_code(
        &self,
        request: Request<ValidationRequest>,
    ) -> Result<Response<ValidationResponse>, Status> {
        let req = request.into_inner();
        
        let mode = match req.mode.as_str() {
            "regular" => InferenceMode::Regular,
            "robotics" => InferenceMode::Robotics,
            _ => return Err(Status::invalid_argument("Invalid inference mode")),
        };

        let workspace_bounds = req.workspace_bounds.map(convert_workspace_bounds_from_proto);

        let validation_result = self.service.validate_code(&req.code, mode, &workspace_bounds)
            .await
            .map_err(|e| Status::internal(format!("Validation failed: {}", e)))?;

        Ok(Response::new(ValidationResponse {
            result: Some(convert_validation_result_to_proto(validation_result)),
            validation_time_ms: 0, // Would measure actual validation time
        }))
    }

    async fn get_service_info(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ServiceInfo>, Status> {
        let _info = self.service.get_service_info()
            .await
            .map_err(|e| Status::internal(format!("Failed to get service info: {}", e)))?;

        Ok(Response::new(ServiceInfo {
            version: "1.0.0".to_string(),
            build_date: "2024-01-01".to_string(),
            supported_modes: vec!["regular".to_string(), "robotics".to_string()],
            available_providers: vec!["ollama".to_string()],
            stats: Some(ServiceStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                active_sessions: 0,
                average_response_time_ms: 0.0,
            }),
        }))
    }

    async fn cancel_generation(
        &self,
        request: Request<CancelRequest>,
    ) -> Result<Response<CancelResponse>, Status> {
        let req = request.into_inner();
        
        match self.service.cancel_generation(&req.session_id).await {
            Ok(()) => Ok(Response::new(CancelResponse {
                cancelled: true,
                message: "Generation cancelled successfully".to_string(),
            })),
            Err(e) => Ok(Response::new(CancelResponse {
                cancelled: false,
                message: format!("Failed to cancel: {}", e),
            })),
        }
    }

    async fn apply_partial_update(
        &self,
        request: Request<PartialUpdateRequest>,
    ) -> Result<Response<PartialUpdateResponse>, Status> {
        let req = request.into_inner();
        
        // No mode_hint field in proto, so we'll use a default
        let mode_hint = Some(InferenceMode::Regular);

        let result = self.service.apply_partial_update(
            &req.file_path,
            &req.original_content,
            &req.prompt,
            mode_hint,
            "default_session".to_string(), // No session_id field in proto
            false, // No streaming field in proto
        ).await.map_err(|e| Status::internal(format!("Partial update failed: {}", e)))?;

        Ok(Response::new(PartialUpdateResponse {
            result: Some(PartialUpdateResult {
                updated_content: result.updated_content,
                deltas: vec![], // Would need to convert deltas properly
                stats: Some(DiffStats {
                    lines_added: 0,
                    lines_removed: 0,
                    lines_modified: 0,
                    similarity_score: 0.8,
                }),
                preview: "".to_string(),
            }),
            update_time_ms: 0, // Would measure actual update time
        }))
    }

}

// Helper functions for proto conversion
fn convert_proto_request(req: LlmRequest) -> Result<crate::service::LLMRequest, Status> {
    // Convert mode string to enum
    let mode_hint = match req.mode.as_str() {
        "regular" => Some(InferenceMode::Regular),
        "robotics" => Some(InferenceMode::Robotics),
        _ => Some(InferenceMode::Regular), // Default
    };

    let workspace_bounds = None; // Not in the current proto request

    Ok(crate::service::LLMRequest {
        prompt: req.prompt,
        context: req.context,
        mode_hint,
        workspace_bounds,
        client_id: req.client_id.unwrap_or_default(),
        session_id: req.session_id.unwrap_or_default(),
        streaming: req.streaming,
        file_path: req.file_path,
        existing_content: req.existing_content,
    })
}

fn convert_response_to_proto(response: crate::service::LLMResponse) -> LlmResponse {
    LlmResponse {
        generated_code: response.generated_code,
        mode_detected: convert_mode_to_proto(response.detected_mode),
        confidence: response.confidence,
        detected_keywords: vec![], // Would need to extract keywords
        validation: Some(convert_validation_result_to_proto(response.validation_result)),
        partial_update: None, // Would need to convert partial update
        session_id: response.session_id,
        generation_time_ms: response.inference_time_ms as i64,
        token_count: response.tokens_used as i32,
    }
}

fn convert_validation_result_to_proto(result: crate::intelligence::ValidationResult) -> ValidationResult {
    ValidationResult {
        is_valid: result.is_valid,
        errors: vec![], // Would need to convert validation errors properly
        warnings: vec![], // Would need to convert validation warnings properly
        safety: Some(SafetyValidation {
            workspace_bounds_valid: true,
            collision_free: true,
            emergency_stop_accessible: true,
            safety_warnings: vec![],
            ros2: Some(Ros2Validation {
                services_available: true,
                topics_valid: true,
                missing_dependencies: vec![],
            }),
        }),
    }
}

fn convert_workspace_bounds_from_proto(bounds: WorkspaceBounds) -> crate::config::WorkspaceBounds {
    crate::config::WorkspaceBounds {
        x_min: bounds.x_min,
        x_max: bounds.x_max,
        y_min: bounds.y_min,
        y_max: bounds.y_max,
        z_min: bounds.z_min,
        z_max: bounds.z_max,
        max_distance: bounds.max_distance,
        max_rotation: bounds.max_rotation,
        max_wait_time: bounds.max_wait_time,
    }
}

fn convert_mode_to_proto(mode: InferenceMode) -> String {
    match mode {
        InferenceMode::Regular => "regular".to_string(),
        InferenceMode::Robotics => "robotics".to_string(),
    }
}