use droe_compiler::{CompilerFactory, CompilerTarget, Framework, CompilerOptions, CompilationResult};
use droe_compiler::ast::{Program, Node, DataDefinition, DataField};

fn main() {
    // Create test program
    let program = Program {
        statements: vec![
            Node::DataDefinition(DataDefinition {
                name: "User".to_string(),
                fields: vec![
                    DataField {
                        name: "name".to_string(),
                        field_type: "text".to_string(),
                        annotations: vec![],
                        line_number: Some(1),
                    },
                    DataField {
                        name: "email".to_string(),
                        field_type: "text".to_string(),
                        annotations: vec![],
                        line_number: Some(2),
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

    println!("=== PROPER COMPILER FACTORY ARCHITECTURE ===\n");

    let factory = CompilerFactory::new();

    // Show proper separation: Core generators vs Framework adapters

    println!("1. CORE LANGUAGE GENERATION (Plain framework):");
    println!("   ✅ Core generators handle only language features");
    println!("   ✅ No framework knowledge in core generators\n");

    // Java core generation
    println!("Java core (plain):");
    let java_options = CompilerOptions {
        target: CompilerTarget::Java,
        framework: Framework::Plain,
        ..Default::default()
    };

    match factory.compile(&program, java_options) {
        Ok(CompilationResult::Code(code)) => {
            println!("Generated: {} characters of pure Java code", code.len());
            let lines: Vec<&str> = code.lines().take(5).collect();
            println!("Preview: {}", lines.join("\\n"));
        }
        Ok(CompilationResult::Project(_)) => {
            println!("❌ ERROR: Core generator returned project (should not happen)");
        }
        Err(e) => {
            println!("❌ ERROR: {}", e);
        }
    }

    println!();

    // Python core generation  
    println!("Python core (plain):");
    let python_options = CompilerOptions {
        target: CompilerTarget::Python,
        framework: Framework::Plain,
        ..Default::default()
    };

    match factory.compile(&program, python_options) {
        Ok(CompilationResult::Code(code)) => {
            println!("Generated: {} characters of pure Python code", code.len());
            let lines: Vec<&str> = code.lines().take(5).collect();
            println!("Preview: {}", lines.join("\\n"));
        }
        Ok(CompilationResult::Project(_)) => {
            println!("❌ ERROR: Core generator returned project (should not happen)");
        }
        Err(e) => {
            println!("❌ ERROR: {}", e);
        }
    }

    println!("\n2. FRAMEWORK ADAPTATION (Specific frameworks):");
    println!("   ✅ Framework adapters handle framework-specific features");
    println!("   ✅ Proper routing based on target + framework combination\n");

    // Spring Boot generation
    println!("Java + Spring Boot:");
    let spring_options = CompilerOptions {
        target: CompilerTarget::Java,
        framework: Framework::Spring,
        package_name: Some("com.example.demo".to_string()),
        ..Default::default()
    };

    match factory.compile(&program, spring_options) {
        Ok(CompilationResult::Project(project)) => {
            println!("Generated Spring Boot project with {} files:", project.files.len());
            for filename in project.files.keys() {
                println!("  - {}", filename);
            }
        }
        Ok(CompilationResult::Code(_)) => {
            println!("❌ ERROR: Framework adapter returned simple code (should not happen)");
        }
        Err(e) => {
            println!("❌ ERROR: {}", e);
        }
    }

    println!();

    // FastAPI generation
    println!("Python + FastAPI:");
    let fastapi_options = CompilerOptions {
        target: CompilerTarget::Python,
        framework: Framework::FastAPI,
        package_name: Some("user_service".to_string()),
        ..Default::default()
    };

    match factory.compile(&program, fastapi_options) {
        Ok(CompilationResult::Project(project)) => {
            println!("Generated FastAPI project with {} files:", project.files.len());
            for filename in project.files.keys() {
                println!("  - {}", filename);
            }
        }
        Ok(CompilationResult::Code(_)) => {
            println!("❌ ERROR: Framework adapter returned simple code (should not happen)");
        }
        Err(e) => {
            println!("❌ ERROR: {}", e);
        }
    }

    println!("\n3. INVALID COMBINATIONS (Proper error handling):");
    println!("   ✅ Factory validates target + framework compatibility\n");

    // Invalid combination
    println!("Python + Spring (invalid):");
    let invalid_options = CompilerOptions {
        target: CompilerTarget::Python,
        framework: Framework::Spring, // Spring is for Java, not Python
        ..Default::default()
    };

    match factory.compile(&program, invalid_options) {
        Ok(_) => {
            println!("❌ ERROR: Should have failed for invalid combination");
        }
        Err(e) => {
            println!("✅ Correctly rejected: {}", e);
        }
    }

    println!("\n=== ARCHITECTURE COMPARISON ===");
    println!("❌ BEFORE: Java codegen knew about Spring (violated separation)");
    println!("✅ AFTER:  CompilerFactory routes to correct adapter");
    println!("❌ BEFORE: Direct codegen instantiation");  
    println!("✅ AFTER:  Factory-based routing like Python implementation");
    println!("❌ BEFORE: Framework logic mixed in core generators");
    println!("✅ AFTER:  Clean separation of language vs framework concerns");
}