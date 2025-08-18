//! Framework adapters for Droe DSL
//! 
//! Adapters transform Droe DSL into specific framework code (Spring, Fiber, FastAPI, etc.)

use crate::ast::Program;
use serde_json::Value;
use std::collections::HashMap;

/// Base trait for framework adapters
pub trait FrameworkAdapter {
    /// Generate framework-specific code from Droe AST
    fn generate(&self, program: &Program, options: AdapterOptions) -> Result<AdapterOutput, String>;
    
    /// Get the framework name (e.g., "fiber", "spring", "fastapi")
    fn framework_name(&self) -> &str;
    
    /// Get the target language (e.g., "go", "java", "python")
    fn target_language(&self) -> &str;
}

/// Options for adapter generation
#[derive(Debug, Clone)]
pub struct AdapterOptions {
    pub package_name: Option<String>,
    pub output_dir: String,
    pub database_type: Option<String>,
    pub custom_vars: HashMap<String, Value>,
}

impl Default for AdapterOptions {
    fn default() -> Self {
        Self {
            package_name: None,
            output_dir: "output".to_string(),
            database_type: Some("postgres".to_string()),
            custom_vars: HashMap::new(),
        }
    }
}

/// Output from adapter generation
#[derive(Debug)]
pub struct AdapterOutput {
    /// Generated files: filename -> content
    pub files: HashMap<String, String>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Template context builder for Tera
pub struct TemplateContext {
    context: tera::Context,
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateContext {
    pub fn new() -> Self {
        Self {
            context: tera::Context::new(),
        }
    }
    
    pub fn insert<T: serde::Serialize>(&mut self, key: &str, value: &T) {
        self.context.insert(key, value);
    }
    
    pub fn from_program(program: &Program, options: &AdapterOptions) -> Self {
        let mut ctx = Self::new();
        
        // Extract data definitions
        let data_definitions: Vec<_> = program.statements.iter()
            .filter_map(|stmt| {
                if let crate::ast::Node::DataDefinition(data) = stmt {
                    Some(data)
                } else {
                    None
                }
            })
            .collect();
        
        // Extract action definitions
        let actions: Vec<_> = program.statements.iter()
            .filter_map(|stmt| {
                match stmt {
                    crate::ast::Node::ActionDefinition(action) => Some(action),
                    _ => None,
                }
            })
            .collect();
        
        ctx.insert("data_definitions", &data_definitions);
        ctx.insert("actions", &actions);
        ctx.insert("package_name", &options.package_name.as_deref().unwrap_or("droe_app"));
        ctx.insert("database_type", &options.database_type.as_deref().unwrap_or("postgres"));
        
        // Add custom variables
        for (key, value) in &options.custom_vars {
            ctx.context.insert(key, value);
        }
        
        ctx
    }
    
    pub fn inner(&self) -> &tera::Context {
        &self.context
    }
}

pub mod go;
pub mod java;
pub mod python;
pub mod node;
pub mod mobile;

// Re-export all framework adapters from each language
pub use go::fiber::FiberAdapter;
pub use java::spring::SpringAdapter;
pub use python::fastapi::FastAPIAdapter;
pub use node::FastifyAdapter;
pub use mobile::{IOSAdapter, AndroidAdapter};