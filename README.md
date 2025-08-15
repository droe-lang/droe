# 🦌 Roelang

**Roelang** is a lightweight DSL that compiles human-readable commands like:

```droe
Display Hello World
```

...into WebAssembly (`.wasm`) for fast, portable execution across platforms.

- 🔤 Simple syntax for scripting and automation
- ⚡️ Compiles to `.wasm` via intermediate `.wat`
- 🧩 Easily extensible grammar
- 🖥️ CLI and GUI installer available
- 🛠️ Built in Python + Node

---

## 🔧 Installation

### macOS (via DMG Installer)

1. [Download the latest `.dmg`](https://droe-lang.dev)
2. Open and **double-click Roelang Installer**
3. Follow instructions — this sets up:

   - `~/.droelang/` with runtime files
   - CLI tool `droe` added to your terminal

Once installed, run:

```bash
droe run main.droe
```

---

## 🗂️ Project Structure

### Roelang Project Structure

```
my-project/
├── src/
│   └── main.droe       # Your Roelang source file
├── build/             # Generated .wat and .wasm files
└── droeconfig.json     # Project configuration
```

### Repository Structure

```
roelang-installer/
├── compiler/          # Core compiler components
│   ├── ast.py        # Abstract Syntax Tree definitions
│   ├── parser.py     # DSL parser with comment support
│   ├── codegen_wat.py # WebAssembly Text generation
│   └── symbols.py    # Symbol table and type system
├── examples/          # Example programs and documentation
│   ├── src/          # Example .droe files (01-11)
│   └── README.md     # Learning guide and feature documentation
├── tests/            # Comprehensive test suite
│   ├── unit/         # Unit tests
│   ├── integration/  # Integration tests
│   ├── type_system/  # Type system validation
│   └── README.md     # Test documentation
├── assets/           # Icons and DMG assets
├── droe               # Command-line interface
└── run.js           # WebAssembly runtime
```

---

## 🚀 Example

**main.droe**

```droe
Display Hello World
```

Then run:

```bash
droe run main.droe
```

Expected output:

```
Hello World
```

---

## 🤝 Contributing

We welcome contributions!

- File an issue
- Fork and PR to `main`
- Follow the language design spec (coming soon)

---

## 📄 License

Licensed under the [Apache License 2.0](LICENSE).

---

## 🌐 Website

👉 [droe-lang.dev](https://droe-lang.dev)
