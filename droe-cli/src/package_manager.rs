//! Package Manager - Handles dependency installation and management
//! 
//! This module provides functionality for:
//! - Installing packages to global cache (~/.droelang/packages/)
//! - Managing dependencies in droeconfig.json
//! - Resolving packages from cache
//! - Version resolution similar to Maven/.m2

use anyhow::{Result, Context};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct PackageManager {
    project_root: PathBuf,
    global_cache_dir: PathBuf,  // ~/.droelang/packages/
    config_path: PathBuf,
}

impl PackageManager {
    pub fn new() -> Result<Self> {
        let project_root = crate::find_project_root()?;
        let config_path = project_root.join("droeconfig.json");
        
        // Use global cache directory like Maven's .m2
        let home = dirs::home_dir()
            .context("Unable to determine home directory")?;
        let global_cache_dir = home.join(".droelang").join("packages");
        
        // Ensure global cache directory exists
        fs::create_dir_all(&global_cache_dir)?;
        
        Ok(Self {
            project_root,
            global_cache_dir,
            config_path,
        })
    }

    pub async fn install_package(&mut self, package_name: &str, is_dev: bool) -> Result<()> {
        println!("📦 Installing {}...", package_name);
        
        // Load existing config
        let mut config = self.load_config()?;
        
        // Determine package version (in real world, this would come from registry)
        let version = "1.0.0";
        
        // Create versioned package path like Maven: ~/.droelang/packages/math-utils/1.0.0/
        let package_dir = self.global_cache_dir
            .join(package_name)
            .join(version);
        
        // Check if already in cache
        if !package_dir.exists() {
            println!("⬇️  Downloading {} v{} to global cache...", package_name, version);
            
            // Install to global cache based on package name
            match package_name {
            "math-utils" => {
                self.install_math_utils(&package_dir).await?;
            }
            "string-utils" => {
                self.install_string_utils(&package_dir).await?;
            }
            "http-client" => {
                self.install_http_client(&package_dir).await?;
            }
            "ui-components" => {
                self.install_ui_components(&package_dir).await?;
            }
            "database-orm" => {
                self.install_database_orm(&package_dir).await?;
            }
            _ => {
                anyhow::bail!("Package '{}' not found in registry. Available packages: math-utils, string-utils, http-client, ui-components, database-orm", package_name);
            }
            }
            
            println!("💾 Cached at: {}", package_dir.display());
        } else {
            println!("✅ Using cached version from: {}", package_dir.display());
        }
        
        // Update config with dependency (no local files created!)
        let dep_key = if is_dev { "devDependencies" } else { "dependencies" };
        if config.get(dep_key).is_none() {
            config[dep_key] = json!({});
        }
        
        config[dep_key][package_name] = json!(format!("^{}", version));
        
        // Save updated config
        self.save_config(&config)?;
        
        println!("✅ Added {} v{} to {}", package_name, version, dep_key);
        println!("💡 Import with: include \"@{}/{}.droe\"", package_name, package_name.replace('-', '_'));
        println!("📍 No local files created - using global cache at ~/.droelang/packages/");
        
        Ok(())
    }

    pub async fn install_all(&mut self) -> Result<()> {
        let config = self.load_config()?;
        
        let mut packages_to_install = Vec::new();
        
        // Collect dependencies
        if let Some(deps) = config.get("dependencies").and_then(|d| d.as_object()) {
            for (pkg_name, _version) in deps {
                packages_to_install.push((pkg_name.clone(), false));
            }
        }
        
        // Collect dev dependencies
        if let Some(dev_deps) = config.get("devDependencies").and_then(|d| d.as_object()) {
            for (pkg_name, _version) in dev_deps {
                packages_to_install.push((pkg_name.clone(), true));
            }
        }
        
        if packages_to_install.is_empty() {
            println!("ℹ️  No dependencies to install");
            return Ok(());
        }
        
        println!("📦 Installing {} dependencies...", packages_to_install.len());
        
        for (pkg_name, is_dev) in packages_to_install {
            self.install_package(&pkg_name, is_dev).await?;
        }
        
        println!("✅ All dependencies installed successfully");
        Ok(())
    }

    async fn install_math_utils(&self, package_dir: &Path) -> Result<()> {
        fs::create_dir_all(package_dir)?;
        
        // Create math.droe
        let math_droe = package_dir.join("math_utils.droe");
        let content = r#"# Math utilities package
module MathUtils
    action add with a which is number and b which is number gives number
        give a + b
    end action
    
    action multiply with x which is decimal and y which is decimal gives decimal
        give x * y
    end action
    
    action power with base which is number and exp which is number gives number
        set result which is number to 1
        set i which is number to 0
        while i < exp
            set result to result * base
            set i to i + 1
        end while
        give result
    end action
    
    action max with a which is number and b which is number gives number
        when a > b
            give a
        otherwise
            give b
        end when
    end action
    
    action min with a which is number and b which is number gives number
        when a < b
            give a
        otherwise
            give b
        end when
    end action
end module
"#;
        
        fs::write(&math_droe, content)?;
        
        // Create package.json
        let package_info = package_dir.join("package.json");
        let info = json!({
            "name": "math-utils",
            "version": "1.0.0",
            "description": "Mathematical utility functions for Droelang",
            "main": "math_utils.droe",
            "files": ["math_utils.droe"],
            "keywords": ["math", "utilities", "numbers"],
            "license": "MIT"
        });
        
        fs::write(&package_info, serde_json::to_string_pretty(&info)?)?;
        
        println!("✅ Downloaded math-utils to modules/math-utils");
        Ok(())
    }

    async fn install_string_utils(&self, package_dir: &Path) -> Result<()> {
        fs::create_dir_all(package_dir)?;
        
        // Create string_utils.droe
        let string_droe = package_dir.join("string_utils.droe");
        let content = r#"# String utilities package
module StringUtils
    action uppercase with text which is text gives text
        # Convert text to uppercase (placeholder implementation)
        give text + " (UPPER)"
    end action
    
    action lowercase with text which is text gives text
        # Convert text to lowercase (placeholder implementation)
        give text + " (lower)"
    end action
    
    action trim with text which is text gives text
        # Trim whitespace (placeholder implementation)
        give text
    end action
    
    action concat_with_separator with items which are list of text and sep which is text gives text
        # Join array with separator (placeholder implementation)
        when items has content
            give "Joined: " + items[0] + sep + items[1]
        otherwise
            give ""
        end when
    end action
    
    action contains with text which is text and search which is text gives boolean
        # Check if text contains search string (placeholder)
        give true
    end action
end module
"#;
        
        fs::write(&string_droe, content)?;
        
        // Create package.json
        let package_info = package_dir.join("package.json");
        let info = json!({
            "name": "string-utils",
            "version": "1.0.0",
            "description": "String manipulation utilities for Droelang",
            "main": "string_utils.droe",
            "files": ["string_utils.droe"],
            "keywords": ["string", "text", "utilities"],
            "license": "MIT"
        });
        
        fs::write(&package_info, serde_json::to_string_pretty(&info)?)?;
        
        println!("✅ Downloaded string-utils to modules/string-utils");
        Ok(())
    }

    async fn install_http_client(&self, package_dir: &Path) -> Result<()> {
        fs::create_dir_all(package_dir)?;
        
        // Create http_client.droe
        let http_droe = package_dir.join("http_client.droe");
        let content = r#"# HTTP client utilities package
module HttpClient
    action get with url which is text gives text
        # Make GET request (placeholder implementation)
        display "Making GET request to [url]"
        give "Response from " + url
    end action
    
    action post with url which is text and data which is text gives text
        # Make POST request (placeholder implementation)
        display "Making POST request to [url] with data: [data]"
        give "POST response from " + url
    end action
    
    action put with url which is text and data which is text gives text
        # Make PUT request (placeholder implementation)
        display "Making PUT request to [url] with data: [data]"
        give "PUT response from " + url
    end action
    
    action delete with url which is text gives text
        # Make DELETE request (placeholder implementation)
        display "Making DELETE request to [url]"
        give "DELETE response from " + url
    end action
end module
"#;
        
        fs::write(&http_droe, content)?;
        
        // Create package.json
        let package_info = package_dir.join("package.json");
        let info = json!({
            "name": "http-client",
            "version": "1.0.0",
            "description": "HTTP client utilities for Droelang",
            "main": "http_client.droe",
            "files": ["http_client.droe"],
            "keywords": ["http", "client", "api", "rest"],
            "license": "MIT"
        });
        
        fs::write(&package_info, serde_json::to_string_pretty(&info)?)?;
        
        println!("✅ Downloaded http-client to modules/http-client");
        Ok(())
    }

    async fn install_ui_components(&self, package_dir: &Path) -> Result<()> {
        fs::create_dir_all(package_dir)?;
        
        // Create ui_components.droe
        let ui_droe = package_dir.join("ui_components.droe");
        let content = r#"# UI components package
module UIComponents
    action button with text which is text and onclick which is text gives ui_component
        # Create button component
        set component which is ui_component to new ui_component
        set component.type to "button"
        set component.text to text
        set component.onclick to onclick
        give component
    end action
    
    action input with placeholder which is text and type which is text gives ui_component
        # Create input component
        set component which is ui_component to new ui_component
        set component.type to "input"
        set component.placeholder to placeholder
        set component.input_type to type
        give component
    end action
    
    action form with components which are list of ui_component gives ui_component
        # Create form component
        set form which is ui_component to new ui_component
        set form.type to "form"
        set form.children to components
        give form
    end action
    
    action render with component which is ui_component
        # Render UI component
        display "Rendering component of type: [component.type]"
    end action
end module

data ui_component
    type is text
    text is text
    placeholder is text
    input_type is text
    onclick is text
    children is list of ui_component
end data
"#;
        
        fs::write(&ui_droe, content)?;
        
        // Create package.json
        let package_info = package_dir.join("package.json");
        let info = json!({
            "name": "ui-components",
            "version": "1.0.0",
            "description": "UI component library for Droelang",
            "main": "ui_components.droe",
            "files": ["ui_components.droe"],
            "keywords": ["ui", "components", "forms", "interface"],
            "license": "MIT"
        });
        
        fs::write(&package_info, serde_json::to_string_pretty(&info)?)?;
        
        println!("✅ Downloaded ui-components to modules/ui-components");
        Ok(())
    }

    async fn install_database_orm(&self, package_dir: &Path) -> Result<()> {
        fs::create_dir_all(package_dir)?;
        
        // Create database_orm.droe
        let db_droe = package_dir.join("database_orm.droe");
        let content = r#"# Database ORM package
module DatabaseORM
    action connect with connection_string which is text gives database_connection
        # Connect to database
        set connection which is database_connection to new database_connection
        set connection.url to connection_string
        set connection.connected to true
        display "Connected to database: [connection_string]"
        give connection
    end action
    
    action find with connection which is database_connection and table which is text and id which is number gives record
        # Find record by ID
        set record which is record to new record
        set record.id to id
        set record.table to table
        display "Finding record with ID [id] in table [table]"
        give record
    end action
    
    action save with connection which is database_connection and record which is record gives boolean
        # Save record to database
        display "Saving record to table [record.table]"
        give true
    end action
    
    action delete with connection which is database_connection and table which is text and id which is number gives boolean
        # Delete record by ID
        display "Deleting record with ID [id] from table [table]"
        give true
    end action
    
    action query with connection which is database_connection and sql which is text gives list of record
        # Execute SQL query
        display "Executing query: [sql]"
        set results which are list of record to new list of record
        give results
    end action
end module

data database_connection
    url is text
    connected is boolean
end data

data record
    id is number
    table is text
    data is text
end data
"#;
        
        fs::write(&db_droe, content)?;
        
        // Create package.json
        let package_info = package_dir.join("package.json");
        let info = json!({
            "name": "database-orm",
            "version": "1.0.0",
            "description": "Database ORM utilities for Droelang",
            "main": "database_orm.droe",
            "files": ["database_orm.droe"],
            "keywords": ["database", "orm", "sql", "data"],
            "license": "MIT"
        });
        
        fs::write(&package_info, serde_json::to_string_pretty(&info)?)?;
        
        println!("✅ Downloaded database-orm to modules/database-orm");
        Ok(())
    }

    fn load_config(&self) -> Result<Value> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(json!({
                "src": "src",
                "build": "build",
                "target": "droe",
                "framework": "plain"
            }))
        }
    }

    fn save_config(&self, config: &Value) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
}

pub async fn list_available_packages() -> Vec<(&'static str, &'static str)> {
    vec![
        ("math-utils", "Mathematical utility functions"),
        ("string-utils", "String manipulation utilities"),
        ("http-client", "HTTP client for API requests"),
        ("ui-components", "UI component library"),
        ("database-orm", "Database ORM utilities"),
    ]
}