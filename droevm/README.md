# Droe Virtual Machine

A high-performance Rust-based virtual machine for executing Droe language bytecode.

## Features

- **Stack-based VM** with comprehensive instruction set
- **JSON bytecode format** for easy debugging and inspection
- **Standalone executable generation** with embedded bytecode
- **Zero-dependency deployment** - single binary distribution
- **Cross-platform support** with native performance

## Usage

### Running Bytecode Files

```bash
# Compile Droe source to bytecode
droe compile program.droe --target bytecode

# Run bytecode with the VM
droevm run program.droebc
```

### Creating Standalone Executables

```bash
# Create a standalone executable with embedded bytecode
droevm build program.droebc -o my-program

# Run the standalone executable (no dependencies needed!)
./my-program
```

### Building the VM

```bash
cargo build --release
```

## Architecture

The VM implements a stack-based execution model with support for:

- **Variables** and **data types** (strings, numbers, booleans, arrays)
- **Arithmetic** and **comparison** operations
- **Control flow** (if/else, loops, jumps)
- **Function calls** and **task execution**
- **I/O operations** (display, future HTTP/DB support)

## Bytecode Format

The bytecode uses JSON serialization for cross-platform compatibility:

```json
{
  "version": 1,
  "metadata": {
    "source_file": "program.droe",
    "created_at": 1754474723,
    "compiler_version": "0.1.0"
  },
  "constants": [],
  "instructions": [
    { "Push": { "String": "Hello, World!" } },
    "Display",
    "Halt"
  ],
  "debug_info": null
}
```

## Standalone Executable Format

Standalone executables embed bytecode using binary markers:

```
[VM Binary Data]
__DROEBC_DATA_START__
[8-byte length in little-endian]
[Bytecode JSON data]
__DROEBC_DATA_END__
```

The VM automatically detects and executes embedded bytecode at startup.

## Integration

The VM integrates seamlessly with the Droe compilation pipeline:

1. **Droe source** → Python compiler → **AST**
2. **AST** → Bytecode generator → **`.droebc` file**
3. **`.droebc`** → Rust VM → **Execution**
4. **`.droebc`** → Standalone builder → **Single executable**
