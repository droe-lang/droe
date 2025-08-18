//! Go code generation for Droe DSL

use crate::ast::*;
use super::CodeGenerator;

/// Go Generator
pub struct GoGenerator {
    indent_level: usize,
}

impl GoGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }
    
    fn indent(&self) -> String {
        "\t".repeat(self.indent_level)
    }
    
    fn with_indent<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.indent_level += 1;
        let result = f(self);
        self.indent_level -= 1;
        result
    }

    fn emit_package_header(&self) -> String {
        let mut output = String::new();
        output.push_str("package main\n\n");
        output.push_str("import (\n");
        output.push_str("\t\"fmt\"\n");
        output.push_str("\t\"os\"\n");
        output.push_str(")\n\n");
        output
    }

    fn emit_main_function(&mut self, program: &Program) -> String {
        let mut output = String::new();
        output.push_str("func main() {\n");
        
        self.with_indent(|gen| {
            for statement in &program.statements {
                output.push_str(&gen.emit_statement(statement));
            }
        });
        
        output.push_str("}\n");
        output
    }

    fn emit_statement(&mut self, statement: &Node) -> String {
        match statement {
            Node::DisplayStatement(display) => self.emit_display_statement(display),
            Node::Assignment(assignment) => self.emit_assignment(assignment),
            Node::IfStatement(if_stmt) => self.emit_if_statement(if_stmt),
            Node::WhileLoop(while_loop) => self.emit_while_loop(while_loop),
            _ => format!("{}// TODO: Implement {:#?}\n", self.indent(), statement),
        }
    }

    fn emit_display_statement(&self, display: &DisplayStatement) -> String {
        match display.expression.as_ref() {
            Node::Literal(literal) => {
                match &literal.value {
                    LiteralValue::String(s) => {
                        format!("{}fmt.Println(\"{}\")\n", self.indent(), s)
                    }
                    LiteralValue::Integer(i) => {
                        format!("{}fmt.Println({})\n", self.indent(), i)
                    }
                    LiteralValue::Float(f) => {
                        format!("{}fmt.Println({})\n", self.indent(), f)
                    }
                    LiteralValue::Boolean(b) => {
                        format!("{}fmt.Println({})\n", self.indent(), b)
                    }
                }
            }
            Node::Identifier(identifier) => {
                format!("{}fmt.Println({})\n", self.indent(), identifier.name)
            }
            _ => format!("{}fmt.Println(\"complex expression\")\n", self.indent()),
        }
    }

    fn emit_assignment(&self, assignment: &Assignment) -> String {
        let value_str = match assignment.value.as_ref() {
            Node::Literal(literal) => {
                match &literal.value {
                    LiteralValue::String(s) => format!("\"{}\"", s),
                    LiteralValue::Integer(i) => i.to_string(),
                    LiteralValue::Float(f) => f.to_string(),
                    LiteralValue::Boolean(b) => b.to_string(),
                }
            }
            Node::Identifier(identifier) => identifier.name.clone(),
            _ => "nil".to_string(),
        };
        
        format!("{}{} := {}\n", self.indent(), assignment.variable, value_str)
    }

    fn emit_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        let mut output = String::new();
        
        let condition = self.emit_expression(&if_stmt.condition);
        output.push_str(&format!("{}if {} {{\n", self.indent(), condition));
        
        self.with_indent(|gen| {
            for stmt in &if_stmt.then_body {
                output.push_str(&gen.emit_statement(stmt));
            }
        });
        
        if let Some(else_body) = &if_stmt.else_body {
            output.push_str(&format!("{}}} else {{\n", self.indent()));
            self.with_indent(|gen| {
                for stmt in else_body {
                    output.push_str(&gen.emit_statement(stmt));
                }
            });
        }
        
        output.push_str(&format!("{}}}\n", self.indent()));
        output
    }

    fn emit_while_loop(&mut self, while_loop: &WhileLoop) -> String {
        let mut output = String::new();
        
        let condition = self.emit_expression(&while_loop.condition);
        output.push_str(&format!("{}for {} {{\n", self.indent(), condition));
        
        self.with_indent(|gen| {
            for stmt in &while_loop.body {
                output.push_str(&gen.emit_statement(stmt));
            }
        });
        
        output.push_str(&format!("{}}}\n", self.indent()));
        output
    }

    fn emit_expression(&self, expression: &Box<Node>) -> String {
        match expression.as_ref() {
            Node::Literal(literal) => {
                match &literal.value {
                    LiteralValue::String(s) => format!("\"{}\"", s),
                    LiteralValue::Integer(i) => i.to_string(),
                    LiteralValue::Float(f) => f.to_string(),
                    LiteralValue::Boolean(b) => b.to_string(),
                }
            }
            Node::Identifier(identifier) => identifier.name.clone(),
            Node::BinaryOp(binary_op) => {
                let left = self.emit_expression(&binary_op.left);
                let right = self.emit_expression(&binary_op.right);
                format!("{} {} {}", left, binary_op.operator, right)
            }
            Node::ArithmeticOp(arith_op) => {
                let left = self.emit_expression(&arith_op.left);
                let right = self.emit_expression(&arith_op.right);
                format!("{} {} {}", left, arith_op.operator, right)
            }
            _ => "nil".to_string(),
        }
    }
}

impl CodeGenerator for GoGenerator {
    fn generate(&self, program: &Program) -> Result<String, String> {
        let mut generator = GoGenerator::new();
        let mut output = String::new();
        
        output.push_str(&generator.emit_package_header());
        output.push_str(&generator.emit_main_function(program));
        
        Ok(output)
    }
}

impl Default for GoGenerator {
    fn default() -> Self {
        Self::new()
    }
}