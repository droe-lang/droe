//! iOS/SwiftUI adapter

use crate::adapters::{FrameworkAdapter, AdapterOptions, AdapterOutput};
use crate::ast::Program;
use crate::codegen::mobile::MobileGenerator;

pub struct IOSAdapter {
    generator: MobileGenerator,
}

impl IOSAdapter {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            generator: MobileGenerator::new()?,
        })
    }
}

impl FrameworkAdapter for IOSAdapter {
    fn generate(&self, program: &Program, options: AdapterOptions) -> Result<AdapterOutput, String> {
        let files = self.generator.generate_project(program, "ios")?;
        
        Ok(AdapterOutput {
            files,
            metadata: std::collections::HashMap::from([
                ("platform".to_string(), serde_json::Value::String("iOS".to_string())),
                ("framework".to_string(), serde_json::Value::String("SwiftUI".to_string())),
                ("app_name".to_string(), serde_json::Value::String(options.package_name.unwrap_or("MyApp".to_string()))),
            ]),
        })
    }
    
    fn framework_name(&self) -> &str {
        "swiftui"
    }
    
    fn target_language(&self) -> &str {
        "swift"
    }
}