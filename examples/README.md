# Roelang Examples

This directory contains example Roelang (.droe) files demonstrating all modern language features. Each example focuses on core functionality with the latest syntax.

## Example Files

### 01_display.droe

**Basic Display Statements**

- `display` statements with strings
- Double and single quote strings
- Displaying variables

### 02_variables.droe

**Modern Type System & String Features**

- Strong type system: `set name which is text to "Alice"`
- New type names: `int`, `decimal`, `flag`, `yesno`, `text`
- String interpolation: `"Hello [name]!"`
- String concatenation: `"Welcome " + name + "!"`
- Collections: `list of`, `group of`

### 03_conditionals.droe

**Conditional Statements**

- `when...then` statements
- Comparison operators: `is greater than`, `equals`, `is not equal to`
- Boolean conditions
- Variable comparisons

### 04_while_loops.droe

**While Loops**

- `while...end while` syntax
- Loop counters and conditions
- Countdown loops
- Accumulator patterns

### 05_arrays.droe

**Arrays and Type Safety**

- Number and string arrays
- Type-safe array declarations: `set items which are group of text to [...]`
- Array creation and type validation

### 06_for_each_loops.droe

**For Each Loops**

- `for each...end for` syntax
- `loop...end loop` alternative syntax
- Iterating over arrays
- Processing array elements

### 07_arithmetic.droe

**Arithmetic Operations**

- Basic operators: `plus`, `minus`, `times`, `divided by`
- Variable arithmetic
- Using arithmetic in conditions

### 08_complete_example.droe

**Complete Program**

- Demonstrates multiple features working together
- Student grade analysis program
- Shows real-world usage patterns

### 10_actions.droe

**Modern Actions with Parameters**

- Module system: `module name ... end module`
- Parameterized actions: `action greet with name which is text gives text`
- Action invocation: `run module.action with arguments`
- Data structures: `data User ... end data`

### 11_modern_features.droe

**Complete Modern Feature Demo**

- All new features in one comprehensive example
- Real-world usage patterns with modern syntax
- Best practices demonstration

### MathUtils.droe, StringUtils.droe, UserManager.droe

**Utility Modules**

- Reusable modules with mathematical and string operations
- Demonstrates module organization and code reuse
- Example actions with parameters and return values

### main_with_includes.droe

**Include System Demo**

- `include ModuleName.droe` syntax for importing modules
- Cross-module action calls: `run ModuleName.ActionName with params`
- Parameter passing between modules
- Complete Include functionality demonstration

## Language Features

### ✅ Fully Implemented Modern Features

- **Display**: `display "text"` and `display variable`
- **Strong Type System**: `int`, `decimal`, `text`, `flag`, `yesno`, `date`, `file`
- **String Interpolation**: `"Hello [name]!"` - Variables in square brackets
- **String Concatenation**: `"Welcome " + name + "!"` - Tech-friendly + operator
- **Collections**: `list of type`, `group of type` with type safety
- **Modules**: Organized code with `module name ... end module`
- **Include System**: `include ModuleName.droe` for importing modules
- **Cross-Module Calls**: `run ModuleName.ActionName with params`
- **Parameterized Actions**: Functions with typed parameters and return values
- **Data Structures**: `data Name ... end data` with typed fields
- **Type Compatibility**: Related types work together seamlessly

### 🔧 Enhanced Type System

- **Numeric Types**: `int` (integers), `decimal` (floating point), `number` (legacy)
- **Boolean Types**: `flag` (true/false), `yesno` (yes/no), `boolean` (legacy)
- **Text Types**: `text` (modern), `string` (legacy)
- **Collection Types**: `list of type`, `group of type`, `array` (legacy)
- **Special Types**: `date` (timestamps), `file` (file paths)
- **Type Checking**: Prevents type mismatches with helpful error messages
- **Type Compatibility**: Compatible types can be used interchangeably

## Running Examples

### Using the Roelang CLI (Recommended)

```bash
# Navigate to examples directory
cd examples

# Run any example directly
droe run src/01_display.droe
droe run src/02_variables.droe
droe run src/08_complete_example.droe

# Try the new Include functionality
droe run src/main_with_includes.droe

# Just compile (creates .wat and .wasm files)
droe compile src/01_display.droe
```

### Manual Compilation

```bash
# Using the compiler directly
python -m compiler.compiler examples/01_display.droe examples/01_display.wat

# Convert to WebAssembly binary
wat2wasm examples/01_display.wat -o examples/01_display.wasm

# Run with Node.js
node run.js examples/01_display.wasm
```

## Learning Path

1. **Start with**: `01_display.droe` - Learn basic output
2. **Then try**: `02_variables.droe` - Understand variables and type safety
3. **Continue with**: `03_conditionals.droe` - Learn decision making
4. **Move to**: `04_while_loops.droe` and `06_for_each_loops.droe` - Master loops
5. **Explore**: `05_arrays.droe` and `07_arithmetic.droe` - Data structures and math
6. **Advanced**: `10_actions.droe` and `11_modern_features.droe` - Modules and actions
7. **Module System**: `MathUtils.droe`, `StringUtils.droe` - Reusable modules
8. **Complete**: `main_with_includes.droe` - Include system and cross-module calls

Each example builds on concepts from previous ones, so following this order will give you a solid understanding of Roelang including the powerful new Include system!
