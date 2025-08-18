use droe_compiler::adapters::{FastAPIAdapter, FrameworkAdapter, AdapterOptions};
use droe_compiler::ast::{Program, Node, DataDefinition, DataField, ServeStatement};
use droe_compiler::codegen::{PythonGenerator, CodeGenerator};

fn main() {
    // Create a simple Droe program with data and API endpoints
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
                        name: "name".to_string(),
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
            Node::ServeStatement(ServeStatement {
                method: "GET".to_string(),
                endpoint: "/api/users".to_string(),
                body: vec![],
                params: vec![],
                accept_type: None,
                response_action: None,
                line_number: Some(4),
            }),
            Node::ServeStatement(ServeStatement {
                method: "POST".to_string(),
                endpoint: "/api/users".to_string(),
                body: vec![],
                params: vec![],
                accept_type: None,
                response_action: None,
                line_number: Some(5),
            }),
        ],
        metadata: vec![],
        included_modules: None,
        line_number: None,
    };

    println!("=== DEMONSTRATING PROPER SEPARATION OF CONCERNS ===\n");

    // 1. First, show core Python generation (language fundamentals only)
    println!("1. CORE PYTHON GENERATION (Language features only):");
    println!("   - Direct database operations using sqlite3");
    println!("   - Basic HTTP using http.client");
    println!("   - Pure Python classes and methods\n");

    let core_generator = PythonGenerator::new().with_class_name("UserService");
    match core_generator.generate(&program) {
        Ok(core_code) => {
            println!("=== CORE PYTHON CODE ===");
            println!("{}", core_code);
            println!("\n=== END CORE PYTHON CODE ===\n");
        }
        Err(e) => {
            eprintln!("Error generating core Python: {}", e);
            return;
        }
    }

    // 2. Now show FastAPI framework adapter (builds on top of core)
    println!("2. FASTAPI FRAMEWORK ADAPTER (Framework features on top of core):");
    println!("   - SQLAlchemy ORM models");
    println!("   - Pydantic schemas");  
    println!("   - FastAPI routers and endpoints");
    println!("   - Database session management\n");

    // Configure FastAPI adapter
    let adapter = FastAPIAdapter::new()
        .with_package_name("user_service")
        .with_database("postgresql");

    let mut options = AdapterOptions::default();
    options.package_name = Some("user_service".to_string());
    options.database_type = Some("postgresql".to_string());

    // Generate FastAPI project
    match adapter.generate(&program, options) {
        Ok(output) => {
            println!("Successfully generated FastAPI project!");
            println!("\nGenerated files:");
            for filename in output.files.keys() {
                println!("  - {}", filename);
            }

            // Show key framework files
            if let Some(main_content) = output.files.get("user_service/main.py") {
                println!("\n=== FastAPI main.py (Framework Layer) ===");
                println!("{}", main_content);
            }

            if let Some(core_content) = output.files.get("user_service/core.py") {
                println!("\n=== Core business logic (Core Layer) ===");
                let lines: Vec<&str> = core_content.lines().take(20).collect();
                println!("{}...", lines.join("\n"));
                if core_content.lines().count() > 20 {
                    println!("\n[Core file continues with business logic...]");
                }
            }

            if let Some(models_content) = output.files.get("user_service/models.py") {
                println!("\n=== SQLAlchemy Models (ORM Layer) ===");
                println!("{}", models_content);
            }

            // Show requirements.txt
            if let Some(requirements) = output.files.get("requirements.txt") {
                println!("\n=== requirements.txt ===");
                println!("{}", requirements);
            }
        }
        Err(e) => {
            eprintln!("Error generating FastAPI project: {}", e);
        }
    }
}