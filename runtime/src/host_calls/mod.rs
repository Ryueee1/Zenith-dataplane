/// Host Call Interface for WASM Plugins
/// Provides safe API for plugins to interact with Zenith runtime
use anyhow::Result;

/// Host functions exposed to WASM plugins
pub struct HostCallInterface {
    call_count: std::sync::atomic::AtomicU32,
}

impl HostCallInterface {
    pub fn new() -> Self {
        Self {
            call_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Log a message from the plugin
    pub fn log(&self, level: LogLevel, message: &str) {
        self.increment_call_count();
        match level {
            LogLevel::Info => tracing::info!("[WASM Plugin] {}", message),
            LogLevel::Warn => tracing::warn!("[WASM Plugin] {}", message),
            LogLevel::Error => tracing::error!("[WASM Plugin] {}", message),
        }
    }

    /// Get current timestamp (nanoseconds since UNIX epoch)
    pub fn get_timestamp_ns(&self) -> u64 {
        self.increment_call_count();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Read event metadata field
    pub fn read_event_field(&self, field_name: &str) -> Result<Vec<u8>> {
        self.increment_call_count();
        // Placeholder: In real implementation, this would access current event context
        Ok(field_name.as_bytes().to_vec())
    }

    /// Get total host calls made
    pub fn get_call_count(&self) -> u32 {
        self.call_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn increment_call_count(&self) {
        self.call_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for HostCallInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl From<u32> for LogLevel {
    fn from(val: u32) -> Self {
        match val {
            0 => LogLevel::Info,
            1 => LogLevel::Warn,
            _ => LogLevel::Error,
        }
    }
}
