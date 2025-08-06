# Roelang Compiler Architecture

This document describes the modular architecture of the Roelang compiler after the major refactoring to support multiple compilation targets and core libraries.

## Directory Structure

```
compiler/
├── __init__.py                 # Main compiler module
├── compiler.py                 # Primary compilation API
├── ast.py                      # Abstract Syntax Tree definitions
├── parser.py                   # Roelang DSL parser
├── symbols.py                  # Symbol table and type system
├── module_resolver.py          # Module include resolution
├── codegen_base.py             # Base code generator class
├── codegen_wat.py              # Legacy WebAssembly codegen (deprecated)
├── libs/                       # Core runtime libraries
│   ├── __init__.py
│   └── core/
│       ├── __init__.py
│       ├── string_utils.py     # String utility functions
│       ├── math_utils.py       # Math utility functions  
│       └── formatting.py      # Formatting utility functions
└── targets/                    # Target-specific code generators
    ├── __init__.py
    └── wasm/                   # WebAssembly target
        ├── __init__.py
        ├── codegen.py          # WASM code generation
        └── runtime_builder.py  # JavaScript runtime generation
```

## Architecture Overview

### 1. Base Code Generator (`codegen_base.py`)

The `BaseCodeGenerator` abstract base class provides:
- Common functionality for all code generators
- Type system utilities and type checking
- Symbol table management
- Core library integration
- Abstract methods that must be implemented by target-specific generators

```python
class BaseCodeGenerator(ABC):
    @abstractmethod
    def generate(self, program: Program) -> str:
        """Generate code for the given AST program."""
        pass
    
    @abstractmethod 
    def emit_expression(self, expr: ASTNode):
        """Emit code for an expression."""
        pass
    
    @abstractmethod
    def emit_statement(self, stmt: ASTNode):
        """Emit code for a statement."""
        pass
```

### 2. Core Libraries (`libs/core/`)

Core libraries provide built-in functionality that is available to all Roelang programs:

#### String Utilities (`string_utils.py`)
- String concatenation, length, substring operations
- Print functions with/without newlines
- WebAssembly import declarations
- JavaScript runtime implementations

#### Math Utilities (`math_utils.py`) 
- Absolute value, min/max, power functions
- Decimal math operations (scaled arithmetic)
- Square root, multiplication, division for decimals
- Math constants (π, e, √2, etc.)

#### Formatting Utilities (`formatting.py`)
- Date formatting (MM/dd/yyyy, long, short, ISO, etc.)
- Decimal formatting (#,##0.00, $0.00, percent, etc.)
- Number formatting (hex, octal, binary, zero-padded)
- Pattern-based formatting system

### 3. Target-Specific Generators (`targets/`)

#### WebAssembly Target (`targets/wasm/`)

**Code Generator (`codegen.py`)**:
- Extends `BaseCodeGenerator`
- Generates WebAssembly Text (WAT) format
- Integrates core libraries via WASM imports
- Handles WASM-specific features (memory, locals, etc.)

**Runtime Builder (`runtime_builder.py`)**:
- Generates JavaScript runtime for Node.js execution
- Creates import object with core library functions
- Enables/disables libraries based on compiler configuration
- Updates existing runtime files

### 4. Compilation Flow

1. **Parse**: Source code → AST (`parser.py`)
2. **Resolve**: Include statements → Expanded AST (`module_resolver.py`)
3. **Generate**: AST → Target code (e.g., `WATCodeGenerator`)
4. **Runtime**: Core libraries → Runtime functions (`WASMRuntimeBuilder`)

## Usage Examples

### Basic Compilation

```python
from compiler.targets.wasm.codegen import WATCodeGenerator
from compiler.parser import parse

# Parse source to AST
ast = parse(source_code)

# Generate WebAssembly
codegen = WATCodeGenerator()
wat_code = codegen.generate(ast)
```

### Core Library Management

```python
# Enable specific libraries
codegen = WATCodeGenerator()
codegen.enable_core_lib('string_utils')
codegen.enable_core_lib('formatting')
codegen.disable_core_lib('math_utils')

# Check if library is enabled
if codegen.is_core_lib_enabled('string_utils'):
    # Library is available
    pass
```

### Runtime Generation

```python
from compiler.targets.wasm.runtime_builder import WASMRuntimeBuilder

# Create runtime builder
builder = WASMRuntimeBuilder()
builder.enable_library('string_utils')
builder.enable_library('formatting')

# Generate runtime file
runtime_js = builder.generate_runtime_js('path/to/run.js')
```

## Extension Points

### Adding New Targets

1. Create directory under `targets/` (e.g., `targets/llvm/`)
2. Implement code generator extending `BaseCodeGenerator`
3. Create target-specific runtime builder if needed
4. Update main compiler to support new target

### Adding Core Libraries

1. Create new library in `libs/core/` (e.g., `network_utils.py`)
2. Define function metadata and WASM imports
3. Implement JavaScript runtime functions
4. Enable library in target code generators

### Custom Code Generators

```python
from compiler.codegen_base import BaseCodeGenerator

class MyCodeGenerator(BaseCodeGenerator):
    def generate(self, program: Program) -> str:
        # Custom implementation
        pass
    
    def emit_expression(self, expr: ASTNode):
        # Custom expression handling
        pass
    
    def emit_statement(self, stmt: ASTNode):
        # Custom statement handling
        pass
```

## Migration from Legacy

The old `codegen_wat.py` is deprecated but maintained for backwards compatibility:

```python
# Old (deprecated)
from compiler.codegen_wat import generate_wat
wat = generate_wat(ast)  # Shows deprecation warning

# New (recommended)
from compiler.targets.wasm.codegen import WATCodeGenerator
codegen = WATCodeGenerator()
wat = codegen.generate(ast)
```

## Core Library Functions

### String Functions
- `print_no_newline`, `print_string_from_offset_no_newline`, `print_newline`
- `string_concat`, `string_length`, `string_substring`

### Math Functions
- `math_abs_i32`, `math_abs_decimal`, `math_min_i32`, `math_max_i32`
- `math_power_i32`, `math_sqrt_decimal`, `math_decimal_multiply`, `math_decimal_divide`

### Formatting Functions
- `print_i32_no_newline`, `print_decimal_no_newline`, `print_date_no_newline`
- `format_date`, `format_decimal`, `format_number`

This modular architecture provides:
- **Extensibility**: Easy to add new compilation targets
- **Maintainability**: Separate concerns and modular design
- **Reusability**: Core libraries shared across all targets
- **Testability**: Individual components can be tested in isolation
- **Backwards Compatibility**: Legacy APIs still work with deprecation warnings