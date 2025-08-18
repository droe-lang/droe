//! Core Python code generator for Droe DSL
//! 
//! This module generates pure Python code from Droe AST, handling only core language features.
//! Framework-specific code generation is handled by separate adapters.

use crate::ast::{Program, Node, LiteralValue, IfStatement, 
                WhileLoop, ForEachLoop, ForEachCharLoop, ActionDefinition, DataDefinition, ModuleDefinition,
                ServeStatement, DatabaseStatement, ApiCallStatement, StringInterpolation, FormatExpression};
use crate::codegen::CodeGenerator;

/// Core Python code generator - handles only language fundamentals
pub struct PythonGenerator {
    /// Class name for the generated program
    class_name: String,
    /// Import statements
    imports: Vec<String>,
    /// Class-level variables/fields
    fields: Vec<String>,
    /// Method definitions (from action definitions)
    methods: Vec<String>,
    /// Main program code (constructor/procedural code)
    main_code: Vec<String>,
    /// Data class definitions
    data_classes: Vec<String>,
    /// Module classes
    module_classes: Vec<String>,
}

impl PythonGenerator {
    pub fn new() -> Self {
        Self {
            class_name: "DroeProgram".to_string(),
            imports: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            main_code: Vec::new(),
            data_classes: Vec::new(),
            module_classes: Vec::new(),
        }
    }

    pub fn with_class_name(mut self, name: &str) -> Self {
        self.class_name = self.to_pascal_case(name);
        self
    }

    /// Generate core Python code (single file)
    pub fn generate_python(&mut self, program: &Program) -> Result<String, String> {
        self.clear();
        self.add_core_imports();

        // Process all statements
        for statement in &program.statements {
            self.process_statement(statement)?;
        }

        Ok(self.build_python_file())
    }

    fn clear(&mut self) {
        self.imports.clear();
        self.fields.clear();
        self.methods.clear();
        self.main_code.clear();
        self.data_classes.clear();
        self.module_classes.clear();
    }

    fn add_core_imports(&mut self) {
        self.imports.extend_from_slice(&[
            "from typing import List, Dict, Any, Optional".to_string(),
            "from datetime import datetime, date".to_string(),
            "from dataclasses import dataclass".to_string(),
            "import sqlite3".to_string(),
            "import json".to_string(),
            "import http.client".to_string(),
            "from urllib.parse import urlparse".to_string(),
        ]);
    }

    fn process_statement(&mut self, statement: &Node) -> Result<(), String> {
        match statement {
            Node::DisplayStatement(display) => {
                let expr = self.generate_expression(&display.expression)?;
                self.main_code.push(format!("        print({})", expr));
            }
            Node::Assignment(assignment) => {
                let value = self.generate_expression(&assignment.value)?;
                self.main_code.push(format!("        self.{} = {}", assignment.variable, value));
                // Add field declaration
                self.fields.push(format!("    {}: Any", assignment.variable));
            }
            Node::IfStatement(if_stmt) => {
                self.generate_if_statement(if_stmt)?;
            }
            Node::WhileLoop(while_loop) => {
                self.generate_while_loop(while_loop)?;
            }
            Node::ForEachLoop(for_loop) => {
                self.generate_for_loop(for_loop)?;
            }
            Node::ForEachCharLoop(char_loop) => {
                self.generate_for_char_loop(char_loop)?;
            }
            Node::ActionDefinition(action) => {
                self.generate_action_definition(action)?;
            }
            Node::DataDefinition(data) => {
                self.generate_data_definition(data)?;
            }
            Node::ModuleDefinition(module) => {
                self.generate_module_definition(module)?;
            }
            Node::ServeStatement(serve) => {
                self.generate_serve_statement(serve)?;
            }
            Node::DatabaseStatement(db) => {
                self.generate_database_statement(db)?;
            }
            Node::ApiCallStatement(api) => {
                self.generate_api_call_statement(api)?;
            }
            _ => {
                // Skip unknown statements with comment
                self.main_code.push(format!("        # TODO: Implement {:?}", statement));
            }
        }
        Ok(())
    }

    fn generate_expression(&self, expr: &Box<Node>) -> Result<String, String> {
        match **expr {
            Node::Literal(ref lit) => {
                match &lit.value {
                    LiteralValue::String(s) => Ok(format!("\"{}\"", s.replace("\"", "\\\""))),
                    LiteralValue::Integer(i) => Ok(i.to_string()),
                    LiteralValue::Float(f) => Ok(f.to_string()),
                    LiteralValue::Boolean(b) => Ok(if *b { "True".to_string() } else { "False".to_string() }),
                }
            }
            Node::Identifier(ref id) => Ok(format!("self.{}", id.name)),
            Node::BinaryOp(ref binop) => {
                let left = self.generate_expression(&binop.left)?;
                let right = self.generate_expression(&binop.right)?;
                Ok(format!("({} {} {})", left, binop.operator, right))
            }
            Node::ArithmeticOp(ref arith) => {
                let left = self.generate_expression(&arith.left)?;
                let right = self.generate_expression(&arith.right)?;
                Ok(format!("({} {} {})", left, arith.operator, right))
            }
            Node::ArrayLiteral(ref array) => {
                let elements: Result<Vec<String>, String> = array.elements
                    .iter()
                    .map(|elem| self.generate_expression(&Box::new(elem.clone())))
                    .collect();
                Ok(format!("[{}]", elements?.join(", ")))
            }
            Node::StringInterpolation(ref interp) => {
                self.generate_string_interpolation(interp)
            }
            Node::FormatExpression(ref fmt) => {
                self.generate_format_expression(fmt)
            }
            _ => Ok("None  # TODO: unsupported expression".to_string()),
        }
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> Result<(), String> {
        let condition = self.generate_expression(&if_stmt.condition)?;
        self.main_code.push(format!("        if {}:", condition));

        // Generate then body
        for _stmt in &if_stmt.then_body {
            self.main_code.push("            # Then body".to_string());
            // Note: This is simplified - would need proper indentation handling
        }

        // Generate else body if present
        if let Some(else_body) = &if_stmt.else_body {
            self.main_code.push("        else:".to_string());
            for _stmt in else_body {
                self.main_code.push("            # Else body".to_string());
            }
        }

        Ok(())
    }

    fn generate_while_loop(&mut self, while_loop: &WhileLoop) -> Result<(), String> {
        let condition = self.generate_expression(&while_loop.condition)?;
        self.main_code.push(format!("        while {}:", condition));
        
        for _stmt in &while_loop.body {
            self.main_code.push("            # Loop body".to_string());
        }

        Ok(())
    }

    fn generate_for_loop(&mut self, for_loop: &ForEachLoop) -> Result<(), String> {
        let iterable = self.generate_expression(&for_loop.iterable)?;
        self.main_code.push(format!("        for {} in {}:", for_loop.variable, iterable));
        
        for _stmt in &for_loop.body {
            self.main_code.push("            # Loop body".to_string());
        }

        Ok(())
    }

    fn generate_for_char_loop(&mut self, char_loop: &ForEachCharLoop) -> Result<(), String> {
        let string_expr = self.generate_expression(&char_loop.string_expr)?;
        self.main_code.push(format!("        for {} in {}:", char_loop.variable, string_expr));
        
        // Generate loop body
        for _stmt in &char_loop.body {
            self.main_code.push("            # Character loop body".to_string());
        }

        Ok(())
    }

    fn generate_action_definition(&mut self, action: &ActionDefinition) -> Result<(), String> {
        let mut method_lines = vec![
            format!("    def {}(self):", action.name),
        ];

        // Process action body
        for stmt in &action.body {
            match stmt {
                Node::ReturnStatement(ret) => {
                    let expr = self.generate_expression(&ret.expression)?;
                    method_lines.push(format!("        return {}", expr));
                }
                _ => {
                    method_lines.push("        # Action body".to_string());
                }
            }
        }

        if method_lines.len() == 1 {
            method_lines.push("        pass".to_string());
        }

        self.methods.push(method_lines.join("\n"));
        Ok(())
    }

    fn generate_data_definition(&mut self, data: &DataDefinition) -> Result<(), String> {
        let mut class_lines = vec![
            "@dataclass".to_string(),
            format!("class {}:", data.name),
        ];

        // Generate fields
        for field in &data.fields {
            let python_type = self.map_droe_type_to_python(&field.field_type);
            class_lines.push(format!("    {}: {}", field.name, python_type));
        }

        if data.fields.is_empty() {
            class_lines.push("    pass".to_string());
        }

        self.data_classes.push(class_lines.join("\n"));
        Ok(())
    }

    fn generate_module_definition(&mut self, module: &ModuleDefinition) -> Result<(), String> {
        let mut class_lines = vec![
            format!("class {}:", module.name),
        ];

        // Process module body
        for stmt in &module.body {
            if let Node::ActionDefinition(action) = stmt {
                class_lines.push("    @staticmethod".to_string());
                class_lines.push(format!("    def {}():", action.name));
                class_lines.push("        # Module action".to_string());
            }
        }

        if class_lines.len() == 1 {
            class_lines.push("    pass".to_string());
        }

        self.module_classes.push(class_lines.join("\n"));
        Ok(())
    }

    fn generate_serve_statement(&mut self, serve: &ServeStatement) -> Result<(), String> {
        // Core Python: Simple HTTP server using built-in libraries
        self.main_code.push(format!(
            "        # Serve {} {} - using basic HTTP server",
            serve.method, serve.endpoint
        ));
        self.main_code.push("        # Implementation would use http.server for basic functionality".to_string());
        Ok(())
    }

    fn generate_database_statement(&mut self, db: &DatabaseStatement) -> Result<(), String> {
        // Core Python: Direct SQL using sqlite3
        match db.operation.as_str() {
            "CREATE" => {
                self.main_code.push("        # Create table".to_string());
                self.main_code.push("        conn = sqlite3.connect('droe.db')".to_string());
                self.main_code.push("        cursor = conn.cursor()".to_string());
                self.main_code.push(format!(
                    "        cursor.execute('CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY)')",
                    db.entity_name
                ));
                self.main_code.push("        conn.commit()".to_string());
                self.main_code.push("        conn.close()".to_string());
            }
            "INSERT" => {
                self.main_code.push("        # Insert data".to_string());
                self.main_code.push("        conn = sqlite3.connect('droe.db')".to_string());
                self.main_code.push("        cursor = conn.cursor()".to_string());
                self.main_code.push(format!(
                    "        cursor.execute('INSERT INTO {} DEFAULT VALUES')",
                    db.entity_name
                ));
                self.main_code.push("        conn.commit()".to_string());
                self.main_code.push("        conn.close()".to_string());
            }
            "SELECT" => {
                self.main_code.push("        # Select data".to_string());
                self.main_code.push("        conn = sqlite3.connect('droe.db')".to_string());
                self.main_code.push("        cursor = conn.cursor()".to_string());
                self.main_code.push(format!(
                    "        cursor.execute('SELECT * FROM {}')",
                    db.entity_name
                ));
                self.main_code.push("        results = cursor.fetchall()".to_string());
                if let Some(var) = &db.return_var {
                    self.main_code.push(format!("        self.{} = results", var));
                } else {
                    self.main_code.push("        for row in results: print(row)".to_string());
                }
                self.main_code.push("        conn.close()".to_string());
            }
            _ => {
                self.main_code.push(format!("        # {} operation on {}", db.operation, db.entity_name));
            }
        }
        Ok(())
    }

    fn generate_api_call_statement(&mut self, api: &ApiCallStatement) -> Result<(), String> {
        // Core Python: Direct HTTP calls using http.client
        self.main_code.push(format!("        # API {} call to {}", api.method, api.endpoint));
        self.main_code.push(format!("        url_parts = urlparse('{}')", api.endpoint));
        
        if api.endpoint.starts_with("https://") {
            self.main_code.push("        conn = http.client.HTTPSConnection(url_parts.netloc)".to_string());
        } else {
            self.main_code.push("        conn = http.client.HTTPConnection(url_parts.netloc)".to_string());
        }

        self.main_code.push("        headers = {'Content-Type': 'application/json'}".to_string());
        self.main_code.push(format!(
            "        conn.request('{}', url_parts.path, None, headers)",
            api.method.to_uppercase()
        ));
        self.main_code.push("        response = conn.getresponse()".to_string());
        self.main_code.push("        data = response.read().decode('utf-8')".to_string());
        
        if let Some(var) = &api.response_variable {
            self.main_code.push(format!("        self.{} = json.loads(data) if data else None", var));
        } else {
            self.main_code.push("        print(f'Response: {response.status} - {data}')".to_string());
        }
        
        self.main_code.push("        conn.close()".to_string());
        Ok(())
    }

    fn generate_string_interpolation(&self, interp: &StringInterpolation) -> Result<String, String> {
        let mut parts = Vec::new();
        for part in &interp.parts {
            match part {
                Node::Literal(lit) => {
                    if let LiteralValue::String(s) = &lit.value {
                        parts.push(s.clone());
                    }
                }
                _ => {
                    let expr = self.generate_expression(&Box::new(part.clone()))?;
                    parts.push(format!("{{{}}}", expr));
                }
            }
        }
        Ok(format!("f\"{}\"", parts.join("")))
    }

    fn generate_format_expression(&self, fmt: &FormatExpression) -> Result<String, String> {
        let expr = self.generate_expression(&fmt.expression)?;
        // Simple format pattern mapping
        match fmt.format_pattern.as_str() {
            "0.00" => Ok(format!("f\"{{{expr}:.2f}}\"")),
            "#,##0" => Ok(format!("f\"{{{expr}:,}}\"")),
            _ => Ok(expr),
        }
    }

    fn map_droe_type_to_python(&self, droe_type: &str) -> &str {
        match droe_type {
            "text" => "str",
            "int" => "int", 
            "decimal" => "float",
            "flag" => "bool",
            "date" => "datetime",
            "datetime" => "datetime",
            _ => "Any",
        }
    }

    fn to_pascal_case(&self, name: &str) -> String {
        name.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect()
    }

    fn build_python_file(&self) -> String {
        let mut lines = Vec::new();

        // File header
        lines.push("#!/usr/bin/env python3".to_string());
        lines.push("\"\"\"Generated Python code from Droe DSL\"\"\"".to_string());
        lines.push("".to_string());

        // Imports
        for import in &self.imports {
            lines.push(import.clone());
        }
        lines.push("".to_string());

        // Data classes
        for data_class in &self.data_classes {
            lines.push(data_class.clone());
            lines.push("".to_string());
        }

        // Module classes
        for module_class in &self.module_classes {
            lines.push(module_class.clone());
            lines.push("".to_string());
        }

        // Main program class
        lines.push(format!("class {}:", self.class_name));
        lines.push("    \"\"\"Main program class\"\"\"".to_string());
        lines.push("".to_string());

        // Constructor
        lines.push("    def __init__(self):".to_string());
        
        // Initialize fields
        for field in &self.fields {
            lines.push(format!("        # {}", field));
        }

        // Main code
        if self.main_code.is_empty() {
            lines.push("        pass".to_string());
        } else {
            for code_line in &self.main_code {
                lines.push(code_line.clone());
            }
        }

        lines.push("".to_string());

        // Methods
        for method in &self.methods {
            lines.push(method.clone());
            lines.push("".to_string());
        }

        // Main execution
        lines.push("def main():".to_string());
        lines.push("    \"\"\"Main entry point\"\"\"".to_string());
        lines.push(format!("    program = {}()", self.class_name));
        lines.push("".to_string());
        lines.push("if __name__ == '__main__':".to_string());
        lines.push("    main()".to_string());

        lines.join("\n")
    }
}

impl CodeGenerator for PythonGenerator {
    fn generate(&self, program: &Program) -> Result<String, String> {
        let mut generator = self.clone();
        generator.generate_python(program)
    }
}

impl Clone for PythonGenerator {
    fn clone(&self) -> Self {
        Self {
            class_name: self.class_name.clone(),
            imports: self.imports.clone(),
            fields: self.fields.clone(),
            methods: self.methods.clone(),
            main_code: self.main_code.clone(),
            data_classes: self.data_classes.clone(),
            module_classes: self.module_classes.clone(),
        }
    }
}

impl Default for PythonGenerator {
    fn default() -> Self {
        Self::new()
    }
}