"""Math utility functions for Roelang runtime."""

from typing import List, Dict, Any
from dataclasses import dataclass


@dataclass
class MathFunction:
    """Represents a math utility function."""
    name: str
    wasm_import_name: str
    description: str
    parameters: List[str]
    return_type: str


class MathUtils:
    """Math utility functions and their WebAssembly implementations."""
    
    def __init__(self):
        self.functions = {
            'math_abs_i32': MathFunction(
                name='math_abs_i32',
                wasm_import_name='math_abs_i32',
                description='Absolute value of 32-bit integer',
                parameters=['value: i32'],
                return_type='i32'
            ),
            'math_abs_decimal': MathFunction(
                name='math_abs_decimal',
                wasm_import_name='math_abs_decimal',
                description='Absolute value of scaled decimal',
                parameters=['value: i32'],
                return_type='i32'
            ),
            'math_min_i32': MathFunction(
                name='math_min_i32',
                wasm_import_name='math_min_i32',
                description='Minimum of two 32-bit integers',
                parameters=['a: i32', 'b: i32'],
                return_type='i32'
            ),
            'math_max_i32': MathFunction(
                name='math_max_i32',
                wasm_import_name='math_max_i32',
                description='Maximum of two 32-bit integers',
                parameters=['a: i32', 'b: i32'],
                return_type='i32'
            ),
            'math_power_i32': MathFunction(
                name='math_power_i32',
                wasm_import_name='math_power_i32',
                description='Power function for integers (a^b)',
                parameters=['base: i32', 'exponent: i32'],
                return_type='i32'
            ),
            'math_sqrt_decimal': MathFunction(
                name='math_sqrt_decimal',
                wasm_import_name='math_sqrt_decimal',
                description='Square root of scaled decimal',
                parameters=['value: i32'],
                return_type='i32'
            ),
            'math_decimal_multiply': MathFunction(
                name='math_decimal_multiply',
                wasm_import_name='math_decimal_multiply',
                description='Multiply two scaled decimals',
                parameters=['a: i32', 'b: i32'],
                return_type='i32'
            ),
            'math_decimal_divide': MathFunction(
                name='math_decimal_divide',
                wasm_import_name='math_decimal_divide',
                description='Divide two scaled decimals',
                parameters=['a: i32', 'b: i32'],
                return_type='i32'
            )
        }
    
    def get_wasm_imports(self) -> List[str]:
        """Generate WebAssembly import declarations for math utilities."""
        imports = []
        for func in self.functions.values():
            if func.parameters:
                param_list = ' '.join([f'(param {p.split(":")[1].strip()})' for p in func.parameters])
            else:
                param_list = ''
            
            return_decl = f' (result {func.return_type})'
            
            import_decl = f'(import "env" "{func.wasm_import_name}" (func ${func.name} {param_list}{return_decl}))'
            imports.append(import_decl)
        
        return imports
    
    def get_js_runtime_functions(self) -> Dict[str, str]:
        """Generate JavaScript runtime functions for Node.js runtime."""
        js_functions = {}
        
        # Basic math functions
        js_functions['math_abs_i32'] = '''(value) => {
        return Math.abs(value);
      }'''
        
        js_functions['math_abs_decimal'] = '''(scaledValue) => {
        return Math.abs(scaledValue);
      }'''
        
        js_functions['math_min_i32'] = '''(a, b) => {
        return Math.min(a, b);
      }'''
        
        js_functions['math_max_i32'] = '''(a, b) => {
        return Math.max(a, b);
      }'''
        
        js_functions['math_power_i32'] = '''(base, exponent) => {
        if (exponent < 0) return 0; // Integer power doesn't support negative exponents
        let result = 1;
        for (let i = 0; i < exponent; i++) {
          result *= base;
        }
        return result;
      }'''
        
        # Decimal math (scaled by 100)
        js_functions['math_sqrt_decimal'] = '''(scaledValue) => {
        const realValue = scaledValue / 100.0;
        const result = Math.sqrt(realValue);
        return Math.round(result * 100); // Scale back to integer
      }'''
        
        js_functions['math_decimal_multiply'] = '''(a, b) => {
        // Both are scaled by 100, so result is scaled by 10000
        // Divide by 100 to get back to scale of 100
        return Math.round((a * b) / 100);
      }'''
        
        js_functions['math_decimal_divide'] = '''(a, b) => {
        if (b === 0) return 0; // Avoid division by zero
        // a is scaled by 100, b is scaled by 100
        // To maintain scaling, multiply a by 100 before division
        return Math.round((a * 100) / b);
      }'''
        
        return js_functions
    
    def get_builtin_constants(self) -> Dict[str, int]:
        """Get built-in math constants (scaled for decimal representation)."""
        return {
            'MATH_PI': 314,      # π * 100 = 3.14 * 100
            'MATH_E': 272,       # e * 100 = 2.72 * 100  
            'MATH_SQRT2': 141,   # √2 * 100 = 1.41 * 100
            'MATH_SQRT3': 173,   # √3 * 100 = 1.73 * 100
        }