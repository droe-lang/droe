//! Formatting utility functions for Droe runtime
//! 
//! This module provides comprehensive formatting functions for dates, numbers,
//! and decimals with WebAssembly integration and JavaScript runtime support.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatFunction {
    pub name: String,
    pub wasm_import_name: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub return_type: String,
}

impl FormatFunction {
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
pub struct FormattingUtils {
    functions: HashMap<String, FormatFunction>,
    date_patterns: HashMap<String, String>,
    decimal_patterns: HashMap<String, String>,
    number_patterns: HashMap<String, String>,
}

impl FormattingUtils {
    pub fn new() -> Self {
        let mut formatting_utils = Self {
            functions: HashMap::new(),
            date_patterns: HashMap::new(),
            decimal_patterns: HashMap::new(),
            number_patterns: HashMap::new(),
        };
        
        formatting_utils.register_functions();
        formatting_utils.register_patterns();
        formatting_utils
    }

    fn register_functions(&mut self) {
        // Basic print functions
        self.functions.insert(
            "print_i32_no_newline".to_string(),
            FormatFunction::new(
                "print_i32_no_newline",
                "print_i32_no_newline",
                "Print integer without newline",
                vec!["value: i32"],
                "void"
            )
        );

        self.functions.insert(
            "print_decimal_no_newline".to_string(),
            FormatFunction::new(
                "print_decimal_no_newline",
                "print_decimal_no_newline",
                "Print scaled decimal without newline",
                vec!["scaledValue: i32"],
                "void"
            )
        );

        self.functions.insert(
            "print_decimal".to_string(),
            FormatFunction::new(
                "print_decimal",
                "print_decimal",
                "Print scaled decimal with newline",
                vec!["scaledValue: i32"],
                "void"
            )
        );

        self.functions.insert(
            "print_date".to_string(),
            FormatFunction::new(
                "print_date",
                "print_date",
                "Print date from string offset",
                vec!["offset: i32"],
                "void"
            )
        );

        self.functions.insert(
            "print_date_no_newline".to_string(),
            FormatFunction::new(
                "print_date_no_newline",
                "print_date_no_newline",
                "Print date without newline",
                vec!["offset: i32"],
                "void"
            )
        );

        // Advanced formatting functions
        self.functions.insert(
            "format_date".to_string(),
            FormatFunction::new(
                "format_date",
                "format_date",
                "Format date according to pattern",
                vec!["dateOffset: i32", "patternOffset: i32"],
                "i32" // Returns formatted string offset
            )
        );

        self.functions.insert(
            "format_decimal".to_string(),
            FormatFunction::new(
                "format_decimal",
                "format_decimal",
                "Format decimal according to pattern",
                vec!["scaledValue: i32", "patternOffset: i32"],
                "i32" // Returns formatted string offset
            )
        );

        self.functions.insert(
            "format_number".to_string(),
            FormatFunction::new(
                "format_number",
                "format_number",
                "Format number according to pattern",
                vec!["value: i32", "patternOffset: i32"],
                "i32" // Returns formatted string offset
            )
        );
    }

    fn register_patterns(&mut self) {
        // Standard format patterns
        self.date_patterns.insert("MM/dd/yyyy".to_string(), "US date format (12/31/2024)".to_string());
        self.date_patterns.insert("dd/MM/yyyy".to_string(), "European date format (31/12/2024)".to_string());
        self.date_patterns.insert("MMM dd, yyyy".to_string(), "Long date format (Dec 31, 2024)".to_string());
        self.date_patterns.insert("long".to_string(), "Full date format (Tuesday, December 31, 2024)".to_string());
        self.date_patterns.insert("short".to_string(), "Short date format (12/31/24)".to_string());
        self.date_patterns.insert("iso".to_string(), "ISO date format (2024-12-31)".to_string());

        self.decimal_patterns.insert("0.00".to_string(), "Basic decimal format (123.45)".to_string());
        self.decimal_patterns.insert("#,##0.00".to_string(), "Decimal with thousands separator (1,234.56)".to_string());
        self.decimal_patterns.insert("$0.00".to_string(), "Currency format ($123.45)".to_string());
        self.decimal_patterns.insert("0.0000".to_string(), "High precision decimal (123.4500)".to_string());
        self.decimal_patterns.insert("percent".to_string(), "Percentage format (12.34%)".to_string());

        self.number_patterns.insert("#,##0".to_string(), "Number with thousands separator (1,234)".to_string());
        self.number_patterns.insert("0000".to_string(), "Zero-padded number (0123)".to_string());
        self.number_patterns.insert("hex".to_string(), "Hexadecimal format (0x7B)".to_string());
        self.number_patterns.insert("oct".to_string(), "Octal format (0o173)".to_string());
        self.number_patterns.insert("bin".to_string(), "Binary format (0b1111011)".to_string());
    }

    pub fn get_functions(&self) -> &HashMap<String, FormatFunction> {
        &self.functions
    }

    pub fn get_date_patterns(&self) -> &HashMap<String, String> {
        &self.date_patterns
    }

    pub fn get_decimal_patterns(&self) -> &HashMap<String, String> {
        &self.decimal_patterns
    }

    pub fn get_number_patterns(&self) -> &HashMap<String, String> {
        &self.number_patterns
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

        // Basic print functions
        js_functions.insert(
            "print_i32_no_newline".to_string(),
            r#"(value) => {
        process.stdout.write(value.toString());
      }"#.to_string()
        );

        js_functions.insert(
            "print_decimal_no_newline".to_string(),
            r#"(scaledValue) => {
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        const formattedDecimal = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        process.stdout.write(formattedDecimal);
      }"#.to_string()
        );

        js_functions.insert(
            "print_decimal".to_string(),
            r#"(scaledValue) => {
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        const formattedDecimal = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        console.log(formattedDecimal);
      }"#.to_string()
        );

        js_functions.insert(
            "print_date".to_string(),
            r#"(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const dateString = new TextDecoder("utf-8").decode(stringBytes);
        console.log(dateString);
      }"#.to_string()
        );

        js_functions.insert(
            "print_date_no_newline".to_string(),
            r#"(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const dateString = new TextDecoder("utf-8").decode(stringBytes);
        process.stdout.write(dateString);
      }"#.to_string()
        );

        // Advanced formatting functions
        js_functions.insert(
            "format_date".to_string(),
            r#"(dateOffset, patternOffset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get the date string
        let dateLength = 0;
        while (bytes[dateOffset + dateLength] !== 0 && dateOffset + dateLength < bytes.length) {
          dateLength++;
        }
        const dateBytes = new Uint8Array(memory.buffer, dateOffset, dateLength);
        const dateString = new TextDecoder("utf-8").decode(dateBytes);
        
        // Get the pattern string
        let patternLength = 0;
        while (bytes[patternOffset + patternLength] !== 0 && patternOffset + patternLength < bytes.length) {
          patternLength++;
        }
        const patternBytes = new Uint8Array(memory.buffer, patternOffset, patternLength);
        const pattern = new TextDecoder("utf-8").decode(patternBytes);
        
        // Format the date based on pattern
        let formatted = dateString;
        try {
          const date = new Date(dateString);
          if (pattern === "MM/dd/yyyy") {
            const month = (date.getMonth() + 1).toString().padStart(2, '0');
            const day = date.getDate().toString().padStart(2, '0');
            const year = date.getFullYear();
            formatted = `${month}/${day}/${year}`;
          } else if (pattern === "dd/MM/yyyy") {
            const month = (date.getMonth() + 1).toString().padStart(2, '0');
            const day = date.getDate().toString().padStart(2, '0');
            const year = date.getFullYear();
            formatted = `${day}/${month}/${year}`;
          } else if (pattern === "MMM dd, yyyy") {
            const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                          "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            const month = months[date.getMonth()];
            const day = date.getDate();
            const year = date.getFullYear();
            formatted = `${month} ${day}, ${year}`;
          } else if (pattern === "long") {
            formatted = date.toLocaleDateString('en-US', { 
              weekday: 'long', 
              year: 'numeric', 
              month: 'long', 
              day: 'numeric' 
            });
          } else if (pattern === "short") {
            formatted = date.toLocaleDateString('en-US', { 
              year: '2-digit', 
              month: '2-digit', 
              day: '2-digit' 
            });
          } else if (pattern === "iso") {
            formatted = date.toISOString().split('T')[0];
          }
        } catch (e) {
          // If date parsing fails, return original string
        }
        
        // Store the formatted result in memory and return its offset
        const formattedBytes = new TextEncoder().encode(formatted + '\0');
        const resultOffset = memory.buffer.byteLength - 1024;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      }"#.to_string()
        );

        js_functions.insert(
            "format_decimal".to_string(),
            r##"(scaledValue, patternOffset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get the pattern string
        let patternLength = 0;
        while (bytes[patternOffset + patternLength] !== 0 && patternOffset + patternLength < bytes.length) {
          patternLength++;
        }
        const patternBytes = new Uint8Array(memory.buffer, patternOffset, patternLength);
        const pattern = new TextDecoder("utf-8").decode(patternBytes);
        
        // Convert scaled value to decimal
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        
        let formatted;
        if (pattern === "0.00") {
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        } else if (pattern === "#,##0.00") {
          formatted = `${integerPart.toLocaleString()}.${fractionalPart.toString().padStart(2, '0')}`;
        } else if (pattern === "$0.00") {
          formatted = `$${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        } else if (pattern === "0.0000") {
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}00`;
        } else if (pattern === "percent") {
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}%`;
        } else {
          // Default formatting
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        }
        
        // Store the formatted result in memory and return its offset
        const formattedBytes = new TextEncoder().encode(formatted + '\0');
        const resultOffset = memory.buffer.byteLength - 2048;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      }"##.to_string()
        );

        js_functions.insert(
            "format_number".to_string(),
            r##"(value, patternOffset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get the pattern string
        let patternLength = 0;
        while (bytes[patternOffset + patternLength] !== 0 && patternOffset + patternLength < bytes.length) {
          patternLength++;
        }
        const patternBytes = new Uint8Array(memory.buffer, patternOffset, patternLength);
        const pattern = new TextDecoder("utf-8").decode(patternBytes);
        
        let formatted;
        if (pattern === "#,##0") {
          formatted = value.toLocaleString();
        } else if (pattern === "0000") {
          formatted = value.toString().padStart(4, '0');
        } else if (pattern === "hex") {
          formatted = "0x" + value.toString(16).toUpperCase();
        } else if (pattern === "oct") {
          formatted = "0o" + value.toString(8);
        } else if (pattern === "bin") {
          formatted = "0b" + value.toString(2);
        } else {
          // Default formatting
          formatted = value.toString();
        }
        
        // Store the formatted result in memory and return its offset
        const formattedBytes = new TextEncoder().encode(formatted + '\0');
        const resultOffset = memory.buffer.byteLength - 3072;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      }"##.to_string()
        );

        js_functions
    }
}

impl Default for FormattingUtils {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatting_functions_registration() {
        let formatting_utils = FormattingUtils::new();
        assert!(formatting_utils.functions.contains_key("print_i32_no_newline"));
        assert!(formatting_utils.functions.contains_key("format_date"));
        assert!(formatting_utils.functions.contains_key("format_decimal"));
        assert!(formatting_utils.functions.contains_key("format_number"));
        assert_eq!(formatting_utils.functions.len(), 8);
    }

    #[test]
    fn test_pattern_registration() {
        let formatting_utils = FormattingUtils::new();
        
        // Check date patterns
        assert!(formatting_utils.date_patterns.contains_key("MM/dd/yyyy"));
        assert!(formatting_utils.date_patterns.contains_key("iso"));
        
        // Check decimal patterns
        assert!(formatting_utils.decimal_patterns.contains_key("0.00"));
        assert!(formatting_utils.decimal_patterns.contains_key("$0.00"));
        
        // Check number patterns
        assert!(formatting_utils.number_patterns.contains_key("#,##0"));
        assert!(formatting_utils.number_patterns.contains_key("hex"));
    }

    #[test]
    fn test_wasm_import_generation() {
        let formatting_utils = FormattingUtils::new();
        let imports = formatting_utils.get_wasm_imports();
        assert!(!imports.is_empty());
        
        // Check that imports contain expected function
        let format_import = imports.iter().find(|import| import.contains("format_date"));
        assert!(format_import.is_some());
        assert!(format_import.unwrap().contains("(param i32)"));
        assert!(format_import.unwrap().contains("(result i32)"));
    }

    #[test]
    fn test_js_runtime_generation() {
        let formatting_utils = FormattingUtils::new();
        let js_functions = formatting_utils.get_js_runtime_functions();
        assert!(!js_functions.is_empty());
        assert!(js_functions.contains_key("format_date"));
        assert!(js_functions.contains_key("format_decimal"));
        assert!(js_functions.contains_key("format_number"));
    }

    #[test]
    fn test_function_return_types() {
        let formatting_utils = FormattingUtils::new();
        
        let print_func = formatting_utils.functions.get("print_i32_no_newline").unwrap();
        assert_eq!(print_func.return_type, "void");
        
        let format_func = formatting_utils.functions.get("format_date").unwrap();
        assert_eq!(format_func.return_type, "i32");
    }
}