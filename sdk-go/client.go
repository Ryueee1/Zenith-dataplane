package zenith

/*
#cgo LDFLAGS: -L../../core/target/release -lzenith_core
#include <stdint.h>
#include <stdlib.h>

// Forward declarations of C ABI
void* zenith_init(uint32_t buffer_size);
void zenith_free(void* engine_ptr);
int32_t zenith_publish(void* engine_ptr, void* array_ptr, void* schema_ptr, uint32_t source_id, uint64_t seq_no);
int32_t zenith_load_plugin(void* engine_ptr, const uint8_t* wasm_bytes, size_t len);

*/
import "C"
import (
	"errors"
	"unsafe"
)

type Client struct {
	enginePtr unsafe.Pointer
}

func NewClient(bufferSize uint32) *Client {
	ptr := C.zenith_init(C.uint32_t(bufferSize))
	if ptr == nil {
		return nil
	}
	return &Client{enginePtr: ptr}
}

func (c *Client) Close() {
	if c.enginePtr != nil {
		C.zenith_free(c.enginePtr)
		c.enginePtr = nil
	}
}

// Publish sends an Arrow RecordBatch to the engine.
// Note: Integrating Go Arrow with C Data Interface requires 'github.com/apache/arrow/go/v14/arrow/cdata'
// This is a placeholder for the FFI bridge logic.
func (c *Client) Publish(cArray unsafe.Pointer, cSchema unsafe.Pointer, sourceID uint32, seqNo uint64) error {
	ret := C.zenith_publish(c.enginePtr, cArray, cSchema, C.uint32_t(sourceID), C.uint64_t(seqNo))
	if ret != 0 {
		return errors.New("failed to publish event")
	}
	return nil
}

func (c *Client) LoadPlugin(wasmBytes []byte) error {
	cBytes := (*C.uint8_t)(unsafe.Pointer(&wasmBytes[0]))
	cLen := C.size_t(len(wasmBytes))
	
	ret := C.zenith_load_plugin(c.enginePtr, cBytes, cLen)
	if ret != 0 {
		return errors.New("failed to load plugin")
	}
	return nil
}
