# Zenith Node.js Bindings

Node.js FFI bindings for the Zenith Data Plane.

## Installation

```bash
npm install @zenith/ffi
```

Or for local development:
```bash
cd ffi-bindings/node
npm install
```

## Prerequisites

Build Zenith core first:

```bash
cd ../../core
cargo build --release
```

## Quick Start

```javascript
const { ZenithClient } = require('@zenith/ffi');

// Create client
const client = new ZenithClient(1024);

try {
  // Load plugin
  client.loadPlugin('filter.wasm');
  
  // Get stats
  const stats = client.getStats();
  console.log(`Plugins: ${stats.pluginCount}`);
} finally {
  client.close();
}
```

## API Reference

### `new ZenithClient(bufferSize, libPath)`

**Parameters:**
- `bufferSize` (number): Ring buffer size (default: 1024)
- `libPath` (string): Path to libzenith_core.so (optional, auto-detected)

### `loadPlugin(filepath)`

Load a WASM plugin.

**Throws:** `ZenithError`

### `getStats()`

Get engine statistics.

**Returns:** Object with `bufferLen`, `pluginCount`, `eventsProcessed`

### `close()`

Free resources. Always call when done!

## Error Handling

```javascript
const { ZenithClient, ZenithError } = require('@zenith/ffi');

try {
  const client = new ZenithClient();
  client.loadPlugin('bad.wasm');
} catch (err) {
  if (err instanceof ZenithError) {
    console.error(`Zenith error ${err.code}: ${err.message}`);
  }
}
```

## Testing

```bash
npm test
```

## Requirements

- Node.js >= 14.0.0
- Native addon build tools (`node-gyp`)

## License

Apache 2.0
