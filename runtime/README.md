# Zenith Runtime Library

The Zenith Runtime provides advanced orchestration and management for WASM plugins in the Zenith Data Plane.

## Features

### 1. **Plugin Lifecycle Management**
- Hot-reload: Automatically detect and reload plugins when `.wasm` files change
- Safe loading with validation

### 2. **Sandboxing & Security**
- Resource limits (CPU timeout, memory limits, host call limits)
- Execution context tracking
- WASM bytecode validation

### 3. **Task Scheduling**
- Priority-based task queue (Critical, High, Normal, Low)
- Concurrency control with semaphores
- Fair scheduling across priority levels

### 4. **Virtual Machine Abstraction**
- Wasmtime integration
- Function export discovery
- Safe execution wrapper

### 5. **Host Call Interface**
- Logging from plugins
- Timestamp access
- Event metadata reading
- Call counting for quota enforcement

## Architecture

```
Runtime
├── sandbox/      - Security & resource limits
├── scheduler/    - Task priority queue
├── vm/          - WASM execution engine
├── engine/      - Orchestration layer
└── host_calls/  - Plugin→Host API
```

## Usage

```rust
use zenith_runtime::{Runtime, RuntimeEngine, PluginMetadata};

#[tokio::main]
async fn main() {
    // Create runtime with hot-reload
    let runtime = Runtime::new(1024, "./plugins").unwrap();
    
    // Run (blocks until Ctrl+C)
    runtime.run().await.unwrap();
}
```

## Testing

```bash
cargo test
```

All modules include unit tests for core functionality.
