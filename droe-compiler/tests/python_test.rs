#[cfg(test)]
mod tests {
    use droe_compiler::codegen::{python::PythonGenerator, CodeGenerator};
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
            line_number: None,
        }
    }

    fn create_data_program() -> Program {
        Program {
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
                Node::ServeStatement(ServeStatement {
                    method: "GET".to_string(),
                    endpoint: "/users".to_string(),
                    body: vec![],
                    params: vec![],
                    accept_type: None,
                    response_action: None,
                    line_number: Some(3),
                }),
            ],
            metadata: vec![],
            included_modules: None,
            line_number: None,
        }
    }

    #[test]
    fn test_core_python_generation() {
        let generator = PythonGenerator::new().with_class_name("TestProgram");
        let program = create_simple_program();
        
        let result = generator.generate(&program);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("print(\"Hello World\")"));
        assert!(code.contains("def main():"));
        assert!(code.contains("class Testprogram:"));
        assert!(code.contains("sqlite3"));  // Core uses direct database access
        assert!(code.contains("http.client"));  // Core uses basic HTTP
    }

    #[test]
    fn test_core_python_with_data() {
        let generator = PythonGenerator::new().with_class_name("DataProgram");
        let program = create_data_program();
        
        let result = generator.generate(&program);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("@dataclass"));
        assert!(code.contains("class User:"));
        assert!(code.contains("name: str"));
        assert!(code.contains("email: str"));
        // Should contain basic HTTP server comment for serve statements
        assert!(code.contains("# Serve GET /users"));
        assert!(code.contains("# Implementation would use http.server"));
    }

}