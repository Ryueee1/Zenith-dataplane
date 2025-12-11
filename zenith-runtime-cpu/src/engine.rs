//! CPU Engine - Main runtime orchestrator

use crate::{
    allocator::{AllocatorConfig, NumaAllocator},
    buffer::SpscRingBuffer,
    config::EngineConfig,
    numa::NumaTopology,
    telemetry::TelemetryCollector,
    Error, Result,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// CPU Runtime Engine
///
/// The main orchestrator for the ultra-low-latency CPU runtime.
/// Manages NUMA-aware memory, thread pinning, and I/O processing.
pub struct CpuEngine {
    config: EngineConfig,
    topology: NumaTopology,
    allocator: NumaAllocator,
    running: Arc<AtomicBool>,
    telemetry: Option<TelemetryCollector>,
}

impl CpuEngine {
    /// Create a new CPU engine with the given configuration
    pub fn new(config: EngineConfig) -> Result<Self> {
        config.validate()?;
        
        info!("Initializing Zenith CPU Engine v{}", crate::VERSION);
        
        // Discover NUMA topology
        let topology = NumaTopology::discover()?;
        info!(
            "System topology: {} NUMA nodes, {} CPUs, {} total memory",
            topology.num_nodes(),
            topology.num_cpus(),
            format_bytes(topology.total_memory())
        );
        
        // Setup allocator
        let allocator_config = AllocatorConfig {
            preferred_node: config.preferred_numa_node,
            use_hugepages: config.hugepages,
            ..Default::default()
        };
        let allocator = NumaAllocator::new(allocator_config);
        
        // Setup telemetry if enabled
        let telemetry = if config.telemetry_enabled {
            Some(TelemetryCollector::new(config.telemetry_interval_ms))
        } else {
            None
        };
        
        Ok(Self {
            config,
            topology,
            allocator,
            running: Arc::new(AtomicBool::new(false)),
            telemetry,
        })
    }
    
    /// Start the engine
    pub async fn run(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(Error::Config("Engine is already running".into()));
        }
        
        info!("Starting CPU engine...");
        
        // Start telemetry collection if enabled
        if let Some(ref telemetry) = self.telemetry {
            telemetry.start();
        }
        
        // Main processing loop
        while self.running.load(Ordering::SeqCst) {
            // Process events
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        info!("CPU engine stopped");
        Ok(())
    }
    
    /// Stop the engine
    pub fn stop(&self) {
        info!("Stopping CPU engine...");
        self.running.store(false, Ordering::SeqCst);
        
        if let Some(ref telemetry) = self.telemetry {
            telemetry.stop();
        }
    }
    
    /// Check if the engine is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Get the NUMA topology
    pub fn topology(&self) -> &NumaTopology {
        &self.topology
    }
    
    /// Get the configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
    
    /// Get the allocator
    pub fn allocator(&self) -> &NumaAllocator {
        &self.allocator
    }
    
    /// Create a new ring buffer with the configured size
    pub fn create_ring_buffer<T>(&self) -> SpscRingBuffer<T> {
        SpscRingBuffer::new(self.config.ring_buffer_size)
    }
    
    /// Get telemetry collector if available
    pub fn telemetry(&self) -> Option<&TelemetryCollector> {
        self.telemetry.as_ref()
    }
}

impl Drop for CpuEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
    
    #[tokio::test]
    async fn test_engine_creation() {
        let config = EngineConfig::default();
        let engine = CpuEngine::new(config).unwrap();
        
        assert!(!engine.is_running());
        assert!(engine.topology().num_cpus() > 0);
    }
}
