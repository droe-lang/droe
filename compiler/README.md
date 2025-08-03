# Roe DSL Compiler

This is the AST-based compiler for the Roe Domain-Specific Language.

## Architecture

The compiler follows a clean pipeline architecture:

```
DSL Source → Parser → AST → Code Generator → WAT Output
```

### Components

1. **ast.py** - AST node definitions using Python dataclasses
   - `Literal`: String, number, and boolean values
   - `Identifier`: Variable names
   - `BinaryOp`: Binary operations (>, <, ==, etc.)
   - `DisplayStatement`: Display output
   - `IfStatement`: Conditional execution
   - `PropertyAccess`: Object property access (e.g., user.age)
   - `Program`: Root node containing statements

2. **parser.py** - Recursive-descent parser
   - Converts Roe DSL text to AST
   - Supports basic statements and expressions
   - Handles string/number literals and binary operations

3. **codegen_wat.py** - WebAssembly Text generator
   - Walks the AST and generates WAT code
   - Manages string constants in memory
   - Imports `display` function from host environment

4. **compiler.py** - Main entry point
   - `compile(source)`: Compile DSL string to WAT
   - `compile_file(input, output)`: File-based compilation
   - CLI interface for command-line usage

## Usage

### As a Module

```python
from compiler import compile

# Compile DSL to WAT
source = 'display "Hello, World!"'
wat = compile(source)
print(wat)
```

### Command Line

```bash
python -m compiler.compiler input.roe output.wat
```

## Supported Syntax

Currently supported:
- `display "string"` - Display a string
- `display 'string'` - Single quotes also work
- `if condition then statement` - Simple conditionals
- Number comparisons: `>`, `<`, `>=`, `<=`, `==`, `!=`

## TODO

- [ ] Variable declarations and assignments
- [ ] Property access implementation (requires symbol table)
- [ ] Multi-line if statements and else clauses
- [ ] Function definitions
- [ ] Loops (for, while)
- [ ] Arrays and objects
- [ ] More operators (+, -, *, /, &&, ||)
- [ ] Type checking
- [ ] Better error messages with line numbers