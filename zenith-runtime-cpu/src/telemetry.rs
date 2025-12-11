//! Telemetry and Metrics Collection

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

/// Telemetry collector for runtime metrics
pub struct TelemetryCollector {
    running: Arc<AtomicBool>,
    _interval_ms: u64,
    start_time: Instant,
    
    // Counters
    events_processed: AtomicU64,
    bytes_processed: AtomicU64,
    allocations: AtomicU64,
    deallocations: AtomicU64,
    
    // Latency tracking (microseconds)
    latency_sum: AtomicU64,
    latency_count: AtomicU64,
    latency_max: AtomicU64,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new(interval_ms: u64) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            _interval_ms: interval_ms,
            start_time: Instant::now(),
            events_processed: AtomicU64::new(0),
            bytes_processed: AtomicU64::new(0),
            allocations: AtomicU64::new(0),
            deallocations: AtomicU64::new(0),
            latency_sum: AtomicU64::new(0),
            latency_count: AtomicU64::new(0),
            latency_max: AtomicU64::new(0),
        }
    }
    
    /// Start telemetry collection
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        debug!("Telemetry collection started");
    }
    
    /// Stop telemetry collection
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        debug!("Telemetry collection stopped");
    }
    
    /// Record an event processed
    pub fn record_event(&self, bytes: u64) {
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        self.bytes_processed.fetch_add(bytes, Ordering::Relaxed);
    }
    
    /// Record a latency measurement in microseconds
    pub fn record_latency(&self, latency_us: u64) {
        self.latency_sum.fetch_add(latency_us, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
        
        // Update max latency (compare-and-swap loop)
        loop {
            let current_max = self.latency_max.load(Ordering::Relaxed);
            if latency_us <= current_max {
                break;
            }
            if self.latency_max.compare_exchange(
                current_max,
                latency_us,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
    }
    
    /// Record an allocation
    pub fn record_allocation(&self) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a deallocation
    pub fn record_deallocation(&self) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get current metrics snapshot
    pub fn snapshot(&self) -> TelemetrySnapshot {
        let uptime_ms = self.start_time.elapsed().as_millis() as u64;
        let events = self.events_processed.load(Ordering::Relaxed);
        let bytes = self.bytes_processed.load(Ordering::Relaxed);
        let latency_count = self.latency_count.load(Ordering::Relaxed);
        let latency_sum = self.latency_sum.load(Ordering::Relaxed);
        
        TelemetrySnapshot {
            uptime_ms,
            events_processed: events,
            bytes_processed: bytes,
            events_per_second: if uptime_ms > 0 {
                (events * 1000) / uptime_ms
            } else {
                0
            },
            throughput_mbps: if uptime_ms > 0 {
                (bytes * 1000) / (uptime_ms * 1024 * 1024)
            } else {
                0
            },
            avg_latency_us: if latency_count > 0 {
                latency_sum / latency_count
            } else {
                0
            },
            max_latency_us: self.latency_max.load(Ordering::Relaxed),
            allocations: self.allocations.load(Ordering::Relaxed),
            deallocations: self.deallocations.load(Ordering::Relaxed),
        }
    }
    
    /// Reset all counters
    pub fn reset(&self) {
        self.events_processed.store(0, Ordering::Relaxed);
        self.bytes_processed.store(0, Ordering::Relaxed);
        self.latency_sum.store(0, Ordering::Relaxed);
        self.latency_count.store(0, Ordering::Relaxed);
        self.latency_max.store(0, Ordering::Relaxed);
        self.allocations.store(0, Ordering::Relaxed);
        self.deallocations.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of telemetry metrics
#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    /// Uptime in milliseconds
    pub uptime_ms: u64,
    /// Total events processed
    pub events_processed: u64,
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Events per second
    pub events_per_second: u64,
    /// Throughput in MB/s
    pub throughput_mbps: u64,
    /// Average latency in microseconds
    pub avg_latency_us: u64,
    /// Maximum latency in microseconds
    pub max_latency_us: u64,
    /// Total allocations
    pub allocations: u64,
    /// Total deallocations
    pub deallocations: u64,
}

impl std::fmt::Display for TelemetrySnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Uptime: {}ms | Events: {} ({}/s) | Throughput: {} MB/s | Latency: avg={}µs max={}µs",
            self.uptime_ms,
            self.events_processed,
            self.events_per_second,
            self.throughput_mbps,
            self.avg_latency_us,
            self.max_latency_us,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_telemetry_collector() {
        let collector = TelemetryCollector::new(1000);
        
        collector.record_event(1024);
        collector.record_event(2048);
        collector.record_latency(50);
        collector.record_latency(100);
        
        let snapshot = collector.snapshot();
        assert_eq!(snapshot.events_processed, 2);
        assert_eq!(snapshot.bytes_processed, 3072);
        assert_eq!(snapshot.avg_latency_us, 75);
        assert_eq!(snapshot.max_latency_us, 100);
    }
}
