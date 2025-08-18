//! Spring Boot (Java) framework adapter

use crate::adapters::{FrameworkAdapter, AdapterOptions, AdapterOutput};
use crate::ast::{Program, Node, DataDefinition, ModuleDefinition};
use std::collections::HashMap;
use serde_json::{json, Value};
use tera::Tera;

/// Spring Boot framework adapter
pub struct SpringAdapter {
    templates: Tera,
}

impl Default for SpringAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SpringAdapter {
    pub fn new() -> Self {
        let mut templates = Tera::default();
        
        // Register Spring Boot templates
        templates.add_raw_template("application.java", include_str!("templates/application.java.tera")).unwrap();
        templates.add_raw_template("entity.java", include_str!("templates/entity.java.tera")).unwrap();
        templates.add_raw_template("repository.java", include_str!("templates/repository.java.tera")).unwrap();
        templates.add_raw_template("service.java", include_str!("templates/service.java.tera")).unwrap();
        templates.add_raw_template("controller.java", include_str!("templates/controller.java.tera")).unwrap();
        templates.add_raw_template("pom.xml", include_str!("templates/pom.xml.tera")).unwrap();
        templates.add_raw_template("application.properties", include_str!("templates/application.properties.tera")).unwrap();
        templates.add_raw_template("readme.md", include_str!("templates/readme.md.tera")).unwrap();
        
        Self { templates }
    }
    
    fn build_context(&self, program: &Program, options: &AdapterOptions) -> tera::Context {
        let mut context = tera::Context::new();
        
        // Basic project info
        let package_name = options.package_name.as_deref().unwrap_or("com.example.app");
        let project_name = package_name.split('.').last().unwrap_or("app");
        
        context.insert("package_name", package_name);
        context.insert("project_name", project_name);
        context.insert("app_name", &Self::to_pascal_case(project_name));
        context.insert("artifact_id", &format!("{}-spring-boot", project_name));
        context.insert("group_id", &package_name.rsplit_once('.').map(|(g, _)| g).unwrap_or("com.example"));
        context.insert("spring_boot_version", "3.1.5");
        context.insert("java_version", "17");
        
        // Database configuration
        let db_type = options.database_type.as_deref().unwrap_or("h2");
        context.insert("database", db_type);
        
        match db_type {
            "postgres" | "postgresql" => {
                context.insert("database_url", "jdbc:postgresql://localhost:5432/mydb");
                context.insert("db_username", "postgres");
                context.insert("db_password", "password");
                context.insert("db_driver", "org.postgresql.Driver");
            }
            "mysql" => {
                context.insert("database_url", "jdbc:mysql://localhost:3306/mydb");
                context.insert("db_username", "root");
                context.insert("db_password", "password");
                context.insert("db_driver", "com.mysql.cj.jdbc.Driver");
            }
            _ => { // Default to H2
                context.insert("database_url", "jdbc:h2:mem:testdb");
                context.insert("db_username", "sa");
                context.insert("db_password", "");
                context.insert("db_driver", "org.h2.Driver");
            }
        }
        
        // Extract entities and modules
        let mut entities = Vec::new();
        let mut modules = Vec::new();
        
        for stmt in &program.statements {
            match stmt {
                Node::DataDefinition(data_def) => {
                    entities.push(self.process_data_definition(data_def, package_name));
                }
                Node::ModuleDefinition(module_def) => {
                    modules.push(self.process_module(module_def));
                    
                    // Extract data definitions from module
                    for module_stmt in &module_def.body {
                        if let Node::DataDefinition(data_def) = module_stmt {
                            entities.push(self.process_data_definition(data_def, package_name));
                        }
                    }
                }
                _ => {}
            }
        }
        
        context.insert("entities", &entities);
        context.insert("modules", &modules);
        context.insert("has_entities", &!entities.is_empty());
        context.insert("has_modules", &!modules.is_empty());
        
        context
    }
    
    fn process_data_definition(&self, data_def: &DataDefinition, package_name: &str) -> Value {
        let mut fields = Vec::new();
        let mut has_id_field = false;
        
        for field in &data_def.fields {
            let java_type = self.get_java_type(&field.field_type);
            let is_id = field.name.to_lowercase() == "id";
            
            if is_id {
                has_id_field = true;
            }
            
            fields.push(json!({
                "name": field.name,
                "java_type": java_type,
                "is_id": is_id,
                "getter_name": format!("get{}", Self::to_pascal_case(&field.name)),
                "setter_name": format!("set{}", Self::to_pascal_case(&field.name)),
            }));
        }
        
        // Add default ID field if not present
        if !has_id_field {
            fields.insert(0, json!({
                "name": "id",
                "java_type": "Long",
                "is_id": true,
                "getter_name": "getId",
                "setter_name": "setId",
            }));
        }
        
        json!({
            "class_name": data_def.name,
            "table_name": format!("{}s", data_def.name.to_lowercase()),
            "fields": fields,
            "package_name": package_name,
        })
    }
    
    fn process_module(&self, module_def: &ModuleDefinition) -> Value {
        let mut actions = Vec::new();
        
        for stmt in &module_def.body {
            match stmt {
                Node::ActionDefinition(action) => {
                    actions.push(json!({
                        "name": action.name,
                        "has_params": false,
                        "return_type": "Object",
                    }));
                }
                Node::ActionDefinitionWithParams(action) => {
                    let params: Vec<Value> = action.parameters.iter().map(|p| {
                        json!({
                            "name": p.name,
                            "java_type": self.get_java_type(&p.param_type),
                        })
                    }).collect();
                    
                    actions.push(json!({
                        "name": action.name,
                        "has_params": true,
                        "params": params,
                        "return_type": action.return_type.as_ref().map(|t| self.get_java_type(t)).unwrap_or_else(|| "Object".to_string()),
                    }));
                }
                _ => {}
            }
        }
        
        json!({
            "name": module_def.name,
            "service_name": format!("{}Service", module_def.name),
            "actions": actions,
        })
    }
    
    fn get_java_type(&self, type_name: &str) -> String {
        match type_name {
            "int" | "number" => "Integer".to_string(),
            "decimal" => "BigDecimal".to_string(),
            "text" | "string" => "String".to_string(),
            "flag" | "yesno" | "boolean" => "Boolean".to_string(),
            "date" => "LocalDate".to_string(),
            "datetime" => "LocalDateTime".to_string(),
            "time" => "LocalTime".to_string(),
            _ if type_name.starts_with("list_of_") => {
                let element_type = &type_name[8..];
                format!("List<{}>", self.get_java_type(element_type))
            }
            _ => "Object".to_string(),
        }
    }
    
    fn to_pascal_case(name: &str) -> String {
        let binding = name.replace(['-', ' '], "_");
        let parts: Vec<&str> = binding.split('_').collect();
        let mut result = String::new();
        
        for part in parts {
            if !part.is_empty() {
                let mut chars = part.chars();
                if let Some(first) = chars.next() {
                    result.push(first.to_ascii_uppercase());
                    result.extend(chars);
                }
            }
        }
        
        result
    }
}

impl FrameworkAdapter for SpringAdapter {
    fn generate(&self, program: &Program, options: AdapterOptions) -> Result<AdapterOutput, String> {
        let context = self.build_context(program, &options);
        let package_name = options.package_name.as_deref().unwrap_or("com.example.app");
        let package_path = package_name.replace('.', "/");
        
        let mut files = HashMap::new();
        
        // Generate main application class
        let app_content = self.templates.render("application.java", &context)
            .map_err(|e| format!("Failed to render application.java: {}", e))?;
        files.insert(format!("src/main/java/{}/Application.java", package_path), app_content);
        
        // Generate entities, repositories, services, and controllers
        if let Some(entities) = context.get("entities") {
            if let Some(entities_array) = entities.as_array() {
                for entity in entities_array {
                    let mut entity_context = context.clone();
                    entity_context.insert("entity", entity);
                    
                    // Entity
                    let entity_name = entity["class_name"].as_str().unwrap_or("Entity");
                    let entity_content = self.templates.render("entity.java", &entity_context)
                        .map_err(|e| format!("Failed to render entity {}: {}", entity_name, e))?;
                    files.insert(format!("src/main/java/{}/entity/{}.java", package_path, entity_name), entity_content);
                    
                    // Repository
                    let repo_content = self.templates.render("repository.java", &entity_context)
                        .map_err(|e| format!("Failed to render repository for {}: {}", entity_name, e))?;
                    files.insert(format!("src/main/java/{}/repository/{}Repository.java", package_path, entity_name), repo_content);
                }
            }
        }
        
        // Generate services from modules
        if let Some(modules) = context.get("modules") {
            if let Some(modules_array) = modules.as_array() {
                for module in modules_array {
                    let mut module_context = context.clone();
                    module_context.insert("module", module);
                    
                    let service_name = module["service_name"].as_str().unwrap_or("Service");
                    let service_content = self.templates.render("service.java", &module_context)
                        .map_err(|e| format!("Failed to render service {}: {}", service_name, e))?;
                    files.insert(format!("src/main/java/{}/service/{}.java", package_path, service_name), service_content);
                }
            }
        }
        
        // Generate pom.xml
        let pom_content = self.templates.render("pom.xml", &context)
            .map_err(|e| format!("Failed to render pom.xml: {}", e))?;
        files.insert("pom.xml".to_string(), pom_content);
        
        // Generate application.properties
        let props_content = self.templates.render("application.properties", &context)
            .map_err(|e| format!("Failed to render application.properties: {}", e))?;
        files.insert("src/main/resources/application.properties".to_string(), props_content);
        
        // Generate README.md
        let readme_content = self.templates.render("readme.md", &context)
            .map_err(|e| format!("Failed to render README.md: {}", e))?;
        files.insert("README.md".to_string(), readme_content);
        
        Ok(AdapterOutput {
            files,
            metadata: HashMap::new(),
        })
    }
    
    fn framework_name(&self) -> &str {
        "spring"
    }
    
    fn target_language(&self) -> &str {
        "java"
    }
}