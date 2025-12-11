# Zenith Data Plane: API Specification

**Version:** 1.0.0

## 1. Core C ABI
The core engine exports the following C-compatible symbols for SDK integration.

```c
// Initialize the engine with a ring buffer size
void* zenith_init(uint32_t buffer_size);

// Push an Arrow RecordBatch (via C Data Interface)
int32_t zenith_publish(void* engine, 
                       struct ArrowArray* array, 
                       struct ArrowSchema* schema);

// Load a WASM plugin
int32_t zenith_load_plugin(void* engine, 
                           const uint8_t* wasm_bytes, 
                           size_t len);

// Shutdown
void zenith_free(void* engine);
```

## 2. Event Envelope (Internal / Wire)
When an event transcends the ring buffer (e.g. to network), it follows this standard binary layout:

| Offset | Type   | Name         | Description                  |
| :---   | :---   | :---         | :---                         |
| 0      | u32    | source_id    | Publisher ID                 |
| 4      | u64    | seq_no       | Monotonic Sequence           |
| 12     | u64    | ts_ns        | Ingest Timestamp (Unix Nano) |
| 20     | u32    | flags        | Bitmask (0x1=Heartbeat)      |
| 24     | u32    | payload_len  | Length of Arrow IPC buffer   |
| 28     | bytes  | payload      | Arrow IPC Stream             |

## 3. WASM Host Interface
Plugins typically export:

```wit
// WIT definition
package zenith:plugin

world filter {
  export on-event: func(payload: list<u8>) -> bool
}
```

*For MVP, we use a simplified linear memory interface.*
