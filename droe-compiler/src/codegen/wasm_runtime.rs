//! WebAssembly runtime builder for JavaScript execution
//! 
//! This module generates the JavaScript runtime functions needed
//! to execute compiled WebAssembly modules with core library support.

use std::collections::HashMap;
use crate::codegen_base::CoreLibraries;

pub struct WasmRuntimeBuilder {
    enabled_libs: Vec<String>,
    core_libs: CoreLibraries,
}

impl WasmRuntimeBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            enabled_libs: Vec::new(),
            core_libs: CoreLibraries::new(),
        };
        
        // Enable default libraries
        builder.enable_library("string_utils");
        builder.enable_library("math_utils");
        builder.enable_library("formatting");
        
        builder
    }
    
    pub fn enable_library(&mut self, lib_name: &str) {
        if !self.enabled_libs.contains(&lib_name.to_string()) {
            self.enabled_libs.push(lib_name.to_string());
        }
        self.core_libs.enable(lib_name);
    }
    
    pub fn disable_library(&mut self, lib_name: &str) {
        self.enabled_libs.retain(|lib| lib != lib_name);
        self.core_libs.disable(lib_name);
    }
    
    pub fn generate_runtime_js(&self) -> String {
        let mut runtime_code = Vec::new();
        
        // Add header
        runtime_code.extend(vec![
            "const fs = require(\"fs\");".to_string(),
            "const path = process.argv[2];".to_string(),
            "".to_string(),
            "(async () => {".to_string(),
            "  const wasmBuffer = fs.readFileSync(path);".to_string(),
            "".to_string(),
            "  const importObject = {".to_string(),
            "    env: {".to_string(),
            "      // Basic runtime functions".to_string(),
        ]);
        
        // Add basic print functions
        runtime_code.extend(self.generate_basic_print_functions());
        
        // Add core library functions using new utilities
        runtime_code.extend(self.generate_core_library_functions());
        
        // Add footer
        runtime_code.extend(vec![
            "    },".to_string(),
            "  };".to_string(),
            "".to_string(),
            "  const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);".to_string(),
            "".to_string(),
            "  instance.exports.main();".to_string(),
            "})();".to_string(),
            "".to_string(),
        ]);
        
        runtime_code.join("\n")
    }
    
    fn generate_core_library_functions(&self) -> Vec<String> {
        let js_functions = self.core_libs.get_js_runtime_functions();
        let mut lines = Vec::new();
        
        for (func_name, func_code) in js_functions {
            lines.push(format!("      {}: {},", func_name, func_code));
        }
        
        lines
    }

    fn generate_basic_print_functions(&self) -> Vec<String> {
        vec![
            "      print: (offset, length) => {".to_string(),
            "        const memory = instance.exports.memory;".to_string(),
            "        const bytes = new Uint8Array(memory.buffer, offset, length);".to_string(),
            "        const text = new TextDecoder(\"utf-8\").decode(bytes);".to_string(),
            "        console.log(text);".to_string(),
            "      },".to_string(),
            "      print_i32: (value) => {".to_string(),
            "        console.log(value);".to_string(),
            "      },".to_string(),
            "      print_string_from_offset: (offset) => {".to_string(),
            "        const memory = instance.exports.memory;".to_string(),
            "        const bytes = new Uint8Array(memory.buffer);".to_string(),
            "        let length = 0;".to_string(),
            "        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {".to_string(),
            "          length++;".to_string(),
            "        }".to_string(),
            "        const stringBytes = new Uint8Array(memory.buffer, offset, length);".to_string(),
            "        const text = new TextDecoder(\"utf-8\").decode(stringBytes);".to_string(),
            "        console.log(text);".to_string(),
            "      },".to_string(),
        ]
    }
    
    fn generate_string_utils_js(&self) -> Vec<String> {
        let js_functions = self.get_string_utils_functions();
        let mut lines = Vec::new();
        
        for (func_name, func_code) in js_functions {
            lines.push(format!("      {}: {},", func_name, func_code));
        }
        
        lines
    }
    
    fn generate_math_utils_js(&self) -> Vec<String> {
        let js_functions = self.get_math_utils_functions();
        let mut lines = Vec::new();
        
        for (func_name, func_code) in js_functions {
            lines.push(format!("      {}: {},", func_name, func_code));
        }
        
        lines
    }
    
    fn generate_formatting_js(&self) -> Vec<String> {
        let js_functions = self.get_formatting_utils_functions();
        let mut lines = Vec::new();
        
        for (func_name, func_code) in js_functions {
            lines.push(format!("      {}: {},", func_name, func_code));
        }
        
        lines
    }
    
    fn get_string_utils_functions(&self) -> HashMap<String, String> {
        let mut functions = HashMap::new();
        
        functions.insert("print_no_newline".to_string(), 
            "(offset, length) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder(\"utf-8\").decode(bytes);
        process.stdout.write(text);
      }".to_string());
        
        functions.insert("print_string_from_offset_no_newline".to_string(),
            "(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder(\"utf-8\").decode(stringBytes);
        process.stdout.write(text);
      }".to_string());
        
        functions.insert("print_newline".to_string(),
            "() => {
        console.log();
      }".to_string());
        
        functions.insert("string_concat".to_string(),
            "(str1Offset, str2Offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get first string
        let str1Length = 0;
        while (bytes[str1Offset + str1Length] !== 0) str1Length++;
        const str1 = new TextDecoder(\"utf-8\").decode(new Uint8Array(memory.buffer, str1Offset, str1Length));
        
        // Get second string  
        let str2Length = 0;
        while (bytes[str2Offset + str2Length] !== 0) str2Length++;
        const str2 = new TextDecoder(\"utf-8\").decode(new Uint8Array(memory.buffer, str2Offset, str2Length));
        
        // Concatenate and store result
        const result = str1 + str2;
        const resultBytes = new TextEncoder().encode(result + '\\0');
        const resultOffset = memory.buffer.byteLength - 4096; // Use end of memory
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(resultBytes, resultOffset);
        return resultOffset;
      }".to_string());
        
        functions.insert("string_length".to_string(),
            "(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        return length;
      }".to_string());
        
        functions.insert("string_substring".to_string(),
            "(offset, start, length) => {
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
        const result = new TextDecoder(\"utf-8\").decode(substring);
        
        // Store result
        const resultBytes = new TextEncoder().encode(result + '\\0');
        const resultOffset = memory.buffer.byteLength - 5120; // Use end of memory
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(resultBytes, resultOffset);
        return resultOffset;
      }".to_string());
        
        functions
    }
    
    fn get_math_utils_functions(&self) -> HashMap<String, String> {
        let mut functions = HashMap::new();
        
        functions.insert("math_abs_i32".to_string(),
            "(value) => {
        return Math.abs(value);
      }".to_string());
        
        functions.insert("math_abs_decimal".to_string(),
            "(scaledValue) => {
        return Math.abs(scaledValue);
      }".to_string());
        
        functions.insert("math_min_i32".to_string(),
            "(a, b) => {
        return Math.min(a, b);
      }".to_string());
        
        functions.insert("math_max_i32".to_string(),
            "(a, b) => {
        return Math.max(a, b);
      }".to_string());
        
        functions.insert("math_power_i32".to_string(),
            "(base, exponent) => {
        if (exponent < 0) return 0; // Integer power doesn't support negative exponents
        let result = 1;
        for (let i = 0; i < exponent; i++) {
          result *= base;
        }
        return result;
      }".to_string());
        
        functions.insert("math_sqrt_decimal".to_string(),
            "(scaledValue) => {
        const realValue = scaledValue / 100.0;
        const result = Math.sqrt(realValue);
        return Math.round(result * 100); // Scale back to integer
      }".to_string());
        
        functions.insert("math_decimal_multiply".to_string(),
            "(a, b) => {
        // Both are scaled by 100, so result is scaled by 10000
        // Divide by 100 to get back to scale of 100
        return Math.round((a * b) / 100);
      }".to_string());
        
        functions.insert("math_decimal_divide".to_string(),
            "(a, b) => {
        if (b === 0) return 0; // Avoid division by zero
        // a is scaled by 100, b is scaled by 100
        // To maintain scaling, multiply a by 100 before division
        return Math.round((a * 100) / b);
      }".to_string());
        
        functions
    }
    
    fn get_formatting_utils_functions(&self) -> HashMap<String, String> {
        let mut functions = HashMap::new();
        
        functions.insert("print_i32_no_newline".to_string(),
            "(value) => {
        process.stdout.write(value.toString());
      }".to_string());
        
        functions.insert("print_decimal_no_newline".to_string(),
            "(scaledValue) => {
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        const formattedDecimal = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        process.stdout.write(formattedDecimal);
      }".to_string());
        
        functions.insert("print_decimal".to_string(),
            "(scaledValue) => {
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        const formattedDecimal = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        console.log(formattedDecimal);
      }".to_string());
        
        functions.insert("print_date".to_string(),
            "(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const dateString = new TextDecoder(\"utf-8\").decode(stringBytes);
        console.log(dateString);
      }".to_string());
        
        functions.insert("print_date_no_newline".to_string(),
            "(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const dateString = new TextDecoder(\"utf-8\").decode(stringBytes);
        process.stdout.write(dateString);
      }".to_string());
        
        functions.insert("format_date".to_string(),
            "(dateOffset, patternOffset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get the date string
        let dateLength = 0;
        while (bytes[dateOffset + dateLength] !== 0 && dateOffset + dateLength < bytes.length) {
          dateLength++;
        }
        const dateBytes = new Uint8Array(memory.buffer, dateOffset, dateLength);
        const dateString = new TextDecoder(\"utf-8\").decode(dateBytes);
        
        // Get the pattern string
        let patternLength = 0;
        while (bytes[patternOffset + patternLength] !== 0 && patternOffset + patternLength < bytes.length) {
          patternLength++;
        }
        const patternBytes = new Uint8Array(memory.buffer, patternOffset, patternLength);
        const pattern = new TextDecoder(\"utf-8\").decode(patternBytes);
        
        // Format the date based on pattern
        let formatted = dateString;
        try {
          const date = new Date(dateString);
          if (pattern === \"MM/dd/yyyy\") {
            const month = (date.getMonth() + 1).toString().padStart(2, '0');
            const day = date.getDate().toString().padStart(2, '0');
            const year = date.getFullYear();
            formatted = `${month}/${day}/${year}`;
          } else if (pattern === \"dd/MM/yyyy\") {
            const month = (date.getMonth() + 1).toString().padStart(2, '0');
            const day = date.getDate().toString().padStart(2, '0');
            const year = date.getFullYear();
            formatted = `${day}/${month}/${year}`;
          } else if (pattern === \"MMM dd, yyyy\") {
            const months = [\"Jan\", \"Feb\", \"Mar\", \"Apr\", \"May\", \"Jun\",
                          \"Jul\", \"Aug\", \"Sep\", \"Oct\", \"Nov\", \"Dec\"];
            const month = months[date.getMonth()];
            const day = date.getDate();
            const year = date.getFullYear();
            formatted = `${month} ${day}, ${year}`;
          } else if (pattern === \"long\") {
            formatted = date.toLocaleDateString('en-US', { 
              weekday: 'long', 
              year: 'numeric', 
              month: 'long', 
              day: 'numeric' 
            });
          } else if (pattern === \"short\") {
            formatted = date.toLocaleDateString('en-US', { 
              year: '2-digit', 
              month: '2-digit', 
              day: '2-digit' 
            });
          } else if (pattern === \"iso\") {
            formatted = date.toISOString().split('T')[0];
          }
        } catch (e) {
          // If date parsing fails, return original string
        }
        
        // Store the formatted result in memory and return its offset
        const formattedBytes = new TextEncoder().encode(formatted + '\\0');
        const resultOffset = memory.buffer.byteLength - 1024;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      }".to_string());
        
        functions.insert("format_decimal".to_string(),
            "(scaledValue, patternOffset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get the pattern string
        let patternLength = 0;
        while (bytes[patternOffset + patternLength] !== 0 && patternOffset + patternLength < bytes.length) {
          patternLength++;
        }
        const patternBytes = new Uint8Array(memory.buffer, patternOffset, patternLength);
        const pattern = new TextDecoder(\"utf-8\").decode(patternBytes);
        
        // Convert scaled value to decimal
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        
        let formatted;
        if (pattern === \"0.00\") {
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        } else if (pattern === \"#,##0.00\") {
          formatted = `${integerPart.toLocaleString()}.${fractionalPart.toString().padStart(2, '0')}`;
        } else if (pattern === \"$0.00\") {
          formatted = `$${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        } else if (pattern === \"0.0000\") {
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}00`;
        } else if (pattern === \"percent\") {
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}%`;
        } else {
          // Default formatting
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        }
        
        // Store the formatted result in memory and return its offset
        const formattedBytes = new TextEncoder().encode(formatted + '\\0');
        const resultOffset = memory.buffer.byteLength - 2048;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      }".to_string());
        
        functions.insert("format_number".to_string(),
            "(value, patternOffset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get the pattern string
        let patternLength = 0;
        while (bytes[patternOffset + patternLength] !== 0 && patternOffset + patternLength < bytes.length) {
          patternLength++;
        }
        const patternBytes = new Uint8Array(memory.buffer, patternOffset, patternLength);
        const pattern = new TextDecoder(\"utf-8\").decode(patternBytes);
        
        let formatted;
        if (pattern === \"#,##0\") {
          formatted = value.toLocaleString();
        } else if (pattern === \"0000\") {
          formatted = value.toString().padStart(4, '0');
        } else if (pattern === \"hex\") {
          formatted = \"0x\" + value.toString(16).toUpperCase();
        } else if (pattern === \"oct\") {
          formatted = \"0o\" + value.toString(8);
        } else if (pattern === \"bin\") {
          formatted = \"0b\" + value.toString(2);
        } else {
          // Default formatting
          formatted = value.toString();
        }
        
        // Store the formatted result in memory and return its offset
        const formattedBytes = new TextEncoder().encode(formatted + '\\0');
        const resultOffset = memory.buffer.byteLength - 3072;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      }".to_string());
        
        functions
    }
    
    pub fn update_existing_runtime(&self, runtime_path: &str) -> Result<(), std::io::Error> {
        let runtime_js = self.generate_runtime_js();
        std::fs::write(runtime_path, runtime_js)?;
        Ok(())
    }
}

impl Default for WasmRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}