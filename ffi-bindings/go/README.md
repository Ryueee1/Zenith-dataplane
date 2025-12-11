# Zenith Go Bindings

Go FFI bindings for the Zenith Data Plane.

## Installation

```bash
go get github.com/vibeswithkk/zenith-dataplane/ffi-bindings/go
```

## Prerequisites

The Zenith core library must be built first:

```bash
cd ../../core
cargo build --release
```

## Usage

```go
package main

import (
	"fmt"
	"log"
	
	zenith "github.com/vibeswithkk/zenith-dataplane/ffi-bindings/go"
)

func main() {
	// Create client
	client, err := zenith.NewClient(1024)
	if err != nil {
		log.Fatal(err)
	}
	defer client.Close()
	
	// Load a plugin
	err = client.LoadPluginFromFile("plugin.wasm")
	if err != nil {
		log.Printf("Plugin load failed: %v", err)
	}
	
	// Get statistics
	stats, _ := client.GetStats()
	fmt.Printf("Plugins loaded: %d\n", stats.PluginCount)
}
```

## Running Tests

```bash
cd ffi-bindings/go
go test -v
```

## API Reference

### `NewClient(bufferSize uint32) (*Client, error)`
Creates a new Zenith engine instance.

### `LoadPlugin(wasmBytes []byte) error`  
Loads a WASM plugin from byte slice.

### `LoadPluginFromFile(filepath string) error`
Loads a WASM plugin from file.

### `GetStats() (*Stats, error)`
Retrieves engine statistics.

### `Close() error`
Frees engine resources.

## Error Handling

All functions return Go errors for proper error propagation. Always check errors!

## Thread Safety

The client is **not** thread-safe. Use mutex if accessing from multiple goroutines.
