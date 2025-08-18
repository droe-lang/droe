//! Type checker for Droe DSL

use crate::ast::*;
use crate::diagnostics::Diagnostic;
use std::collections::HashMap;

pub struct TypeChecker {
    symbol_table: SymbolTable,
    action_types: HashMap<String, String>,
    task_types: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, VariableType>>,
}

#[derive(Debug, Clone)]
pub struct VariableType {
    pub name: String,
    pub type_name: String,
    pub line_declared: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
        }
    }
    
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    
    pub fn define(&mut self, name: String, var_type: VariableType) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, var_type);
        }
    }
    
    pub fn get(&self, name: &str) -> Option<&VariableType> {
        for scope in self.scopes.iter().rev() {
            if let Some(var_type) = scope.get(name) {
                return Some(var_type);
            }
        }
        None
    }
    
    pub fn exists(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            action_types: HashMap::new(),
            task_types: HashMap::new(),
        }
    }
    
    pub fn check(&self, program: &Program) -> Vec<Diagnostic> {
        let mut checker = TypeChecker::new();
        let mut diagnostics = Vec::new();
        
        // First pass: collect all action and task signatures
        checker.collect_signatures(program, &mut diagnostics);
        
        // Second pass: validate all statements and expressions
        for statement in &program.statements {
            checker.check_statement(statement, &mut diagnostics);
        }
        
        diagnostics
    }
    
    fn collect_signatures(&mut self, program: &Program, _diagnostics: &mut Vec<Diagnostic>) {
        for statement in &program.statements {
            match statement {
                Node::ActionDefinitionWithParams(action) => {
                    if let Some(return_type) = &action.return_type {
                        self.action_types.insert(action.name.clone(), return_type.clone());
                    }
                }
                Node::ModuleDefinition(module) => {
                    for module_stmt in &module.body {
                        if let Node::ActionDefinitionWithParams(action) = module_stmt {
                            let full_name = format!("{}.{}", module.name, action.name);
                            if let Some(return_type) = &action.return_type {
                                self.action_types.insert(full_name, return_type.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    fn check_statement(&mut self, statement: &Node, diagnostics: &mut Vec<Diagnostic>) {
        match statement {
            Node::Assignment(assignment) => {
                self.check_assignment(assignment, diagnostics);
            }
            Node::ActionDefinitionWithParams(action) => {
                self.check_action_definition(action, diagnostics);
            }
            Node::TaskAction(task) => {
                self.check_task_definition(task, diagnostics);
            }
            Node::ModuleDefinition(module) => {
                self.check_module_definition(module, diagnostics);
            }
            Node::IfStatement(if_stmt) => {
                self.check_if_statement(if_stmt, diagnostics);
            }
            Node::WhileLoop(while_loop) => {
                self.check_while_loop(while_loop, diagnostics);
            }
            Node::ForEachLoop(for_loop) => {
                self.check_for_loop(for_loop, diagnostics);
            }
            Node::DisplayStatement(display) => {
                self.check_expression(&display.expression, diagnostics);
            }
            Node::ReturnStatement(ret) => {
                self.check_expression(&ret.expression, diagnostics);
            }
            Node::ActionInvocationWithArgs(invocation) => {
                self.check_action_invocation(invocation, diagnostics);
            }
            Node::DataDefinition(_) => {
                // Data definitions are valid by structure
            }
            Node::IncludeStatement(_) => {
                // Include statements are handled separately
            }
            _ => {
                // Other statements - add more as needed
            }
        }
    }
    
    fn check_assignment(&mut self, assignment: &Assignment, diagnostics: &mut Vec<Diagnostic>) {
        // Check if value expression is valid
        self.check_expression(&assignment.value, diagnostics);
        
        // Add variable to symbol table
        let var_type = VariableType {
            name: assignment.variable.clone(),
            type_name: "unknown".to_string(), // TODO: infer type from expression
            line_declared: assignment.line_number.unwrap_or(0),
        };
        
        self.symbol_table.define(assignment.variable.clone(), var_type);
    }
    
    fn check_action_definition(&mut self, action: &ActionDefinitionWithParams, diagnostics: &mut Vec<Diagnostic>) {
        self.symbol_table.push_scope();
        
        // Add parameters to scope
        for param in &action.parameters {
            let var_type = VariableType {
                name: param.name.clone(),
                type_name: param.param_type.clone(),
                line_declared: param.line_number.unwrap_or(0),
            };
            self.symbol_table.define(param.name.clone(), var_type);
        }
        
        // Check action body
        for stmt in &action.body {
            self.check_statement(stmt, diagnostics);
        }
        
        self.symbol_table.pop_scope();
    }
    
    fn check_task_definition(&mut self, task: &TaskAction, diagnostics: &mut Vec<Diagnostic>) {
        self.symbol_table.push_scope();
        
        // Add parameters to scope
        for param in &task.parameters {
            let var_type = VariableType {
                name: param.name.clone(),
                type_name: param.param_type.clone(),
                line_declared: param.line_number.unwrap_or(0),
            };
            self.symbol_table.define(param.name.clone(), var_type);
        }
        
        // Check task body
        for stmt in &task.body {
            self.check_statement(stmt, diagnostics);
        }
        
        self.symbol_table.pop_scope();
    }
    
    fn check_module_definition(&mut self, module: &ModuleDefinition, diagnostics: &mut Vec<Diagnostic>) {
        self.symbol_table.push_scope();
        
        for stmt in &module.body {
            self.check_statement(stmt, diagnostics);
        }
        
        self.symbol_table.pop_scope();
    }
    
    fn check_if_statement(&mut self, if_stmt: &IfStatement, diagnostics: &mut Vec<Diagnostic>) {
        // Check condition
        self.check_expression(&if_stmt.condition, diagnostics);
        
        // Check then body
        self.symbol_table.push_scope();
        for stmt in &if_stmt.then_body {
            self.check_statement(stmt, diagnostics);
        }
        self.symbol_table.pop_scope();
        
        // Check else body if present
        if let Some(else_body) = &if_stmt.else_body {
            self.symbol_table.push_scope();
            for stmt in else_body {
                self.check_statement(stmt, diagnostics);
            }
            self.symbol_table.pop_scope();
        }
    }
    
    fn check_while_loop(&mut self, while_loop: &WhileLoop, diagnostics: &mut Vec<Diagnostic>) {
        // Check condition
        self.check_expression(&while_loop.condition, diagnostics);
        
        // Check body
        self.symbol_table.push_scope();
        for stmt in &while_loop.body {
            self.check_statement(stmt, diagnostics);
        }
        self.symbol_table.pop_scope();
    }
    
    fn check_for_loop(&mut self, for_loop: &ForEachLoop, diagnostics: &mut Vec<Diagnostic>) {
        // Check iterable
        self.check_expression(&for_loop.iterable, diagnostics);
        
        // Check body with loop variable in scope
        self.symbol_table.push_scope();
        
        let var_type = VariableType {
            name: for_loop.variable.clone(),
            type_name: "unknown".to_string(), // TODO: infer from iterable type
            line_declared: for_loop.line_number.unwrap_or(0),
        };
        self.symbol_table.define(for_loop.variable.clone(), var_type);
        
        for stmt in &for_loop.body {
            self.check_statement(stmt, diagnostics);
        }
        
        self.symbol_table.pop_scope();
    }
    
    fn check_action_invocation(&mut self, invocation: &ActionInvocationWithArgs, diagnostics: &mut Vec<Diagnostic>) {
        // Check if action exists
        let action_name = if let Some(module) = &invocation.module_name {
            format!("{}.{}", module, invocation.action_name)
        } else {
            invocation.action_name.clone()
        };
        
        if !self.action_types.contains_key(&action_name) {
            diagnostics.push(Diagnostic::error(
                format!("Undefined action: {}", action_name),
                invocation.line_number.unwrap_or(0),
                0,
            ));
        }
        
        // Check arguments
        for arg in &invocation.arguments {
            self.check_expression(arg, diagnostics);
        }
    }
    
    fn check_expression(&mut self, expression: &Node, diagnostics: &mut Vec<Diagnostic>) {
        match expression {
            Node::Identifier(identifier) => {
                if !self.symbol_table.exists(&identifier.name) {
                    diagnostics.push(Diagnostic::error(
                        format!("Undefined variable or action: {}", identifier.name),
                        identifier.line_number.unwrap_or(0),
                        0,
                    ));
                }
            }
            Node::BinaryOp(binary_op) => {
                self.check_expression(&binary_op.left, diagnostics);
                self.check_expression(&binary_op.right, diagnostics);
            }
            Node::ArithmeticOp(arith_op) => {
                self.check_expression(&arith_op.left, diagnostics);
                self.check_expression(&arith_op.right, diagnostics);
            }
            Node::PropertyAccess(prop_access) => {
                self.check_expression(&prop_access.object, diagnostics);
            }
            Node::ActionInvocationWithArgs(invocation) => {
                self.check_action_invocation(invocation, diagnostics);
            }
            Node::Literal(_) => {
                // Literals are always valid
            }
            _ => {
                // Other expression types - add more as needed
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}