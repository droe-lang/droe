# JSON-RPC Migration Guide

## Overview

The droe-llm service has been refactored from gRPC to JSON-RPC to provide a simpler, more accessible protocol for LLM communication. This change eliminates the need for protobuf compilation while maintaining all functionality including real-time streaming.

## Key Changes

### 1. Protocol Migration
- **Before**: gRPC with protobuf definitions
- **After**: JSON-RPC 2.0 over TCP
- **Port Change**: Default port changed from 50051 (gRPC) to 9080 (JSON-RPC) to avoid conflicts

### 2. Feature Parity
All original gRPC functionality has been preserved:
- ✅ **Code Generation**: `generate_dsl` and `generate_dsl_stream`
- ✅ **Code Validation**: `validate_code` with safety checks
- ✅ **Partial Updates**: `apply_partial_update` for file modifications
- ✅ **Service Info**: `get_service_info` for health monitoring
- ✅ **Streaming**: Real-time token generation via multiple JSON-RPC responses
- ✅ **Session Management**: Session tracking and cancellation

### 3. Benefits
- **Simpler Integration**: No protobuf compilation required
- **Better Debugging**: Human-readable JSON messages
- **Cross-Platform**: Works with any TCP client
- **Reduced Dependencies**: Fewer build requirements
- **Backward Compatibility**: gRPC support can be enabled with `--features grpc`

## API Reference

### 1. Generate DSL Code

**Method**: `generate_dsl`

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "generate_dsl",
  "params": {
    "prompt": "create a hello world program",
    "context": "optional context",
    "mode": "regular",
    "client_id": "client-identifier",
    "streaming": false
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "generated_code": "display \"Hello, World!\"",
    "detected_mode": "regular",
    "confidence": 0.95,
    "validation_result": {
      "is_valid": true,
      "errors": [],
      "warnings": [],
      "safety_warnings": []
    },
    "tokens_used": 15,
    "inference_time_ms": 1250,
    "session_id": "uuid-string"
  },
  "id": 1
}
```

### 2. Streaming Generation

**Method**: `generate_dsl_stream`

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "generate_dsl_stream",
  "params": {
    "prompt": "create a counting loop",
    "mode": "regular",
    "streaming": true
  },
  "id": 2
}
```

**Response Stream** (multiple responses):
```json
{"jsonrpc": "2.0", "result": {"event_type": "mode_detected", "data": {"mode": "regular", "confidence": 0.9}}, "id": 2}
{"jsonrpc": "2.0", "result": {"event_type": "token_generated", "data": {"token": "set ", "accumulated": "set "}}, "id": 2}
{"jsonrpc": "2.0", "result": {"event_type": "token_generated", "data": {"token": "count ", "accumulated": "set count "}}, "id": 2}
{"jsonrpc": "2.0", "result": {"event_type": "completed", "data": {"generated_code": "...", "tokens_used": 25}}, "id": 2}
```

### 3. Code Validation

**Method**: `validate_code`

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "validate_code",
  "params": {
    "code": "display \"Hello World\"",
    "mode": "regular",
    "file_path": "/path/to/file.droe"
  },
  "id": 3
}
```

### 4. Service Information

**Method**: `get_service_info`

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "get_service_info",
  "params": {},
  "id": 4
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "version": "0.1.0",
    "model_name": "droe-scribe:latest",
    "ollama_url": "http://localhost:11434",
    "streaming_enabled": true,
    "supported_modes": ["regular", "robotics"],
    "server_info": {...},
    "session_stats": {...}
  },
  "id": 4
}
```

## Client Integration

### Tauri (droe-scribe)

The Tauri application now includes JSON-RPC commands:
- `generate_dsl_jsonrpc`
- `validate_dsl_jsonrpc`
- `apply_partial_update_jsonrpc`
- `get_service_info_jsonrpc`
- `health_check_jsonrpc`

### VSCode Extension

New VSCode commands available:
- `droe.generateCodeJsonRpc`
- `droe.generateCodeStreamingJsonRpc`
- `droe.validateCodeJsonRpc`
- `droe.startJsonRpcServer`
- `droe.healthCheckJsonRpc`

## Running the Server

### JSON-RPC Server (Default)
```bash
cargo run --no-default-features --features jsonrpc -- serve-json-rpc --port 9080
```

### gRPC Server (Legacy)
```bash
cargo run --features grpc -- serve --port 50051
```

### Both Protocols
```bash
cargo run --features grpc,jsonrpc -- serve-json-rpc --port 9080
# In another terminal:
cargo run --features grpc,jsonrpc -- serve --port 50051
```

## Testing

### Health Check
```bash
echo '{"jsonrpc": "2.0", "method": "get_service_info", "params": {}, "id": 1}' | nc localhost 9080
```

### Code Generation
```bash
echo '{"jsonrpc": "2.0", "method": "generate_dsl", "params": {"prompt": "hello world", "mode": "regular"}, "id": 2}' | nc localhost 9080
```

### Code Validation
```bash
echo '{"jsonrpc": "2.0", "method": "validate_code", "params": {"code": "display \"test\"", "mode": "regular"}, "id": 3}' | nc localhost 9080
```

## Configuration

The service uses the same configuration as before, but now defaults to JSON-RPC. Configuration options:

```toml
[server]
host = "0.0.0.0"
port = 9080
protocol = "jsonrpc"  # or "grpc"

[ollama]
url = "http://localhost:11434"
model = "droe-scribe:latest"

[safety]
max_distance = 5.0
max_wait_time = 30.0
```

## Migration Checklist

For projects migrating from gRPC to JSON-RPC:

- [ ] Update client code to use JSON-RPC instead of gRPC calls
- [ ] Change default port from 50051 to 9080
- [ ] Replace protobuf message types with JSON structures
- [ ] Update error handling for JSON-RPC error format
- [ ] Test streaming functionality with new event format
- [ ] Update deployment scripts and configurations
- [ ] Verify health checks and monitoring
- [ ] Update documentation and API references

## Troubleshooting

### Common Issues

1. **Port Conflicts**: If port 9080 is in use, specify a different port:
   ```bash
   cargo run -- serve-json-rpc --port 9081
   ```

2. **Connection Refused**: Ensure the server is running and accessible:
   ```bash
   netstat -an | grep 9080
   ```

3. **Invalid JSON**: Ensure JSON-RPC requests are properly formatted:
   ```bash
   echo '{"jsonrpc": "2.0", "method": "get_service_info", "params": {}, "id": 1}' | jq . | nc localhost 9080
   ```

4. **Streaming Issues**: For streaming, ensure the client handles multiple JSON responses on the same connection.

### Performance Notes

- JSON-RPC typically has slightly higher latency than gRPC due to JSON parsing
- Streaming performance is comparable to gRPC when using persistent connections
- Memory usage is slightly lower due to reduced dependency footprint
- The actual LLM inference time dominates protocol overhead in most cases

## Future Considerations

1. **HTTP/REST API**: Consider adding HTTP endpoints for web client compatibility
2. **WebSocket Support**: For better streaming in web browsers
3. **Authentication**: Add API key or token-based authentication
4. **Rate Limiting**: Implement per-client rate limiting
5. **Metrics**: Enhanced metrics collection for JSON-RPC protocol