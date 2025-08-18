use droe_compiler::codegen::bytecode::BytecodeGenerator;
use droe_compiler::codegen::CodeGenerator;
use droe_compiler::ast::*;
use serde_json::Value;

#[test]
fn test_simple_display_program() {
    let program = Program {
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
    };

    let generator = BytecodeGenerator::new();
    let result = generator.generate(&program).unwrap();
    
    // Parse the JSON result
    let bytecode: Value = serde_json::from_str(&result).unwrap();
    
    // Verify structure
    assert_eq!(bytecode["version"], 1);
    assert!(bytecode["metadata"].is_object());
    assert!(bytecode["instructions"].is_array());
    
    let instructions = bytecode["instructions"].as_array().unwrap();
    
    // Should have Push, Display, and Halt instructions
    assert_eq!(instructions.len(), 3);
    
    // First instruction should be Push with string
    assert!(instructions[0]["Push"]["String"] == "Hello World");
    
    // Second instruction should be Display
    assert_eq!(instructions[1], "Display");
    
    // Third instruction should be Halt
    assert_eq!(instructions[2], "Halt");
}

#[test]
fn test_assignment_and_arithmetic() {
    let program = Program {
        statements: vec![
            // x = 5
            Node::Assignment(Assignment {
                variable: "x".to_string(),
                value: Box::new(Node::Literal(Literal {
                    value: LiteralValue::Integer(5),
                    literal_type: "int".to_string(),
                    line_number: Some(1),
                })),
                line_number: Some(1),
            }),
            // y = x + 10
            Node::Assignment(Assignment {
                variable: "y".to_string(),
                value: Box::new(Node::ArithmeticOp(ArithmeticOp {
                    left: Box::new(Node::Identifier(Identifier {
                        name: "x".to_string(),
                        line_number: Some(2),
                    })),
                    operator: "+".to_string(),
                    right: Box::new(Node::Literal(Literal {
                        value: LiteralValue::Integer(10),
                        literal_type: "int".to_string(),
                        line_number: Some(2),
                    })),
                    line_number: Some(2),
                })),
                line_number: Some(2),
            }),
            // Display y
            Node::DisplayStatement(DisplayStatement {
                expression: Box::new(Node::Identifier(Identifier {
                    name: "y".to_string(),
                    line_number: Some(3),
                })),
                line_number: Some(3),
            }),
        ],
        metadata: vec![],
        included_modules: None,
        line_number: None,
    };

    let generator = BytecodeGenerator::new();
    let result = generator.generate(&program).unwrap();
    
    // Parse the JSON result
    let bytecode: Value = serde_json::from_str(&result).unwrap();
    let instructions = bytecode["instructions"].as_array().unwrap();
    
    // Should have multiple instructions for assignments and arithmetic
    assert!(instructions.len() > 5);
    
    // Verify we have Push, StoreVar, LoadVar, Add, Display, Halt instructions
    let mut has_push = false;
    let mut has_store_var = false;
    let mut has_load_var = false;
    let mut has_add = false;
    let mut has_display = false;
    let mut has_halt = false;
    
    for instruction in instructions {
        match instruction {
            Value::Object(obj) if obj.contains_key("Push") => has_push = true,
            Value::Object(obj) if obj.contains_key("StoreVar") => has_store_var = true,
            Value::Object(obj) if obj.contains_key("LoadVar") => has_load_var = true,
            Value::String(s) if s == "Add" => has_add = true,
            Value::String(s) if s == "Display" => has_display = true,
            Value::String(s) if s == "Halt" => has_halt = true,
            _ => {}
        }
    }
    
    assert!(has_push, "Should have Push instruction");
    assert!(has_store_var, "Should have StoreVar instruction");
    assert!(has_load_var, "Should have LoadVar instruction");
    assert!(has_add, "Should have Add instruction");
    assert!(has_display, "Should have Display instruction");
    assert!(has_halt, "Should have Halt instruction");
}

#[test]
fn test_if_statement_with_labels() {
    let program = Program {
        statements: vec![
            Node::IfStatement(IfStatement {
                condition: Box::new(Node::BinaryOp(BinaryOp {
                    left: Box::new(Node::Literal(Literal {
                        value: LiteralValue::Integer(5),
                        literal_type: "int".to_string(),
                        line_number: Some(1),
                    })),
                    operator: ">".to_string(),
                    right: Box::new(Node::Literal(Literal {
                        value: LiteralValue::Integer(3),
                        literal_type: "int".to_string(),
                        line_number: Some(1),
                    })),
                    line_number: Some(1),
                })),
                then_body: vec![
                    Node::DisplayStatement(DisplayStatement {
                        expression: Box::new(Node::Literal(Literal {
                            value: LiteralValue::String("Greater".to_string()),
                            literal_type: "string".to_string(),
                            line_number: Some(2),
                        })),
                        line_number: Some(2),
                    }),
                ],
                else_body: Some(vec![
                    Node::DisplayStatement(DisplayStatement {
                        expression: Box::new(Node::Literal(Literal {
                            value: LiteralValue::String("Lesser".to_string()),
                            literal_type: "string".to_string(),
                            line_number: Some(4),
                        })),
                        line_number: Some(4),
                    }),
                ]),
                line_number: Some(1),
            }),
        ],
        metadata: vec![],
        included_modules: None,
        line_number: None,
    };

    let generator = BytecodeGenerator::new();
    let result = generator.generate(&program).unwrap();
    
    // Parse the JSON result
    let bytecode: Value = serde_json::from_str(&result).unwrap();
    let instructions = bytecode["instructions"].as_array().unwrap();
    
    // Should have jump instructions for if/else control flow
    let mut has_jump_if_false = false;
    let mut has_jump = false;
    
    for instruction in instructions {
        match instruction {
            Value::Object(obj) if obj.contains_key("JumpIfFalse") => has_jump_if_false = true,
            Value::Object(obj) if obj.contains_key("Jump") => has_jump = true,
            _ => {}
        }
    }
    
    assert!(has_jump_if_false, "Should have JumpIfFalse instruction for if condition");
    assert!(has_jump, "Should have Jump instruction for else branch");
}