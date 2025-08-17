# Droe - Unified Programming System

The core repository for Droe, a unified programming system that combines natural language programming, visual development, and multi-target compilation into a single cohesive platform.

## Overview

Droe is a revolutionary programming system that bridges the gap between natural language, visual development, and traditional code. It features:

- **Natural Language Programming**: Write code using English-like syntax
- **Visual Development**: Switch seamlessly between code and visual editors  
- **Multi-Target Compilation**: Deploy to web, mobile, desktop, and server platforms
- **Unified Toolchain**: One system for frontend, backend, and mobile development

## Features

### 🌐 Multi-Target Compilation
- **Web**: HTML/JavaScript with modern frameworks
- **Mobile**: Native Android (Kotlin) and iOS (Swift) apps
- **Server**: Java Spring Boot, Node.js, Python FastAPI, Rust Axum, Go Fiber
- **Desktop**: WebAssembly and native binaries

### 🎨 Visual Development
- **Puck Editor Integration**: Visual component-based development
- **Bidirectional Sync**: Switch between code and visual editing
- **Real-time Preview**: See changes instantly across all targets

### 🤖 AI-Powered Development
- **droe-scribe**: Intelligent code generation and assistance
- **Natural Language to Code**: Describe what you want, get working code
- **Smart Compilation**: Automatic optimization for target platforms

### 📱 Platform-Specific Features
- **Mobile**: Camera, GPS, sensors, notifications, native UI components
- **Web**: Responsive layouts, modern CSS, Progressive Web Apps
- **Backend**: Database operations, API endpoints, authentication

## Repository Structure

```
droe/
├── compiler/              # Multi-target compiler
│   ├── targets/          # Platform-specific code generators
│   │   ├── html/         # Web development
│   │   ├── mobile/       # Android/iOS
│   │   ├── java/         # Spring Boot
│   │   ├── python/       # FastAPI
│   │   ├── rust/         # Axum
│   │   ├── go/           # Fiber
│   │   └── puck/         # Visual editor integration
│   └── parser/           # Language parser and AST
├── droevm/               # Droe Virtual Machine (Rust)
├── droe                  # Command-line interface
├── examples/             # Sample projects and tutorials
├── tests/                # Comprehensive test suite
├── specs/                # Language specifications
└── models/               # AI models and configurations
```

## Quick Start

### Installation

Download the latest release or build from source:

```bash
# Download and extract
curl -L https://github.com/droe-lang/droe/releases/latest/download/compiler.tar.gz | tar -xz

# Or clone and build
git clone https://github.com/droe-lang/droe.git
cd droe
./build-droevm.sh
```

### Create Your First Project

```bash
# Initialize a new project
./droe init my-app

# Navigate to project
cd my-app

# Write your first Droe program
echo '@target html

layout MainApp
  column class "app-container"
    title "Hello Droe!" class "main-title"
    button "Click Me" action sayHello class "primary-btn"
  end column
end layout

action sayHello
  display "Hello from Droe!"
end action' > src/main.droe

# Compile to HTML
./droe compile src/main.droe

# The compiled web app is now in build/
```

### Multi-Platform Development

```bash
# Compile for different targets
./droe compile --target html src/main.droe      # Web app
./droe compile --target mobile src/main.droe    # Mobile app
./droe compile --target java src/main.droe      # Spring Boot API
./droe compile --target python src/main.droe    # FastAPI server
```

## Language Syntax

Droe uses natural, English-like syntax:

```droe
@target html
@name "Todo App"

module TodoApp

data Task
  id is text key auto
  title is text required
  completed is flag default false
  created_at is date auto
end data

layout TodoList
  column class "todo-container"
    title "My Tasks" class "header"
    
    input id new_task text placeholder "Add new task..." bind NewTask.title class "task-input"
    button "Add Task" action addTask class "add-btn"
    
    for each task in tasks
      row class "task-item"
        checkbox bind task.completed action toggleTask class "task-checkbox"
        text task.title class "task-text"
        button "Delete" action deleteTask with task.id class "delete-btn"
      end row
    end for
  end column
end layout

action addTask
  when NewTask.title is not empty then
    db create Task from NewTask
    set NewTask.title to ""
    refresh tasks
  end when
end action

end module
```

## Development Ecosystem

### Core Components
- **droe-scribe**: AI-powered development assistant and runtime
- **droe-studio**: Visual development environment with Puck editor
- **droe CLI**: Command-line compiler and project management
- **DroeVM**: High-performance virtual machine for bytecode execution

### IDE Integration
- **VS Code Extension**: Full language support with syntax highlighting, IntelliSense, and debugging
- **Claude Code Integration**: AI-powered code assistance and generation

## Building from Source

### Prerequisites
- Python 3.8+
- Rust 1.70+
- Node.js 18+
- Git

### Build Steps

```bash
# Clone the repository
git clone https://github.com/droe-lang/droe.git
cd droe

# Build the virtual machine
./build-droevm.sh

# Create compiler bundle
tar -czf compiler.tar.gz compiler/ droe run.js

# Run tests
./tests/run_tests.sh
```

## Documentation

- **Language Specification**: [specs/DROELANG_LANGUAGE_SPECIFICATION.md](specs/DROELANG_LANGUAGE_SPECIFICATION.md)
- **Examples**: [examples/](examples/)
- **API Documentation**: [droe-lang.github.io](https://droe-lang.github.io)

## Contributing

We welcome contributions! Please see our contributing guidelines and:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Roadmap

- [ ] **Enhanced AI Integration**: More sophisticated code generation
- [ ] **Performance Optimization**: Faster compilation and runtime
- [ ] **Extended Platform Support**: Desktop applications, IoT devices
- [ ] **Advanced Debugging**: Visual debugging tools and profilers
- [ ] **Package Manager**: Centralized module and library distribution

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Community

- **GitHub**: [droe-lang](https://github.com/droe-lang)
- **Documentation**: [droe-lang.github.io](https://droe-lang.github.io)
- **Issues**: [GitHub Issues](https://github.com/droe-lang/droe/issues)

---

**Droe**: Where natural language meets powerful programming. Build once, deploy everywhere.