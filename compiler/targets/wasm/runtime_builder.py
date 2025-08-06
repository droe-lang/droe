"""Runtime builder for WebAssembly target.

This module generates the JavaScript runtime functions needed
to execute compiled WebAssembly modules.
"""

from typing import Dict, List
from ...libs.core.string_utils import StringUtils
from ...libs.core.math_utils import MathUtils
from ...libs.core.formatting import FormattingUtils


class WASMRuntimeBuilder:
    """Builds JavaScript runtime for WebAssembly execution."""
    
    def __init__(self):
        self.string_utils = StringUtils()
        self.math_utils = MathUtils()
        self.formatting_utils = FormattingUtils()
        self.enabled_libs = set()
    
    def enable_library(self, lib_name: str):
        """Enable a core library in the runtime."""
        self.enabled_libs.add(lib_name)
    
    def disable_library(self, lib_name: str):
        """Disable a core library in the runtime."""
        self.enabled_libs.discard(lib_name)
    
    def generate_runtime_js(self, output_path: str = None) -> str:
        """Generate the complete JavaScript runtime."""
        runtime_code = []
        
        # Add header
        runtime_code.extend([
            'const fs = require("fs");',
            'const path = process.argv[2];',
            '',
            '(async () => {',
            '  const wasmBuffer = fs.readFileSync(path);',
            '',
            '  const importObject = {',
            '    env: {',
            '      // Basic runtime functions'
        ])
        
        # Add basic print functions
        runtime_code.extend([
            '      print: (offset, length) => {',
            '        const memory = instance.exports.memory;',
            '        const bytes = new Uint8Array(memory.buffer, offset, length);',
            '        const text = new TextDecoder("utf-8").decode(bytes);',
            '        console.log(text);',
            '      },',
            '      print_i32: (value) => {',
            '        console.log(value);',
            '      },',
            '      print_string_from_offset: (offset) => {',
            '        const memory = instance.exports.memory;',
            '        const bytes = new Uint8Array(memory.buffer);',
            '        let length = 0;',
            '        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {',
            '          length++;',
            '        }',
            '        const stringBytes = new Uint8Array(memory.buffer, offset, length);',
            '        const text = new TextDecoder("utf-8").decode(stringBytes);',
            '        console.log(text);',
            '      },'
        ])
        
        # Add core library functions
        if 'string_utils' in self.enabled_libs:
            runtime_code.extend(self._generate_string_utils_js())
        
        if 'math_utils' in self.enabled_libs:
            runtime_code.extend(self._generate_math_utils_js())
        
        if 'formatting' in self.enabled_libs:
            runtime_code.extend(self._generate_formatting_js())
        
        # Add footer
        runtime_code.extend([
            '    },',
            '  };',
            '',
            '  const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);',
            '',
            '  instance.exports.main();',
            '})();',
            ''
        ])
        
        runtime_js = '\n'.join(runtime_code)
        
        # Write to file if path provided
        if output_path:
            with open(output_path, 'w') as f:
                f.write(runtime_js)
        
        return runtime_js
    
    def _generate_string_utils_js(self) -> List[str]:
        """Generate JavaScript code for string utilities."""
        js_functions = self.string_utils.get_js_runtime_functions()
        lines = []
        
        for func_name, func_code in js_functions.items():
            lines.extend([
                f'      {func_name}: {func_code},'
            ])
        
        return lines
    
    def _generate_math_utils_js(self) -> List[str]:
        """Generate JavaScript code for math utilities."""
        js_functions = self.math_utils.get_js_runtime_functions()
        lines = []
        
        for func_name, func_code in js_functions.items():
            lines.extend([
                f'      {func_name}: {func_code},'
            ])
        
        return lines
    
    def _generate_formatting_js(self) -> List[str]:
        """Generate JavaScript code for formatting utilities."""
        js_functions = self.formatting_utils.get_js_runtime_functions()
        lines = []
        
        for func_name, func_code in js_functions.items():
            lines.extend([
                f'      {func_name}: {func_code},'
            ])
        
        return lines
    
    def update_existing_runtime(self, runtime_path: str) -> bool:
        """Update an existing run.js file with new core library functions."""
        try:
            # Generate new runtime
            new_runtime = self.generate_runtime_js()
            
            # Write to the runtime file
            with open(runtime_path, 'w') as f:
                f.write(new_runtime)
            
            return True
        except Exception as e:
            print(f"Error updating runtime: {e}")
            return False