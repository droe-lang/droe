//! Interactive Initialization - Guided project setup with prompts
//! 
//! This module provides functionality for:
//! - Interactive guided project initialization
//! - Database configuration setup
//! - Framework selection with validation
//! - User-friendly prompts and validation

use anyhow::Result;
use dialoguer::{Input, Select, Confirm, MultiSelect};
use console::{style, Term};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub async fn guided_init() -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("{}", style("🚀 Welcome to Droelang Project Setup!").bold().cyan());
    println!("{}", style("Let's create your new project step by step.\n").dim());

    // Project name
    let project_name: Option<String> = loop {
        let name: String = Input::new()
            .with_prompt("📝 Project name (or press Enter for current directory)")
            .allow_empty(true)
            .interact()?;
        
        if name.is_empty() {
            break None;
        }
        
        if name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
            break Some(name);
        } else {
            println!("{}", style("❌ Project name should only contain letters, numbers, hyphens, and underscores").red());
        }
    };

    // Author
    let author: Option<String> = Input::new()
        .with_prompt("👤 Author name (optional)")
        .allow_empty(true)
        .interact()
        .map(|s: String| if s.is_empty() { None } else { Some(s) })?;

    // Target selection
    println!("\n{}", style("🎯 Choose your target platform:").bold());
    let targets = vec![
        ("droe", "Droe VM (Fast bytecode execution)"),
        ("html", "Web Applications (HTML/CSS/JS)"),
        ("mobile", "Mobile Apps (Android/iOS)"),
        ("java", "Java Applications"),
        ("python", "Python Applications"),
        ("go", "Go Applications"),
        ("node", "Node.js Applications"),
        ("rust", "Rust Applications"),
        ("wasm", "WebAssembly (Generic)"),
    ];

    let target_selection = Select::new()
        .with_prompt("Select target")
        .items(&targets.iter().map(|(key, desc)| format!("{:<8} - {}", key, desc)).collect::<Vec<_>>())
        .default(0)
        .interact()?;

    let target = targets[target_selection].0.to_string();

    // Framework selection
    let framework = if let Some(framework_options) = get_framework_options(&target) {
        println!("\n{}", style(format!("🛠️  Choose framework for {}:", target)).bold());
        
        let mut frameworks = vec![("plain", "No Framework")];
        frameworks.extend(framework_options);
        
        let framework_selection = Select::new()
            .with_prompt("Select framework")
            .items(&frameworks.iter().map(|(key, desc)| format!("{:<10} - {}", key, desc)).collect::<Vec<_>>())
            .default(0)
            .interact()?;
        
        Some(frameworks[framework_selection].0.to_string())
    } else {
        None
    };

    // Database configuration
    let (database_config, database_details) = if should_ask_database(&target, &framework) {
        configure_database(&project_name)?
    } else {
        (None, None)
    };

    // Show summary
    display_summary(&project_name, &author, &target, &framework, &database_config, &database_details)?;

    // Confirm creation
    let should_create = Confirm::new()
        .with_prompt("✅ Create project?")
        .default(true)
        .interact()?;

    if should_create {
        create_project_with_config(
            project_name,
            author,
            target,
            framework,
            database_config,
            database_details,
        ).await?;
    } else {
        println!("{}", style("❌ Project creation cancelled").red());
    }

    Ok(())
}

fn get_framework_options(target: &str) -> Option<Vec<(&'static str, &'static str)>> {
    match target {
        "java" => Some(vec![("spring", "Spring Boot Framework")]),
        "python" => Some(vec![("fastapi", "FastAPI Framework")]),
        "go" => Some(vec![("fiber", "Fiber Framework")]),
        "node" => Some(vec![("fastify", "Fastify Framework")]),
        "rust" => Some(vec![("axum", "Axum Framework")]),
        "droe" => Some(vec![("axum", "Axum Framework")]),
        _ => None,
    }
}

fn should_ask_database(target: &str, framework: &Option<String>) -> bool {
    let needs_db_targets = ["html", "mobile", "java", "python", "go", "node", "rust", "droe"];
    let has_framework = framework.as_ref().map(|f| f != "plain").unwrap_or(false);
    
    needs_db_targets.contains(&target) && has_framework
}

fn configure_database(project_name: &Option<String>) -> Result<(Option<String>, Option<DatabaseDetails>)> {
    println!("\n{}", style("🗄️  Database Configuration:").bold());
    
    let needs_db = Confirm::new()
        .with_prompt("Do you need database support?")
        .default(false)
        .interact()?;

    if !needs_db {
        return Ok((None, None));
    }

    let db_options = vec![
        ("postgresql", "PostgreSQL"),
        ("mysql", "MySQL/MariaDB"),
        ("sqlite", "SQLite (local file)"),
        ("mongodb", "MongoDB"),
    ];

    let db_selection = Select::new()
        .with_prompt("Select database")
        .items(&db_options.iter().map(|(key, desc)| format!("{:<12} - {}", key, desc)).collect::<Vec<_>>())
        .default(0)
        .interact()?;

    let database_type = db_options[db_selection].0.to_string();

    // Get connection details for non-SQLite databases
    let database_details = if database_type != "sqlite" {
        println!("\n{}", style(format!("📡 {} Connection Details:", database_type.to_uppercase())).bold());
        
        let host: String = Input::new()
            .with_prompt("Host")
            .default("localhost".to_string())
            .interact()?;

        let default_ports = [
            ("postgresql", "5432"),
            ("mysql", "3306"),
            ("mongodb", "27017"),
        ].iter().cloned().collect::<HashMap<_, _>>();

        let default_port = default_ports.get(database_type.as_str()).unwrap_or(&"5432");
        let port: String = Input::new()
            .with_prompt("Port")
            .default(default_port.to_string())
            .interact()?;

        let default_db_name = if let Some(name) = project_name {
            format!("my{}", name.replace("-", "_"))
        } else {
            "myapp".to_string()
        };

        let database_name: String = Input::new()
            .with_prompt("Database name")
            .default(default_db_name)
            .interact()?;

        let username: String = Input::new()
            .with_prompt("Username")
            .default("postgres".to_string())
            .interact()?;

        let password: String = Input::new()
            .with_prompt("Password (optional)")
            .allow_empty(true)
            .interact()?;

        Some(DatabaseDetails {
            host,
            port: port.parse().unwrap_or(5432),
            database_name,
            username,
            password: if password.is_empty() { None } else { Some(password) },
        })
    } else {
        None
    };

    Ok((Some(database_type), database_details))
}

#[derive(Debug)]
struct DatabaseDetails {
    host: String,
    port: u16,
    database_name: String,
    username: String,
    password: Option<String>,
}

fn display_summary(
    project_name: &Option<String>,
    author: &Option<String>,
    target: &str,
    framework: &Option<String>,
    database_config: &Option<String>,
    database_details: &Option<DatabaseDetails>,
) -> Result<()> {
    println!("\n{}", style("📋 Project Summary:").bold().cyan());
    
    println!("   📁 Name: {}", 
             style(project_name.as_deref().unwrap_or("current directory")).green());
    
    if let Some(author_name) = author {
        println!("   👤 Author: {}", style(author_name).green());
    }
    
    println!("   🎯 Target: {}", style(target).green());
    
    if let Some(fw) = framework {
        if fw != "plain" {
            println!("   🛠️  Framework: {}", style(fw).green());
        }
    }
    
    if let Some(db) = database_config {
        println!("   🗄️  Database: {}", style(db).green());
        
        if let Some(details) = database_details {
            if db != "sqlite" {
                println!("   📡 DB Host: {}:{}", 
                         style(&details.host).dim(), 
                         style(details.port).dim());
                println!("   🏷️  DB Name: {}", style(&details.database_name).dim());
                println!("   👤 DB User: {}", style(&details.username).dim());
            }
        }
    }

    Ok(())
}

async fn create_project_with_config(
    project_name: Option<String>,
    author: Option<String>,
    target: String,
    framework: Option<String>,
    database_config: Option<String>,
    database_details: Option<DatabaseDetails>,
) -> Result<()> {
    // Determine project root
    let project_root = if let Some(name) = &project_name {
        let path = PathBuf::from(name);
        if path.exists() {
            if path.is_dir() && path.read_dir()?.next().is_some() {
                anyhow::bail!("Directory '{}' already exists and is not empty", name);
            } else if path.is_file() {
                anyhow::bail!("A file named '{}' already exists", name);
            }
        }
        fs::create_dir_all(&path)?;
        println!("📁 Created project directory: {}/", name);
        path
    } else {
        let current = std::env::current_dir()?;
        if current.read_dir()?.next().is_some() {
            let config_path = current.join("droeconfig.json");
            if config_path.exists() {
                anyhow::bail!("This directory already contains a Droelang project");
            } else {
                anyhow::bail!("Directory is not empty. Use an empty directory or provide a project name");
            }
        }
        current
    };

    // Create directory structure
    let src_dir = project_root.join("src");
    let build_dir = project_root.join("build");
    let modules_dir = project_root.join("modules");
    
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&build_dir)?;
    fs::create_dir_all(&modules_dir)?;

    // Create configuration
    let mut config = json!({
        "src": "src",
        "build": "build",
        "dist": "dist",
        "modules": "modules",
        "main": "src/main.droe",
        "target": target
    });

    if let Some(author_name) = author {
        config["author"] = json!(author_name);
    }

    if let Some(fw) = &framework {
        if fw != "plain" {
            config["framework"] = json!(fw);
        }
    }

    // Add package name for Java Spring projects
    if target == "java" && framework.as_ref() == Some(&"spring".to_string()) {
        let package_name = if let Some(name) = &project_name {
            format!("com.example.{}", name.replace("-", "").replace("_", "").to_lowercase())
        } else {
            "com.example.app".to_string()
        };
        config["package"] = json!(package_name);
    }

    // Add database configuration
    if let Some(db_type) = database_config {
        config["database"] = json!(db_type);
        
        if let Some(details) = database_details {
            config["database_config"] = json!({
                "host": details.host,
                "port": details.port,
                "name": details.database_name,
                "user": details.username,
                "password": details.password
            });
        }
    }

    // Write configuration
    let config_path = project_root.join("droeconfig.json");
    fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
    println!("✅ Created droeconfig.json");

    // Create sample main.droe file
    let main_file = src_dir.join("main.droe");
    let actual_project_name = project_name.as_deref()
        .unwrap_or_else(|| project_root.file_name().and_then(|n| n.to_str()).unwrap_or("droe-project"));

    let main_content = create_sample_content(&target, &framework, &actual_project_name, &config);
    fs::write(&main_file, main_content)?;
    println!("✅ Created src/main.droe");

    // Success message
    println!("\n{}", style("✅ Initialized Droelang project successfully!").bold().green());
    println!("📁 Project name: {}", style(actual_project_name).cyan());
    
    let framework_targets = ["java", "python", "go", "node"];
    let show_framework = framework.as_ref()
        .filter(|fw| fw.as_str() != "plain")
        .filter(|_| framework_targets.contains(&target.as_str()));
    
    if let Some(fw) = show_framework {
        println!("🎯 Target: {} with {} framework", style(&target).cyan(), style(fw).cyan());
    } else {
        println!("🎯 Target: {}", style(&target).cyan());
    }

    println!("\n📂 Directory structure:");
    println!("  src/         - Source code (.droe files)");
    println!("  modules/     - Downloaded packages");
    println!("  build/       - Compiled files");
    println!("  dist/        - Distribution packages");
    
    println!("\n🚀 Next steps:");
    if project_name.is_some() {
        println!("  cd {}/              - Navigate to project directory", actual_project_name);
    }
    println!("  droe run src/main.droe     - Run the sample");
    println!("  droe build               - Build project");

    Ok(())
}

fn create_sample_content(
    target: &str,
    framework: &Option<String>,
    project_name: &str,
    config: &serde_json::Value,
) -> String {
    match target {
        "java" if framework.as_ref() == Some(&"spring".to_string()) => {
            format!(r#"@target java
@framework spring
@package {}

module IndexModule

action Index gives text
    set projectName which is text to "{}"
    give projectName
end action

end module

api IndexAPI
    endpoint GET /
        return from IndexModule.Index
    end
end
"#, config["package"].as_str().unwrap(), project_name)
        }
        "html" => {
            format!(r#"@target html

set projectName which is text to "{}"
display "Hello from [projectName]!"

# Example UI components
button "Click me!" onclick="alert('Hello!')"
input placeholder="Enter your name" type="text"
"#, project_name)
        }
        "mobile" => {
            format!(r#"@target mobile

set projectName which is text to "{}"
display "Hello from [projectName]!"

# Example mobile UI
screen "Home"
    text "Welcome to [projectName]"
    button "Get Started" onclick="navigate_to('main')"
end screen
"#, project_name)
        }
        "python" => {
            format!(r#"@target python

set projectName which is text to "{}"
display "Hello from [projectName]!"

# Example Python-specific features
action process_data with items which are list of text gives list of text
    set results which are list of text to new list of text
    for each item in items
        add item + " processed" to results
    end for
    give results
end action
"#, project_name)
        }
        "go" => {
            format!(r#"@target go

set projectName which is text to "{}"
display "Hello from [projectName]!"

# Example Go-specific features
action concurrent_task with data which is text gives text
    display "Processing: [data]"
    give data + " completed"
end action
"#, project_name)
        }
        "node" => {
            format!(r#"@target node

set projectName which is text to "{}"
display "Hello from [projectName]!"

# Example Node.js-specific features
action async_operation with url which is text gives text
    display "Fetching: [url]"
    give "Response from " + url
end action
"#, project_name)
        }
        _ => {
            format!(r#"set projectName which is text to "{}"
display "Hello from [projectName]!"

# Example basic functionality
action greet with name which is text gives text
    give "Hello, " + name + "!"
end action

set userName which is text to "World"
set greeting which is text to greet(userName)
display greeting
"#, project_name)
        }
    }
}