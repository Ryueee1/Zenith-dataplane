//! Scheduler configuration

use serde::{Deserialize, Serialize};

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Listen address for gRPC
    pub grpc_address: String,
    /// Listen address for HTTP/REST
    pub http_address: String,
    /// Heartbeat timeout in seconds
    pub heartbeat_timeout_seconds: i64,
    /// Scheduling interval in milliseconds
    pub schedule_interval_ms: u64,
    /// Maximum jobs per scheduling cycle
    pub max_schedule_batch: usize,
    /// Enable backfill scheduling
    pub backfill_enabled: bool,
    /// Enable topology-aware placement
    pub topology_aware: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            grpc_address: "[::]:50051".to_string(),
            http_address: "0.0.0.0:8080".to_string(),
            heartbeat_timeout_seconds: 60,
            schedule_interval_ms: 1000,
            max_schedule_batch: 100,
            backfill_enabled: true,
            topology_aware: true,
        }
    }
}
