//! Base code generator for Droe compiler
//! 
//! This module provides the abstract base functionality and common utilities
//! for all target-specific code generators.

use crate::ast::{Node, Program, Literal};
use crate::symbols::{SymbolTable, VariableType, VariableValue};
use crate::codegen::utils::CoreUtilities;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("Type error: {message}")]
    TypeError { message: String },
    
    #[error("Unknown variable: {name}")]
    UnknownVariable { name: String },
    
    #[error("Unknown type: {type_name}")]
    UnknownType { type_name: String },
    
    #[error("Core library error: {library} - {message}")]
    CoreLibError { library: String, message: String },
    
    #[error("Code generation failed: {message}")]
    GenerationFailed { message: String },
    
    #[error("Symbol table error: {message}")]
    SymbolError { message: String },
    
    #[error("Unsupported operation: {op}")]
    UnsupportedOperation { op: String },
}

/// Abstract base class for all code generators
pub trait BaseCodeGenerator {
    /// Generate code for the given AST program
    fn generate(&self, program: &Program) -> Result<String, CodeGenError>;
    
    /// Emit code for an expression
    fn emit_expression(&mut self, expr: &Node) -> Result<String, CodeGenError>;
    
    /// Emit code for a statement
    fn emit_statement(&mut self, stmt: &Node) -> Result<(), CodeGenError>;
    
    /// Get the target language name
    fn target_language(&self) -> &str;
    
    /// Get supported core libraries
    fn supported_core_libraries(&self) -> Vec<&str> {
        vec!["string_utils", "math_utils", "formatting"]
    }
}

/// Core library registry for tracking enabled libraries
#[derive(Debug, Clone)]
pub struct CoreLibraries {
    enabled_libs: HashSet<String>,
    available_libs: HashMap<String, CoreLibrary>,
    core_utils: CoreUtilities,
}

#[derive(Debug, Clone)]
pub struct CoreLibrary {
    pub name: String,
    pub description: String,
    pub functions: Vec<String>,
}

impl CoreLibraries {
    pub fn new() -> Self {
        let mut core_libs = Self {
            enabled_libs: HashSet::new(),
            available_libs: HashMap::new(),
            core_utils: CoreUtilities::new(),
        };
        
        core_libs.register_default_libraries();
        core_libs
    }

    fn register_default_libraries(&mut self) {
        // String utilities
        let string_functions: Vec<String> = self.core_utils.string.get_functions()
            .keys()
            .cloned()
            .collect();
        
        self.available_libs.insert(
            "string_utils".to_string(),
            CoreLibrary {
                name: "string_utils".to_string(),
                description: "String manipulation functions".to_string(),
                functions: string_functions,
            },
        );

        // Math utilities
        let math_functions: Vec<String> = self.core_utils.math.get_functions()
            .keys()
            .cloned()
            .collect();
        
        self.available_libs.insert(
            "math_utils".to_string(),
            CoreLibrary {
                name: "math_utils".to_string(),
                description: "Mathematical functions".to_string(),
                functions: math_functions,
            },
        );

        // Formatting utilities
        let formatting_functions: Vec<String> = self.core_utils.formatting.get_functions()
            .keys()
            .cloned()
            .collect();
        
        self.available_libs.insert(
            "formatting".to_string(),
            CoreLibrary {
                name: "formatting".to_string(),
                description: "Value formatting functions".to_string(),
                functions: formatting_functions,
            },
        );
    }

    pub fn enable(&mut self, lib_name: &str) {
        if self.available_libs.contains_key(lib_name) {
            self.enabled_libs.insert(lib_name.to_string());
        }
    }

    pub fn disable(&mut self, lib_name: &str) {
        self.enabled_libs.remove(lib_name);
    }

    pub fn is_enabled(&self, lib_name: &str) -> bool {
        self.enabled_libs.contains(lib_name)
    }

    pub fn get_enabled_libraries(&self) -> Vec<&CoreLibrary> {
        self.enabled_libs
            .iter()
            .filter_map(|name| self.available_libs.get(name))
            .collect()
    }
    
    /// Generate WASM import declarations for enabled libraries
    pub fn get_wasm_imports(&self) -> Vec<String> {
        let mut imports = Vec::new();
        
        if self.enabled_libs.contains("string_utils") {
            imports.extend(self.core_utils.string.get_wasm_imports());
        }
        
        if self.enabled_libs.contains("math_utils") {
            imports.extend(self.core_utils.math.get_wasm_imports());
        }
        
        if self.enabled_libs.contains("formatting") {
            imports.extend(self.core_utils.formatting.get_wasm_imports());
        }
        
        imports
    }
    
    /// Generate JavaScript runtime functions for enabled libraries
    pub fn get_js_runtime_functions(&self) -> HashMap<String, String> {
        let mut js_functions = HashMap::new();
        
        if self.enabled_libs.contains("string_utils") {
            js_functions.extend(self.core_utils.string.get_js_runtime_functions());
        }
        
        if self.enabled_libs.contains("math_utils") {
            js_functions.extend(self.core_utils.math.get_js_runtime_functions());
        }
        
        if self.enabled_libs.contains("formatting") {
            js_functions.extend(self.core_utils.formatting.get_js_runtime_functions());
        }
        
        js_functions
    }

    /// Get access to the core utilities
    pub fn get_core_utilities(&self) -> &CoreUtilities {
        &self.core_utils
    }

    /// Check if a function exists in any enabled library
    pub fn has_function(&self, function_name: &str) -> bool {
        self.core_utils.has_function(function_name)
    }

    /// Get function description if it exists
    pub fn get_function_description(&self, function_name: &str) -> Option<String> {
        self.core_utils.get_function_description(function_name)
    }
}

impl Default for CoreLibraries {
    fn default() -> Self {
        Self::new()
    }
}

/// Common functionality shared by all code generators
pub struct CodeGenContext {
    /// Symbol table for tracking variables
    pub symbol_table: SymbolTable,
    /// Output buffer for generated code
    pub output: Vec<String>,
    /// Current indentation level
    pub indent_level: usize,
    /// String constants for optimization
    pub string_constants: HashMap<String, usize>,
    /// Next string constant index
    pub next_string_index: usize,
    /// Core libraries management
    pub core_libs: CoreLibraries,
}

impl CodeGenContext {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            output: Vec::new(),
            indent_level: 0,
            string_constants: HashMap::new(),
            next_string_index: 0,
            core_libs: CoreLibraries::new(),
        }
    }

    /// Emit a line of code with proper indentation
    pub fn emit(&mut self, code: &str) {
        let indent = "  ".repeat(self.indent_level);
        self.output.push(format!("{}{}", indent, code));
    }

    /// Emit code without indentation
    pub fn emit_raw(&mut self, code: &str) {
        self.output.push(code.to_string());
    }

    /// Get the generated code as a string
    pub fn get_output(&self) -> String {
        self.output.join("\n")
    }

    /// Clear the output buffer
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Increase indentation level
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Add a string constant and return its index
    pub fn add_string_constant(&mut self, value: &str) -> usize {
        if let Some(&index) = self.string_constants.get(value) {
            index
        } else {
            let index = self.next_string_index;
            self.string_constants.insert(value.to_string(), index);
            self.next_string_index += 1;
            index
        }
    }

    /// Enable a core library
    pub fn enable_core_lib(&mut self, lib_name: &str) {
        self.core_libs.enable(lib_name);
    }

    /// Disable a core library
    pub fn disable_core_lib(&mut self, lib_name: &str) {
        self.core_libs.disable(lib_name);
    }

    /// Check if a core library is enabled
    pub fn is_core_lib_enabled(&self, lib_name: &str) -> bool {
        self.core_libs.is_enabled(lib_name)
    }
}

impl Default for CodeGenContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Common type system helpers
pub trait TypeSystemHelpers {
    /// Map user-facing type names to internal VariableType enum
    fn map_user_type_to_internal(&self, user_type: &str) -> Result<VariableType, CodeGenError> {
        let var_type = VariableType::from_str(user_type);
        if var_type == VariableType::Unknown {
            Err(CodeGenError::UnknownType {
                type_name: user_type.to_string(),
            })
        } else {
            Ok(var_type)
        }
    }

    /// Infer the type of an AST node
    fn infer_type(&self, node: &Node, symbol_table: &SymbolTable) -> VariableType {
        match node {
            Node::Literal(literal) => self.infer_literal_type(literal),
            Node::Identifier(identifier) => {
                if let Some(var) = symbol_table.get_variable(&identifier.name) {
                    var.var_type.clone()
                } else {
                    VariableType::Unknown
                }
            }
            Node::ActionInvocation(action) => self.infer_action_type(&action.action_name),
            Node::ActionInvocationWithArgs(action) => self.infer_action_type(&action.action_name),
            _ => VariableType::Unknown,
        }
    }

    /// Infer type from a literal value
    fn infer_literal_type(&self, literal: &Literal) -> VariableType {
        match &literal.value {
            crate::ast::LiteralValue::String(_) => VariableType::Text,
            crate::ast::LiteralValue::Integer(_) => VariableType::Int,
            crate::ast::LiteralValue::Float(_) => VariableType::Decimal,
            crate::ast::LiteralValue::Boolean(_) => VariableType::Flag,
        }
    }

    /// Infer return type from action name (heuristic-based)
    fn infer_action_type(&self, action_name: &str) -> VariableType {
        let name_lower = action_name.to_lowercase();
        
        // Common patterns for type inference
        if name_lower.contains("age") || name_lower.contains("count") || name_lower.contains("calculate") {
            VariableType::Int
        } else if name_lower.contains("greeting") || name_lower.contains("text") || name_lower.contains("message") {
            VariableType::Text
        } else if name_lower.contains("price") || name_lower.contains("rate") || name_lower.contains("percentage") {
            VariableType::Decimal
        } else if name_lower.contains("check") || name_lower.contains("verify") || name_lower.contains("validate") {
            VariableType::Flag
        } else {
            VariableType::Text // Default fallback
        }
    }

    /// Check if two types are compatible for assignment
    fn are_types_compatible(&self, declared_type: &VariableType, inferred_type: &VariableType) -> bool {
        declared_type.is_compatible_with(inferred_type)
    }
}

/// Abstract base code generator with default implementations
pub struct DefaultCodeGenerator {
    pub context: CodeGenContext,
}

impl DefaultCodeGenerator {
    pub fn new() -> Self {
        Self {
            context: CodeGenContext::new(),
        }
    }

    /// Process a program and generate basic structure
    pub fn process_program(&mut self, program: &Program) -> Result<(), CodeGenError> {
        // Enable default core libraries
        self.context.enable_core_lib("string_utils");
        self.context.enable_core_lib("math_utils");

        // Process all statements
        for statement in &program.statements {
            self.emit_statement(statement)?;
        }

        Ok(())
    }

    /// Generate variable declaration code (target-specific)
    pub fn generate_variable_declaration(&mut self, name: &str, var_type: &VariableType, value: Option<&VariableValue>) -> String {
        // Default implementation - should be overridden by target generators
        match value {
            Some(val) => format!("{} {} = {}", var_type.to_string(), name, val.to_string()),
            None => format!("{} {}", var_type.to_string(), name),
        }
    }

    /// Generate function call code (target-specific)
    pub fn generate_function_call(&mut self, function_name: &str, args: &[String]) -> String {
        // Default implementation - should be overridden by target generators
        format!("{}({})", function_name, args.join(", "))
    }
}

impl TypeSystemHelpers for DefaultCodeGenerator {}

impl BaseCodeGenerator for DefaultCodeGenerator {
    fn generate(&self, program: &Program) -> Result<String, CodeGenError> {
        let mut generator = Self::new();
        generator.process_program(program)?;
        Ok(generator.context.get_output())
    }

    fn emit_expression(&mut self, expr: &Node) -> Result<String, CodeGenError> {
        match expr {
            Node::Literal(literal) => {
                Ok(match &literal.value {
                    crate::ast::LiteralValue::String(s) => format!("\"{}\"", s),
                    crate::ast::LiteralValue::Integer(i) => i.to_string(),
                    crate::ast::LiteralValue::Float(f) => f.to_string(),
                    crate::ast::LiteralValue::Boolean(b) => b.to_string(),
                })
            }
            Node::Identifier(identifier) => {
                if self.context.symbol_table.has_variable(&identifier.name) {
                    Ok(identifier.name.clone())
                } else {
                    Err(CodeGenError::UnknownVariable {
                        name: identifier.name.clone(),
                    })
                }
            }
            Node::ActionInvocation(action) => {
                Ok(self.generate_function_call(&action.action_name, &[]))
            }
            Node::ActionInvocationWithArgs(action) => {
                let args: Result<Vec<String>, CodeGenError> = action.arguments
                    .iter()
                    .map(|arg| self.emit_expression(arg))
                    .collect();
                Ok(self.generate_function_call(&action.action_name, &args?))
            }
            _ => Ok("/* unsupported expression */".to_string()),
        }
    }

    fn emit_statement(&mut self, stmt: &Node) -> Result<(), CodeGenError> {
        match stmt {
            Node::DisplayStatement(display) => {
                let expr = self.emit_expression(&display.expression)?;
                self.context.emit(&format!("print({})", expr));
            }
            Node::Assignment(assignment) => {
                let value_expr = self.emit_expression(&assignment.value)?;
                
                // Infer type from the value
                let inferred_type = self.infer_type(&assignment.value, &self.context.symbol_table);
                
                // Add to symbol table if new variable
                if !self.context.symbol_table.has_variable(&assignment.variable) {
                    self.context.symbol_table.add_variable(assignment.variable.clone(), inferred_type.clone())
                        .map_err(|e| CodeGenError::SymbolError { message: e })?;
                }
                
                self.context.emit(&format!("{} = {}", assignment.variable, value_expr));
            }
            _ => {
                // Emit comment for unsupported statements
                self.context.emit(&format!("// TODO: Implement {:?}", stmt));
            }
        }
        Ok(())
    }

    fn target_language(&self) -> &str {
        "generic"
    }
}

impl Default for DefaultCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_core_libraries() {
        let mut core_libs = CoreLibraries::new();
        
        assert!(!core_libs.is_enabled("string_utils"));
        
        core_libs.enable("string_utils");
        assert!(core_libs.is_enabled("string_utils"));
        
        core_libs.disable("string_utils");
        assert!(!core_libs.is_enabled("string_utils"));
    }

    #[test]
    fn test_codegen_context() {
        let mut context = CodeGenContext::new();
        
        context.emit("test line");
        context.indent();
        context.emit("indented line");
        context.dedent();
        context.emit("normal line");
        
        let output = context.get_output();
        assert!(output.contains("test line"));
        assert!(output.contains("  indented line"));
        assert!(output.contains("normal line"));
    }

    #[test]
    fn test_string_constants() {
        let mut context = CodeGenContext::new();
        
        let index1 = context.add_string_constant("hello");
        let index2 = context.add_string_constant("world");
        let index3 = context.add_string_constant("hello"); // Duplicate
        
        assert_eq!(index1, 0);
        assert_eq!(index2, 1);
        assert_eq!(index3, 0); // Should return same index as first "hello"
    }

    #[test]
    fn test_type_inference() {
        let generator = DefaultCodeGenerator::new();
        let symbol_table = SymbolTable::new();
        
        // Test literal type inference
        let string_literal = Node::Literal(Literal {
            value: LiteralValue::String("test".to_string()),
            literal_type: "string".to_string(),
            line_number: Some(1),
        });
        
        let inferred_type = generator.infer_type(&string_literal, &symbol_table);
        assert_eq!(inferred_type, VariableType::Text);
    }

    #[test]
    fn test_variable_type_mapping() {
        let generator = DefaultCodeGenerator::new();
        
        assert_eq!(generator.map_user_type_to_internal("int").unwrap(), VariableType::Int);
        assert_eq!(generator.map_user_type_to_internal("text").unwrap(), VariableType::Text);
        assert!(generator.map_user_type_to_internal("invalid").is_err());
    }
}