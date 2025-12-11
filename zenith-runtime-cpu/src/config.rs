//! Configuration for the CPU Runtime

use serde::{Deserialize, Serialize};

/// Configuration for the CPU engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Enable NUMA-aware memory allocation
    #[serde(default = "default_true")]
    pub numa_aware: bool,
    
    /// Enable hugepage support
    #[serde(default = "default_true")]
    pub hugepages: bool,
    
    /// Number of io_uring entries
    #[serde(default = "default_io_uring_entries")]
    pub io_uring_entries: u32,
    
    /// Enable thread pinning
    #[serde(default = "default_true")]
    pub thread_pinning: bool,
    
    /// Number of worker threads (0 = auto-detect)
    #[serde(default)]
    pub worker_threads: usize,
    
    /// Preferred NUMA node (-1 = any)
    #[serde(default = "default_numa_node")]
    pub preferred_numa_node: i32,
    
    /// Ring buffer size for data ingestion
    #[serde(default = "default_ring_buffer_size")]
    pub ring_buffer_size: usize,
    
    /// Enable telemetry collection
    #[serde(default = "default_true")]
    pub telemetry_enabled: bool,
    
    /// Telemetry collection interval in milliseconds
    #[serde(default = "default_telemetry_interval")]
    pub telemetry_interval_ms: u64,
    
    /// Prometheus metrics port (0 = disabled)
    #[serde(default)]
    pub metrics_port: u16,
}

fn default_true() -> bool {
    true
}

fn default_io_uring_entries() -> u32 {
    4096
}

fn default_numa_node() -> i32 {
    -1
}

fn default_ring_buffer_size() -> usize {
    1024 * 1024 // 1M entries
}

fn default_telemetry_interval() -> u64 {
    1000 // 1 second
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            numa_aware: true,
            hugepages: true,
            io_uring_entries: 4096,
            thread_pinning: true,
            worker_threads: 0,
            preferred_numa_node: -1,
            ring_buffer_size: 1024 * 1024,
            telemetry_enabled: true,
            telemetry_interval_ms: 1000,
            metrics_port: 0,
        }
    }
}

impl EngineConfig {
    /// Create a new configuration builder
    pub fn builder() -> EngineConfigBuilder {
        EngineConfigBuilder::default()
    }
    
    /// Load configuration from a file
    pub fn from_file(path: &str) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::Error::Config(format!("Failed to read config: {}", e)))?;
        
        if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)
                .map_err(|e| crate::Error::Config(format!("Failed to parse YAML: {}", e)))
        } else {
            serde_json::from_str(&content)
                .map_err(|e| crate::Error::Config(format!("Failed to parse JSON: {}", e)))
        }
    }
    
    /// Validate configuration
    pub fn validate(&self) -> crate::Result<()> {
        if self.io_uring_entries == 0 {
            return Err(crate::Error::Config(
                "io_uring_entries must be > 0".into()
            ));
        }
        
        if self.ring_buffer_size == 0 {
            return Err(crate::Error::Config(
                "ring_buffer_size must be > 0".into()
            ));
        }
        
        Ok(())
    }
}

/// Builder for EngineConfig
#[derive(Default)]
pub struct EngineConfigBuilder {
    config: EngineConfig,
}

impl EngineConfigBuilder {
    /// Enable/disable NUMA awareness
    pub fn numa_aware(mut self, enabled: bool) -> Self {
        self.config.numa_aware = enabled;
        self
    }
    
    /// Enable/disable hugepages
    pub fn hugepages(mut self, enabled: bool) -> Self {
        self.config.hugepages = enabled;
        self
    }
    
    /// Set io_uring entries
    pub fn io_uring_entries(mut self, entries: u32) -> Self {
        self.config.io_uring_entries = entries;
        self
    }
    
    /// Enable/disable thread pinning
    pub fn thread_pinning(mut self, enabled: bool) -> Self {
        self.config.thread_pinning = enabled;
        self
    }
    
    /// Set number of worker threads
    pub fn worker_threads(mut self, count: usize) -> Self {
        self.config.worker_threads = count;
        self
    }
    
    /// Set ring buffer size
    pub fn ring_buffer_size(mut self, size: usize) -> Self {
        self.config.ring_buffer_size = size;
        self
    }
    
    /// Set metrics port
    pub fn metrics_port(mut self, port: u16) -> Self {
        self.config.metrics_port = port;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> crate::Result<EngineConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert!(config.numa_aware);
        assert!(config.hugepages);
        assert_eq!(config.io_uring_entries, 4096);
    }
    
    #[test]
    fn test_builder() {
        let config = EngineConfig::builder()
            .numa_aware(false)
            .io_uring_entries(1024)
            .build()
            .unwrap();
        
        assert!(!config.numa_aware);
        assert_eq!(config.io_uring_entries, 1024);
    }
}
