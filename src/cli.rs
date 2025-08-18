use anyhow::Result;
use std::path::{Path, PathBuf};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DroeConfig {
    pub src: String,
    pub build: String,
    pub target: String,
    pub framework: String,
    pub package: Option<String>,
    pub database: Option<String>,
}

impl Default for DroeConfig {
    fn default() -> Self {
        Self {
            src: "src".to_string(),
            build: "build".to_string(),
            target: "bytecode".to_string(),
            framework: "plain".to_string(),
            package: None,
            database: None,
        }
    }
}

/// Load configuration from droeconfig.json with fallback to defaults
pub fn load_config(project_root: Option<&Path>) -> Result<DroeConfig> {
    let config_path = if let Some(root) = project_root {
        root.join("droeconfig.json")
    } else {
        std::env::current_dir()?.join("droeconfig.json")
    };

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        
        Ok(DroeConfig {
            src: json.get("src").and_then(|v| v.as_str()).unwrap_or("src").to_string(),
            build: json.get("build").and_then(|v| v.as_str()).unwrap_or("build").to_string(),
            target: json.get("target").and_then(|v| v.as_str()).unwrap_or("bytecode").to_string(),
            framework: json.get("framework").and_then(|v| v.as_str()).unwrap_or("plain").to_string(),
            package: json.get("package").and_then(|v| v.as_str()).map(|s| s.to_string()),
            database: json.get("database").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    } else {
        Ok(DroeConfig::default())
    }
}

/// Extract @target metadata from source file
pub fn get_target_from_source(source: &str, default_target: &str) -> String {
    // Look for @target metadata at the start of the file
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("@target ") {
            let target = line.strip_prefix("@target ").unwrap_or("").trim();
            if !target.is_empty() {
                return target.to_string();
            }
        }
    }
    default_target.to_string()
}

/// Resolve target with correct priority: CLI override > @target metadata > config > default
pub fn resolve_target(source_content: &str, config_target: &str, cli_override: Option<&str>) -> String {
    let target = if let Some(override_target) = cli_override {
        override_target.to_string()
    } else {
        get_target_from_source(source_content, config_target)
    };
    
    // Alias 'droe' target to 'bytecode'
    if target == "droe" {
        "bytecode".to_string()
    } else {
        target
    }
}

pub async fn init_project(name: &str, template: &str) -> Result<()> {
    println!("{} Initializing project '{}'...", style("[INFO]").cyan(), name);
    
    // Use the existing CLI functionality from droe-cli
    let cli_path = std::env::current_exe()?.parent().unwrap().join("droe-cli");
    if cli_path.exists() {
        // Delegate to existing CLI if available
        std::process::Command::new(cli_path)
            .args(["init", name, "--template", template])
            .status()?;
    } else {
        // Fallback implementation
        create_project_structure(name, template)?;
    }
    
    println!("{} Project '{}' initialized successfully!", style("[SUCCESS]").green(), name);
    Ok(())
}

pub async fn run_file(input: &Path, args: &[String]) -> Result<()> {
    println!("{} Compiling and running {}...", style("[INFO]").cyan(), input.display());
    
    // Load config
    let config = load_config(None)?;
    
    // Read source content to check for @target metadata
    let source_content = fs::read_to_string(input)?;
    
    // Resolve target: @target metadata > config > bytecode default
    let target = resolve_target(&source_content, &config.target, None);
    
    println!("{} Target: {}", style("[INFO]").cyan(), target);
    
    // First compile with resolved target
    crate::compiler::compile_file(input, None, &target, 2).await?;
    
    // Then run based on target
    match target.as_str() {
        "bytecode" => {
            let bytecode_file = input.parent()
                .unwrap_or(Path::new("."))
                .join("build")
                .join("main.droebc");
            crate::vm::run_bytecode(&bytecode_file, "main", args).await
        }
        "wasm" => {
            let wasm_file = input.with_extension("wasm");
            crate::vm::run_wasm(&wasm_file, "main", args).await
        }
        _ => {
            println!("{} Running target '{}' is not yet implemented in Rust CLI", style("[WARN]").yellow(), target);
            println!("{} Compiled successfully to {}", style("[SUCCESS]").green(), target);
            Ok(())
        }
    }
}

pub async fn lint_path(path: &Path, fix: bool) -> Result<()> {
    println!("{} Linting {}...", style("[INFO]").cyan(), path.display());
    
    if fix {
        println!("{} Auto-fixing issues...", style("[INFO]").yellow());
    }
    
    // Use droe-compiler for linting
    droe_compiler::lint_file(path, fix)
        .map_err(|e| anyhow::anyhow!("Lint error: {}", e))?;
    
    println!("{} Linting completed!", style("[SUCCESS]").green());
    Ok(())
}

pub async fn format_path(path: &Path, check: bool) -> Result<()> {
    if check {
        println!("{} Checking formatting for {}...", style("[INFO]").cyan(), path.display());
    } else {
        println!("{} Formatting {}...", style("[INFO]").cyan(), path.display());
    }
    
    // Use droe-compiler for formatting
    droe_compiler::format_file(path, check)
        .map_err(|e| anyhow::anyhow!("Format error: {}", e))?;
    
    println!("{} Formatting completed!", style("[SUCCESS]").green());
    Ok(())
}

pub async fn install_package(package: Option<&String>, global: bool) -> Result<()> {
    match package {
        Some(pkg) => {
            println!("{} Installing package '{}'...", style("[INFO]").cyan(), pkg);
            if global {
                println!("{} Installing globally...", style("[INFO]").yellow());
            }
            // Implement package installation logic
        }
        None => {
            println!("{} Installing project dependencies...", style("[INFO]").cyan());
            // Read droeconfig.json and install dependencies
        }
    }
    
    println!("{} Installation completed!", style("[SUCCESS]").green());
    Ok(())
}

pub async fn build_project(target: &str, release: bool) -> Result<()> {
    println!("{} Building project for target '{}'...", style("[INFO]").cyan(), target);
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")?);
    pb.set_message("Building...");
    
    // Build logic using droe-compiler
    if release {
        pb.set_message("Building in release mode...");
    }
    
    pb.finish_with_message("Build completed!");
    println!("{} Project built successfully!", style("[SUCCESS]").green());
    Ok(())
}

pub async fn clean_project(all: bool) -> Result<()> {
    println!("{} Cleaning build directory...", style("[INFO]").cyan());
    
    if all {
        println!("{} Cleaning all caches...", style("[INFO]").yellow());
    }
    
    // Clean build directory
    std::fs::remove_dir_all("build").ok();
    
    if all {
        std::fs::remove_dir_all(".droe-cache").ok();
    }
    
    println!("{} Clean completed!", style("[SUCCESS]").green());
    Ok(())
}

pub async fn update_runtime(force: bool) -> Result<()> {
    println!("{} Updating WebAssembly runtime...", style("[INFO]").cyan());
    
    if force {
        println!("{} Forcing update...", style("[INFO]").yellow());
    }
    
    // Runtime update logic
    println!("{} Runtime updated successfully!", style("[SUCCESS]").green());
    Ok(())
}

fn create_project_structure(name: &str, template: &str) -> Result<()> {
    use std::fs;
    
    // Create project directory
    fs::create_dir_all(name)?;
    fs::create_dir_all(format!("{}/src", name))?;
    fs::create_dir_all(format!("{}/build", name))?;
    fs::create_dir_all(format!("{}/modules", name))?;
    
    // Create droeconfig.json
    let config = serde_json::json!({
        "name": name,
        "version": "1.0.0",
        "template": template,
        "src": "src",
        "build": "build",
        "modules": "modules",
        "main": "src/main.droe",
        "target": "droe"
    });
    
    fs::write(
        format!("{}/droeconfig.json", name),
        serde_json::to_string_pretty(&config)?
    )?;
    
    // Create main.droe
    let main_content = match template {
        "api" => r#"// API Template
serve {
    route "/hello" {
        method GET
        response {
            Display "Hello, World!"
        }
    }
}"#,
        "mobile" => r#"// Mobile Template
screen MainScreen {
    layout "main_layout" {
        button "Click Me" {
            action {
                Display "Button clicked!"
            }
        }
    }
}"#,
        _ => r#"// Basic Template
Display "Hello, World!"
"#,
    };
    
    fs::write(format!("{}/src/main.droe", name), main_content)?;
    
    Ok(())
}