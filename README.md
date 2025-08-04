# 🦌 Roelang

**Roelang** is a lightweight DSL that compiles human-readable commands like:

```roe
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

1. [Download the latest `.dmg`](https://roe-lang.dev)
2. Open and **double-click Roelang Installer**
3. Follow instructions — this sets up:

   - `~/.roelang/` with runtime files
   - CLI tool `roe` added to your terminal

Once installed, run:

```bash
roe run main.roe
```

---

## 🗂️ Project Structure

### Roelang Project Structure
```
my-project/
├── src/
│   └── main.roe       # Your Roelang source file
├── build/             # Generated .wat and .wasm files
└── roeconfig.json     # Project configuration
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
│   ├── src/          # Example .roe files (01-11)
│   └── README.md     # Learning guide and feature documentation
├── tests/            # Comprehensive test suite
│   ├── unit/         # Unit tests
│   ├── integration/  # Integration tests
│   ├── type_system/  # Type system validation
│   └── README.md     # Test documentation
├── assets/           # Icons and DMG assets
├── roe               # Command-line interface
└── run.js           # WebAssembly runtime
```

---

## 🚀 Example

**main.roe**

```roe
Display Hello World
```

Then run:

```bash
roe run main.roe
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

👉 [roe-lang.dev](https://roe-lang.dev)
