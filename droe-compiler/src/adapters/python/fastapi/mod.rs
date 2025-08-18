//! FastAPI framework adapter
//! 
//! This adapter builds on top of the core Python generator to add FastAPI-specific features:
//! - SQLAlchemy ORM models
//! - Pydantic schemas  
//! - FastAPI routers and endpoints
//! - Database session management

use crate::adapters::{FrameworkAdapter, AdapterOptions, AdapterOutput};
use crate::ast::{Program, Node, DataDefinition, ServeStatement};
use crate::codegen::{python::PythonGenerator, CodeGenerator};
use std::collections::HashMap;
use tera::{Tera, Value, try_get_value};
use serde_json::value::to_value;

/// FastAPI framework adapter
pub struct FastAPIAdapter {
    package_name: String,
    database_type: String,
    tera: Tera,
}

impl Default for FastAPIAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl FastAPIAdapter {
    pub fn new() -> Self {
        let mut tera = Tera::default();
        
        // Load templates from embedded strings
        tera.add_raw_templates(vec![
            ("requirements.txt", include_str!("templates/requirements.txt.tera")),
            ("main.py", include_str!("templates/main.py.tera")),
            ("models.py", include_str!("templates/models.py.tera")),
            ("schemas.py", include_str!("templates/schemas.py.tera")),
            ("database.py", include_str!("templates/database.py.tera")),
            ("routers.py", include_str!("templates/routers.py.tera")),
        ]).expect("Failed to load FastAPI templates");

        // Register custom filters
        tera.register_filter("snake_case", snake_case_filter);
        tera.register_filter("pascal_case", pascal_case_filter);
        tera.register_filter("python_type", python_type_filter);
        tera.register_filter("sqlalchemy_type", sqlalchemy_type_filter);
        tera.register_filter("handler_name", handler_name_filter);

        Self {
            package_name: "droe_app".to_string(),
            database_type: "postgresql".to_string(),
            tera,
        }
    }

    pub fn with_package_name(mut self, name: &str) -> Self {
        self.package_name = name.to_string();
        self
    }

    pub fn with_database(mut self, db_type: &str) -> Self {
        self.database_type = db_type.to_string();
        self
    }

    fn render_template(&self, template_name: &str, context: &tera::Context) -> Result<String, String> {
        self.tera.render(template_name, context)
            .map_err(|e| format!("Template rendering error: {}", e))
    }
}

impl FrameworkAdapter for FastAPIAdapter {
    fn generate(&self, program: &Program, options: AdapterOptions) -> Result<AdapterOutput, String> {
        // Get package name from options or use default
        let package_name = options.package_name.as_ref()
            .unwrap_or(&self.package_name);
        let db_type = options.database_type.as_ref()
            .unwrap_or(&self.database_type);

        // Analyze program for framework features
        let analysis = self.analyze_program(program);

        // Generate core Python code first
        let core_generator = PythonGenerator::new().with_class_name(&format!("{}Core", package_name));
        let core_code = core_generator.generate(program)?;

        // Generate framework-specific files
        let mut files = HashMap::new();

        // Core business logic (generated from core Python generator)
        files.insert(format!("{}/core.py", package_name), core_code);

        // Create template context
        let mut context = tera::Context::new();
        context.insert("package_name", package_name);
        context.insert("database_type", db_type);
        context.insert("has_data_definitions", &analysis.has_data_definitions);
        context.insert("has_serve_statements", &analysis.has_serve_statements);
        context.insert("data_definitions", &analysis.data_definitions);
        context.insert("serve_statements", &analysis.serve_statements);
        context.insert("database_url", &self.get_database_url(db_type));

        // FastAPI-specific files using templates
        files.insert("requirements.txt".to_string(), self.render_template("requirements.txt", &context)?);
        files.insert(format!("{}/__init__.py", package_name), "".to_string());
        files.insert(format!("{}/main.py", package_name), self.render_template("main.py", &context)?);
        
        if analysis.has_data_definitions {
            files.insert(format!("{}/models.py", package_name), self.render_template("models.py", &context)?);
            files.insert(format!("{}/schemas.py", package_name), self.render_template("schemas.py", &context)?);
            files.insert(format!("{}/database.py", package_name), self.render_template("database.py", &context)?);
        }

        if analysis.has_serve_statements {
            files.insert(format!("{}/routers.py", package_name), self.render_template("routers.py", &context)?);
        }

        Ok(AdapterOutput {
            files,
            metadata: HashMap::new(),
        })
    }

    fn framework_name(&self) -> &str {
        "fastapi"
    }

    fn target_language(&self) -> &str {
        "python"
    }
}

impl FastAPIAdapter {
    fn analyze_program(&self, program: &Program) -> ProgramAnalysis {
        let mut analysis = ProgramAnalysis::default();

        for statement in &program.statements {
            match statement {
                Node::DataDefinition(data) => {
                    analysis.has_data_definitions = true;
                    analysis.data_definitions.push(data.clone());
                }
                Node::ServeStatement(serve) => {
                    analysis.has_serve_statements = true;
                    analysis.serve_statements.push(serve.clone());
                }
                _ => {}
            }
        }

        analysis
    }

    fn get_database_url(&self, db_type: &str) -> String {
        match db_type {
            "postgresql" => "postgresql://localhost/droe_db".to_string(),
            "mysql" => "mysql://localhost/droe_db".to_string(),
            "sqlite" => "sqlite:///./droe.db".to_string(),
            _ => "postgresql://localhost/droe_db".to_string(),
        }
    }
}

#[derive(Debug, Default)]
struct ProgramAnalysis {
    has_data_definitions: bool,
    has_serve_statements: bool,
    data_definitions: Vec<DataDefinition>,
    serve_statements: Vec<ServeStatement>,
}

// Template filters
fn snake_case_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let string_val = try_get_value!("snake_case", "value", String, value);
    let mut result = String::new();
    for (i, c) in string_val.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap());
    }
    Ok(to_value(result).unwrap())
}

fn pascal_case_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let string_val = try_get_value!("pascal_case", "value", String, value);
    let result = string_val.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect::<String>();
    Ok(to_value(result).unwrap())
}

fn python_type_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let string_val = try_get_value!("python_type", "value", String, value);
    let result = match string_val.as_str() {
        "text" => "str",
        "int" => "int",
        "decimal" => "float",
        "flag" => "bool",
        "date" | "datetime" => "datetime",
        _ => "str",
    };
    Ok(to_value(result).unwrap())
}

fn sqlalchemy_type_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let string_val = try_get_value!("sqlalchemy_type", "value", String, value);
    let result = match string_val.as_str() {
        "text" => "String",
        "int" => "Integer",
        "decimal" => "Float",
        "flag" => "Boolean",
        "date" | "datetime" => "DateTime",
        _ => "String",
    };
    Ok(to_value(result).unwrap())
}

fn handler_name_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let string_val = try_get_value!("handler_name", "value", String, value);
    let path_parts: Vec<&str> = string_val
        .trim_matches('/')
        .split('/')
        .filter(|p| !p.starts_with(':') && !p.starts_with('{') && !p.is_empty())
        .collect();

    let result = if path_parts.is_empty() {
        "root".to_string()
    } else {
        path_parts.join("_")
    };
    Ok(to_value(result).unwrap())
}

// #[cfg(test)]
// mod test;