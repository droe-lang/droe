# Roelang Examples

This directory contains example Roelang (.roe) files demonstrating all language features. Each example focuses on one core functionality.

## Example Files

### 01_display.roe
**Basic Display Statements**
- `show` statements with strings
- Double and single quote strings
- Displaying variables

### 02_variables.roe  
**Variables and Type Safety**
- Basic variable assignment: `set name to value`
- Type-safe declarations: `set name which is text to "Alice"`
- Variable reassignment with type checking
- Arithmetic with variables

### 03_conditionals.roe
**Conditional Statements**
- `when...then` statements
- Comparison operators: `is greater than`, `equals`, `is not equal to`
- Boolean conditions
- Variable comparisons

### 04_while_loops.roe
**While Loops**
- `while...end while` syntax
- Loop counters and conditions
- Countdown loops
- Accumulator patterns

### 05_arrays.roe
**Arrays and Type Safety**
- Number and string arrays
- Type-safe array declarations: `set items which are group of text to [...]`
- Array creation and type validation

### 06_for_each_loops.roe
**For Each Loops**
- `for each...end for` syntax
- `loop...end loop` alternative syntax
- Iterating over arrays
- Processing array elements

### 07_arithmetic.roe
**Arithmetic Operations**
- Basic operators: `plus`, `minus`, `times`, `divided by`
- Variable arithmetic
- Using arithmetic in conditions

### 08_complete_example.roe
**Complete Program**
- Demonstrates multiple features working together
- Student grade analysis program
- Shows real-world usage patterns

## Language Features

### ✅ Fully Implemented
- **Display**: `show "text"` and `show variable`
- **Variables**: Type-safe variables with explicit declarations
- **Arrays**: Homogeneous arrays with type checking
- **Conditionals**: Natural language comparisons
- **While Loops**: Full loop control with proper exit
- **For Each Loops**: Array iteration with type safety
- **Arithmetic**: Natural language math operations
- **Type Safety**: Compile-time type checking prevents runtime errors

### 🔧 Type System
- **Explicit Types**: `set name which is text to "value"`
- **Array Types**: `set items which are group of numbers to [1, 2, 3]`
- **Type Inference**: Types inferred from first assignment
- **Type Checking**: Prevents type mismatches at compile time
- **Error Messages**: Clear, helpful error descriptions

## Running Examples

### Using the Roelang CLI (Recommended)

```bash
# Navigate to examples directory
cd examples

# Run any example directly
roe run 01_display.roe
roe run 02_variables.roe
roe run 08_complete_example.roe

# Just compile (creates .wat and .wasm files)
roe compile 01_display.roe
```

### Manual Compilation

```bash
# Using the compiler directly
python -m compiler.compiler examples/01_display.roe examples/01_display.wat

# Convert to WebAssembly binary
wat2wasm examples/01_display.wat -o examples/01_display.wasm

# Run with Node.js
node run.js examples/01_display.wasm
```

## Learning Path

1. **Start with**: `01_display.roe` - Learn basic output
2. **Then try**: `02_variables.roe` - Understand variables and type safety
3. **Continue with**: `03_conditionals.roe` - Learn decision making
4. **Move to**: `04_while_loops.roe` and `06_for_each_loops.roe` - Master loops
5. **Explore**: `05_arrays.roe` and `07_arithmetic.roe` - Data structures and math
6. **Complete**: `08_complete_example.roe` - See it all working together

Each example builds on concepts from previous ones, so following this order will give you a solid understanding of Roelang!