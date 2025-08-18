# gRPC Client Wrapper Usage Guide

The `GrpcClientWrapper` provides a high-level interface for managing droe-llm gRPC servers with automatic spawning, health monitoring, and lifecycle management.

## Quick Start

### Basic Usage

```rust
use droe_llm::{GrpcClientWrapper, LLMRequest, InferenceMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create wrapper with defaults
    let wrapper = GrpcClientWrapper::with_defaults();
    
    // Generate DSL code (server auto-starts if needed)
    let request = LLMRequest {
        prompt: "Generate a hello world program".to_string(),
        context: Some("Example usage".to_string()),
        mode_hint: Some(InferenceMode::Regular),
        workspace_bounds: None,
        client_id: "my_client".to_string(),
        session_id: "session_1".to_string(),
        streaming: false,
        file_path: None,
        existing_content: None,
    };
    
    let response = wrapper.generate_dsl(request).await?;
    println!("Generated: {}", response.generated_code);
    
    Ok(())
}
```

### Custom Configuration

```rust
use droe_llm::{GrpcClientWrapper, GrpcWrapperConfigBuilder};

let config = GrpcWrapperConfigBuilder::new()
    .default_port(8080)
    .startup_timeout_ms(15000)
    .health_check_interval_ms(200)
    .binary_path("/usr/local/bin/droe-llm")
    .server_args(vec!["--debug".to_string()])
    .build();

let wrapper = GrpcClientWrapper::new(config);
```

## Key Features

### 1. Auto-Start Server Management

The wrapper automatically:
- Detects if a server is running on the target port
- Spawns a new droe-llm server if none is available
- Waits for the server to become ready before proceeding
- Handles server lifecycle cleanup

### 2. Health Monitoring

```rust
// Check if server is running
if wrapper.is_server_running(50051).await {
    println!("Server is healthy");
}

// Get service information
let service_info = wrapper.get_service_info().await?;
println!("Version: {}", service_info.version);
```

### 3. Multi-Server Support

```rust
// Start multiple servers on different ports
let port1 = wrapper.start_new_server().await?;
let port2 = wrapper.start_new_server().await?;

// Get list of active servers
let active_ports = wrapper.get_active_ports().await;
println!("Active servers on ports: {:?}", active_ports);

// Use load balancing for requests
let response = wrapper.generate_dsl_load_balanced(request).await?;
```

### 4. Port Management

```rust
// Find next available port
let available_port = wrapper.find_available_port().await?;

// Start server on specific port
let port = wrapper.start_new_server().await?;

// Stop specific server
wrapper.stop_server(port).await?;

// Stop all managed servers
wrapper.stop_all_servers().await?;
```

## API Reference

### Core Methods

- `generate_dsl(request)` - Generate DSL code using default client
- `generate_dsl_stream(request)` - Generate with streaming using default client
- `validate_code(code, mode)` - Validate code using default client
- `get_service_info()` - Get service information from default client

### Server Management

- `start_new_server()` - Start server on next available port
- `stop_server(port)` - Stop server on specific port
- `stop_all_servers()` - Stop all managed servers
- `is_server_running(port)` - Check if server is healthy
- `find_available_port()` - Find next available port

### Multi-Client Support

- `get_client(port)` - Get client for specific port
- `get_default_client()` - Get client for default port
- `generate_dsl_load_balanced(request)` - Use load balancing

### Configuration

- `GrpcWrapperConfigBuilder` - Fluent configuration builder
- `default_port(port)` - Set default port
- `startup_timeout_ms(timeout)` - Set server startup timeout
- `health_check_interval_ms(interval)` - Set health check frequency
- `binary_path(path)` - Set custom droe-llm binary path
- `server_args(args)` - Set additional server arguments

## Error Handling

The wrapper provides detailed error information:

```rust
match wrapper.generate_dsl(request).await {
    Ok(response) => {
        println!("Success: {}", response.generated_code);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        // Handle specific error types
        match e {
            LLMError::ConfigError(msg) => {
                eprintln!("Configuration issue: {}", msg);
            }
            LLMError::GrpcError(status) => {
                eprintln!("gRPC error: {}", status);
            }
            _ => eprintln!("Other error: {}", e),
        }
    }
}
```

## Testing

Run the included test example:

```bash
cargo run --example test_grpc_wrapper --features grpc
```

This will test all major functionality including:
- Server auto-start
- Health checking
- DSL generation
- Code validation
- Server management

## Best Practices

1. **Resource Management**: Use `stop_all_servers()` in cleanup code
2. **Error Handling**: Always handle `LLMError` variants appropriately
3. **Port Selection**: Use `find_available_port()` to avoid conflicts
4. **Configuration**: Use `GrpcWrapperConfigBuilder` for complex setups
5. **Health Monitoring**: Check server health before critical operations

## Troubleshooting

### Common Issues

1. **Binary Not Found**: Set `binary_path` in config or ensure `droe-llm` is in PATH
2. **Port Conflicts**: Use `find_available_port()` or check active servers
3. **Startup Timeout**: Increase `startup_timeout_ms` for slow systems
4. **Connection Refused**: Verify server is running and port is correct

### Debug Logging

Enable detailed logging:

```rust
tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer()
        .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG))
    .init();
```