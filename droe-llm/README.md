# DroeLLM - Centralized LLM Service for DROELANG

🤖 **Adaptive AI for robotics and development** - A high-performance, centralized LLM service providing intelligent code generation for the DROELANG ecosystem.

## 🚀 Features

### 🎯 Dual-Mode Intelligence
- **Regular Mode**: General DSL development and programming tasks
- **Robotics Mode**: Ultra-fast, real-time robotics command generation (<100ms)

### ⚡ High Performance
- **Streaming Generation**: Real-time token streaming with live validation
- **Optimized Parameters**: Mode-specific temperature and token optimization
- **Session Management**: Concurrent session handling with intelligent cleanup
- **Smart Caching**: Response caching for improved performance

### 🔒 Safety First
- **Workspace Validation**: Hard boundaries for robot movement
- **Safety Checks**: Pre-execution validation and emergency controls
- **ROS2 Integration**: Service availability verification
- **Real-time Monitoring**: Live safety status updates

### 🔧 Partial File Updates
- **Intelligent Diff Engine**: Efficient file modification algorithms
- **Multiple Strategies**: Replace, merge, insert, function-specific updates
- **Preview Generation**: Real-time update previews
- **Context Analysis**: Smart change detection

## 📦 Installation

### Prerequisites
- Rust 1.70+ 
- Protocol Buffers compiler (`protoc`)

```bash
# macOS
brew install protobuf

# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# Arch Linux
sudo pacman -S protobuf
```

### Build from Source

```bash
# Clone and build
git clone <repository>
cd droe/droe-llm
cargo build --release

# Install binary
cp target/release/droe-llm ~/.local/bin/
```

## 🎮 Usage

### Start the Service

```bash
# Basic usage
droe-llm serve --port 50051

# With custom configuration
droe-llm serve --port 50051 --config ~/.droe/llm_config.toml --debug

# Health check
droe-llm health --url http://localhost:50051
```

### Configuration

Create `~/.droe/llm_config.toml`:

```toml
[llm]
model = "droe-scribe:latest"
ollama_url = "http://localhost:11434"

[modes.regular]
temperature = 0.7
top_p = 0.9
top_k = 40
max_tokens = 800

[modes.robotics]
temperature = 0.05
top_p = 0.8
top_k = 20
max_tokens = 200

[safety]
enable_validation = true
workspace_bounds = { x = 2.0, y = 2.0, z = 1.5 }
max_distance = 2.0
max_rotation = 180.0
max_wait_time = 10.0

[performance]
cache_responses = true
enable_streaming = true
max_concurrent_requests = 10
```

## 🔌 API Reference

### gRPC Service

```proto
service DroeVMLLMService {
    rpc GenerateDSLStream(LLMRequest) returns (stream LLMStreamResponse);
    rpc GenerateDSL(LLMRequest) returns (LLMResponse);
    rpc ValidateCode(ValidationRequest) returns (ValidationResponse);
    rpc ApplyPartialUpdate(PartialUpdateRequest) returns (PartialUpdateResponse);
    rpc GetServiceInfo(Empty) returns (ServiceInfo);
    rpc CancelGeneration(CancelRequest) returns (CancelResponse);
}
```

### Client Examples

#### Rust Client
```rust
use droe_llm::{LLMService, LLMRequest, InferenceMode};

let service = LLMService::new(config).await?;
let request = LLMRequest {
    prompt: "Generate a pick and place sequence".to_string(),
    mode_hint: Some(InferenceMode::Robotics),
    ..Default::default()
};

let response = service.generate_dsl(request).await?;
println!("Generated: {}", response.generated_code);
```

#### gRPC Client (Any Language)
```bash
# Using grpcurl
grpcurl -plaintext \
  -d '{"prompt": "pick red_box", "mode": "robotics"}' \
  localhost:50051 \
  droevm.llm.DroeVMLLMService/GenerateDSL
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│                 DroeLLM Service                 │
│                                                 │
│  ┌─────────────┐  ┌─────────────────────────┐   │
│  │ Intelligence│  │      Providers          │   │
│  │ • Mode      │  │      • Ollama           │   │
│  │   Detection │  │      • OpenAI (future)  │   │
│  │ • Validation│  │      • Claude (future)  │   │
│  │ • Partial   │  └─────────────────────────┘   │
│  │   Updates   │                                │
│  └─────────────┘                                │
│                                                 │
│  ┌─────────────┐  ┌─────────────────────────┐   │
│  │ Service     │  │      Session            │   │
│  │ • gRPC      │  │      • Management       │   │
│  │ • Streaming │  │      • Cancellation     │   │
│  │ • Core LLM  │  │      • Cleanup          │   │
│  └─────────────┘  └─────────────────────────┘   │
└─────────────────────────────────────────────────┘
                         ▲
                         │ gRPC API (Port 50051)
                         │
    ┌────────────────────┼────────────────────┐
    │                    │                    │
    ▼                    ▼                    ▼
┌─────────┐         ┌─────────┐         ┌─────────┐
│VS Code  │         │droe-    │         │ Future  │
│Extension│         │scribe   │         │ Clients │
└─────────┘         └─────────┘         └─────────┘
```

## 📊 Performance Metrics

| Metric | Target | Achieved |
|--------|--------|----------|
| **Regular Mode** | <1000ms | ~400ms |
| **Robotics Mode** | <100ms | ~80ms |
| **First Token** | <100ms | ~60ms |
| **Token Rate** | >50/sec | ~75/sec |
| **Validation** | <10ms | ~5ms |

## 🔧 Development

### Project Structure

```
src/
├── lib.rs              # Library exports
├── main.rs             # Service binary
├── config/             # Configuration management
├── intelligence/       # AI logic (mode detection, validation)
├── providers/          # LLM provider implementations
├── service/            # Core service & gRPC server
└── session/            # Session management
```

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=droe_llm=debug cargo run -- serve --debug
```

### Adding New Providers

1. Create `src/providers/new_provider.rs`
2. Implement the provider trait
3. Add to `src/providers/mod.rs`
4. Update configuration options

## 🤝 Integration

### VS Code Extension
Configure in VS Code settings:
```json
{
  "droeLanguageServer": {
    "useCentralizedService": true,
    "droeVmServiceUrl": "localhost:50051",
    "enableStreaming": true,
    "enableRoboticsMode": true
  }
}
```

### Droe-Scribe
Set environment variables:
```bash
USE_DROEVM_SERVICE=true
DROEVM_SERVICE_URL=localhost:50051
DROEVM_FALLBACK_TO_OLLAMA=true
```

## 🔍 Monitoring

### Health Checks
```bash
# Service health
droe-llm health

# Ollama connectivity  
curl http://localhost:11434/api/tags

# gRPC endpoint
grpcurl -plaintext localhost:50051 list
```

### Logs
```bash
# Service logs
RUST_LOG=droe_llm=info droe-llm serve

# Debug logging
RUST_LOG=droe_llm=debug droe-llm serve --debug
```

## 🚀 Deployment

### Docker (Future)
```bash
docker build -t droe-llm .
docker run -p 50051:50051 droe-llm serve
```

### Systemd Service
```ini
[Unit]
Description=DroeLLM Service
After=network.target

[Service]
Type=simple
User=droe
ExecStart=/usr/local/bin/droe-llm serve --port 50051
Restart=always

[Install]
WantedBy=multi-user.target
```

## 📝 License

MIT License - see LICENSE file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Submit a pull request

## 📞 Support

- Issues: [GitHub Issues](https://github.com/droelang/droe-llm/issues)
- Discussions: [GitHub Discussions](https://github.com/droelang/droe-llm/discussions)
- Documentation: [docs.droelang.org](https://docs.droelang.org)

---

**Built with ❤️ for the DROELANG ecosystem**