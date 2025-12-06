pub mod event;
pub mod ring_buffer;
pub mod engine;
pub mod wasm_host;
pub mod error;

use std::ffi::c_void;
use std::sync::Arc;
use arrow::ffi::{FFI_ArrowArray, FFI_ArrowSchema};
use arrow::record_batch::RecordBatch;
// use arrow::ffi_stream::ArrowArrayStreamReader;
use crate::engine::ZenithEngine;
use crate::event::ZenithEvent;

pub use engine::ZenithEngine as Engine;
pub use event::ZenithEvent as Event;

/// Initialize the Zenith Engine
/// Returns a raw pointer to the engine instance.
/// Caller is responsible for calling zenith_free.
#[no_mangle]
pub extern "C" fn zenith_init(buffer_size: u32) -> *mut c_void {
    match ZenithEngine::new(buffer_size as usize) {
        Ok(engine) => {
            // Start the consumer thread immediately upon init for this MVP
            engine.start();
            let boxed = Box::new(engine);
            Box::into_raw(boxed) as *mut c_void
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free the Zenith Engine
#[no_mangle]
pub unsafe extern "C" fn zenith_free(engine_ptr: *mut c_void) {
    if !engine_ptr.is_null() {
        let engine = Box::from_raw(engine_ptr as *mut ZenithEngine);
        engine.shutdown();
        // Drop handled by Box
    }
}

/// Publish an Arrow RecordBatch via C Data Interface
/// Takes ownership of the FFI structs (they are moved into Rust)
#[no_mangle]
pub unsafe extern "C" fn zenith_publish(
    engine_ptr: *mut c_void,
    array_ptr: *mut FFI_ArrowArray,
    schema_ptr: *mut FFI_ArrowSchema,
    source_id: u32,
    seq_no: u64
) -> i32 {
    if engine_ptr.is_null() || array_ptr.is_null() || schema_ptr.is_null() {
        return -1;
    }

    let engine = &*(engine_ptr as *mut ZenithEngine);
    
    // SAFETY: We assume the caller (Python) has prepared valid FFI structs
    // and effectively "forgot" them so Rust can take ownership.
    let array = std::ptr::read(array_ptr);
    let schema = std::ptr::read(schema_ptr);

    // Import from FFI
    // In a real scenario, we might avoid full import if we just want to put pointers in the ring buffer.
    // However, Zenith Core needs to verify or inspect data for the logic.
    // For Zero-Copy "Passing", we ideally pass the pointers. 
    // But Arrow-RS requires importing to a RecordBatch to work with it safely in Rust.
    // This underlying import is usually a move of pointers (cheap), not deep copy of buffers,
    // AS LONG AS the underlying buffers were allocated compatibly or we are careful.
    
    match arrow::ffi::from_ffi(array, &schema) {
        Ok(array_data) => {
            // Note: from_ffi returns ArrayData. We need RecordBatch.
            // This part is tricky because ArrayData is for a single array (column). 
            // Usually we pass a StructArray for a RecordBatch or use FFI_ArrowArrayStream.
            // For MVP, let's assume the Python side sends a StructArray representing the Batch,
            // OR we accept proper RecordBatch conversion if data is Struct.
            
            // Simplification for MVP: We assume the payload IS the RecordBatch exposed as a StructArray.
            
             let struct_array = arrow::array::StructArray::from(array_data);
             // Verify it is a struct array layout
             let batch = RecordBatch::from(&struct_array);
             let event = ZenithEvent::new(source_id, seq_no, batch);
             
             match engine.get_ring_buffer().push(event) {
                 Ok(_) => 0,
                 Err(_) => -2, // Buffer full
             }
        },
        Err(_) => -4, // FFI Error
    }
}
