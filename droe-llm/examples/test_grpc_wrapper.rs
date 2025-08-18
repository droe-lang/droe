use droe_llm::{
    GrpcClientWrapper, GrpcWrapperConfigBuilder,
    LLMRequest, InferenceMode
};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .init();

    info!("🧪 Testing gRPC Client Wrapper functionality");

    // Create wrapper with custom configuration
    let config = GrpcWrapperConfigBuilder::new()
        .default_port(50052) // Use a different port for testing
        .startup_timeout_ms(15000)
        .health_check_interval_ms(500)
        .build();

    let wrapper = GrpcClientWrapper::new(config);

    // Test 1: Check for available port
    info!("🔍 Test 1: Finding available port");
    match wrapper.find_available_port().await {
        Ok(port) => info!("✅ Found available port: {}", port),
        Err(e) => error!("❌ Failed to find available port: {}", e),
    }

    // Test 2: Start a new server
    info!("🚀 Test 2: Starting new server");
    match wrapper.start_new_server().await {
        Ok(port) => {
            info!("✅ Server started on port: {}", port);
            
            // Test 3: Check if server is running
            info!("🔍 Test 3: Checking if server is running");
            if wrapper.is_server_running(port).await {
                info!("✅ Server is running on port {}", port);
            } else {
                error!("❌ Server is not responding on port {}", port);
                return Ok(());
            }

            // Test 4: Get service info
            info!("📊 Test 4: Getting service info");
            match wrapper.get_service_info().await {
                Ok(service_info) => {
                    info!("✅ Service info retrieved:");
                    info!("   Version: {}", service_info.version);
                    info!("   Model: {}", service_info.model_name);
                    info!("   Streaming: {}", service_info.streaming_enabled);
                    info!("   Modes: {:?}", service_info.supported_modes);
                }
                Err(e) => error!("❌ Failed to get service info: {}", e),
            }

            // Test 5: Simple DSL generation
            info!("🎯 Test 5: Testing DSL generation");
            let request = LLMRequest {
                prompt: "Generate a simple hello world program".to_string(),
                context: Some("Testing the gRPC client wrapper".to_string()),
                mode_hint: Some(InferenceMode::Regular),
                workspace_bounds: None,
                client_id: "test_client".to_string(),
                session_id: "test_session".to_string(),
                streaming: false,
                file_path: None,
                existing_content: None,
            };

            match wrapper.generate_dsl(request).await {
                Ok(response) => {
                    info!("✅ DSL generation successful:");
                    info!("   Generated code length: {} chars", response.generated_code.len());
                    info!("   Detected mode: {:?}", response.detected_mode);
                    info!("   Confidence: {:.2}", response.confidence);
                    info!("   Tokens used: {}", response.tokens_used);
                    info!("   Inference time: {}ms", response.inference_time_ms);
                }
                Err(e) => error!("❌ DSL generation failed: {}", e),
            }

            // Test 6: Code validation
            info!("✅ Test 6: Testing code validation");
            let test_code = "display \"Hello, World!\"";
            match wrapper.validate_code(test_code, InferenceMode::Regular).await {
                Ok(validation) => {
                    info!("✅ Code validation successful:");
                    info!("   Is valid: {}", validation.is_valid);
                    info!("   Is safe: {}", validation.is_safe);
                    info!("   Errors: {:?}", validation.errors);
                }
                Err(e) => error!("❌ Code validation failed: {}", e),
            }

            // Test 7: Get active ports
            info!("📋 Test 7: Getting active ports");
            let active_ports = wrapper.get_active_ports().await;
            info!("✅ Active ports: {:?}", active_ports);

            // Test 8: Stop the server
            info!("🛑 Test 8: Stopping server");
            match wrapper.stop_server(port).await {
                Ok(()) => info!("✅ Server stopped successfully"),
                Err(e) => error!("❌ Failed to stop server: {}", e),
            }

            // Verify server is stopped
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            if !wrapper.is_server_running(port).await {
                info!("✅ Server successfully stopped");
            } else {
                error!("❌ Server is still running after stop command");
            }
        }
        Err(e) => {
            error!("❌ Failed to start server: {}", e);
            return Ok(());
        }
    }

    info!("🎉 All tests completed!");
    Ok(())
}