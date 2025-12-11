//! Zenith Turbo Engine - High-Performance ML Acceleration
//!
//! This module implements the Zenith Turbo Engine that provides:
//! - SIMD-accelerated preprocessing
//! - Zero-copy data transfer
//! - Mixed precision support
//! - Async prefetching pipeline
//! - ONNX Runtime integration ready

pub mod simd;
pub mod prefetch;
pub mod precision;
pub mod onnx;

// Re-exports
pub use simd::{SimdOps, SimdFeatures};
pub use prefetch::{PrefetchPipeline, PrefetchConfig, PrefetchBuffer};
pub use precision::{Float16, BFloat16, LossScaler, PrecisionConverter, MixedPrecisionConfig};
pub use onnx::{OnnxSession, OnnxConfig, ExecutionProvider};

use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::Instant;

/// Turbo Engine configuration
#[derive(Debug, Clone)]
pub struct TurboConfig {
    /// Enable SIMD acceleration
    pub enable_simd: bool,
    /// Enable async prefetching
    pub enable_prefetch: bool,
    /// Number of prefetch buffers
    pub prefetch_buffers: usize,
    /// Enable mixed precision (BF16/FP16)
    pub mixed_precision: MixedPrecisionMode,
    /// Batch size for processing
    pub batch_size: usize,
    /// Number of worker threads
    pub num_workers: usize,
    /// Enable GPU direct transfer
    pub gpu_direct: bool,
}

impl Default for TurboConfig {
    fn default() -> Self {
        Self {
            enable_simd: true,
            enable_prefetch: true,
            prefetch_buffers: 4,
            mixed_precision: MixedPrecisionMode::Auto,
            batch_size: 256,
            num_workers: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
            gpu_direct: false,
        }
    }
}

/// Mixed precision modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixedPrecisionMode {
    /// Full precision (FP32)
    Full,
    /// Half precision (FP16)
    Half,
    /// Brain float (BF16)
    BFloat16,
    /// Automatic selection based on hardware
    Auto,
}

/// Data type for tensors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Float32,
    Float16,
    BFloat16,
    Int32,
    Int64,
    UInt8,
}

impl DataType {
    /// Size in bytes
    pub fn size(&self) -> usize {
        match self {
            DataType::Float32 | DataType::Int32 => 4,
            DataType::Float16 | DataType::BFloat16 => 2,
            DataType::Int64 => 8,
            DataType::UInt8 => 1,
        }
    }
}

/// Turbo statistics
#[derive(Debug, Clone, Default)]
pub struct TurboStats {
    /// Total samples processed
    pub samples_processed: u64,
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Average throughput (samples/sec)
    pub throughput: f64,
    /// SIMD operations performed
    pub simd_ops: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Prefetch queue depth
    pub prefetch_depth: usize,
}

/// Turbo Engine - Main acceleration engine
pub struct TurboEngine {
    config: TurboConfig,
    stats: Arc<RwLock<TurboStats>>,
    running: AtomicBool,
    start_time: Instant,
    samples_counter: AtomicU64,
    bytes_counter: AtomicU64,
}

impl TurboEngine {
    /// Create a new Turbo Engine
    pub fn new(config: TurboConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(TurboStats::default())),
            running: AtomicBool::new(false),
            start_time: Instant::now(),
            samples_counter: AtomicU64::new(0),
            bytes_counter: AtomicU64::new(0),
        }
    }
    
    /// Start the engine
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        tracing::info!("Turbo Engine started with config: {:?}", self.config);
    }
    
    /// Stop the engine
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        tracing::info!("Turbo Engine stopped");
    }
    
    /// Check if engine is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Get current statistics
    pub fn stats(&self) -> TurboStats {
        let mut stats = self.stats.read().clone();
        stats.samples_processed = self.samples_counter.load(Ordering::Relaxed);
        stats.bytes_processed = self.bytes_counter.load(Ordering::Relaxed);
        
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            stats.throughput = stats.samples_processed as f64 / elapsed;
        }
        
        stats
    }
    
    /// Record samples processed
    pub fn record_samples(&self, count: u64, bytes: u64) {
        self.samples_counter.fetch_add(count, Ordering::Relaxed);
        self.bytes_counter.fetch_add(bytes, Ordering::Relaxed);
    }
    
    /// Get configuration
    pub fn config(&self) -> &TurboConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_turbo_engine_creation() {
        let engine = TurboEngine::new(TurboConfig::default());
        assert!(!engine.is_running());
        
        engine.start();
        assert!(engine.is_running());
        
        engine.stop();
        assert!(!engine.is_running());
    }
    
    #[test]
    fn test_turbo_stats() {
        let engine = TurboEngine::new(TurboConfig::default());
        engine.start();
        
        engine.record_samples(1000, 4000);
        
        let stats = engine.stats();
        assert_eq!(stats.samples_processed, 1000);
        assert_eq!(stats.bytes_processed, 4000);
    }
}
