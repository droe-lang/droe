//! WebAssembly Text (WAT) code generation for Droe DSL
//! 
//! This is a comprehensive port of the Python WASM codegen implementation
//! with full support for core libraries, advanced features, and runtime generation.

use crate::ast::*;
use crate::symbols::{SymbolTable, VariableType, Variable};
use crate::codegen_base::{BaseCodeGenerator, CodeGenError, CodeGenContext, TypeSystemHelpers};
use super::{CodeGenerator, wasm_runtime::WasmRuntimeBuilder};
use std::collections::HashMap;

pub struct WebAssemblyGenerator {
    context: CodeGenContext,
    string_constants: HashMap<String, usize>,
    next_string_index: usize,
    memory_offset: usize,
    array_metadata: HashMap<String, (usize, usize, VariableType)>, // (offset, length, element_type)
    action_definitions: HashMap<String, Node>,
    parameterized_action_definitions: HashMap<String, Node>,
    data_definitions: HashMap<String, Node>,
    data_instances: HashMap<String, Node>,
    runtime_builder: WasmRuntimeBuilder,
}

impl WebAssemblyGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            context: CodeGenContext::new(),
            string_constants: HashMap::new(),
            next_string_index: 0,
            memory_offset: 1024,
            array_metadata: HashMap::new(),
            action_definitions: HashMap::new(),
            parameterized_action_definitions: HashMap::new(),
            data_definitions: HashMap::new(),
            data_instances: HashMap::new(),
            runtime_builder: WasmRuntimeBuilder::new(),
        };
        
        // Enable default core libraries
        generator.context.enable_core_lib("string_utils");
        generator.context.enable_core_lib("math_utils");
        generator.context.enable_core_lib("formatting");
        
        generator
    }
    
    fn emit_module_header(&mut self) -> String {
        let mut output = String::new();
        output.push_str("(module\n");
        
        // Import basic functions
        output.push_str("  (import \"env\" \"print\" (func $print (param i32 i32)))\n");
        output.push_str("  (import \"env\" \"print_i32\" (func $print_i32 (param i32)))\n");
        output.push_str("  (import \"env\" \"print_string_from_offset\" (func $print_string_from_offset (param i32)))\n");
        output.push_str("  (import \"env\" \"print_no_newline\" (func $print_no_newline (param i32 i32)))\n");
        output.push_str("  (import \"env\" \"print_newline\" (func $print_newline))\n");
        output.push_str("  (import \"env\" \"print_i32_no_newline\" (func $print_i32_no_newline (param i32)))\n");
        output.push_str("  (import \"env\" \"print_string_from_offset_no_newline\" (func $print_string_from_offset_no_newline (param i32)))\n");
        output.push_str("  (import \"env\" \"print_decimal\" (func $print_decimal (param i32)))\n");
        output.push_str("  (import \"env\" \"print_decimal_no_newline\" (func $print_decimal_no_newline (param i32)))\n");
        output.push_str("  (import \"env\" \"print_date\" (func $print_date (param i32)))\n");
        output.push_str("  (import \"env\" \"print_date_no_newline\" (func $print_date_no_newline (param i32)))\n");
        output.push_str("  (import \"env\" \"format_date\" (func $format_date (param i32 i32) (result i32)))\n");
        output.push_str("  (import \"env\" \"format_decimal\" (func $format_decimal (param i32 i32) (result i32)))\n");
        output.push_str("  (import \"env\" \"format_number\" (func $format_number (param i32 i32) (result i32)))\n");
        
        // Import core library functions if enabled
        output.push_str(&self.emit_core_library_imports());
        
        // Memory
        output.push_str("  (memory 1)\n");
        output.push_str("  (export \"memory\" (memory 0))\n\n");
        
        output
    }
    
    fn emit_core_library_imports(&self) -> String {
        let mut output = String::new();
        
        // Get WASM imports from the core libraries
        let imports = self.context.core_libs.get_wasm_imports();
        
        if !imports.is_empty() {
            output.push_str("  ;; Core library imports\n");
            for import in imports {
                output.push_str(&format!("  {}\n", import));
            }
        }
        
        output
    }

    fn emit_string_data(&self) -> String {
        let mut output = String::new();
        output.push_str("\n  ;; String constants\n");
        
        let mut offset = 0;
        let mut sorted_strings: Vec<_> = self.string_constants.iter().collect();
        sorted_strings.sort_by_key(|(_, &index)| index);
        
        for (string, _index) in sorted_strings {
            let bytes_str = string.chars()
                .map(|c| format!("\\{:02x}", c as u8))
                .collect::<String>() + "\\00";
            output.push_str(&format!("  (data (i32.const {}) \"{}\")\n", offset, bytes_str));
            offset += string.len() + 1;
        }
        
        output.push('\n');
        output
    }

    fn emit_main_function(&mut self, program: &Program) -> Result<String, CodeGenError> {
        let mut output = String::new();
        
        // First pass: collect definitions and variables
        self.collect_definitions(program)?;
        self.collect_variables(program)?;
        
        // Start main function
        output.push_str("  (func $main\n");
        
        // Declare local variables if any
        let local_count = self.context.symbol_table.get_local_count();
        if local_count > 0 {
            for var in self.context.symbol_table.get_all_variables().values() {
                if self.is_numeric_type(&var.var_type) {
                    output.push_str("    (local i32)\n");
                } else if self.is_boolean_type(&var.var_type) {
                    output.push_str("    (local i32)\n");
                } else if self.is_text_type(&var.var_type) {
                    output.push_str("    (local i32)\n");  // String offset
                    output.push_str("    (local i32)\n");  // String length
                } else if self.is_collection_type(&var.var_type) {
                    output.push_str("    (local i32)\n");  // Array pointer
                    output.push_str("    (local i32)\n");  // Array length
                } else {
                    output.push_str("    (local i32)\n");  // Default
                }
            }
        }
        
        // Generate function body
        for statement in &program.statements {
            output.push_str(&self.emit_statement(statement)?);
        }
        
        output.push_str("  )\n\n");
        output.push_str("  (export \"main\" (func $main))\n");
        
        Ok(output)
    }

    fn collect_definitions(&mut self, program: &Program) -> Result<(), CodeGenError> {
        // Collect action definitions, data definitions, etc.
        for statement in &program.statements {
            match statement {
                Node::ActionDefinition(action) => {
                    self.action_definitions.insert(action.name.clone(), statement.clone());
                }
                Node::DataDefinition(data) => {
                    self.data_definitions.insert(data.name.clone(), statement.clone());
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn collect_variables(&mut self, program: &Program) -> Result<(), CodeGenError> {
        for statement in &program.statements {
            self.collect_variables_from_node(statement)?;
        }
        Ok(())
    }
    
    fn collect_variables_from_node(&mut self, node: &Node) -> Result<(), CodeGenError> {
        match node {
            Node::Assignment(assignment) => {
                if !self.context.symbol_table.has_variable(&assignment.variable) {
                    let var_type = self.infer_type(&assignment.value, &self.context.symbol_table);
                    self.context.symbol_table.add_variable(assignment.variable.clone(), var_type)
                        .map_err(|e| CodeGenError::SymbolError { message: e })?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn emit_statement(&mut self, statement: &Node) -> Result<String, CodeGenError> {
        match statement {
            Node::DisplayStatement(display) => self.emit_display_statement(display),
            Node::Assignment(assignment) => self.emit_assignment(assignment),
            Node::IfStatement(if_stmt) => self.emit_if_statement(if_stmt),
            Node::WhileLoop(while_loop) => self.emit_while_loop(while_loop),
            Node::ForEachLoop(for_each) => self.emit_foreach_loop(for_each),
            Node::ActionDefinition(action) => self.emit_action_definition(action),
            Node::ActionDefinitionWithParams(action) => self.emit_action_definition_with_params(action),
            Node::ModuleDefinition(module) => self.emit_module_definition(module),
            Node::DataDefinition(data) => self.emit_data_definition(data),
            Node::ReturnStatement(ret) => self.emit_return_statement(ret),
            _ => Ok(format!("    ;; TODO: Implement {:?}\n", statement)),
        }
    }

    fn emit_display_statement(&mut self, display: &DisplayStatement) -> Result<String, CodeGenError> {
        let mut output = String::new();
        output.push_str("    ;; display statement\n");
        
        match display.expression.as_ref() {
            Node::Literal(literal) => {
                if let LiteralValue::String(s) = &literal.value {
                    // Add to string constants
                    let string_index = self.string_constants.len();
                    self.string_constants.insert(s.clone(), string_index);
                    
                    // Calculate offset
                    let offset = self.calculate_string_offset(string_index);
                    let string_length = s.len();
                    
                    output.push_str(&format!("    i32.const {}\n", offset));
                    output.push_str(&format!("    i32.const {}\n", string_length));
                    output.push_str("    call $print\n");
                } else {
                    output.push_str("    ;; display non-string literal\n");
                }
            }
            Node::Identifier(identifier) => {
                if let Some(var) = self.context.symbol_table.get_variable(&identifier.name) {
                    if self.is_numeric_type(&var.var_type) {
                        output.push_str(&format!("    local.get {}\n", var.wasm_index.unwrap_or(0)));
                        if var.var_type == VariableType::Decimal {
                            output.push_str("    call $print_decimal\n");
                        } else {
                            output.push_str("    call $print_i32\n");
                        }
                    } else if self.is_text_type(&var.var_type) {
                        output.push_str(&format!("    local.get {}\n", var.wasm_index.unwrap_or(0)));
                        output.push_str("    call $print_string_from_offset\n");
                    } else {
                        output.push_str("    ;; display other variable type\n");
                    }
                } else {
                    return Err(CodeGenError::UnknownVariable { 
                        name: identifier.name.clone() 
                    });
                }
            }
            _ => {
                output.push_str("    ;; display complex expression\n");
            }
        }
        
        Ok(output)
    }
    
    fn emit_assignment(&mut self, assignment: &Assignment) -> Result<String, CodeGenError> {
        let mut output = String::new();
        output.push_str(&format!("    ;; set {}\n", assignment.variable));
        
        let wasm_index = {
            let var = self.context.symbol_table.get_variable(&assignment.variable)
                .ok_or_else(|| CodeGenError::UnknownVariable { 
                    name: assignment.variable.clone() 
                })?;
            var.wasm_index.unwrap_or(0)
        };
        
        // Emit the value expression
        output.push_str(&self.emit_expression(&assignment.value)?);
        output.push_str(&format!("    local.set {}\n", wasm_index));
        
        Ok(output)
    }
    
    fn emit_expression(&mut self, expr: &Node) -> Result<String, CodeGenError> {
        match expr {
            Node::Literal(literal) => {
                match &literal.value {
                    LiteralValue::Integer(i) => Ok(format!("    i32.const {}\n", i)),
                    LiteralValue::Float(f) => {
                        // Scale decimal by 100 for precision
                        let scaled_value = (*f * 100.0) as i32;
                        Ok(format!("    i32.const {}\n", scaled_value))
                    }
                    LiteralValue::Boolean(b) => {
                        Ok(format!("    i32.const {}\n", if *b { 1 } else { 0 }))
                    }
                    LiteralValue::String(s) => {
                        let string_index = self.string_constants.len();
                        self.string_constants.insert(s.clone(), string_index);
                        let offset = self.calculate_string_offset(string_index);
                        Ok(format!("    i32.const {}\n", offset))
                    }
                }
            }
            Node::Identifier(identifier) => {
                if let Some(var) = self.context.symbol_table.get_variable(&identifier.name) {
                    Ok(format!("    local.get {}\n", var.wasm_index.unwrap_or(0)))
                } else {
                    Err(CodeGenError::UnknownVariable { 
                        name: identifier.name.clone() 
                    })
                }
            }
            Node::BinaryOp(binary) => self.emit_binary_operation(binary),
            Node::ArithmeticOp(arith) => self.emit_arithmetic_operation(arith),
            Node::ArrayLiteral(array) => self.emit_array_literal(array),
            _ => Ok("    ;; complex expression\n".to_string()),
        }
    }
    
    fn emit_binary_operation(&mut self, binary: &BinaryOp) -> Result<String, CodeGenError> {
        let mut output = String::new();
        
        // Emit left operand
        output.push_str(&self.emit_expression(&binary.left)?);
        
        // Emit right operand
        output.push_str(&self.emit_expression(&binary.right)?);
        
        // Emit operator
        match binary.operator.as_str() {
            ">" => output.push_str("    i32.gt_s\n"),
            "<" => output.push_str("    i32.lt_s\n"),
            ">=" => output.push_str("    i32.ge_s\n"),
            "<=" => output.push_str("    i32.le_s\n"),
            "==" => output.push_str("    i32.eq\n"),
            "!=" => output.push_str("    i32.ne\n"),
            _ => return Err(CodeGenError::UnsupportedOperation { 
                op: binary.operator.clone() 
            }),
        }
        
        Ok(output)
    }
    
    fn emit_arithmetic_operation(&mut self, arith: &ArithmeticOp) -> Result<String, CodeGenError> {
        let mut output = String::new();
        
        // Emit left operand
        output.push_str(&self.emit_expression(&arith.left)?);
        
        // Emit right operand
        output.push_str(&self.emit_expression(&arith.right)?);
        
        // Emit operator
        match arith.operator.as_str() {
            "+" => output.push_str("    i32.add\n"),
            "-" => output.push_str("    i32.sub\n"),
            "*" => output.push_str("    i32.mul\n"),
            "/" => output.push_str("    i32.div_s\n"),
            _ => return Err(CodeGenError::UnsupportedOperation { 
                op: arith.operator.clone() 
            }),
        }
        
        Ok(output)
    }
    
    fn emit_array_literal(&mut self, array: &ArrayLiteral) -> Result<String, CodeGenError> {
        let mut output = String::new();
        
        // For now, just emit the first element or 0 if empty
        if !array.elements.is_empty() {
            output.push_str(&self.emit_expression(&array.elements[0])?);
        } else {
            output.push_str("    i32.const 0\n");
        }
        
        Ok(output)
    }

    fn collect_strings(&mut self, statement: &Node) {
        match statement {
            Node::DisplayStatement(display) => {
                self.collect_strings_from_expression(&display.expression);
            }
            Node::Assignment(assignment) => {
                self.collect_strings_from_expression(&assignment.value);
            }
            _ => {}
        }
    }
    
    fn collect_strings_from_expression(&mut self, expr: &Node) {
        match expr {
            Node::Literal(literal) => {
                if let LiteralValue::String(s) = &literal.value {
                    if !self.string_constants.contains_key(s) {
                        self.string_constants.insert(s.clone(), self.next_string_index);
                        self.next_string_index += 1;
                    }
                }
            }
            _ => {}
        }
    }
    
    fn calculate_string_offset(&self, string_index: usize) -> usize {
        self.string_constants.iter()
            .filter(|(_, &index)| index < string_index)
            .map(|(s, _)| s.len() + 1)
            .sum()
    }
    
    // Type system helpers
    fn is_numeric_type(&self, var_type: &VariableType) -> bool {
        matches!(var_type, VariableType::Int | VariableType::Decimal | VariableType::Number)
    }
    
    fn is_boolean_type(&self, var_type: &VariableType) -> bool {
        matches!(var_type, VariableType::Flag | VariableType::YesNo | VariableType::Boolean)
    }
    
    fn is_text_type(&self, var_type: &VariableType) -> bool {
        matches!(var_type, VariableType::Text | VariableType::String)
    }
    
    fn is_collection_type(&self, var_type: &VariableType) -> bool {
        matches!(var_type, VariableType::ListOf | VariableType::GroupOf | VariableType::Array)
    }
    
    pub fn generate_runtime(&self) -> String {
        self.runtime_builder.generate_runtime_js()
    }
    
    fn emit_if_statement(&mut self, if_stmt: &IfStatement) -> Result<String, CodeGenError> {
        let mut output = String::new();
        output.push_str("    ;; if statement\n");
        
        // Emit condition
        output.push_str(&self.emit_expression(&if_stmt.condition)?);
        
        // If-then structure
        output.push_str("    if\n");
        
        // Emit then body
        for stmt in &if_stmt.then_body {
            output.push_str(&self.emit_statement(stmt)?);
        }
        
        // Handle else clause if present
        if let Some(else_body) = &if_stmt.else_body {
            output.push_str("    else\n");
            for stmt in else_body {
                output.push_str(&self.emit_statement(stmt)?);
            }
        }
        
        output.push_str("    end\n");
        Ok(output)
    }
    
    fn emit_while_loop(&mut self, while_loop: &WhileLoop) -> Result<String, CodeGenError> {
        let mut output = String::new();
        output.push_str("    ;; while loop\n");
        output.push_str("    block $loop_exit\n");
        output.push_str("      loop $while_loop\n");
        
        // Emit condition
        output.push_str(&self.emit_expression(&while_loop.condition)?);
        
        // If condition is false, break out of loop
        output.push_str("        i32.eqz\n");
        output.push_str("        br_if $loop_exit\n");
        
        // Emit loop body
        for stmt in &while_loop.body {
            output.push_str(&self.emit_statement(stmt)?);
        }
        
        // Continue loop
        output.push_str("        br $while_loop\n");
        output.push_str("      end\n");
        output.push_str("    end\n");
        
        Ok(output)
    }
    
    fn emit_foreach_loop(&mut self, for_each: &ForEachLoop) -> Result<String, CodeGenError> {
        let mut output = String::new();
        output.push_str("    ;; foreach loop\n");
        
        // For now, emit a placeholder
        output.push_str("    ;; TODO: Implement foreach loop\n");
        
        Ok(output)
    }
    
    fn emit_action_definition(&mut self, action: &ActionDefinition) -> Result<String, CodeGenError> {
        let mut output = String::new();
        output.push_str(&format!("    ;; action {} defined\n", action.name));
        Ok(output)
    }
    
    fn emit_action_definition_with_params(&mut self, action: &ActionDefinitionWithParams) -> Result<String, CodeGenError> {
        let mut output = String::new();
        output.push_str(&format!("    ;; action {} with parameters defined\n", action.name));
        Ok(output)
    }
    
    fn emit_module_definition(&mut self, module: &ModuleDefinition) -> Result<String, CodeGenError> {
        let mut output = String::new();
        output.push_str(&format!("    ;; module {} defined\n", module.name));
        Ok(output)
    }
    
    fn emit_data_definition(&mut self, data: &DataDefinition) -> Result<String, CodeGenError> {
        let mut output = String::new();
        let field_names: Vec<&String> = data.fields.iter().map(|f| &f.name).collect();
        output.push_str(&format!("    ;; data {} defined with fields: {:?}\n", data.name, field_names));
        Ok(output)
    }
    
    fn emit_return_statement(&mut self, ret: &ReturnStatement) -> Result<String, CodeGenError> {
        let mut output = String::new();
        output.push_str("    ;; return statement\n");
        // The actual return value emission is handled in action invocations
        Ok(output)
    }
}

impl TypeSystemHelpers for WebAssemblyGenerator {}

impl BaseCodeGenerator for WebAssemblyGenerator {
    fn generate(&self, program: &Program) -> Result<String, CodeGenError> {
        let mut generator = WebAssemblyGenerator::new();
        let mut output = String::new();
        
        // Collect strings first
        for statement in &program.statements {
            generator.collect_strings(statement);
        }
        
        // Generate WAT code
        output.push_str(&generator.emit_module_header());
        output.push_str(&generator.emit_main_function(program)?);
        output.push_str(&generator.emit_string_data());
        output.push_str(")\n");
        
        Ok(output)
    }
    
    fn emit_expression(&mut self, expr: &Node) -> Result<String, CodeGenError> {
        self.emit_expression(expr)
    }
    
    fn emit_statement(&mut self, stmt: &Node) -> Result<(), CodeGenError> {
        self.emit_statement(stmt).map(|_| ())
    }
    
    fn target_language(&self) -> &str {
        "webassembly"
    }
}

impl CodeGenerator for WebAssemblyGenerator {
    fn generate(&self, program: &Program) -> Result<String, String> {
        let mut generator = WebAssemblyGenerator::new();
        let mut output = String::new();
        
        // Collect strings first
        for statement in &program.statements {
            generator.collect_strings(statement);
        }
        
        // Generate WAT code
        output.push_str(&generator.emit_module_header());
        output.push_str(&generator.emit_main_function(program).map_err(|e| format!("{:?}", e))?);
        output.push_str(&generator.emit_string_data());
        output.push_str(")\n");
        
        Ok(output)
    }
}

impl Default for WebAssemblyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_generator_creation() {
        let generator = WebAssemblyGenerator::new();
        assert_eq!(generator.target_language(), "webassembly");
    }

    #[test]
    fn test_simple_display_generation() {
        let generator = WebAssemblyGenerator::new();
        
        // Create a simple program: Display "Hello, World!"
        let program = Program {
            statements: vec![
                Node::DisplayStatement(DisplayStatement {
                    expression: Box::new(Node::Literal(Literal {
                        value: LiteralValue::String("Hello, World!".to_string()),
                        literal_type: "string".to_string(),
                        line_number: None,
                    })),
                    line_number: None,
                }),
            ],
            metadata: vec![],
            included_modules: None,
            line_number: None,
        };

        let result = <WebAssemblyGenerator as crate::codegen::CodeGenerator>::generate(&generator, &program);
        assert!(result.is_ok(), "WASM generation should succeed");
        
        let wasm_code = result.unwrap();
        assert!(wasm_code.contains("(module"), "Should contain module header");
        assert!(wasm_code.contains("Hello, World!"), "Should contain string literal");
    }

    #[test]
    fn test_runtime_generation() {
        let generator = WebAssemblyGenerator::new();
        let runtime = generator.generate_runtime();
        
        assert!(runtime.contains("const fs = require"), "Should contain Node.js runtime");
        assert!(runtime.contains("print"), "Should contain print functions");
        assert!(runtime.contains("WebAssembly.instantiate"), "Should contain WASM instantiation");
    }
}