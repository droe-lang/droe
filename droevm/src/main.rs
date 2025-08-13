mod bytecode;
mod vm;
mod embed;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "droevm")]
#[command(about = "Droe Virtual Machine - Execute Droe bytecode", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a bytecode file
    Run {
        /// The bytecode file to execute
        file: PathBuf,
        
        /// Enable debug output
        #[arg(short, long)]
        debug: bool,
    },
    
    /// Create a standalone executable
    Build {
        /// The bytecode file to embed
        bytecode: PathBuf,
        
        /// Output executable path
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Show version information
    Version,
}

fn main() -> Result<()> {
    // Check if this is a standalone executable with embedded bytecode
    match embed::extract_embedded_bytecode() {
        Ok(Some(embedded_bytecode)) => {
            return run_embedded_bytecode(embedded_bytecode);
        }
        Ok(None) => {
            // No embedded bytecode, continue with normal CLI
        }
        Err(_) => {
            // Error reading embedded data, continue with normal CLI
        }
    }
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { file, debug } => {
            run_bytecode(file, debug)?;
        }
        Commands::Build { bytecode, output } => {
            build_standalone(bytecode, output)?;
        }
        Commands::Version => {
            println!("Droe VM version {}", env!("CARGO_PKG_VERSION"));
        }
    }
    
    Ok(())
}

fn run_bytecode(path: PathBuf, debug: bool) -> Result<()> {
    let bytes = fs::read(&path)?;
    let bytecode = bytecode::BytecodeFile::deserialize(&bytes)?;
    
    if debug {
        println!("Loaded bytecode from: {:?}", path);
        println!("Instructions: {}", bytecode.instructions.len());
    }
    
    let mut vm = vm::VM::new(bytecode);
    vm.run()?;
    
    Ok(())
}

fn run_embedded_bytecode(bytecode_data: Vec<u8>) -> Result<()> {
    let bytecode = bytecode::BytecodeFile::deserialize(&bytecode_data)?;
    let mut vm = vm::VM::new(bytecode);
    vm.run()?;
    Ok(())
}

fn build_standalone(bytecode_path: PathBuf, output_path: PathBuf) -> Result<()> {
    println!("Building standalone executable...");
    println!("Bytecode: {:?}", bytecode_path);
    println!("Output: {:?}", output_path);
    
    // Get the current VM binary path
    let vm_binary = std::env::current_exe()?;
    
    // Embed the bytecode into the VM binary
    embed::embed_bytecode_in_binary(&vm_binary, &bytecode_path, &output_path)?;
    
    println!("✅ Standalone executable created at: {:?}", output_path);
    println!("📦 Contains embedded bytecode from: {:?}", bytecode_path);
    println!("🚀 Run with: {:?}", output_path);
    
    Ok(())
}
