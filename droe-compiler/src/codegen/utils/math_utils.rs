//! Math utility functions for Droe runtime
//! 
//! This module provides comprehensive mathematical functions with WebAssembly
//! integration and JavaScript runtime support.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathFunction {
    pub name: String,
    pub wasm_import_name: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub return_type: String,
}

impl MathFunction {
    pub fn new(
        name: &str,
        wasm_import_name: &str,
        description: &str,
        parameters: Vec<&str>,
        return_type: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            wasm_import_name: wasm_import_name.to_string(),
            description: description.to_string(),
            parameters: parameters.iter().map(|p| p.to_string()).collect(),
            return_type: return_type.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MathUtils {
    functions: HashMap<String, MathFunction>,
    constants: HashMap<String, i32>,
}

impl MathUtils {
    pub fn new() -> Self {
        let mut math_utils = Self {
            functions: HashMap::new(),
            constants: HashMap::new(),
        };
        
        math_utils.register_functions();
        math_utils.register_constants();
        math_utils
    }

    fn register_functions(&mut self) {
        // Basic math functions
        self.functions.insert(
            "math_abs_i32".to_string(),
            MathFunction::new(
                "math_abs_i32",
                "math_abs_i32",
                "Absolute value of 32-bit integer",
                vec!["value: i32"],
                "i32"
            )
        );

        self.functions.insert(
            "math_abs_decimal".to_string(),
            MathFunction::new(
                "math_abs_decimal",
                "math_abs_decimal",
                "Absolute value of scaled decimal",
                vec!["value: i32"],
                "i32"
            )
        );

        self.functions.insert(
            "math_min_i32".to_string(),
            MathFunction::new(
                "math_min_i32",
                "math_min_i32",
                "Minimum of two 32-bit integers",
                vec!["a: i32", "b: i32"],
                "i32"
            )
        );

        self.functions.insert(
            "math_max_i32".to_string(),
            MathFunction::new(
                "math_max_i32",
                "math_max_i32",
                "Maximum of two 32-bit integers",
                vec!["a: i32", "b: i32"],
                "i32"
            )
        );

        self.functions.insert(
            "math_power_i32".to_string(),
            MathFunction::new(
                "math_power_i32",
                "math_power_i32",
                "Power function for integers (a^b)",
                vec!["base: i32", "exponent: i32"],
                "i32"
            )
        );

        self.functions.insert(
            "math_sqrt_decimal".to_string(),
            MathFunction::new(
                "math_sqrt_decimal",
                "math_sqrt_decimal",
                "Square root of scaled decimal",
                vec!["value: i32"],
                "i32"
            )
        );

        self.functions.insert(
            "math_decimal_multiply".to_string(),
            MathFunction::new(
                "math_decimal_multiply",
                "math_decimal_multiply",
                "Multiply two scaled decimals",
                vec!["a: i32", "b: i32"],
                "i32"
            )
        );

        self.functions.insert(
            "math_decimal_divide".to_string(),
            MathFunction::new(
                "math_decimal_divide",
                "math_decimal_divide",
                "Divide two scaled decimals",
                vec!["a: i32", "b: i32"],
                "i32"
            )
        );
    }

    fn register_constants(&mut self) {
        // Built-in math constants (scaled for decimal representation)
        self.constants.insert("MATH_PI".to_string(), 314);      // π * 100 = 3.14 * 100
        self.constants.insert("MATH_E".to_string(), 272);       // e * 100 = 2.72 * 100
        self.constants.insert("MATH_SQRT2".to_string(), 141);   // √2 * 100 = 1.41 * 100
        self.constants.insert("MATH_SQRT3".to_string(), 173);   // √3 * 100 = 1.73 * 100
    }

    pub fn get_functions(&self) -> &HashMap<String, MathFunction> {
        &self.functions
    }

    pub fn get_constants(&self) -> &HashMap<String, i32> {
        &self.constants
    }

    pub fn get_wasm_imports(&self) -> Vec<String> {
        let mut imports = Vec::new();
        
        for func in self.functions.values() {
            let param_list = if !func.parameters.is_empty() {
                func.parameters
                    .iter()
                    .map(|p| {
                        let parts: Vec<&str> = p.split(':').collect();
                        if parts.len() >= 2 {
                            format!("(param {})", parts[1].trim())
                        } else {
                            String::new()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            } else {
                String::new()
            };

            let return_decl = if func.return_type != "void" {
                format!(" (result {})", func.return_type)
            } else {
                String::new()
            };

            let import_decl = format!(
                "(import \"env\" \"{}\" (func ${} {}{}))",
                func.wasm_import_name,
                func.name,
                param_list,
                return_decl
            );
            imports.push(import_decl);
        }
        
        imports
    }

    pub fn get_js_runtime_functions(&self) -> HashMap<String, String> {
        let mut js_functions = HashMap::new();

        // Basic math functions
        js_functions.insert(
            "math_abs_i32".to_string(),
            r#"(value) => {
        return Math.abs(value);
      }"#.to_string()
        );

        js_functions.insert(
            "math_abs_decimal".to_string(),
            r#"(scaledValue) => {
        return Math.abs(scaledValue);
      }"#.to_string()
        );

        js_functions.insert(
            "math_min_i32".to_string(),
            r#"(a, b) => {
        return Math.min(a, b);
      }"#.to_string()
        );

        js_functions.insert(
            "math_max_i32".to_string(),
            r#"(a, b) => {
        return Math.max(a, b);
      }"#.to_string()
        );

        js_functions.insert(
            "math_power_i32".to_string(),
            r#"(base, exponent) => {
        if (exponent < 0) return 0; // Integer power doesn't support negative exponents
        let result = 1;
        for (let i = 0; i < exponent; i++) {
          result *= base;
        }
        return result;
      }"#.to_string()
        );

        // Decimal math (scaled by 100)
        js_functions.insert(
            "math_sqrt_decimal".to_string(),
            r#"(scaledValue) => {
        const realValue = scaledValue / 100.0;
        const result = Math.sqrt(realValue);
        return Math.round(result * 100); // Scale back to integer
      }"#.to_string()
        );

        js_functions.insert(
            "math_decimal_multiply".to_string(),
            r#"(a, b) => {
        // Both are scaled by 100, so result is scaled by 10000
        // Divide by 100 to get back to scale of 100
        return Math.round((a * b) / 100);
      }"#.to_string()
        );

        js_functions.insert(
            "math_decimal_divide".to_string(),
            r#"(a, b) => {
        if (b === 0) return 0; // Avoid division by zero
        // a is scaled by 100, b is scaled by 100
        // To maintain scaling, multiply a by 100 before division
        return Math.round((a * 100) / b);
      }"#.to_string()
        );

        js_functions
    }
}

impl Default for MathUtils {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_functions_registration() {
        let math_utils = MathUtils::new();
        assert!(math_utils.functions.contains_key("math_abs_i32"));
        assert!(math_utils.functions.contains_key("math_decimal_multiply"));
        assert_eq!(math_utils.functions.len(), 8);
    }

    #[test]
    fn test_math_constants() {
        let math_utils = MathUtils::new();
        assert_eq!(math_utils.constants.get("MATH_PI"), Some(&314));
        assert_eq!(math_utils.constants.get("MATH_E"), Some(&272));
    }

    #[test]
    fn test_wasm_import_generation() {
        let math_utils = MathUtils::new();
        let imports = math_utils.get_wasm_imports();
        assert!(!imports.is_empty());
        
        // Check that imports contain expected function
        let abs_import = imports.iter().find(|import| import.contains("math_abs_i32"));
        assert!(abs_import.is_some());
        assert!(abs_import.unwrap().contains("(param i32)"));
        assert!(abs_import.unwrap().contains("(result i32)"));
    }

    #[test]
    fn test_js_runtime_generation() {
        let math_utils = MathUtils::new();
        let js_functions = math_utils.get_js_runtime_functions();
        assert!(!js_functions.is_empty());
        assert!(js_functions.contains_key("math_abs_i32"));
        assert!(js_functions.contains_key("math_decimal_multiply"));
    }
}