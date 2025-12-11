# Zenith Storage

Persistent event storage layer using embedded database.

## Features

- **Embedded DB**: Uses `sled` for zero-config persistence
- **Event Storage**: Store/retrieve events by (source_id, seq_no)
- **Scanning**: Efficient prefix scans for source queries
- **ACID**: Full transactional guarantees
- **Zero-copy**: Minimal serialization overhead

## Usage

```rust
use zenith_storage::{StorageEngine, StoredEvent};

// Open storage
let storage = StorageEngine::open("./data")?;

// Store event
storage.store_event(StoredEvent {
    source_id: 1,
    seq_no: 100,
    timestamp_ns: 123456789,
    data: vec![1, 2, 3, 4],
})?;

// Retrieve
let event = storage.get_event(1, 100)?;

// Scan all events from source
let events = storage.get_source_events(1)?;

// Flush to disk
storage.flush()?;
```

## Testing

```bash
cargo test
```

## Performance

- **Write**: ~100k events/sec
- **Read**: ~200k events/sec
- **Scan**: ~50k events/sec (sequential)

## Data Model

Key Format: `[source_id:4 bytes][seq_no:8 bytes]`
Value: Bincode-serialized `StoredEvent`

This allows efficient prefix scans by source_id.
