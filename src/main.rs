use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;

mod cli;
mod compiler;
mod vm;
mod lsp;
mod llm;
mod daemon;

#[derive(Parser)]
#[command(name = "droe")]
#[command(about = "Droelang - Unified CLI, Compiler, VM, LLM, and LSP")]
#[command(version = "1.0.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize new Droelang project
    Init {
        /// Project name
        name: String,
        /// Project template
        #[arg(short, long, default_value = "basic")]
        template: String,
    },
    /// Compile a .droe file to specified target
    Compile {
        /// Input file
        input: PathBuf,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Target platform
        #[arg(short, long, default_value = "bytecode")]
        target: String,
        /// Optimization level
        #[arg(long, default_value = "2")]
        opt_level: u8,
    },
    /// Compile and run .droe file
    Run {
        /// Input file
        input: PathBuf,
        /// Arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Lint and validate .droe file
    Lint {
        /// Input file or directory
        path: PathBuf,
        /// Fix issues automatically
        #[arg(long)]
        fix: bool,
    },
    /// Format a .droe file
    Format {
        /// Input file or directory
        path: PathBuf,
        /// Check formatting without modifying files
        #[arg(long)]
        check: bool,
    },
    /// Install package dependencies
    Install {
        /// Package name
        package: Option<String>,
        /// Install globally
        #[arg(short, long)]
        global: bool,
    },
    /// Build entire project
    Build {
        /// Target platform
        #[arg(short, long, default_value = "bytecode")]
        target: String,
        /// Release mode
        #[arg(long)]
        release: bool,
    },
    /// Clean build directory
    Clean {
        /// Clean all caches
        #[arg(long)]
        all: bool,
    },
    /// Convert Puck JSON back to DSL
    Reverse {
        /// Input JSON file
        input: PathBuf,
        /// Output .droe file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate framework-specific code
    Generate {
        /// Framework type
        framework: String,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Run as Language Server Protocol server
    Lsp {
        /// LSP mode (stdio, tcp)
        #[arg(long, default_value = "stdio")]
        mode: String,
        /// TCP port (when mode=tcp)
        #[arg(long, default_value = "9257")]
        port: u16,
    },
    /// Run in daemon mode for language server integration
    Daemon {
        /// Daemon port
        #[arg(short, long, default_value = "9258")]
        port: u16,
        /// Enable LLM integration
        #[arg(long)]
        llm: bool,
        /// LLM provider (ollama, anthropic, openai)
        #[arg(long, default_value = "ollama")]
        llm_provider: String,
    },
    /// Interactive LLM chat mode
    Chat {
        /// LLM provider
        #[arg(short, long, default_value = "ollama")]
        provider: String,
        /// Model name
        #[arg(short, long)]
        model: Option<String>,
    },
    /// Start gRPC LLM service for VSCode extension
    LlmServer {
        /// Server port
        #[arg(short, long, default_value = "50051")]
        port: u16,
        /// LLM provider
        #[arg(long, default_value = "ollama")]
        provider: String,
    },
    /// VM operations
    Vm {
        #[command(subcommand)]
        command: VmCommands,
    },
    /// Update WebAssembly runtime with latest core library functions
    UpdateRuntime {
        /// Force update
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum VmCommands {
    /// Run WebAssembly module
    Run {
        /// WASM file path
        file: PathBuf,
        /// Function to call
        #[arg(short, long, default_value = "main")]
        function: String,
        /// Arguments
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Validate WebAssembly module
    Validate {
        /// WASM file path
        file: PathBuf,
    },
    /// Show module information
    Info {
        /// WASM file path
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, template } => {
            cli::init_project(&name, &template).await
        }
        Commands::Compile { input, output, target, opt_level } => {
            compiler::compile_file(&input, output.as_ref(), &target, opt_level).await
        }
        Commands::Run { input, args } => {
            cli::run_file(&input, &args).await
        }
        Commands::Lint { path, fix } => {
            cli::lint_path(&path, fix).await
        }
        Commands::Format { path, check } => {
            cli::format_path(&path, check).await
        }
        Commands::Install { package, global } => {
            cli::install_package(package.as_ref(), global).await
        }
        Commands::Build { target, release } => {
            cli::build_project(&target, release).await
        }
        Commands::Clean { all } => {
            cli::clean_project(all).await
        }
        Commands::Reverse { input, output } => {
            compiler::reverse_compile(&input, output.as_ref()).await
        }
        Commands::Generate { framework, output } => {
            compiler::generate_framework(&framework, output.as_ref()).await
        }
        Commands::Lsp { mode, port } => {
            lsp::start_server(&mode, port).await
        }
        Commands::Daemon { port, llm, llm_provider } => {
            daemon::start_daemon(port, llm, &llm_provider).await
        }
        Commands::Chat { provider, model } => {
            llm::start_chat(&provider, model.as_ref()).await
        }
        Commands::LlmServer { port, provider } => {
            llm::start_grpc_server(port, &provider).await
        }
        Commands::Vm { command } => {
            match command {
                VmCommands::Run { file, function, args } => {
                    vm::run_wasm(&file, &function, &args).await
                }
                VmCommands::Validate { file } => {
                    vm::validate_wasm(&file).await
                }
                VmCommands::Info { file } => {
                    vm::show_info(&file).await
                }
            }
        }
        Commands::UpdateRuntime { force } => {
            cli::update_runtime(force).await
        }
    }
}