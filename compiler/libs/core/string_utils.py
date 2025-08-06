"""String utility functions for Roelang runtime."""

from typing import List, Dict, Any
from dataclasses import dataclass


@dataclass
class StringFunction:
    """Represents a string utility function."""
    name: str
    wasm_import_name: str
    description: str
    parameters: List[str]
    return_type: str


class StringUtils:
    """String utility functions and their WebAssembly implementations."""
    
    def __init__(self):
        self.functions = {
            'print_no_newline': StringFunction(
                name='print_no_newline',
                wasm_import_name='print_no_newline',
                description='Print text without adding newline',
                parameters=['offset: i32', 'length: i32'],
                return_type='void'
            ),
            'print_string_from_offset_no_newline': StringFunction(
                name='print_string_from_offset_no_newline', 
                wasm_import_name='print_string_from_offset_no_newline',
                description='Print null-terminated string without newline',
                parameters=['offset: i32'],
                return_type='void'
            ),
            'print_newline': StringFunction(
                name='print_newline',
                wasm_import_name='print_newline',
                description='Print just a newline character',
                parameters=[],
                return_type='void'
            ),
            'string_concat': StringFunction(
                name='string_concat',
                wasm_import_name='string_concat',
                description='Concatenate two strings',
                parameters=['str1_offset: i32', 'str2_offset: i32'],
                return_type='i32'  # Returns offset to concatenated string
            ),
            'string_length': StringFunction(
                name='string_length',
                wasm_import_name='string_length', 
                description='Get length of null-terminated string',
                parameters=['offset: i32'],
                return_type='i32'
            ),
            'string_substring': StringFunction(
                name='string_substring',
                wasm_import_name='string_substring',
                description='Extract substring from string',
                parameters=['offset: i32', 'start: i32', 'length: i32'],
                return_type='i32'  # Returns offset to substring
            )
        }
    
    def get_wasm_imports(self) -> List[str]:
        """Generate WebAssembly import declarations for string utilities."""
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
        js_functions['print_no_newline'] = '''(offset, length) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder("utf-8").decode(bytes);
        process.stdout.write(text);
      }'''
        
        js_functions['print_string_from_offset_no_newline'] = '''(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder("utf-8").decode(stringBytes);
        process.stdout.write(text);
      }'''
        
        js_functions['print_newline'] = '''() => {
        console.log();
      }'''
        
        # Advanced string functions
        js_functions['string_concat'] = '''(str1Offset, str2Offset) => {
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
        const resultBytes = new TextEncoder().encode(result + '\\0');
        const resultOffset = memory.buffer.byteLength - 4096; // Use end of memory
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(resultBytes, resultOffset);
        return resultOffset;
      }'''
        
        js_functions['string_length'] = '''(offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        return length;
      }'''
        
        js_functions['string_substring'] = '''(offset, start, length) => {
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
        const resultBytes = new TextEncoder().encode(result + '\\0');
        const resultOffset = memory.buffer.byteLength - 5120; // Use end of memory
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(resultBytes, resultOffset);
        return resultOffset;
      }'''
        
        return js_functions