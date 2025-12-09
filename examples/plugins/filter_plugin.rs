//! Simple WASM filter plugin for Zenith
//! 
//! This plugin demonstrates the on_event callback that filters events.
//! Compile with: cargo build --target wasm32-wasi --release

/// Event handler called by Zenith engine
/// 
/// Returns:
///   1 = allow event (pass through)
///   0 = block event (drop)
#[no_mangle]
pub extern "C" fn on_event(source_id: i32, seq_no: i64) -> i32 {
    // Example filter logic:
    // - Block events from source_id 0 (reserved)
    // - Block events with seq_no divisible by 100 (sampling)
    
    if source_id == 0 {
        return 0; // Block reserved source
    }
    
    if seq_no % 100 == 0 {
        return 0; // Sample: drop every 100th event
    }
    
    1 // Allow event
}

/// Initialize the plugin (optional)
#[no_mangle]
pub extern "C" fn init() -> i32 {
    // Plugin initialization code here
    1 // Success
}

/// Get plugin version
#[no_mangle]
pub extern "C" fn version() -> i32 {
    1 // Version 1
}
