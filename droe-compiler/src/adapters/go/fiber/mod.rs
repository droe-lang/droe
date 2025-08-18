//! Go Fiber framework adapter

use crate::adapters::{FrameworkAdapter, AdapterOptions, AdapterOutput, TemplateContext};
use crate::ast::Program;
use std::collections::HashMap;
use tera::Tera;

/// Fiber (Go) framework adapter
pub struct FiberAdapter {
    templates: Tera,
}

impl FiberAdapter {
    pub fn new() -> Result<Self, String> {
        let mut tera = Tera::default();
        
        // Load templates - they're in the same directory as this module
        tera.add_raw_template("main.go", include_str!("templates/main.go.tera"))
            .map_err(|e| format!("Failed to load main.go template: {}", e))?;
            
        tera.add_raw_template("go.mod", include_str!("templates/go.mod.tera"))
            .map_err(|e| format!("Failed to load go.mod template: {}", e))?;
            
        tera.add_raw_template("handlers.go", include_str!("templates/handlers.go.tera"))
            .map_err(|e| format!("Failed to load handlers.go template: {}", e))?;
            
        tera.add_raw_template("models.go", include_str!("templates/models.go.tera"))
            .map_err(|e| format!("Failed to load models.go template: {}", e))?;
            
        tera.add_raw_template("database.go", include_str!("templates/database.go.tera"))
            .map_err(|e| format!("Failed to load database.go template: {}", e))?;
            
        tera.add_raw_template("routes.go", include_str!("templates/routes.go.tera"))
            .map_err(|e| format!("Failed to load routes.go template: {}", e))?;
        
        Ok(Self { templates: tera })
    }
}

impl FrameworkAdapter for FiberAdapter {
    fn generate(&self, program: &Program, options: AdapterOptions) -> Result<AdapterOutput, String> {
        let context = TemplateContext::from_program(program, &options);
        let mut files = HashMap::new();
        
        // Generate all files
        let templates = vec![
            ("main.go", "main.go"),
            ("go.mod", "go.mod"),
            ("handlers.go", "handlers.go"),
            ("models.go", "models.go"),
            ("database.go", "database.go"),
            ("routes.go", "routes.go"),
        ];
        
        for (template_name, output_name) in templates {
            let content = self.templates.render(template_name, context.inner())
                .map_err(|e| format!("Failed to render {}: {}", template_name, e))?;
            files.insert(output_name.to_string(), content);
        }
        
        Ok(AdapterOutput {
            files,
            metadata: HashMap::new(),
        })
    }
    
    fn framework_name(&self) -> &str {
        "fiber"
    }
    
    fn target_language(&self) -> &str {
        "go"
    }
}

impl Default for FiberAdapter {
    fn default() -> Self {
        Self::new().expect("Failed to initialize FiberAdapter")
    }
}