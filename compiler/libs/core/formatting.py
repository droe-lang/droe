"""Formatting utility functions for Roelang runtime."""

from typing import List, Dict, Any
from dataclasses import dataclass


@dataclass
class FormatFunction:
    """Represents a formatting utility function."""
    name: str
    wasm_import_name: str
    description: str
    parameters: List[str]
    return_type: str


class FormattingUtils:
    """Formatting utility functions and their WebAssembly implementations."""
    
    def __init__(self):
        self.functions = {
            'print_i32_no_newline': FormatFunction(
                name='print_i32_no_newline',
                wasm_import_name='print_i32_no_newline',
                description='Print integer without newline',
                parameters=['value: i32'],
                return_type='void'
            ),
            'print_decimal_no_newline': FormatFunction(
                name='print_decimal_no_newline',
                wasm_import_name='print_decimal_no_newline',
                description='Print scaled decimal without newline',
                parameters=['scaledValue: i32'],
                return_type='void'
            ),
            'print_decimal': FormatFunction(
                name='print_decimal',
                wasm_import_name='print_decimal',
                description='Print scaled decimal with newline',
                parameters=['scaledValue: i32'],
                return_type='void'
            ),
            'print_date': FormatFunction(
                name='print_date',
                wasm_import_name='print_date',
                description='Print date from string offset',
                parameters=['offset: i32'],
                return_type='void'
            ),
            'print_date_no_newline': FormatFunction(
                name='print_date_no_newline',
                wasm_import_name='print_date_no_newline',
                description='Print date without newline',
                parameters=['offset: i32'],
                return_type='void'
            ),
            'format_date': FormatFunction(
                name='format_date',
                wasm_import_name='format_date',
                description='Format date according to pattern',
                parameters=['dateOffset: i32', 'patternOffset: i32'],
                return_type='i32'  # Returns formatted string offset
            ),
            'format_decimal': FormatFunction(
                name='format_decimal',
                wasm_import_name='format_decimal',
                description='Format decimal according to pattern',
                parameters=['scaledValue: i32', 'patternOffset: i32'],
                return_type='i32'  # Returns formatted string offset
            ),
            'format_number': FormatFunction(
                name='format_number',
                wasm_import_name='format_number',
                description='Format number according to pattern',
                parameters=['value: i32', 'patternOffset: i32'],
                return_type='i32'  # Returns formatted string offset
            )
        }
        
        # Standard format patterns
        self.date_patterns = {
            'MM/dd/yyyy': 'US date format (12/31/2024)',
            'dd/MM/yyyy': 'European date format (31/12/2024)',
            'MMM dd, yyyy': 'Long date format (Dec 31, 2024)',
            'long': 'Full date format (Tuesday, December 31, 2024)',
            'short': 'Short date format (12/31/24)',
            'iso': 'ISO date format (2024-12-31)'
        }
        
        self.decimal_patterns = {
            '0.00': 'Basic decimal format (123.45)',
            '#,##0.00': 'Decimal with thousands separator (1,234.56)',
            '$0.00': 'Currency format ($123.45)',
            '0.0000': 'High precision decimal (123.4500)',
            'percent': 'Percentage format (12.34%)'
        }
        
        self.number_patterns = {
            '#,##0': 'Number with thousands separator (1,234)',
            '0000': 'Zero-padded number (0123)',
            'hex': 'Hexadecimal format (0x7B)',
            'oct': 'Octal format (0o173)',
            'bin': 'Binary format (0b1111011)'
        }
    
    def get_wasm_imports(self) -> List[str]:
        """Generate WebAssembly import declarations for formatting utilities."""
        imports = []
        for func in self.functions.values():
            if func.parameters:
                param_list = ' '.join([f'(param {p.split(":")[1].strip()})' for p in func.parameters])
            else:
                param_list = ''
            
            if func.return_type != 'void':
                return_decl = f' (result {func.return_type})'
            else:
                return_decl = ''
            
            import_decl = f'(import "env" "{func.wasm_import_name}" (func ${func.name} {param_list}{return_decl}))'
            imports.append(import_decl)
        
        return imports
    
    def get_js_runtime_functions(self) -> Dict[str, str]:
        """Generate JavaScript runtime functions for Node.js runtime."""
        js_functions = {}
        
        # Basic print functions
        js_functions['print_i32_no_newline'] = '''(value) => {
        process.stdout.write(value.toString());
      }'''
        
        js_functions['print_decimal_no_newline'] = '''(scaledValue) => {
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        const formattedDecimal = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        process.stdout.write(formattedDecimal);
      }'''
        
        js_functions['print_decimal'] = '''(scaledValue) => {
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        const formattedDecimal = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        console.log(formattedDecimal);
      }'''
        
        js_functions['print_date'] = '''(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const dateString = new TextDecoder("utf-8").decode(stringBytes);
        console.log(dateString);
      }'''
        
        js_functions['print_date_no_newline'] = '''(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const dateString = new TextDecoder("utf-8").decode(stringBytes);
        process.stdout.write(dateString);
      }'''
        
        # Advanced formatting functions
        js_functions['format_date'] = '''(dateOffset, patternOffset) => {
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
        const formattedBytes = new TextEncoder().encode(formatted + '\\0');
        const resultOffset = memory.buffer.byteLength - 1024;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      }'''
        
        js_functions['format_decimal'] = '''(scaledValue, patternOffset) => {
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
        const formattedBytes = new TextEncoder().encode(formatted + '\\0');
        const resultOffset = memory.buffer.byteLength - 2048;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      }'''
        
        js_functions['format_number'] = '''(value, patternOffset) => {
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
        const formattedBytes = new TextEncoder().encode(formatted + '\\0');
        const resultOffset = memory.buffer.byteLength - 3072;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      }'''
        
        return js_functions