//! Android/Kotlin adapter

use crate::adapters::{FrameworkAdapter, AdapterOptions, AdapterOutput};
use crate::ast::Program;
use crate::codegen::mobile::MobileGenerator;

pub struct AndroidAdapter {
    generator: MobileGenerator,
}

impl AndroidAdapter {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            generator: MobileGenerator::new()?,
        })
    }
}

impl FrameworkAdapter for AndroidAdapter {
    fn generate(&self, program: &Program, options: AdapterOptions) -> Result<AdapterOutput, String> {
        let files = self.generator.generate_project(program, "android")?;
        
        Ok(AdapterOutput {
            files,
            metadata: std::collections::HashMap::from([
                ("platform".to_string(), serde_json::Value::String("Android".to_string())),
                ("framework".to_string(), serde_json::Value::String("Kotlin".to_string())),
                ("package_name".to_string(), serde_json::Value::String(options.package_name.unwrap_or("com.example.myapp".to_string()))),
            ]),
        })
    }
    
    fn framework_name(&self) -> &str {
        "kotlin"
    }
    
    fn target_language(&self) -> &str {
        "kotlin"
    }
}