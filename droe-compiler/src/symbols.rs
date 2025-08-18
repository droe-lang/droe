//! Symbol table for variable management in Droe DSL
//! 
//! This module provides type definitions and symbol table management
//! for tracking variables, their types, and their properties during compilation.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Supported variable types in Droe DSL
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VariableType {
    // Numeric types
    Int,
    Decimal,
    Number, // Legacy support
    
    // Boolean types
    Boolean, // Legacy support
    Flag,
    YesNo,
    
    // Text types
    String, // Legacy support
    Text,
    
    // Date type
    Date,
    DateTime,
    
    // Collection types
    Array,   // Legacy support
    ListOf,
    GroupOf,
    
    // File type
    File,
    
    // Unknown/unresolved type
    Unknown,
}

impl VariableType {
    /// Parse a type name string into a VariableType
    pub fn from_str(type_name: &str) -> Self {
        match type_name.to_lowercase().as_str() {
            // Modern types
            "int" => Self::Int,
            "decimal" => Self::Decimal,
            "text" => Self::Text,
            "flag" => Self::Flag,
            "yesno" => Self::YesNo,
            "date" => Self::Date,
            "datetime" => Self::DateTime,
            "file" => Self::File,
            
            // Legacy types
            "number" => Self::Number,
            "boolean" => Self::Boolean,
            "string" => Self::String,
            "array" => Self::Array,
            
            // Collection types
            s if s.starts_with("list_of") => Self::ListOf,
            s if s.starts_with("group_of") => Self::GroupOf,
            
            _ => Self::Unknown,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            Self::Int => "int".to_string(),
            Self::Decimal => "decimal".to_string(),
            Self::Text => "text".to_string(),
            Self::Flag => "flag".to_string(),
            Self::YesNo => "yesno".to_string(),
            Self::Date => "date".to_string(),
            Self::DateTime => "datetime".to_string(),
            Self::File => "file".to_string(),
            Self::Number => "number".to_string(),
            Self::Boolean => "boolean".to_string(),
            Self::String => "string".to_string(),
            Self::Array => "array".to_string(),
            Self::ListOf => "list_of".to_string(),
            Self::GroupOf => "group_of".to_string(),
            Self::Unknown => "unknown".to_string(),
        }
    }

    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Int | Self::Decimal | Self::Number)
    }

    /// Check if this is a text type
    pub fn is_text(&self) -> bool {
        matches!(self, Self::String | Self::Text)
    }

    /// Check if this is a boolean type
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean | Self::Flag | Self::YesNo)
    }

    /// Check if this is a collection type
    pub fn is_collection(&self) -> bool {
        matches!(self, Self::Array | Self::ListOf | Self::GroupOf)
    }

    /// Check if this is a date/time type
    pub fn is_temporal(&self) -> bool {
        matches!(self, Self::Date | Self::DateTime)
    }

    /// Check if two types are compatible for assignment
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Exact match
        if self == other {
            return true;
        }

        // Numeric type compatibility
        if self.is_numeric() && other.is_numeric() {
            return true;
        }

        // Text type compatibility
        if self.is_text() && other.is_text() {
            return true;
        }

        // Boolean type compatibility
        if self.is_boolean() && other.is_boolean() {
            return true;
        }

        // Date-string compatibility
        if *self == Self::Date && other.is_text() {
            return true;
        }

        // Collection type compatibility
        if self.is_collection() && other.is_collection() {
            return true;
        }

        false
    }
}

/// Represents a variable in the symbol table
#[derive(Debug, Clone)]
pub struct Variable {
    /// Variable name
    pub name: String,
    /// Variable type
    pub var_type: VariableType,
    /// Current value (if known at compile time)
    pub value: Option<VariableValue>,
    /// WASM local variable index (for WASM target)
    pub wasm_index: Option<usize>,
    /// Whether the variable is mutable
    pub is_mutable: bool,
    /// Line number where variable was declared
    pub declaration_line: Option<usize>,
}

impl Variable {
    pub fn new(name: String, var_type: VariableType) -> Self {
        Self {
            name,
            var_type,
            value: None,
            wasm_index: None,
            is_mutable: true,
            declaration_line: None,
        }
    }

    pub fn with_value(mut self, value: VariableValue) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_wasm_index(mut self, index: usize) -> Self {
        self.wasm_index = Some(index);
        self
    }

    pub fn immutable(mut self) -> Self {
        self.is_mutable = false;
        self
    }

    pub fn at_line(mut self, line: usize) -> Self {
        self.declaration_line = Some(line);
        self
    }
}

/// Runtime values that variables can hold
#[derive(Debug, Clone, PartialEq)]
pub enum VariableValue {
    Int(i64),
    Decimal(f64),
    Text(String),
    Flag(bool),
    Date(String), // ISO date string
    File(String), // File path
    Array(Vec<VariableValue>),
    Unknown,
}

impl VariableValue {
    /// Get the type of this value
    pub fn get_type(&self) -> VariableType {
        match self {
            Self::Int(_) => VariableType::Int,
            Self::Decimal(_) => VariableType::Decimal,
            Self::Text(_) => VariableType::Text,
            Self::Flag(_) => VariableType::Flag,
            Self::Date(_) => VariableType::Date,
            Self::File(_) => VariableType::File,
            Self::Array(_) => VariableType::Array,
            Self::Unknown => VariableType::Unknown,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            Self::Int(i) => i.to_string(),
            Self::Decimal(f) => f.to_string(),
            Self::Text(s) => s.clone(),
            Self::Flag(b) => b.to_string(),
            Self::Date(d) => d.clone(),
            Self::File(f) => f.clone(),
            Self::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Self::Unknown => "unknown".to_string(),
        }
    }
}

/// Manages variables and their types during compilation
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Variables in the current scope
    variables: HashMap<String, Variable>,
    /// Next available local variable index (for WASM)
    next_local_index: usize,
    /// Parent scope (for nested scopes)
    parent: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            next_local_index: 0,
            parent: None,
        }
    }

    /// Create a new child scope
    pub fn new_scope(parent: SymbolTable) -> Self {
        Self {
            variables: HashMap::new(),
            next_local_index: parent.next_local_index,
            parent: Some(Box::new(parent)),
        }
    }

    /// Add a variable to the symbol table
    pub fn add_variable(&mut self, name: String, var_type: VariableType) -> Result<(), String> {
        if self.variables.contains_key(&name) {
            return Err(format!("Variable '{}' already declared in current scope", name));
        }

        let mut variable = Variable::new(name.clone(), var_type);
        variable.wasm_index = Some(self.next_local_index);
        self.next_local_index += 1;

        self.variables.insert(name, variable);
        Ok(())
    }

    /// Add a variable with a specific value
    pub fn add_variable_with_value(
        &mut self,
        name: String,
        var_type: VariableType,
        value: VariableValue,
    ) -> Result<(), String> {
        self.add_variable(name.clone(), var_type)?;
        
        if let Some(var) = self.variables.get_mut(&name) {
            var.value = Some(value);
        }
        
        Ok(())
    }

    /// Get a variable by name (searches parent scopes)
    pub fn get_variable(&self, name: &str) -> Option<&Variable> {
        if let Some(var) = self.variables.get(name) {
            Some(var)
        } else if let Some(parent) = &self.parent {
            parent.get_variable(name)
        } else {
            None
        }
    }

    /// Get a mutable reference to a variable by name
    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.variables.get_mut(name)
    }

    /// Update a variable's value
    pub fn set_variable_value(&mut self, name: &str, value: VariableValue) -> Result<(), String> {
        if let Some(var) = self.variables.get_mut(name) {
            if !var.is_mutable {
                return Err(format!("Cannot assign to immutable variable '{}'", name));
            }
            var.value = Some(value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.set_variable_value(name, value)
        } else {
            Err(format!("Variable '{}' not found", name))
        }
    }

    /// Check if a variable exists in any scope
    pub fn has_variable(&self, name: &str) -> bool {
        self.get_variable(name).is_some()
    }

    /// Get all variables in the current scope
    pub fn get_current_scope_variables(&self) -> &HashMap<String, Variable> {
        &self.variables
    }

    /// Get all variables including parent scopes
    pub fn get_all_variables(&self) -> HashMap<String, Variable> {
        let mut all_vars = HashMap::new();
        
        // Add parent variables first (so current scope can override)
        if let Some(parent) = &self.parent {
            all_vars.extend(parent.get_all_variables());
        }
        
        // Add current scope variables
        all_vars.extend(self.variables.clone());
        
        all_vars
    }

    /// Get the next available WASM local index
    pub fn get_next_local_index(&self) -> usize {
        self.next_local_index
    }

    /// Reserve WASM local indices
    pub fn reserve_locals(&mut self, count: usize) {
        self.next_local_index += count;
    }

    /// Clear all variables in current scope
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Get the count of local variables (for WASM)
    pub fn get_local_count(&self) -> usize {
        self.variables.len()
    }

    /// Enter a new scope (returns the old symbol table)
    pub fn enter_scope(&mut self) -> SymbolTable {
        let old_table = std::mem::take(self);
        *self = SymbolTable::new_scope(old_table);
        std::mem::take(self)
    }

    /// Exit current scope (restore parent symbol table)
    pub fn exit_scope(&mut self) -> Option<SymbolTable> {
        if let Some(parent) = self.parent.take() {
            let old_scope = std::mem::replace(self, *parent);
            Some(old_scope)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_type_parsing() {
        assert_eq!(VariableType::from_str("int"), VariableType::Int);
        assert_eq!(VariableType::from_str("text"), VariableType::Text);
        assert_eq!(VariableType::from_str("flag"), VariableType::Flag);
        assert_eq!(VariableType::from_str("unknown"), VariableType::Unknown);
    }

    #[test]
    fn test_type_compatibility() {
        let int_type = VariableType::Int;
        let decimal_type = VariableType::Decimal;
        let text_type = VariableType::Text;

        assert!(int_type.is_compatible_with(&decimal_type));
        assert!(decimal_type.is_compatible_with(&int_type));
        assert!(!int_type.is_compatible_with(&text_type));
    }

    #[test]
    fn test_symbol_table_basic_operations() {
        let mut table = SymbolTable::new();
        
        // Add variable
        assert!(table.add_variable("x".to_string(), VariableType::Int).is_ok());
        assert!(table.has_variable("x"));
        
        // Try to add duplicate
        assert!(table.add_variable("x".to_string(), VariableType::Text).is_err());
        
        // Get variable
        let var = table.get_variable("x").unwrap();
        assert_eq!(var.name, "x");
        assert_eq!(var.var_type, VariableType::Int);
    }

    #[test]
    fn test_symbol_table_scopes() {
        let mut table = SymbolTable::new();
        table.add_variable("x".to_string(), VariableType::Int).unwrap();
        
        // Enter new scope
        let old_table = table.enter_scope();
        table.add_variable("y".to_string(), VariableType::Text).unwrap();
        
        // Should see both variables
        assert!(table.has_variable("x")); // From parent
        assert!(table.has_variable("y")); // From current
        
        // Exit scope
        table.exit_scope();
        
        // Should only see original variable
        assert!(table.has_variable("x"));
        assert!(!table.has_variable("y"));
    }

    #[test]
    fn test_variable_values() {
        let mut table = SymbolTable::new();
        
        table.add_variable_with_value(
            "count".to_string(),
            VariableType::Int,
            VariableValue::Int(42),
        ).unwrap();
        
        let var = table.get_variable("count").unwrap();
        assert_eq!(var.value, Some(VariableValue::Int(42)));
        
        // Update value
        table.set_variable_value("count", VariableValue::Int(100)).unwrap();
        let var = table.get_variable("count").unwrap();
        assert_eq!(var.value, Some(VariableValue::Int(100)));
    }
}