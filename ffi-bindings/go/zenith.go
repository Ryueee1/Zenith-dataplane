package zenith

/*
#cgo LDFLAGS: -L../../core/target/release -lzenith_core
#include "../../zenith_core.h"
#include <stdlib.h>
*/
import "C"
import (
	"errors"
	"fmt"
	"unsafe"
)

// Client represents a Zenith engine instance
type Client struct {
	enginePtr C.ZenithEngine
	closed    bool
}

// NewClient creates a new Zenith client with specified buffer size
func NewClient(bufferSize uint32) (*Client, error) {
	ptr := C.zenith_init(C.uint32_t(bufferSize))
	if ptr == nil {
		return nil, errors.New("failed to initialize Zenith engine")
	}
	
	return &Client{
		enginePtr: ptr,
		closed:    false,
	}, nil
}

// LoadPlugin loads a WASM plugin from bytes
func (c *Client) LoadPlugin(wasmBytes []byte) error {
	if c.closed {
		return errors.New("client is closed")
	}
	
	if len(wasmBytes) == 0 {
		return errors.New("empty WASM bytes")
	}
	
	cBytes := (*C.uint8_t)(unsafe.Pointer(&wasmBytes[0]))
	cLen := C.size_t(len(wasmBytes))
	
	ret := C.zenith_load_plugin(c.enginePtr, cBytes, cLen)
	if ret != C.ZENITH_OK {
		return fmt.Errorf("failed to load plugin: error code %d", ret)
	}
	
	return nil
}

// LoadPluginFromFile loads a WASM plugin from file
func (c *Client) LoadPluginFromFile(filepath string) error {
	wasmBytes, err := readFile(filepath)
	if err != nil {
		return fmt.Errorf("failed to read plugin file: %w", err)
	}
	
	return c.LoadPlugin(wasmBytes)
}

// GetStats retrieves engine statistics
func (c *Client) GetStats() (*Stats, error) {
	if c.closed {
		return nil, errors.New("client is closed")
	}
	
	var cStats C.ZenithStats
	ret := C.zenith_get_stats(c.enginePtr, &cStats)
	if ret != C.ZENITH_OK {
		return nil, fmt.Errorf("failed to get stats: error code %d", ret)
	}
	
	return &Stats{
		BufferLen:        uint64(cStats.buffer_len),
		PluginCount:      uint64(cStats.plugin_count),
		EventsProcessed:  uint64(cStats.events_processed),
	}, nil
}

// Close frees the engine resources
func (c *Client) Close() error {
	if c.closed {
		return nil
	}
	
	C.zenith_free(c.enginePtr)
	c.closed = true
	return nil
}

// Stats represents engine statistics
type Stats struct {
	BufferLen       uint64
	PluginCount     uint64
	EventsProcessed uint64
}

// Helper function to read file
func readFile(filepath string) ([]byte, error) {
	// Using standard library
	data, err := os.ReadFile(filepath)
	if err != nil {
		return nil, err
	}
	return data, nil
}

// Import os package
import "os"
