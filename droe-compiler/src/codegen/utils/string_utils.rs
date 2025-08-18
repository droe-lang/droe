//! String utility functions for Droe runtime
//! 
//! This module provides comprehensive string manipulation functions with WebAssembly
//! integration and JavaScript runtime support.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringFunction {
    pub name: String,
    pub wasm_import_name: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub return_type: String,
}

impl StringFunction {
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
pub struct StringUtils {
    functions: HashMap<String, StringFunction>,
}

impl StringUtils {
    pub fn new() -> Self {
        let mut string_utils = Self {
            functions: HashMap::new(),
        };
        
        string_utils.register_functions();
        string_utils
    }

    fn register_functions(&mut self) {
        // Basic print functions
        self.functions.insert(
            "print_no_newline".to_string(),
            StringFunction::new(
                "print_no_newline",
                "print_no_newline",
                "Print text without adding newline",
                vec!["offset: i32", "length: i32"],
                "void"
            )
        );

        self.functions.insert(
            "print_string_from_offset_no_newline".to_string(),
            StringFunction::new(
                "print_string_from_offset_no_newline",
                "print_string_from_offset_no_newline",
                "Print null-terminated string without newline",
                vec!["offset: i32"],
                "void"
            )
        );

        self.functions.insert(
            "print_newline".to_string(),
            StringFunction::new(
                "print_newline",
                "print_newline",
                "Print just a newline character",
                vec![],
                "void"
            )
        );

        // Advanced string manipulation functions
        self.functions.insert(
            "string_concat".to_string(),
            StringFunction::new(
                "string_concat",
                "string_concat",
                "Concatenate two strings",
                vec!["str1_offset: i32", "str2_offset: i32"],
                "i32" // Returns offset to concatenated string
            )
        );

        self.functions.insert(
            "string_length".to_string(),
            StringFunction::new(
                "string_length",
                "string_length",
                "Get length of null-terminated string",
                vec!["offset: i32"],
                "i32"
            )
        );

        self.functions.insert(
            "string_substring".to_string(),
            StringFunction::new(
                "string_substring",
                "string_substring",
                "Extract substring from string",
                vec!["offset: i32", "start: i32", "length: i32"],
                "i32" // Returns offset to substring
            )
        );
    }

    pub fn get_functions(&self) -> &HashMap<String, StringFunction> {
        &self.functions
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
            "print_no_newline".to_string(),
            r#"(offset, length) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder("utf-8").decode(bytes);
        process.stdout.write(text);
      }"#.to_string()
        );

        js_functions.insert(
            "print_string_from_offset_no_newline".to_string(),
            r#"(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder("utf-8").decode(stringBytes);
        process.stdout.write(text);
      }"#.to_string()
        );

        js_functions.insert(
            "print_newline".to_string(),
            r#"() => {
        console.log();
      }"#.to_string()
        );

        // Advanced string functions
        js_functions.insert(
            "string_concat".to_string(),
            r#"(str1Offset, str2Offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get first string
        let str1Length = 0;
        while (bytes[str1Offset + str1Length] !== 0) str1Length++;
        const str1 = new TextDecoder("utf-8").decode(new Uint8Array(memory.buffer, str1Offset, str1Length));
        
        // Get second string  
        let str2Length = 0;
        while (bytes[str2Offset + str2Length] !== 0) str2Length++;
        const str2 = new TextDecoder("utf-8").decode(new Uint8Array(memory.buffer, str2Offset, str2Length));
        
        // Concatenate and store result
        const result = str1 + str2;
        const resultBytes = new TextEncoder().encode(result + '\0');
        const resultOffset = memory.buffer.byteLength - 4096; // Use end of memory
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(resultBytes, resultOffset);
        return resultOffset;
      }"#.to_string()
        );

        js_functions.insert(
            "string_length".to_string(),
            r#"(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        return length;
      }"#.to_string()
        );

        js_functions.insert(
            "string_substring".to_string(),
            r#"(offset, start, length) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get original string length
        let strLength = 0;
        while (bytes[offset + strLength] !== 0) strLength++;
        
        // Bounds check
        if (start < 0 || start >= strLength) return offset; // Return original if invalid
        const endPos = Math.min(start + length, strLength);
        
        // Extract substring
        const substring = new Uint8Array(memory.buffer, offset + start, endPos - start);
        const result = new TextDecoder("utf-8").decode(substring);
        
        // Store result
        const resultBytes = new TextEncoder().encode(result + '\0');
        const resultOffset = memory.buffer.byteLength - 5120; // Use end of memory
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(resultBytes, resultOffset);
        return resultOffset;
      }"#.to_string()
        );

        js_functions
    }
}

impl Default for StringUtils {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_functions_registration() {
        let string_utils = StringUtils::new();
        assert!(string_utils.functions.contains_key("print_no_newline"));
        assert!(string_utils.functions.contains_key("string_concat"));
        assert!(string_utils.functions.contains_key("string_length"));
        assert_eq!(string_utils.functions.len(), 6);
    }

    #[test]
    fn test_wasm_import_generation() {
        let string_utils = StringUtils::new();
        let imports = string_utils.get_wasm_imports();
        assert!(!imports.is_empty());
        
        // Check that imports contain expected function
        let print_import = imports.iter().find(|import| import.contains("print_no_newline"));
        assert!(print_import.is_some());
        assert!(print_import.unwrap().contains("(param i32)"));
    }

    #[test]
    fn test_js_runtime_generation() {
        let string_utils = StringUtils::new();
        let js_functions = string_utils.get_js_runtime_functions();
        assert!(!js_functions.is_empty());
        assert!(js_functions.contains_key("print_no_newline"));
        assert!(js_functions.contains_key("string_concat"));
    }

    #[test]
    fn test_function_parameters() {
        let string_utils = StringUtils::new();
        let concat_func = string_utils.functions.get("string_concat").unwrap();
        assert_eq!(concat_func.parameters.len(), 2);
        assert_eq!(concat_func.return_type, "i32");
        
        let newline_func = string_utils.functions.get("print_newline").unwrap();
        assert_eq!(newline_func.parameters.len(), 0);
        assert_eq!(newline_func.return_type, "void");
    }
}