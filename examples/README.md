# Roelang Examples

This directory contains example Roelang (.roe) files demonstrating the language features.

## Files

### demo.roe
Shows all currently implemented features:
- String literals (double and single quotes)
- Show statements (natural language for display)
- Conditional statements (when-then for non-technical users)
- Natural language operators: "is greater than", "is less than", "equals", etc.
- Symbolic operators: >, <, >=, <=, ==, !=
- Boolean literals (true, false, "the condition is true")

### limitations.roe
Documents features that are partially implemented or planned:
- Property access (parsed but not code-generated)
- Variables and identifiers (parsed but not code-generated)
- Arithmetic operations
- Multi-line if statements
- Else clauses
- Loops
- Functions
- Variable assignments

### legacy_syntax.roe
Shows the original "Display" syntax that's still supported for backward compatibility.

## Running Examples

To compile and run these examples:

```bash
# Compile to WebAssembly Text format
python -m compiler.compiler examples/demo.roe examples/demo.wat

# Convert to WebAssembly binary
wat2wasm examples/demo.wat -o examples/demo.wasm

# Run with Node.js (requires run.js)
node run.js examples/demo.wasm
```

Or if you have Roelang installed via the installer:

```bash
# From a Roelang project directory
roe compile demo.roe
roe run demo.roe
```