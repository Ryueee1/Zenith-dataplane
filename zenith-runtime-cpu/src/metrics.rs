//! Prometheus Metrics Export
//!
//! Provides HTTP endpoint for Prometheus scraping.

use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    Router,
    routing::get,
    response::IntoResponse,
    extract::State,
};
use crate::TelemetryCollector;

/// Metrics server configuration
pub struct MetricsServerConfig {
    /// Listen address
    pub listen_addr: SocketAddr,
}

impl Default for MetricsServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:9090".parse().unwrap(),
        }
    }
}

/// Metrics server state
struct MetricsState {
    collector: Arc<TelemetryCollector>,
}

/// Start Prometheus metrics server
pub async fn start_metrics_server(
    collector: Arc<TelemetryCollector>,
    config: MetricsServerConfig,
) -> crate::Result<()> {
    let state = Arc::new(MetricsState { collector });
    
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .with_state(state);
    
    tracing::info!("Starting metrics server on {}", config.listen_addr);
    
    let listener = tokio::net::TcpListener::bind(config.listen_addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Prometheus metrics endpoint
async fn metrics_handler(
    State(state): State<Arc<MetricsState>>,
) -> impl IntoResponse {
    let snapshot = state.collector.snapshot();
    
    // Format metrics in Prometheus format
    format!(
        r#"# HELP zenith_cpu_uptime_seconds Engine uptime in seconds
# TYPE zenith_cpu_uptime_seconds gauge
zenith_cpu_uptime_seconds {}

# HELP zenith_cpu_events_total Total events processed
# TYPE zenith_cpu_events_total counter
zenith_cpu_events_total {}

# HELP zenith_cpu_bytes_total Total bytes processed
# TYPE zenith_cpu_bytes_total counter
zenith_cpu_bytes_total {}

# HELP zenith_cpu_events_per_second Current events per second
# TYPE zenith_cpu_events_per_second gauge
zenith_cpu_events_per_second {}

# HELP zenith_cpu_throughput_mbps Current throughput in MB/s
# TYPE zenith_cpu_throughput_mbps gauge
zenith_cpu_throughput_mbps {}

# HELP zenith_cpu_latency_avg_microseconds Average latency in microseconds
# TYPE zenith_cpu_latency_avg_microseconds gauge
zenith_cpu_latency_avg_microseconds {}

# HELP zenith_cpu_latency_max_microseconds Maximum latency in microseconds
# TYPE zenith_cpu_latency_max_microseconds gauge
zenith_cpu_latency_max_microseconds {}

# HELP zenith_cpu_allocations_total Total memory allocations
# TYPE zenith_cpu_allocations_total counter
zenith_cpu_allocations_total {}

# HELP zenith_cpu_deallocations_total Total memory deallocations
# TYPE zenith_cpu_deallocations_total counter
zenith_cpu_deallocations_total {}
"#,
        snapshot.uptime_ms as f64 / 1000.0,
        snapshot.events_processed,
        snapshot.bytes_processed,
        snapshot.events_per_second,
        snapshot.throughput_mbps,
        snapshot.avg_latency_us,
        snapshot.max_latency_us,
        snapshot.allocations,
        snapshot.deallocations,
    )
}

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = MetricsServerConfig::default();
        assert_eq!(config.listen_addr.port(), 9090);
    }
}
