//! Mobile code generation for iOS and Android
//! 
//! This module provides mobile app generation targeting SwiftUI and Android/Kotlin.

use crate::ast::Program;
use crate::codegen::CodeGenerator;
use std::collections::HashMap;
use tera::{Tera, Context};

/// Mobile code generator that creates complete mobile projects
pub struct MobileGenerator {
    tera: Tera,
}

impl MobileGenerator {
    pub fn new() -> Result<Self, String> {
        let mut tera = Tera::default();
        
        // Load Swift templates
        tera.add_raw_template("swift/content_view.swift.tera", include_str!("../adapters/mobile/ios/templates/content_view.swift.tera"))
            .map_err(|e| format!("Failed to load swift content_view template: {}", e))?;
        tera.add_raw_template("swift/app.swift.tera", include_str!("../adapters/mobile/ios/templates/app.swift.tera"))
            .map_err(|e| format!("Failed to load swift app template: {}", e))?;
        tera.add_raw_template("swift/info.plist.tera", include_str!("../adapters/mobile/ios/templates/info.plist.tera"))
            .map_err(|e| format!("Failed to load swift info.plist template: {}", e))?;
        tera.add_raw_template("swift/layout_view.swift.tera", include_str!("../adapters/mobile/ios/templates/layout_view.swift.tera"))
            .map_err(|e| format!("Failed to load swift layout_view template: {}", e))?;
        tera.add_raw_template("swift/form_view.swift.tera", include_str!("../adapters/mobile/ios/templates/form_view.swift.tera"))
            .map_err(|e| format!("Failed to load swift form_view template: {}", e))?;
        tera.add_raw_template("swift/api_service.swift.tera", include_str!("../adapters/mobile/ios/templates/api_service.swift.tera"))
            .map_err(|e| format!("Failed to load swift api_service template: {}", e))?;
        tera.add_raw_template("swift/models.swift.tera", include_str!("../adapters/mobile/ios/templates/models.swift.tera"))
            .map_err(|e| format!("Failed to load swift models template: {}", e))?;
        tera.add_raw_template("swift/project.pbxproj.tera", include_str!("../adapters/mobile/ios/templates/project.pbxproj.tera"))
            .map_err(|e| format!("Failed to load swift project template: {}", e))?;
        
        // Load Kotlin templates  
        tera.add_raw_template("kotlin/main_activity.kt.tera", include_str!("../adapters/mobile/android/templates/main_activity.kt.tera"))
            .map_err(|e| format!("Failed to load kotlin main_activity template: {}", e))?;
        tera.add_raw_template("kotlin/app_build.gradle.tera", include_str!("../adapters/mobile/android/templates/app_build.gradle.tera"))
            .map_err(|e| format!("Failed to load kotlin app_build template: {}", e))?;
        tera.add_raw_template("kotlin/project_build.gradle.tera", include_str!("../adapters/mobile/android/templates/project_build.gradle.tera"))
            .map_err(|e| format!("Failed to load kotlin project_build template: {}", e))?;
        tera.add_raw_template("kotlin/manifest.xml.tera", include_str!("../adapters/mobile/android/templates/manifest.xml.tera"))
            .map_err(|e| format!("Failed to load kotlin manifest template: {}", e))?;
        tera.add_raw_template("kotlin/layout.xml.tera", include_str!("../adapters/mobile/android/templates/layout.xml.tera"))
            .map_err(|e| format!("Failed to load kotlin layout template: {}", e))?;
        tera.add_raw_template("kotlin/api_service.kt.tera", include_str!("../adapters/mobile/android/templates/api_service.kt.tera"))
            .map_err(|e| format!("Failed to load kotlin api_service template: {}", e))?;
        tera.add_raw_template("kotlin/network_module.kt.tera", include_str!("../adapters/mobile/android/templates/network_module.kt.tera"))
            .map_err(|e| format!("Failed to load kotlin network_module template: {}", e))?;
        tera.add_raw_template("kotlin/repository.kt.tera", include_str!("../adapters/mobile/android/templates/repository.kt.tera"))
            .map_err(|e| format!("Failed to load kotlin repository template: {}", e))?;
        tera.add_raw_template("kotlin/form_activity.kt.tera", include_str!("../adapters/mobile/android/templates/form_activity.kt.tera"))
            .map_err(|e| format!("Failed to load kotlin form_activity template: {}", e))?;
        
        // Register custom filters
        tera.register_filter("camelcase", filters::camel_case);
        tera.register_filter("pascalcase", filters::pascal_case);
        tera.register_filter("snakecase", filters::snake_case);
        
        Ok(Self { tera })
    }
    
    /// Generate a mobile project for the specified platform
    pub fn generate_project(&self, program: &Program, platform: &str) -> Result<HashMap<String, String>, String> {
        match platform.to_lowercase().as_str() {
            "ios" | "swiftui" => self.generate_ios_project(program),
            "android" | "kotlin" => self.generate_android_project(program),
            _ => Err(format!("Unsupported mobile platform: {}", platform)),
        }
    }
    
    /// Generate iOS/SwiftUI project
    fn generate_ios_project(&self, program: &Program) -> Result<HashMap<String, String>, String> {
        let context = self.extract_ui_components(program);
        let mut files = HashMap::new();
        
        // Generate main ContentView
        let content_view = self.render_template("swift/content_view.swift.tera", &context)?;
        files.insert("ContentView.swift".to_string(), content_view);
        
        // Generate App file
        let app_file = self.render_template("swift/app.swift.tera", &context)?;
        files.insert(format!("{}App.swift", context.get("app_name").unwrap_or(&tera::Value::String("MyApp".to_string()))), app_file);
        
        // Generate layout views
        if let Some(layouts) = context.get("layouts") {
            if let tera::Value::Array(layouts_array) = layouts {
                for layout in layouts_array {
                    if let tera::Value::Object(layout_obj) = layout {
                        if let Some(tera::Value::String(layout_name)) = layout_obj.get("name") {
                            let layout_view = self.render_template("swift/layout_view.swift.tera", &context)?;
                            files.insert(format!("Views/{}View.swift", to_pascal_case(layout_name)), layout_view);
                        }
                    }
                }
            }
        }
        
        // Generate API service if needed
        if context.get("has_api").unwrap_or(&tera::Value::Bool(false)) == &tera::Value::Bool(true) {
            let api_service = self.render_template("swift/api_service.swift.tera", &context)?;
            files.insert("Network/ApiService.swift".to_string(), api_service);
            
            let models = self.render_template("swift/models.swift.tera", &context)?;
            files.insert("Models/Models.swift".to_string(), models);
        }
        
        // Generate Info.plist
        let info_plist = self.render_template("swift/info.plist.tera", &context)?;
        files.insert("Info.plist".to_string(), info_plist);
        
        Ok(files)
    }
    
    /// Generate Android/Kotlin project
    fn generate_android_project(&self, program: &Program) -> Result<HashMap<String, String>, String> {
        let context = self.extract_ui_components(program);
        let mut files = HashMap::new();
        
        // Generate MainActivity
        let main_activity = self.render_template("kotlin/main_activity.kt.tera", &context)?;
        files.insert("app/src/main/java/com/example/myapp/MainActivity.kt".to_string(), main_activity);
        
        // Generate layout XML files
        if let Some(layouts) = context.get("layouts") {
            if let tera::Value::Array(layouts_array) = layouts {
                for layout in layouts_array {
                    if let tera::Value::Object(layout_obj) = layout {
                        if let Some(tera::Value::String(layout_name)) = layout_obj.get("name") {
                            let layout_xml = self.render_template("kotlin/layout.xml.tera", &context)?;
                            files.insert(format!("app/src/main/res/layout/{}.xml", to_snake_case(layout_name)), layout_xml);
                        }
                    }
                }
            }
        }
        
        // Generate API service if needed
        if context.get("has_api").unwrap_or(&tera::Value::Bool(false)) == &tera::Value::Bool(true) {
            let api_service = self.render_template("kotlin/api_service.kt.tera", &context)?;
            files.insert("app/src/main/java/com/example/myapp/network/ApiService.kt".to_string(), api_service);
            
            let network_module = self.render_template("kotlin/network_module.kt.tera", &context)?;
            files.insert("app/src/main/java/com/example/myapp/di/NetworkModule.kt".to_string(), network_module);
            
            let repository = self.render_template("kotlin/repository.kt.tera", &context)?;
            files.insert("app/src/main/java/com/example/myapp/repository/ApiRepository.kt".to_string(), repository);
        }
        
        // Generate build files
        let app_gradle = self.render_template("kotlin/app_build.gradle.tera", &context)?;
        files.insert("app/build.gradle".to_string(), app_gradle);
        
        let project_gradle = self.render_template("kotlin/project_build.gradle.tera", &context)?;
        files.insert("build.gradle".to_string(), project_gradle);
        
        // Generate AndroidManifest.xml
        let manifest = self.render_template("kotlin/manifest.xml.tera", &context)?;
        files.insert("app/src/main/AndroidManifest.xml".to_string(), manifest);
        
        Ok(files)
    }
    
    /// Extract UI components and other data from AST
    fn extract_ui_components(&self, _program: &Program) -> Context {
        let mut context = Context::new();
        
        // Set defaults
        context.insert("app_name", "MyApp");
        context.insert("package_name", "com.example.myapp");
        context.insert("layouts", &Vec::<serde_json::Value>::new());
        context.insert("forms", &Vec::<serde_json::Value>::new());
        context.insert("components", &Vec::<serde_json::Value>::new());
        context.insert("api_calls", &Vec::<serde_json::Value>::new());
        context.insert("has_api", &false);
        context.insert("permissions", &Vec::<String>::new());
        
        // Set capability flags
        context.insert("has_camera", &false);
        context.insert("has_location", &false);
        context.insert("has_notifications", &false);
        context.insert("has_storage", &false);
        context.insert("has_sensors", &false);
        context.insert("has_contacts", &false);
        
        context
    }
    
    fn render_template(&self, template_name: &str, context: &Context) -> Result<String, String> {
        self.tera.render(template_name, context)
            .map_err(|e| format!("Template rendering error: {}", e))
    }
}

impl CodeGenerator for MobileGenerator {
    fn generate(&self, program: &Program) -> Result<String, String> {
        // For the base CodeGenerator trait, we'll return a summary
        // The actual project generation happens through generate_project()
        let ios_files = self.generate_ios_project(program)?;
        let android_files = self.generate_android_project(program)?;
        
        Ok(format!(
            "Mobile project generated with {} iOS files and {} Android files",
            ios_files.len(),
            android_files.len()
        ))
    }
}

// Helper functions for text case conversion
fn to_camel_case(text: &str) -> String {
    let normalized = text.replace('-', "_");
    let words: Vec<&str> = normalized.split('_').collect();
    if words.is_empty() { return String::new(); }
    
    let mut result = words[0].to_lowercase();
    for word in &words[1..] {
        result.push_str(&capitalize_first(word));
    }
    result
}

fn to_pascal_case(text: &str) -> String {
    let normalized = text.replace('-', "_");
    let words: Vec<&str> = normalized.split('_').collect();
    words.iter().map(|word| capitalize_first(word)).collect()
}

fn to_snake_case(text: &str) -> String {
    text.replace('-', "_").to_lowercase()
}

fn capitalize_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
    }
}

// Tera custom filters
mod filters {
    use tera::{Result as TeraResult, Value};
    use std::collections::HashMap;
    
    pub fn camel_case(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
        match value {
            Value::String(s) => Ok(Value::String(super::to_camel_case(s))),
            _ => Err("camel_case filter can only be applied to strings".into()),
        }
    }
    
    pub fn pascal_case(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
        match value {
            Value::String(s) => Ok(Value::String(super::to_pascal_case(s))),
            _ => Err("pascal_case filter can only be applied to strings".into()),
        }
    }
    
    pub fn snake_case(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
        match value {
            Value::String(s) => Ok(Value::String(super::to_snake_case(s))),
            _ => Err("snake_case filter can only be applied to strings".into()),
        }
    }
}