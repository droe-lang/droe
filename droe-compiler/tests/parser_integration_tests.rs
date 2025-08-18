//! Integration tests for the modular parser

use droe_compiler::{Parser, Node};

#[test]
fn test_ui_component_parsing() {
    let source = r#"
title "Welcome"
button "Click Me"
input "text"
"#;
    
    let parser = Parser::new();
    let result = parser.parse(source);
    
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 3);
    
    // Check that we got the right component types
    match &program.statements[0] {
        Node::TitleComponent(title) => {
            assert_eq!(title.text, "Welcome");
        }
        _ => panic!("Expected TitleComponent"),
    }
    
    match &program.statements[1] {
        Node::ButtonComponent(button) => {
            assert_eq!(button.text, "Click Me");
        }
        _ => panic!("Expected ButtonComponent"),
    }
    
    match &program.statements[2] {
        Node::InputComponent(input) => {
            assert_eq!(input.input_type, "text");
        }
        _ => panic!("Expected InputComponent"),
    }
}

#[test]
fn test_api_call_parsing() {
    let source = r#"
call GET "/api/users"
fetch POST "/api/data"
"#;
    
    let parser = Parser::new();
    let result = parser.parse(source);
    
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 2);
    
    match &program.statements[0] {
        Node::ApiCallStatement(api) => {
            assert_eq!(api.verb, "call");
            assert_eq!(api.method, "GET");
            assert_eq!(api.endpoint, "/api/users");
        }
        _ => panic!("Expected ApiCallStatement"),
    }
}

#[test]
fn test_database_statement_parsing() {
    let source = r#"
db create users
db select posts
"#;
    
    let parser = Parser::new();
    let result = parser.parse(source);
    
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 2);
    
    match &program.statements[0] {
        Node::DatabaseStatement(db) => {
            assert_eq!(db.operation, "create");
            assert_eq!(db.entity_name, "users");
        }
        _ => panic!("Expected DatabaseStatement"),
    }
}

#[test]
fn test_form_definition_parsing() {
    let source = r#"
form UserForm
    input "text"
    button "Submit"
end form
"#;
    
    let parser = Parser::new();
    let result = parser.parse(source);
    
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);
    
    match &program.statements[0] {
        Node::FormDefinition(form) => {
            assert_eq!(form.name, "UserForm");
            assert_eq!(form.children.len(), 2);
        }
        _ => panic!("Expected FormDefinition"),
    }
}

#[test]
fn test_string_interpolation_expression() {
    // This test would require more sophisticated expression parsing
    // For now, just test that basic expressions work
    let source = r#"
display "Hello World"
"#;
    
    let parser = Parser::new();
    let result = parser.parse(source);
    
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);
    
    match &program.statements[0] {
        Node::DisplayStatement(_) => {
            // Success - we can parse display statements
        }
        _ => panic!("Expected DisplayStatement"),
    }
}

#[test]
fn test_module_and_data_definitions() {
    let source = r#"
module UserModule
    data User
        name is string
        age is number
    end data
    
    action greet
        display "Hello"
    end action
end module
"#;
    
    let parser = Parser::new();
    let result = parser.parse(source);
    
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);
    
    match &program.statements[0] {
        Node::ModuleDefinition(module) => {
            assert_eq!(module.name, "UserModule");
            assert_eq!(module.body.len(), 2); // data and action
        }
        _ => panic!("Expected ModuleDefinition"),
    }
}

#[test] 
fn test_screen_definition_parsing() {
    let source = r#"
screen MainScreen
    title "Welcome"
    button "Start"
end screen
"#;
    
    let parser = Parser::new();
    let result = parser.parse(source);
    
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);
    
    match &program.statements[0] {
        Node::ScreenDefinition(screen) => {
            assert_eq!(screen.name, "MainScreen");
        }
        _ => panic!("Expected ScreenDefinition"),
    }
}

#[test]
fn test_serve_statement_parsing() {
    let source = r#"
serve GET "/api/health"
serve POST "/api/users"
"#;
    
    let parser = Parser::new();
    let result = parser.parse(source);
    
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 2);
    
    match &program.statements[0] {
        Node::ServeStatement(serve) => {
            assert_eq!(serve.method, "GET");
            assert_eq!(serve.endpoint, "/api/health");
        }
        _ => panic!("Expected ServeStatement"),
    }
}