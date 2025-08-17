use clap::{Parser, Subcommand};
use droe_llm::{LLMService, LLMConfig, Result, LLMServiceImpl};
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

// Include the generated protobuf code
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
    /// Start the LLM service
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