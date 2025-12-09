use crate::ring_buffer::ZenithRingBuffer;
// use crate::event::ZenithEvent;
use crate::wasm_host::{WasmHost, WasmPlugin};
use crate::error::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct ZenithEngine {
    buffer: ZenithRingBuffer,
    wasm_host: Arc<WasmHost>,
    plugins: Arc<Mutex<Vec<WasmPlugin>>>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl ZenithEngine {
    pub fn new(buffer_size: usize) -> Result<Self> {
        Ok(Self {
            buffer: ZenithRingBuffer::new(buffer_size),
            wasm_host: Arc::new(WasmHost::new()?),
            plugins: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        })
    }

    pub fn get_ring_buffer(&self) -> ZenithRingBuffer {
        self.buffer.clone()
    }

    pub fn load_plugin(&self, wasm_bytes: &[u8]) -> Result<()> {
        let plugin = self.wasm_host.load_plugin(wasm_bytes)?;
        let mut plugins = self.plugins.lock().unwrap();
        plugins.push(plugin);
        Ok(())
    }

    pub fn start(&self) {
        let buffer = self.buffer.clone();
        let running = self.running.clone();
        let plugins = self.plugins.clone(); 

        // Start Admin API
        let admin_state = crate::admin_api::AdminState {
            buffer: self.buffer.clone(),
            plugins: self.plugins.clone(),
        };
        
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(crate::admin_api::start_admin_server(admin_state, 8080));
        });

        thread::spawn(move || {
            println!("Zenith Core Engine: Consumer thread started.");
            while running.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(event) = buffer.pop() {
                    // Process event
                    let plugin_list = plugins.lock().unwrap();
                    let mut allowed = true;
                    
                    for plugin in plugin_list.iter() {
                        // Pass metadata to WASM
                        match plugin.on_event(event.header.source_id, event.header.seq_no) {
                            Ok(res) => {
                                if !res { allowed = false; }
                            },
                            Err(e) => eprintln!("Plugin Execution Error: {}", e),
                        }
                    }

                    if allowed {
                         // println!("Event Processed: {}", event.header.seq_no);
                         // Logic to forward to storage/network would be here
                    } else {
                         // println!("Event Dropped: {}", event.header.seq_no);
                    }
                } else {
                    thread::park_timeout(Duration::from_micros(10));
                }
            }
        });
    }

    pub fn shutdown(&self) {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let result = ZenithEngine::new(1024);
        assert!(result.is_ok(), "Engine creation should succeed");
        
        let engine = result.unwrap();
        // Verify the engine has a valid ring buffer
        let buffer = engine.get_ring_buffer();
        assert!(buffer.is_empty(), "New engine buffer should be empty");
    }
    
    #[test]
    fn test_engine_shutdown_sets_flag() {
        let engine = ZenithEngine::new(1024).unwrap();
        
        // Verify running flag is initially true
        assert!(engine.running.load(std::sync::atomic::Ordering::Relaxed),
            "Engine running flag should be true initially");
        
        // Shutdown should set running to false
        engine.shutdown();
        
        assert!(!engine.running.load(std::sync::atomic::Ordering::Relaxed),
            "Engine running flag should be false after shutdown");
    }
    
    #[test]
    fn test_engine_get_ring_buffer() {
        let engine = ZenithEngine::new(1024).unwrap();
        
        let buffer1 = engine.get_ring_buffer();
        let buffer2 = engine.get_ring_buffer();
        
        // Both buffers should be clones sharing the same underlying queue
        assert!(buffer1.is_empty());
        assert!(buffer2.is_empty());
    }
    
    #[test]
    fn test_engine_load_plugin_with_invalid_wasm() {
        let engine = ZenithEngine::new(1024).unwrap();
        
        // Invalid WASM bytes should fail
        let invalid_wasm = b"not valid wasm bytes";
        let result = engine.load_plugin(invalid_wasm);
        
        // This should return an error, not Ok(())
        assert!(result.is_err(), "Invalid WASM should fail to load");
    }
    
    #[test]
    fn test_engine_multiple_operations() {
        let engine = ZenithEngine::new(1024).unwrap();
        
        // Get buffer and verify it works
        let buffer = engine.get_ring_buffer();
        assert!(buffer.is_empty());
        
        // Shutdown and verify
        engine.shutdown();
        assert!(!engine.running.load(std::sync::atomic::Ordering::Relaxed));
    }
}

