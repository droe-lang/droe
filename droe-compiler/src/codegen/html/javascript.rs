//! JavaScript code generation for Droe DSL

use crate::ast::*;
use crate::codegen::CodeGenerator;
// use std::collections::HashMap;

pub struct JavaScriptGenerator {
    indent_level: usize,
}

impl JavaScriptGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }
    
    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
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
}

impl CodeGenerator for JavaScriptGenerator {
    fn generate(&self, program: &Program) -> Result<String, String> {
        let mut generator = JavaScriptGenerator::new();
        let mut output = String::new();
        
        // Generate JavaScript runtime setup
        output.push_str("// Generated Droe code\n");
        output.push_str("const DroeRuntime = {\n");
        output.push_str("  display: (value) => console.log(value),\n");
        output.push_str("  variables: new Map(),\n");
        output.push_str("  actions: new Map(),\n");
        output.push_str("};\n\n");
        
        // Generate statements
        for statement in &program.statements {
            output.push_str(&generator.generate_statement(statement)?);
            output.push('\n');
        }
        
        Ok(output)
    }
}

impl JavaScriptGenerator {
    fn generate_statement(&mut self, statement: &Node) -> Result<String, String> {
        match statement {
            Node::ModuleDefinition(module) => self.generate_module(module),
            Node::DataDefinition(data) => self.generate_data_definition(data),
            Node::ActionDefinition(action) => self.generate_action_definition(action),
            Node::ActionDefinitionWithParams(action) => self.generate_action_definition_with_params(action),
            Node::TaskAction(task) => self.generate_task_action(task),
            Node::DisplayStatement(display) => self.generate_display_statement(display),
            Node::Assignment(assignment) => self.generate_assignment(assignment),
            Node::IfStatement(if_stmt) => self.generate_if_statement(if_stmt),
            Node::WhileLoop(while_loop) => self.generate_while_loop(while_loop),
            Node::ForEachLoop(for_loop) => self.generate_for_loop(for_loop),
            Node::ReturnStatement(ret) => self.generate_return_statement(ret),
            Node::ActionInvocationWithArgs(invocation) => self.generate_action_invocation(invocation),
            _ => Ok(format!("// TODO: Generate {:#?}", statement)),
        }
    }
    
    fn generate_module(&mut self, module: &ModuleDefinition) -> Result<String, String> {
        let mut output = String::new();
        
        output.push_str(&format!("{}// Module: {}\n", self.indent(), module.name));
        output.push_str(&format!("{}const {} = {{\n", self.indent(), module.name));
        
        self.with_indent(|gen| {
            for statement in &module.body {
                if let Ok(stmt_code) = gen.generate_statement(statement) {
                    output.push_str(&stmt_code);
                    output.push('\n');
                }
            }
        });
        
        output.push_str(&format!("{}}};", self.indent()));
        
        Ok(output)
    }
    
    fn generate_data_definition(&mut self, data: &DataDefinition) -> Result<String, String> {
        let mut output = String::new();
        
        output.push_str(&format!("{}class {} {{\n", self.indent(), data.name));
        
        self.with_indent(|gen| {
            output.push_str(&format!("{}constructor(", gen.indent()));
            
            let param_list: Vec<String> = data.fields.iter()
                .map(|field| field.name.clone())
                .collect();
            output.push_str(&param_list.join(", "));
            output.push_str(") {\n");
            
            gen.with_indent(|gen_inner| {
                for field in &data.fields {
                    output.push_str(&format!("{}this.{} = {};\n", 
                        gen_inner.indent(), field.name, field.name));
                }
            });
            
            output.push_str(&format!("{}}}\n", gen.indent()));
        });
        
        output.push_str(&format!("{}}}", self.indent()));
        
        Ok(output)
    }
    
    fn generate_action_definition(&mut self, action: &ActionDefinition) -> Result<String, String> {
        let mut output = String::new();
        
        output.push_str(&format!("{}{}() {{\n", self.indent(), action.name));
        
        self.with_indent(|gen| {
            for statement in &action.body {
                if let Ok(stmt_code) = gen.generate_statement(statement) {
                    output.push_str(&stmt_code);
                    output.push('\n');
                }
            }
        });
        
        output.push_str(&format!("{}}},", self.indent()));
        
        Ok(output)
    }
    
    fn generate_action_definition_with_params(&mut self, action: &ActionDefinitionWithParams) -> Result<String, String> {
        let mut output = String::new();
        
        let param_list: Vec<String> = action.parameters.iter()
            .map(|param| param.name.clone())
            .collect();
        
        output.push_str(&format!("{}{}({}) {{\n", 
            self.indent(), action.name, param_list.join(", ")));
        
        self.with_indent(|gen| {
            for statement in &action.body {
                if let Ok(stmt_code) = gen.generate_statement(statement) {
                    output.push_str(&stmt_code);
                    output.push('\n');
                }
            }
        });
        
        output.push_str(&format!("{}}},", self.indent()));
        
        Ok(output)
    }
    
    fn generate_task_action(&mut self, task: &TaskAction) -> Result<String, String> {
        let mut output = String::new();
        
        output.push_str(&format!("{}// Task: {}\n", self.indent(), task.name));
        output.push_str(&format!("{}async function {}() {{\n", self.indent(), task.name));
        
        self.with_indent(|gen| {
            for statement in &task.body {
                if let Ok(stmt_code) = gen.generate_statement(statement) {
                    output.push_str(&stmt_code);
                    output.push('\n');
                }
            }
        });
        
        output.push_str(&format!("{}}}", self.indent()));
        
        Ok(output)
    }
    
    fn generate_display_statement(&mut self, display: &DisplayStatement) -> Result<String, String> {
        let expr_code = self.generate_expression(&display.expression)?;
        Ok(format!("{}DroeRuntime.display({});", self.indent(), expr_code))
    }
    
    fn generate_assignment(&mut self, assignment: &Assignment) -> Result<String, String> {
        let value_code = self.generate_expression(&assignment.value)?;
        Ok(format!("{}const {} = {};", self.indent(), assignment.variable, value_code))
    }
    
    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> Result<String, String> {
        let mut output = String::new();
        
        let condition_code = self.generate_expression(&if_stmt.condition)?;
        output.push_str(&format!("{}if ({}) {{\n", self.indent(), condition_code));
        
        self.with_indent(|gen| {
            for statement in &if_stmt.then_body {
                if let Ok(stmt_code) = gen.generate_statement(statement) {
                    output.push_str(&stmt_code);
                    output.push('\n');
                }
            }
        });
        
        if let Some(else_body) = &if_stmt.else_body {
            output.push_str(&format!("{}}} else {{\n", self.indent()));
            
            self.with_indent(|gen| {
                for statement in else_body {
                    if let Ok(stmt_code) = gen.generate_statement(statement) {
                        output.push_str(&stmt_code);
                        output.push('\n');
                    }
                }
            });
        }
        
        output.push_str(&format!("{}}}", self.indent()));
        
        Ok(output)
    }
    
    fn generate_while_loop(&mut self, while_loop: &WhileLoop) -> Result<String, String> {
        let mut output = String::new();
        
        let condition_code = self.generate_expression(&while_loop.condition)?;
        output.push_str(&format!("{}while ({}) {{\n", self.indent(), condition_code));
        
        self.with_indent(|gen| {
            for statement in &while_loop.body {
                if let Ok(stmt_code) = gen.generate_statement(statement) {
                    output.push_str(&stmt_code);
                    output.push('\n');
                }
            }
        });
        
        output.push_str(&format!("{}}}", self.indent()));
        
        Ok(output)
    }
    
    fn generate_for_loop(&mut self, for_loop: &ForEachLoop) -> Result<String, String> {
        let mut output = String::new();
        
        let iterable_code = self.generate_expression(&for_loop.iterable)?;
        output.push_str(&format!("{}for (const {} of {}) {{\n", 
            self.indent(), for_loop.variable, iterable_code));
        
        self.with_indent(|gen| {
            for statement in &for_loop.body {
                if let Ok(stmt_code) = gen.generate_statement(statement) {
                    output.push_str(&stmt_code);
                    output.push('\n');
                }
            }
        });
        
        output.push_str(&format!("{}}}", self.indent()));
        
        Ok(output)
    }
    
    fn generate_return_statement(&mut self, ret: &ReturnStatement) -> Result<String, String> {
        let expr_code = self.generate_expression(&ret.expression)?;
        Ok(format!("{}return {};", self.indent(), expr_code))
    }
    
    fn generate_action_invocation(&mut self, invocation: &ActionInvocationWithArgs) -> Result<String, String> {
        let mut args = Vec::new();
        for arg in &invocation.arguments {
            args.push(self.generate_expression(arg)?);
        }
        
        let function_name = if let Some(module) = &invocation.module_name {
            format!("{}.{}", module, invocation.action_name)
        } else {
            invocation.action_name.clone()
        };
        
        Ok(format!("{}{}({});", self.indent(), function_name, args.join(", ")))
    }
    
    fn generate_expression(&mut self, expression: &Node) -> Result<String, String> {
        match expression {
            Node::Literal(literal) => match &literal.value {
                LiteralValue::String(s) => Ok(format!("\"{}\"", s)),
                LiteralValue::Integer(i) => Ok(i.to_string()),
                LiteralValue::Float(f) => Ok(f.to_string()),
                LiteralValue::Boolean(b) => Ok(b.to_string()),
            },
            Node::Identifier(identifier) => Ok(identifier.name.clone()),
            Node::BinaryOp(binary_op) => {
                let left = self.generate_expression(&binary_op.left)?;
                let right = self.generate_expression(&binary_op.right)?;
                let op = match binary_op.operator.as_str() {
                    "equals" => "===",
                    _ => &binary_op.operator,
                };
                Ok(format!("({} {} {})", left, op, right))
            }
            Node::ArithmeticOp(arith_op) => {
                let left = self.generate_expression(&arith_op.left)?;
                let right = self.generate_expression(&arith_op.right)?;
                Ok(format!("({} {} {})", left, arith_op.operator, right))
            }
            Node::PropertyAccess(prop_access) => {
                let object = self.generate_expression(&prop_access.object)?;
                Ok(format!("{}.{}", object, prop_access.property))
            }
            Node::ActionInvocationWithArgs(invocation) => {
                let mut args = Vec::new();
                for arg in &invocation.arguments {
                    args.push(self.generate_expression(arg)?);
                }
                
                let function_name = if let Some(module) = &invocation.module_name {
                    format!("{}.{}", module, invocation.action_name)
                } else {
                    invocation.action_name.clone()
                };
                
                Ok(format!("{}({})", function_name, args.join(", ")))
            }
            _ => Err(format!("Unsupported expression type: {:#?}", expression)),
        }
    }
}

impl Default for JavaScriptGenerator {
    fn default() -> Self {
        Self::new()
    }
}