use clap::{Parser, Subcommand};
use droe_llm::{LLMService, LLMConfig, Result, JsonRpcServer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[cfg(feature = "grpc")]
use droe_llm::LLMServiceImpl;
#[cfg(feature = "grpc")]
use std::net::SocketAddr;
#[cfg(feature = "grpc")]
use tonic::transport::Server;

// Include the generated protobuf code only when gRPC feature is enabled
#[cfg(feature = "grpc")]
pub mod llm_service {
    tonic::include_proto!("droevm.llm");
}


#[derive(Parser)]
#[command(name = "droe-llm")]
#[command(about = "DroeLLM - Centralized LLM service for DROELANG")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the LLM service with gRPC
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "50051")]
        port: u16,
        
        /// Configuration file path
        #[arg(short, long)]
        config: Option<String>,
        
        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,
    },
    /// Start the LLM service with JSON-RPC
    ServeJsonRpc {
        /// Port to listen on
        #[arg(short, long, default_value = "9080")]
        port: u16,
        
        /// Configuration file path
        #[arg(short, long)]
        config: Option<String>,
        
        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,
    },
    /// Health check
    Health {
        /// Service URL
        #[arg(short, long, default_value = "http://localhost:50051")]
        url: String,
    },
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        #[cfg(feature = "grpc")]
        Commands::Serve { port, config, debug } => {
            // Initialize logging
            let _log_level = if debug { "debug" } else { "info" };
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
                )
                .init();

            info!("🚀 Starting DroeLLM service...");

            // Load configuration
            let llm_config = if let Some(config_path) = config {
                info!("📖 Loading configuration from: {}", config_path);
                LLMConfig::load_from_file(&config_path)
                    .map_err(|e| droe_llm::LLMError::ConfigError(format!("Failed to load config: {}", e)))?
            } else {
                info!("📖 Using default configuration");
                LLMConfig::default()
            };

            // Create LLM service
            let llm_service = LLMService::new(llm_config);
            
            // Create gRPC service implementation
            let grpc_service = LLMServiceImpl::new(llm_service);
            
            // Parse address
            let addr: SocketAddr = format!("0.0.0.0:{}", port)
                .parse()
                .map_err(|e| droe_llm::LLMError::ConfigError(format!("Invalid port: {}", e)))?;
            
            info!("🎯 DroeLLM Service ready!");
            info!("   • Address: {}", addr);
            info!("   • Regular mode: Development DSL generation");
            info!("   • Robotics mode: Real-time command generation");
            info!("   • Streaming: Real-time token generation");
            info!("   • Validation: Safety checks and boundaries");
            info!("   • Partial updates: Efficient file modifications");
            
            // Start the server
            Server::builder()
                .add_service(grpc_service.into_service())
                .serve(addr)
                .await
                .map_err(|e| droe_llm::LLMError::GrpcError(tonic::Status::internal(format!("Transport error: {}", e))))?;
        },
        #[cfg(not(feature = "grpc"))]
        Commands::Serve { .. } => {
            eprintln!("gRPC support is disabled. Use 'serve-jsonrpc' instead or enable the 'grpc' feature.");
            std::process::exit(1);
        },
        Commands::ServeJsonRpc { port, config, debug } => {
            // Initialize logging
            let _log_level = if debug { "debug" } else { "info" };
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
                )
                .init();

            info!("🚀 Starting DroeLLM JSON-RPC service...");

            // Load configuration
            let llm_config = if let Some(config_path) = config {
                info!("📖 Loading configuration from: {}", config_path);
                LLMConfig::load_from_file(&config_path)
                    .map_err(|e| droe_llm::LLMError::ConfigError(format!("Failed to load config: {}", e)))?
            } else {
                info!("📖 Using default configuration");
                LLMConfig::default()
            };

            // Create LLM service
            let llm_service = LLMService::new(llm_config);
            
            // Create JSON-RPC server
            let jsonrpc_server = JsonRpcServer::new(llm_service, port);
            
            info!("🎯 DroeLLM JSON-RPC Service ready!");
            info!("   • Address: 0.0.0.0:{}", port);
            info!("   • Protocol: JSON-RPC over TCP");
            info!("   • Regular mode: Development DSL generation");
            info!("   • Robotics mode: Real-time command generation");
            info!("   • Streaming: Real-time token generation");
            info!("   • Validation: Safety checks and boundaries");
            info!("   • Partial updates: Efficient file modifications");
            
            // Start the JSON-RPC server
            jsonrpc_server.start().await
                .map_err(|e| droe_llm::LLMError::ConfigError(format!("JSON-RPC server error: {}", e)))?;
        },
        Commands::Health { url } => {
            info!("🔍 Checking health of DroeLLM service at {}", url);
            // TODO: Implement health check client
            println!("Health check not yet implemented");
        },
        Commands::Version => {
            println!("DroeLLM v{}", env!("CARGO_PKG_VERSION"));
            println!("Centralized LLM service for DROELANG");
            println!("Build date: 2024-01-01");
        },
    }

    Ok(())
}