#[cfg(test)]
mod tests {
    use droe_compiler::codegen::rust::{RustGenerator, DatabaseConfig, to_snake_case, capitalize_first};
    use droe_compiler::codegen::CodeGenerator;
    use droe_compiler::ast::{Program, Node, DisplayStatement, Literal, LiteralValue, DataDefinition, DataField, ServeStatement};

    fn create_simple_program() -> Program {
        Program {
            statements: vec![
                Node::DisplayStatement(DisplayStatement {
                    expression: Box::new(Node::Literal(Literal {
                        value: LiteralValue::String("Hello World".to_string()),
                        literal_type: "string".to_string(),
                        line_number: Some(1),
                    })),
                    line_number: Some(1),
                }),
            ],
            metadata: vec![],
            included_modules: None,
            line_number: Some(1),
        }
    }

    fn create_program_with_data_and_serve() -> Program {
        Program {
            statements: vec![
                Node::DataDefinition(DataDefinition {
                    name: "User".to_string(),
                    fields: vec![
                        DataField {
                            name: "id".to_string(),
                            field_type: "text".to_string(),
                            annotations: vec!["key".to_string(), "auto".to_string()],
                            line_number: Some(2),
                        },
                        DataField {
                            name: "name".to_string(),
                            field_type: "text".to_string(),
                            annotations: vec![],
                            line_number: Some(3),
                        },
                        DataField {
                            name: "email".to_string(),
                            field_type: "text".to_string(),
                            annotations: vec![],
                            line_number: Some(4),
                        },
                    ],
                    storage_type: None,
                    line_number: Some(1),
                }),
                Node::ServeStatement(ServeStatement {
                    method: "GET".to_string(),
                    endpoint: "/users".to_string(),
                    body: vec![],
                    accept_type: None,
                    params: vec![],
                    response_action: None,
                    line_number: Some(6),
                }),
                Node::ServeStatement(ServeStatement {
                    method: "POST".to_string(),
                    endpoint: "/users".to_string(),
                    body: vec![],
                    accept_type: None,
                    params: vec![],
                    response_action: None,
                    line_number: Some(7),
                }),
            ],
            metadata: vec![],
            included_modules: None,
            line_number: Some(1),
        }
    }

    #[test]
    fn test_rust_generator_creation() {
        let generator = RustGenerator::new(None, false, None, None, None);
        assert!(generator.is_ok());
    }

    #[test]
    fn test_simple_program_generation() {
        let program = create_simple_program();
        let generator = RustGenerator::new(None, false, None, None, None).unwrap();
        
        let result = generator.generate(&program);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("Generated Rust project"));
    }

    #[test]
    fn test_program_with_data_and_endpoints() {
        let program = create_program_with_data_and_serve();
        let generator = RustGenerator::new(
            None, 
            true, 
            Some("axum".to_string()), 
            Some("test_app".to_string()), 
            None
        ).unwrap();
        
        let result = generator.generate(&program);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("Generated Rust project"));
        // Should contain main components for a web API project
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("main.rs"));
        assert!(output.contains("models.rs"));
        assert!(output.contains("handlers.rs"));
    }

    #[test]
    fn test_database_config() {
        let db_config = DatabaseConfig {
            db_type: "postgres".to_string(),
            url: "postgresql://localhost/test".to_string(),
        };
        
        let generator = RustGenerator::new(
            None, 
            false, 
            None, 
            None, 
            Some(db_config)
        );
        
        assert!(generator.is_ok());
    }

    #[test]
    fn test_framework_configuration() {
        let generator = RustGenerator::new(
            Some("src/main.droe".to_string()), 
            true, 
            Some("axum".to_string()), 
            Some("my_app".to_string()), 
            None
        );
        
        assert!(generator.is_ok());
    }

    #[test]
    fn test_snake_case_conversion() {
        assert_eq!(to_snake_case("CamelCase"), "camel_case");
        assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
        assert_eq!(to_snake_case("simpleword"), "simpleword");
        assert_eq!(to_snake_case("ID"), "id");
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("postgres"), "Postgres");
        assert_eq!(capitalize_first("mysql"), "Mysql");
        assert_eq!(capitalize_first("sqlite"), "Sqlite");
        assert_eq!(capitalize_first(""), "");
    }
}