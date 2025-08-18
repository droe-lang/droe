//! Core utility modules for Droe compiler
//! 
//! This module provides comprehensive core libraries for mathematical operations,
//! string manipulation, and formatting functions with full WebAssembly and
//! JavaScript runtime support.

pub mod math_utils;
pub mod string_utils;
pub mod formatting;

pub use math_utils::{MathUtils, MathFunction};
pub use string_utils::{StringUtils, StringFunction};
pub use formatting::{FormattingUtils, FormatFunction};

/// Complete core library utilities
#[derive(Debug, Clone)]
pub struct CoreUtilities {
    pub math: MathUtils,
    pub string: StringUtils,
    pub formatting: FormattingUtils,
}

impl CoreUtilities {
    pub fn new() -> Self {
        Self {
            math: MathUtils::new(),
            string: StringUtils::new(),
            formatting: FormattingUtils::new(),
        }
    }

    /// Get all WebAssembly imports for all core libraries
    pub fn get_all_wasm_imports(&self) -> Vec<String> {
        let mut imports = Vec::new();
        imports.extend(self.math.get_wasm_imports());
        imports.extend(self.string.get_wasm_imports());
        imports.extend(self.formatting.get_wasm_imports());
        imports
    }

    /// Get all JavaScript runtime functions for all core libraries
    pub fn get_all_js_runtime_functions(&self) -> std::collections::HashMap<String, String> {
        let mut js_functions = std::collections::HashMap::new();
        js_functions.extend(self.math.get_js_runtime_functions());
        js_functions.extend(self.string.get_js_runtime_functions());
        js_functions.extend(self.formatting.get_js_runtime_functions());
        js_functions
    }

    /// Get summary of all available functions
    pub fn get_function_summary(&self) -> Vec<String> {
        let mut summary = Vec::new();
        
        summary.push("=== Math Functions ===".to_string());
        for (name, func) in self.math.get_functions() {
            summary.push(format!("  {}: {}", name, func.description));
        }
        
        summary.push("\n=== String Functions ===".to_string());
        for (name, func) in self.string.get_functions() {
            summary.push(format!("  {}: {}", name, func.description));
        }
        
        summary.push("\n=== Formatting Functions ===".to_string());
        for (name, func) in self.formatting.get_functions() {
            summary.push(format!("  {}: {}", name, func.description));
        }
        
        summary
    }

    /// Check if a function exists in any of the core libraries
    pub fn has_function(&self, function_name: &str) -> bool {
        self.math.get_functions().contains_key(function_name) ||
        self.string.get_functions().contains_key(function_name) ||
        self.formatting.get_functions().contains_key(function_name)
    }

    /// Get function description if it exists
    pub fn get_function_description(&self, function_name: &str) -> Option<String> {
        if let Some(func) = self.math.get_functions().get(function_name) {
            return Some(func.description.clone());
        }
        if let Some(func) = self.string.get_functions().get(function_name) {
            return Some(func.description.clone());
        }
        if let Some(func) = self.formatting.get_functions().get(function_name) {
            return Some(func.description.clone());
        }
        None
    }
}

impl Default for CoreUtilities {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_utilities_creation() {
        let core_utils = CoreUtilities::new();
        assert!(core_utils.has_function("math_abs_i32"));
        assert!(core_utils.has_function("string_concat"));
        assert!(core_utils.has_function("format_date"));
    }

    #[test]
    fn test_function_lookup() {
        let core_utils = CoreUtilities::new();
        
        assert!(core_utils.has_function("math_abs_i32"));
        assert!(core_utils.has_function("print_no_newline"));
        assert!(core_utils.has_function("format_decimal"));
        assert!(!core_utils.has_function("nonexistent_function"));
    }

    #[test]
    fn test_function_description() {
        let core_utils = CoreUtilities::new();
        
        let description = core_utils.get_function_description("math_abs_i32");
        assert!(description.is_some());
        assert!(description.unwrap().contains("Absolute value"));
        
        let no_description = core_utils.get_function_description("nonexistent");
        assert!(no_description.is_none());
    }

    #[test]
    fn test_all_imports_generation() {
        let core_utils = CoreUtilities::new();
        let imports = core_utils.get_all_wasm_imports();
        assert!(!imports.is_empty());
        assert!(imports.len() > 20); // Should have math + string + formatting functions
    }

    #[test]
    fn test_all_js_functions_generation() {
        let core_utils = CoreUtilities::new();
        let js_functions = core_utils.get_all_js_runtime_functions();
        assert!(!js_functions.is_empty());
        assert!(js_functions.len() > 20); // Should have all function implementations
    }

    #[test]
    fn test_function_summary() {
        let core_utils = CoreUtilities::new();
        let summary = core_utils.get_function_summary();
        assert!(!summary.is_empty());
        
        // Should contain section headers
        assert!(summary.iter().any(|line| line.contains("Math Functions")));
        assert!(summary.iter().any(|line| line.contains("String Functions")));
        assert!(summary.iter().any(|line| line.contains("Formatting Functions")));
    }
}