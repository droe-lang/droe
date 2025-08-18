//! Fastify (Node.js) framework adapter

use crate::adapters::{FrameworkAdapter, AdapterOptions, AdapterOutput};
use crate::ast::{Program, Node, DataDefinition, ServeStatement, ModuleDefinition};
use std::collections::HashMap;
use tera::{Tera, Context};
use serde_json::{json, Value};

/// Fastify framework adapter
pub struct FastifyAdapter {
    tera: Tera,
}

impl FastifyAdapter {
    pub fn new() -> Result<Self, String> {
        let mut tera = Tera::default();
        
        // Add template strings
        tera.add_raw_template("package.json.tera", include_str!("templates/package.json.tera"))
            .map_err(|e| format!("Failed to add package.json template: {}", e))?;
        tera.add_raw_template("server.js.tera", include_str!("templates/server.js.tera"))
            .map_err(|e| format!("Failed to add server.js template: {}", e))?;
        tera.add_raw_template("handlers.js.tera", include_str!("templates/handlers.js.tera"))
            .map_err(|e| format!("Failed to add handlers.js template: {}", e))?;
        tera.add_raw_template("routes.js.tera", include_str!("templates/routes.js.tera"))
            .map_err(|e| format!("Failed to add routes.js template: {}", e))?;
        tera.add_raw_template("schema.prisma.tera", include_str!("templates/schema.prisma.tera"))
            .map_err(|e| format!("Failed to add schema.prisma template: {}", e))?;
        tera.add_raw_template(".env.tera", include_str!("templates/.env.tera"))
            .map_err(|e| format!("Failed to add .env template: {}", e))?;
        
        // Add custom filters
        tera.register_filter("snake_case", Self::snake_case_filter);
        tera.register_filter("lower", Self::lower_filter);
        
        Ok(Self { tera })
    }
    
    fn snake_case_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(s) = value.as_str() {
            let snake_case = Self::to_snake_case(s);
            Ok(json!(snake_case))
        } else {
            Ok(value.clone())
        }
    }
    
    fn lower_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(s) = value.as_str() {
            Ok(json!(s.to_lowercase()))
        } else {
            Ok(value.clone())
        }
    }
    
    fn to_snake_case(input: &str) -> String {
        let mut result = String::new();
        for (i, ch) in input.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        }
        result
    }
    
    fn analyze_program(&self, program: &Program) -> ProgramAnalysis {
        let mut analysis = ProgramAnalysis::default();
        
        for stmt in &program.statements {
            self.analyze_statement(stmt, &mut analysis);
        }
        
        analysis
    }
    
    fn analyze_statement(&self, stmt: &Node, analysis: &mut ProgramAnalysis) {
        match stmt {
            Node::DataDefinition(data) => {
                analysis.data_structures.insert(data.name.clone(), data.clone());
                analysis.has_database_ops = true;
            }
            Node::ServeStatement(serve) => {
                analysis.has_serve_endpoints = true;
                analysis.serve_endpoints.push(serve.clone());
                
                // Check serve body for database operations
                for body_stmt in &serve.body {
                    self.analyze_statement(body_stmt, analysis);
                }
            }
            Node::DatabaseStatement(_) => {
                analysis.has_database_ops = true;
            }
            Node::ModuleDefinition(module) => {
                analysis.modules.insert(module.name.clone(), module.clone());
                
                // Recursively analyze module body
                for module_stmt in &module.body {
                    self.analyze_statement(module_stmt, analysis);
                }
            }
            _ => {}
        }
    }
    
    fn get_template_context(&self, analysis: &ProgramAnalysis, options: &AdapterOptions) -> Context {
        let mut context = Context::new();
        
        // Process serve endpoints to add handler names
        let processed_endpoints: Vec<Value> = analysis.serve_endpoints.iter()
            .map(|endpoint| {
                json!({
                    "method": endpoint.method,
                    "endpoint": endpoint.endpoint,
                    "handler_name": self.get_handler_name(endpoint),
                    "params": &endpoint.params,
                    "accept_type": endpoint.accept_type.as_deref().unwrap_or(""),
                    "response_action": endpoint.response_action.as_ref().map(|_| "action").unwrap_or("")
                })
            })
            .collect();
        
        let package_name = options.package_name.as_deref().unwrap_or("droe_app")
            .replace(['.', '-'], "_");
        
        context.insert("package_name", &package_name);
        context.insert("has_serve_endpoints", &analysis.has_serve_endpoints);
        context.insert("has_database_ops", &analysis.has_database_ops);
        context.insert("data_structures", &analysis.data_structures);
        context.insert("serve_endpoints", &processed_endpoints);
        context.insert("modules", &analysis.modules);
        context.insert("db_type", &options.database_type.as_deref().unwrap_or("postgres"));
        context.insert("default_db_url", &self.get_default_db_url(options.database_type.as_deref().unwrap_or("postgres")));
        
        context
    }
    
    fn get_handler_name(&self, endpoint: &ServeStatement) -> String {
        let method = endpoint.method.to_lowercase();
        let path_parts: Vec<&str> = endpoint.endpoint.trim_start_matches('/')
            .split('/')
            .filter(|p| !p.starts_with(':'))
            .collect();
        
        if !path_parts.is_empty() {
            format!("{}_{}", method, path_parts.join("_"))
        } else {
            format!("{}_root", method)
        }
    }
    
    fn get_default_db_url(&self, db_type: &str) -> String {
        match db_type {
            "postgres" => "postgresql://localhost:5432/droe_db".to_string(),
            "mysql" => "mysql://localhost:3306/droe_db".to_string(),
            "sqlite" => "file:./dev.db".to_string(),
            _ => "postgresql://localhost:5432/droe_db".to_string(),
        }
    }
}

#[derive(Debug, Default)]
struct ProgramAnalysis {
    has_serve_endpoints: bool,
    has_database_ops: bool,
    data_structures: HashMap<String, DataDefinition>,
    serve_endpoints: Vec<ServeStatement>,
    modules: HashMap<String, ModuleDefinition>,
}

impl FrameworkAdapter for FastifyAdapter {
    fn generate(&self, program: &Program, options: AdapterOptions) -> Result<AdapterOutput, String> {
        let analysis = self.analyze_program(program);
        let context = self.get_template_context(&analysis, &options);
        let mut files = HashMap::new();
        
        // Generate package.json
        let package_json = self.tera.render("package.json.tera", &context)
            .map_err(|e| format!("Failed to render package.json: {}", e))?;
        files.insert("package.json".to_string(), package_json);
        
        // Generate main server file
        let server_js = self.tera.render("server.js.tera", &context)
            .map_err(|e| format!("Failed to render server.js: {}", e))?;
        files.insert("src/server.js".to_string(), server_js);
        
        // Generate Prisma schema if database operations exist
        if analysis.has_database_ops {
            let schema_prisma = self.tera.render("schema.prisma.tera", &context)
                .map_err(|e| format!("Failed to render schema.prisma: {}", e))?;
            files.insert("prisma/schema.prisma".to_string(), schema_prisma);
            
            let env_file = self.tera.render(".env.tera", &context)
                .map_err(|e| format!("Failed to render .env: {}", e))?;
            files.insert(".env".to_string(), env_file);
        }
        
        // Generate routes for data structures
        if !analysis.data_structures.is_empty() {
            let routes_js = self.tera.render("routes.js.tera", &context)
                .map_err(|e| format!("Failed to render routes.js: {}", e))?;
            files.insert("src/routes/index.js".to_string(), routes_js);
        }
        
        // Generate handlers for custom endpoints
        if !analysis.serve_endpoints.is_empty() {
            let handlers_js = self.tera.render("handlers.js.tera", &context)
                .map_err(|e| format!("Failed to render handlers.js: {}", e))?;
            files.insert("src/handlers/index.js".to_string(), handlers_js);
        }
        
        Ok(AdapterOutput {
            files,
            metadata: HashMap::new(),
        })
    }
    
    fn framework_name(&self) -> &str {
        "fastify"
    }
    
    fn target_language(&self) -> &str {
        "node"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fastify_adapter_creation() {
        let adapter = FastifyAdapter::new();
        assert!(adapter.is_ok());
        
        let adapter = adapter.unwrap();
        assert_eq!(adapter.framework_name(), "fastify");
        assert_eq!(adapter.target_language(), "node");
    }

    #[test]
    fn test_snake_case_conversion() {
        assert_eq!(FastifyAdapter::to_snake_case("CamelCase"), "camel_case");
        assert_eq!(FastifyAdapter::to_snake_case("SimpleTest"), "simple_test");
        assert_eq!(FastifyAdapter::to_snake_case("lowercase"), "lowercase");
    }
}