use droe_compiler::adapters::{SpringAdapter, FrameworkAdapter, AdapterOptions};
use droe_compiler::ast::{Program, Node, DataDefinition, DataField};
use droe_compiler::codegen::{JavaCodeGenerator, CodeGenerator};

fn main() {
    // Create a simple Droe program with data
    let program = Program {
        statements: vec![
            Node::DataDefinition(DataDefinition {
                name: "User".to_string(),
                fields: vec![
                    DataField {
                        name: "id".to_string(),
                        field_type: "int".to_string(),
                        annotations: vec!["key".to_string(), "auto".to_string()],
                        line_number: Some(1),
                    },
                    DataField {
                        name: "username".to_string(),
                        field_type: "text".to_string(),
                        annotations: vec![],
                        line_number: Some(2),
                    },
                    DataField {
                        name: "email".to_string(),
                        field_type: "text".to_string(),
                        annotations: vec![],
                        line_number: Some(3),
                    },
                ],
                storage_type: Some("database".to_string()),
                line_number: Some(1),
            }),
        ],
        metadata: vec![],
        included_modules: None,
        line_number: None,
    };

    println!("=== DEMONSTRATING JAVA/SPRING BOOT SEPARATION ===\n");

    // 1. First, show core Java generation (language fundamentals only)
    println!("1. CORE JAVA GENERATION (Language features only):");
    println!("   - Pure Java classes and methods");
    println!("   - Basic collections and utilities");
    println!("   - No framework dependencies\n");

    let core_generator = JavaCodeGenerator::new();
    match core_generator.generate(&program) {
        Ok(core_code) => {
            println!("=== CORE JAVA CODE ===");
            println!("{}", core_code);
            println!("\n=== END CORE JAVA CODE ===\n");
        }
        Err(e) => {
            eprintln!("Error generating core Java: {}", e);
            return;
        }
    }

    // 2. Now show Spring Boot framework adapter (builds on top of core)
    println!("2. SPRING BOOT FRAMEWORK ADAPTER (Framework features on top of core):");
    println!("   - JPA Entity annotations");
    println!("   - Spring Boot application structure");
    println!("   - Repository pattern");
    println!("   - Maven dependencies in pom.xml\n");

    // Configure Spring Boot adapter
    let adapter = SpringAdapter::new();

    let mut options = AdapterOptions::default();
    options.package_name = Some("com.example.userservice".to_string());
    options.database_type = Some("postgresql".to_string());

    // Generate Spring Boot project
    match adapter.generate(&program, options) {
        Ok(output) => {
            println!("Successfully generated Spring Boot project!");
            println!("\nGenerated files:");
            for filename in output.files.keys() {
                println!("  - {}", filename);
            }

            // Show key framework files
            if let Some(main_content) = output.files.get("src/main/java/com/example/userservice/Application.java") {
                println!("\n=== Spring Boot Application.java ===");
                println!("{}", main_content);
            }

            if let Some(entity_content) = output.files.get("src/main/java/com/example/userservice/entity/User.java") {
                println!("\n=== JPA Entity (User.java) ===");
                println!("{}", entity_content);
            }

            if let Some(pom_content) = output.files.get("pom.xml") {
                println!("\n=== Maven Dependencies (pom.xml excerpt) ===");
                let lines: Vec<&str> = pom_content.lines().take(30).collect();
                println!("{}", lines.join("\n"));
                if pom_content.lines().count() > 30 {
                    println!("...\n[pom.xml continues with more dependencies]");
                }
            }
        }
        Err(e) => {
            eprintln!("Error generating Spring Boot project: {}", e);
        }
    }
}