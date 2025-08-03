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

```
my-project/
├── src/
│   └── main.roe       # Your Roelang source file
├── build/             # Generated .wat and .wasm files
└── roeconfig.json     # (coming soon) project configuration
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
