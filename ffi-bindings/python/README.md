# Zenith Python Bindings

Python FFI bindings for the Zenith Data Plane.

## Installation

```bash
pip install -e .
```

Or directly:
```bash
pip install zenith-ffi
```

## Prerequisites

Zenith core library must be built:

```bash
cd ../../core
cargo build --release
```

## Quick Start

```python
from zenith import ZenithClient

# Using context manager (recommended)
with ZenithClient(buffer_size=1024) as client:
    # Load plugin
    client.load_plugin("filter.wasm")
    
    # Get stats
    stats = client.get_stats()
    print(f"Plugins loaded: {stats.plugin_count}")
```

## API Documentation

### `ZenithClient(buffer_size=1024, lib_path=None)`

Create a new Zenith client.

**Parameters:**
- `buffer_size` (int): Ring buffer size
- `lib_path` (str, optional): Path to libzenith_core.so (auto-detected if None)

### `load_plugin(wasm_path: str)`

Load a WASM plugin from file.

**Raises:**
- `ZenithError`: If plugin loading fails

### `get_stats() -> Stats`

Get engine statistics.

**Returns:**
- `Stats` object with `buffer_len`, `plugin_count`, `events_processed`

### `close()`

Free engine resources. Called automatically when used as context manager.

## Error Handling

```python
from zenith import ZenithClient, ZenithError

try:
    client = ZenithClient()
    client.load_plugin("bad.wasm")
except ZenithError as e:
    print(f"Error {e.code}: {e.message}")
```

## Running Tests

```bash
python -m unittest test_zenith.py
```

Or with pytest:
```bash
pytest test_zenith.py -v
```

## Examples

See `examples/` directory for complete usage examples.

## Thread Safety

Not thread-safe. Use threading.Lock if accessing from multiple threads.

## License

Apache 2.0
