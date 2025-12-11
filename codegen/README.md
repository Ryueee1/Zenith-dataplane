# Zenith Code Generator

A powerful CLI tool for generating boilerplate code for the Zenith Data Plane ecosystem.

## Installation

```bash
cd codegen
cargo build --release
```

The binary will be available at `./target/release/zenith-codegen`.

## Features

### 1. **Plugin Generation**
Generate WASM plugin templates with common patterns:

```bash
# Create a filter plugin
zenith-codegen plugin --name my_filter --type filter --output ./plugins

# Create a transform plugin
zenith-codegen plugin --name data_enricher --type transform --output ./plugins

# Create an aggregator plugin
zenith-codegen plugin --name stats_collector --type aggregator --output ./plugins
```

**Plugin Types:**
- `filter`: Accept/reject events based on criteria
- `transform`: Modify event data
- `aggregator`: Accumulate statistics across events

### 2. **FFI Bindings Generation**
Generate SDK bindings for different languages:

```bash
# Generate Go bindings
zenith-codegen ffi --lang go --output ./sdk-go

# Generate Python bindings
zenith-codegen ffi --lang python --output ./sdk-python

# Generate Node.js bindings
zenith-codegen ffi --lang node --output ./sdk-node
```

### 3. **Schema Code Generation**
Convert Arrow schema definitions (JSON) to code:

```bash
# Generate Rust schema code
zenith-codegen schema --input ./examples/market_tick.json --lang rust --output schema.rs

# Generate Python schema code
zenith-codegen schema --input ./examples/market_tick.json --lang python --output schema.py
```

## Example Schema Definition

```json
{
  "name": "UserEvent",
  "fields": [
    {"name": "user_id", "type": "uint64", "nullable": false},
    {"name": "event_type", "type": "string", "nullable": false},
    {"name": "timestamp", "type": "uint64", "nullable": false}
  ]
}
```

## Usage in CI/CD

The codegen tool can be integrated into your build pipeline:

```yaml
# .github/workflows/codegen.yml
- name: Generate FFI bindings
  run: |
    cargo run -p zenith-codegen -- ffi --lang python --output sdk-python
```

## Development

To add new generator types, extend the `src/generators/` module.
