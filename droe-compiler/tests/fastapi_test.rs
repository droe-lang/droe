#[cfg(test)]
mod tests {
    use droe_compiler::adapters::python::fastapi::FastAPIAdapter;
    use droe_compiler::adapters::{AdapterOptions, FrameworkAdapter};
    use droe_compiler::ast::{Program, Node, DataDefinition, DataField, ServeStatement};

    fn create_test_program() -> Program {
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
                    endpoint: "/api/users".to_string(),
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
    fn test_fastapi_adapter_generation() {
        let adapter = FastAPIAdapter::new();
        let program = create_test_program();
        let options = AdapterOptions::default();

        let result = adapter.generate(&program, options);
        assert!(result.is_ok());

        let output = result.unwrap();
        
        // Check that essential files are generated
        assert!(output.files.contains_key("requirements.txt"));
        assert!(output.files.contains_key("droe_app/main.py"));  // Default package name
        assert!(output.files.contains_key("droe_app/models.py"));
        assert!(output.files.contains_key("droe_app/routers.py"));
        assert!(output.files.contains_key("droe_app/core.py"));
        
        // Check content quality
        let main_py = output.files.get("droe_app/main.py").unwrap();
        assert!(main_py.contains("FastAPI"));
        assert!(main_py.contains("uvicorn"));
        
        let requirements = output.files.get("requirements.txt").unwrap();
        assert!(requirements.contains("fastapi"));
        assert!(requirements.contains("sqlalchemy"));
        
        let models_py = output.files.get("droe_app/models.py").unwrap();
        assert!(models_py.contains("class User(Base):"));
        
        let routers_py = output.files.get("droe_app/routers.py").unwrap();
        assert!(routers_py.contains("@router.get(\"/api/users\")"));
    }

    #[test]
    fn test_adapter_framework_info() {
        let adapter = FastAPIAdapter::new();
        
        assert_eq!(adapter.framework_name(), "fastapi");
        assert_eq!(adapter.target_language(), "python");
    }

    #[test]
    fn test_adapter_with_custom_options() {
        let adapter = FastAPIAdapter::new();
        let program = create_test_program();
        
        let mut options = AdapterOptions::default();
        options.package_name = Some("my_custom_app".to_string());
        options.database_type = Some("sqlite".to_string());

        let result = adapter.generate(&program, options);
        assert!(result.is_ok());

        let output = result.unwrap();
        
        // Should generate files with custom package name
        assert!(output.files.contains_key("my_custom_app/main.py"));
        assert!(output.files.contains_key("my_custom_app/models.py"));
        assert!(output.files.contains_key("my_custom_app/core.py"));
    }
}