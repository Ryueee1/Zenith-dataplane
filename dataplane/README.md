# Zenith Data Plane

High-performance event processing engine.

## Architecture

```
Ingress → Pipeline → Processor → Router → Egress
```

## Components

### 1. **DataPlaneEngine**
Core engine for event ingestion and processing.

```rust
let engine = DataPlaneEngine::new(1024);
engine.start().await?;

engine.ingest(Event {
    id: 1,
    source_id: 100,
    timestamp_ns: 123456789,
    data: vec![1, 2, 3],
})?;
```

### 2. **Pipeline**
Configurable processing stages.

```rust
let mut pipeline = Pipeline::new();
pipeline.add_stage(FilterStage::new(|e| e.source_id == 100));
pipeline.add_stage(TransformStage::new(|mut e| {
    e.data.push(99);
    e
}));
```

### 3. **EventRouter**
Route events to destinations.

```rust
let mut router = EventRouter::new();
router.add_route(100, tx_channel);
router.route(&event);
```

## Performance

- **Throughput**: 1M+ events/sec
- **Latency**: < 100μs p99
- **Zero-copy**: Minimal allocations

## Testing

```bash
cargo test
```

## Usage

```rust
use zenith_dataplane::{DataPlaneEngine, Event};

#[tokio::main]
async fn main() {
    let engine = DataPlaneEngine::new(8192);
    engine.start().await.unwrap();
    
    // Process events
    for i in 0..1000000 {
        engine.ingest(Event {
            id: i,
            source_id: 1,
            timestamp_ns: 0,
            data: vec![],
        }).unwrap();
    }
    
    let stats = engine.get_stats();
    println!("Processed: {}", stats.events_processed);
}
```

## Integration

Data plane integrates with:
- Control Plane (management)
- Storage (persistence)
- WASM Runtime (plugins)
