//! Rust code generator for Rust target with Axum and database support
//! 
//! This module generates Rust code with Axum for HTTP server and configurable database support.

use crate::ast::{Node, Program, DataDefinition, ServeStatement, ModuleDefinition};
use crate::codegen::CodeGenerator;
use crate::codegen_base::CodeGenError;
use std::collections::HashMap;
use tera::{Tera, Context, Value};

/// Generates Rust code with Axum for HTTP server and configurable database support
pub struct RustGenerator {
    source_file_path: Option<String>,
    is_main_file: bool,
    framework: String,
    package: String,
    database: DatabaseConfig,
    
    // Track discovered features
    has_serve_endpoints: bool,
    has_database_ops: bool,
    data_structures: HashMap<String, DataDefinition>,
    serve_endpoints: Vec<ServeEndpoint>,
    modules: HashMap<String, ModuleDefinition>,
    
    // Template engine
    tera: Tera,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_type: String,
    pub url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: "postgres".to_string(),
            url: String::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServeEndpoint {
    pub method: String,
    pub endpoint: String,
    pub handler_name: String,
    pub params: Vec<String>,
    pub accept_type: Option<String>,
    pub response_action: Option<String>,
}

impl RustGenerator {
    pub fn new(
        source_file_path: Option<String>,
        is_main_file: bool,
        framework: Option<String>,
        package: Option<String>,
        database: Option<DatabaseConfig>,
    ) -> Result<Self, CodeGenError> {
        let package = package.unwrap_or_else(|| "roelang_app".to_string());
        let database = database.unwrap_or_default();
        
        // Setup Tera template engine - we'll embed the templates as strings
        let mut tera = Tera::new("templates/**/*").map_err(|e| {
            CodeGenError::GenerationFailed {
                message: format!("Failed to initialize Tera: {}", e)
            }
        })?;
        
        // Add custom filters
        tera.register_filter("snake_case", snake_case_filter);
        tera.register_filter("rust_type", rust_type_filter);
        
        Ok(Self {
            source_file_path,
            is_main_file,
            framework: framework.unwrap_or_else(|| "axum".to_string()),
            package,
            database,
            has_serve_endpoints: false,
            has_database_ops: false,
            data_structures: HashMap::new(),
            serve_endpoints: Vec::new(),
            modules: HashMap::new(),
            tera,
        })
    }
    
    /// Analyze the program to discover features and structures
    fn analyze_program(&mut self, program: &Program) -> Result<(), CodeGenError> {
        for stmt in &program.statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }
    
    /// Recursively analyze statements to discover features
    fn analyze_statement(&mut self, stmt: &Node) -> Result<(), CodeGenError> {
        match stmt {
            Node::DataDefinition(data_def) => {
                self.data_structures.insert(data_def.name.clone(), data_def.clone());
                self.has_database_ops = true; // Assume data structures will be persisted
            }
            Node::ServeStatement(serve_stmt) => {
                self.has_serve_endpoints = true;
                let endpoint = ServeEndpoint {
                    method: serve_stmt.method.clone(),
                    endpoint: serve_stmt.endpoint.clone(),
                    handler_name: self.get_handler_name(serve_stmt),
                    params: Vec::new(), // TODO: Extract from endpoint path
                    accept_type: None,   // TODO: Extract from serve body
                    response_action: None, // TODO: Extract from serve body
                };
                self.serve_endpoints.push(endpoint);
                
                // Check serve body for database operations
                for body_stmt in &serve_stmt.body {
                    self.analyze_statement(body_stmt)?;
                }
            }
            Node::DatabaseStatement(_) => {
                self.has_database_ops = true;
            }
            Node::ModuleDefinition(module_def) => {
                self.modules.insert(module_def.name.clone(), module_def.clone());
                // Recursively analyze module body
                for module_stmt in &module_def.body {
                    self.analyze_statement(module_stmt)?;
                }
            }
            Node::IncludeStatement(_) => {
                // Track included modules
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Generate a handler function name from endpoint
    fn get_handler_name(&self, endpoint: &ServeStatement) -> String {
        let method = endpoint.method.to_lowercase();
        let path_parts: Vec<&str> = endpoint.endpoint
            .trim_matches('/')
            .split('/')
            .filter(|p| !p.starts_with(':') && !p.is_empty())
            .collect();
        
        if path_parts.is_empty() {
            format!("{}_root", method)
        } else {
            format!("{}_{}", method, path_parts.join("_"))
        }
    }
    
    /// Prepare template context with all necessary data
    fn get_template_context(&self) -> Context {
        let mut context = Context::new();
        
        context.insert("package_name", &self.package.replace(['.', '-'], "_"));
        context.insert("has_serve_endpoints", &self.has_serve_endpoints);
        context.insert("has_database_ops", &self.has_database_ops);
        context.insert("data_structures", &self.data_structures);
        context.insert("serve_endpoints", &self.serve_endpoints);
        context.insert("modules", &self.modules);
        context.insert("db_type", &self.database.db_type);
        context.insert("default_db_url", &self.get_default_db_url());
        context.insert("has_uuid_fields", &self.has_uuid_fields());
        context.insert("has_datetime_fields", &self.has_datetime_fields());
        
        context
    }
    
    /// Check if any data structures have UUID fields
    fn has_uuid_fields(&self) -> bool {
        self.data_structures.values().any(|ds| self.has_auto_id(ds))
    }
    
    /// Check if any data structures have datetime fields
    fn has_datetime_fields(&self) -> bool {
        self.data_structures.values().any(|ds| self.has_datetime(ds))
    }
    
    /// Check if data definition has auto-generated ID field
    fn has_auto_id(&self, data_def: &DataDefinition) -> bool {
        data_def.fields.iter().any(|field| {
            field.annotations.contains(&"key".to_string()) && 
            field.annotations.contains(&"auto".to_string())
        })
    }
    
    /// Check if data definition has datetime fields
    fn has_datetime(&self, data_def: &DataDefinition) -> bool {
        data_def.fields.iter().any(|field| {
            matches!(field.field_type.as_str(), "date" | "datetime")
        })
    }
    
    /// Get default database URL based on database type
    fn get_default_db_url(&self) -> String {
        match self.database.db_type.as_str() {
            "postgres" => "postgresql://localhost/roelang_db".to_string(),
            "mysql" => "mysql://localhost/roelang_db".to_string(),
            "sqlite" => "sqlite:roelang.db".to_string(),
            _ => "postgresql://localhost/roelang_db".to_string(),
        }
    }
    
    /// Generate project files using templates
    fn generate_project_files(&self, context: &Context) -> Result<HashMap<String, String>, CodeGenError> {
        let mut files = HashMap::new();
        
        // Generate Cargo.toml
        files.insert(
            "Cargo.toml".to_string(),
            self.render_template("cargo_toml", context)?
        );
        
        // Generate main.rs with Axum server
        files.insert(
            "src/main.rs".to_string(),
            self.render_template("main_rs", context)?
        );
        
        // Generate models if data structures exist
        if !self.data_structures.is_empty() {
            files.insert(
                "src/models.rs".to_string(),
                self.render_template("models_rs", context)?
            );
        }
        
        // Generate handlers for serve endpoints
        if !self.serve_endpoints.is_empty() {
            files.insert(
                "src/handlers.rs".to_string(),
                self.render_template("handlers_rs", context)?
            );
        }
        
        // Generate database module if database operations exist
        if self.has_database_ops {
            files.insert(
                "src/db.rs".to_string(),
                self.render_template("db_rs", context)?
            );
        }
        
        // Generate lib.rs to tie modules together
        files.insert(
            "src/lib.rs".to_string(),
            self.generate_lib_rs()
        );
        
        Ok(files)
    }
    
    /// Render a template with context
    fn render_template(&self, template_name: &str, context: &Context) -> Result<String, CodeGenError> {
        // For now, we'll generate the templates inline
        // In a full implementation, these would be loaded from embedded template files
        match template_name {
            "cargo_toml" => self.generate_cargo_toml(context),
            "main_rs" => self.generate_main_rs(context),
            "models_rs" => self.generate_models_rs(context),
            "handlers_rs" => self.generate_handlers_rs(context),
            "db_rs" => self.generate_db_rs(context),
            _ => Err(CodeGenError::GenerationFailed {
                message: format!("Unknown template: {}", template_name)
            })
        }
    }
    
    /// Generate lib.rs to export modules
    fn generate_lib_rs(&self) -> String {
        let mut content = String::new();
        
        if !self.data_structures.is_empty() {
            content.push_str("pub mod models;\n");
        }
        if !self.serve_endpoints.is_empty() {
            content.push_str("pub mod handlers;\n");
        }
        if self.has_database_ops {
            content.push_str("pub mod db;\n");
        }
        
        content.push('\n');
        if !self.data_structures.is_empty() {
            content.push_str("pub use models::*;\n");
        }
        
        content
    }
}

impl CodeGenerator for RustGenerator {
    fn generate(&self, program: &Program) -> Result<String, String> {
        let mut generator = self.clone();
        
        // Analyze the program to understand what features are used
        generator.analyze_program(program).map_err(|e| e.to_string())?;
        
        // Prepare template context
        let context = generator.get_template_context();
        
        // Generate project structure
        let files = generator.generate_project_files(&context).map_err(|e| e.to_string())?;
        
        // Return a summary of generated files
        let file_list: Vec<String> = files.keys().cloned().collect();
        Ok(format!("Generated Rust project with files: {}", file_list.join(", ")))
    }
}

impl Clone for RustGenerator {
    fn clone(&self) -> Self {
        Self {
            source_file_path: self.source_file_path.clone(),
            is_main_file: self.is_main_file,
            framework: self.framework.clone(),
            package: self.package.clone(),
            database: self.database.clone(),
            has_serve_endpoints: self.has_serve_endpoints,
            has_database_ops: self.has_database_ops,
            data_structures: self.data_structures.clone(),
            serve_endpoints: self.serve_endpoints.clone(),
            modules: self.modules.clone(),
            tera: Tera::new("templates/**/*").unwrap_or_default(),
        }
    }
}

// Template generation methods (inline templates for now)
impl RustGenerator {
    fn generate_cargo_toml(&self, context: &Context) -> Result<String, CodeGenError> {
        let package_name = context.get("package_name").unwrap().as_str().unwrap();
        let has_database_ops = context.get("has_database_ops").unwrap().as_bool().unwrap();
        let db_type = context.get("db_type").unwrap().as_str().unwrap();
        
        let mut content = format!(
r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1.0", features = ["full"] }}
axum = "0.6"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tower = "0.4"
tower-http = {{ version = "0.4", features = ["cors"] }}
tracing = "0.1"
tracing-subscriber = "0.3"
"#, package_name);

        if has_database_ops {
            let sqlx_features = match db_type {
                "postgres" => r#"sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }"#,
                "mysql" => r#"sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "mysql", "uuid", "chrono"] }"#,
                "sqlite" => r#"sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "uuid", "chrono"] }"#,
                _ => r#"sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }"#,
            };
            content.push_str(&format!("{}\n", sqlx_features));
            content.push_str(r#"uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
"#);
        }
        
        Ok(content)
    }
    
    fn generate_main_rs(&self, context: &Context) -> Result<String, CodeGenError> {
        let has_database_ops = context.get("has_database_ops").unwrap().as_bool().unwrap();
        let default_db_url = context.get("default_db_url").unwrap().as_str().unwrap();
        let serve_endpoints = context.get("serve_endpoints").unwrap().as_array().unwrap();
        
        let mut content = String::from(
r#"use axum::{
    Router,
    routing::{get, post, put, delete},
    response::Json,
    extract::{Path, State},
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use std::sync::Arc;

"#);

        if !self.data_structures.is_empty() {
            content.push_str("mod models;\n");
        }
        if !self.serve_endpoints.is_empty() {
            content.push_str("mod handlers;\n");
        }
        if has_database_ops {
            content.push_str("mod db;\nuse db::Database;\n");
        }
        
        content.push_str(
r#"
#[derive(Clone)]
pub struct AppState {
"#);

        if has_database_ops {
            content.push_str("    db: Arc<Database>,\n");
        }

        content.push_str(
r#"}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
"#);

        if has_database_ops {
            content.push_str(&format!(
r#"    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "{}".to_string());
    let db = Arc::new(Database::new(&database_url).await.expect("Failed to connect to database"));
    
"#, default_db_url));
        }

        content.push_str("    // Build application state\n    let state = AppState {\n");
        if has_database_ops {
            content.push_str("        db,\n");
        }
        content.push_str("    };\n    \n    // Build router\n    let app = Router::new()\n");

        for endpoint in serve_endpoints {
            let endpoint_obj = endpoint.as_object().unwrap();
            let method = endpoint_obj.get("method").unwrap().as_str().unwrap().to_lowercase();
            let endpoint_path = endpoint_obj.get("endpoint").unwrap().as_str().unwrap();
            let handler_name = endpoint_obj.get("handler_name").unwrap().as_str().unwrap();
            
            content.push_str(&format!(
                r#"        .route("{}", {}(handlers::{}))
"#, endpoint_path, method, handler_name));
        }

        content.push_str(
r#"        .layer(CorsLayer::permissive())
        .with_state(state);
    
    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
"#);

        Ok(content)
    }
    
    fn generate_models_rs(&self, _context: &Context) -> Result<String, CodeGenError> {
        let has_database_ops = self.has_database_ops;
        let db_type = &self.database.db_type;
        let has_uuid_fields = self.has_uuid_fields();
        let has_datetime_fields = self.has_datetime_fields();
        
        let mut content = String::from("use serde::{Deserialize, Serialize};\n");
        
        if has_database_ops && matches!(db_type.as_str(), "postgres" | "mysql" | "sqlite") {
            content.push_str("use sqlx::FromRow;\n");
            if has_uuid_fields {
                content.push_str("use uuid::Uuid;\n");
            }
            if has_datetime_fields {
                content.push_str("use chrono::{DateTime, Utc};\n");
            }
        }
        
        content.push('\n');
        
        for (name, data_def) in &self.data_structures {
            content.push_str("#[derive(Debug, Clone, Serialize, Deserialize");
            if has_database_ops && matches!(db_type.as_str(), "postgres" | "mysql" | "sqlite") {
                content.push_str(", FromRow");
            }
            content.push_str(")]\n");
            content.push_str(&format!("pub struct {} {{\n", name));
            
            for field in &data_def.fields {
                let rust_type = self.roe_type_to_rust(&field.field_type, &field.annotations);
                let field_name = to_snake_case(&field.name);
                content.push_str(&format!("    pub {}: {},\n", field_name, rust_type));
            }
            
            content.push_str("}\n\n");
        }
        
        Ok(content)
    }
    
    fn generate_handlers_rs(&self, _context: &Context) -> Result<String, CodeGenError> {
        let has_database_ops = self.has_database_ops;
        
        let mut content = String::from(
r#"use axum::{
    response::{Json, Response, IntoResponse},
    extract::{Path, State, Query},
    http::StatusCode,
};
use serde_json::json;
use crate::{AppState"#);

        if !self.data_structures.is_empty() {
            content.push_str(", models::*");
        }
        content.push_str("};\n");
        
        if has_database_ops {
            content.push_str("use crate::db::Database;\n");
        }
        
        content.push('\n');
        
        for endpoint in &self.serve_endpoints {
            content.push_str(&format!(
r#"pub async fn {}(
    State(state): State<AppState>,
) -> impl IntoResponse {{
    // Handler implementation
    Json(json!({{"message": "Endpoint not fully implemented"}}))
}}

"#, endpoint.handler_name));
        }
        
        Ok(content)
    }
    
    fn generate_db_rs(&self, context: &Context) -> Result<String, CodeGenError> {
        let db_type = context.get("db_type").unwrap().as_str().unwrap();
        let db_type_cap = capitalize_first(db_type);
        
        let mut content = format!(
r#"use sqlx::{{{}Pool, Pool, {}::{}}};
use crate::models::*;

pub struct Database {{
    pool: Pool<{}>,
}}

impl Database {{
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {{
        let pool = {}Pool::connect(database_url).await?;
        Ok(Self {{ pool }})
    }}
    
    pub fn pool(&self) -> &Pool<{}> {{
        &self.pool
    }}

"#, db_type_cap, db_type, db_type_cap, db_type_cap, db_type_cap, db_type_cap);

        for name in self.data_structures.keys() {
            let snake_name = to_snake_case(name);
            content.push_str(&format!(
r#"    pub async fn find_{}_by_id(&self, id: &str) -> Result<Option<{}>, sqlx::Error> {{
        let result = sqlx::query_as::<_, {}>(
            "SELECT * FROM {}s WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }}
    
    pub async fn find_all_{}s(&self) -> Result<Vec<{}>, sqlx::Error> {{
        let results = sqlx::query_as::<_, {}>(
            "SELECT * FROM {}s"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(results)
    }}
    
    pub async fn create_{}(&self, data: {}) -> Result<{}, sqlx::Error> {{
        // Implementation depends on specific fields
        todo!("Implement create")
    }}
    
    pub async fn update_{}(&self, id: &str, data: {}) -> Result<{}, sqlx::Error> {{
        // Implementation depends on specific fields
        todo!("Implement update")
    }}
    
    pub async fn delete_{}(&self, id: &str) -> Result<(), sqlx::Error> {{
        sqlx::query("DELETE FROM {}s WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }}
"#, snake_name, name, name, snake_name,
     snake_name, name, name, snake_name,
     snake_name, name, name,
     snake_name, name, name,
     snake_name, snake_name));
        }

        content.push_str("}\n");
        Ok(content)
    }
    
    /// Convert Droe type to Rust type
    fn roe_type_to_rust(&self, roe_type: &str, annotations: &[String]) -> String {
        // Handle auto-generated IDs
        if annotations.contains(&"key".to_string()) && annotations.contains(&"auto".to_string()) {
            return "Uuid".to_string();
        }
        
        match roe_type {
            "text" => "String".to_string(),
            "int" => "i32".to_string(),
            "decimal" => "f64".to_string(),
            "flag" => "bool".to_string(),
            "date" | "datetime" => "DateTime<Utc>".to_string(),
            _ => "String".to_string(),
        }
    }
}

// Filter functions for Tera templates
fn snake_case_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Some(s) = value.as_str() {
        Ok(Value::String(to_snake_case(s)))
    } else {
        Err(tera::Error::msg("snake_case filter expects a string"))
    }
}

fn rust_type_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Some(s) = value.as_str() {
        let rust_type = match s {
            "text" => "String",
            "int" => "i32", 
            "decimal" => "f64",
            "flag" => "bool",
            "date" | "datetime" => "DateTime<Utc>",
            _ => "String",
        };
        Ok(Value::String(rust_type.to_string()))
    } else {
        Err(tera::Error::msg("rust_type filter expects a string"))
    }
}

// Helper functions
pub fn to_snake_case(name: &str) -> String {
    let mut result = Vec::new();
    let chars: Vec<char> = name.chars().collect();
    
    for (i, &char) in chars.iter().enumerate() {
        if char.is_uppercase() && i > 0 {
            // Check if previous char was not uppercase or if next char is lowercase
            let prev_is_lower = chars.get(i - 1).map_or(false, |c| c.is_lowercase());
            let next_is_lower = chars.get(i + 1).map_or(false, |c| c.is_lowercase());
            
            if prev_is_lower || next_is_lower {
                result.push('_');
            }
        }
        result.push(char.to_lowercase().next().unwrap());
    }
    result.into_iter().collect()
}

pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}