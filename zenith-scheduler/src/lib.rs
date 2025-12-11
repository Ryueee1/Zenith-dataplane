//! # Zenith Job Scheduler
//!
//! Lightweight GPU-aware job scheduler with gang scheduling support.
//!
//! Copyright 2025 Wahyu Ardiansyah and Zenith AI Contributors
//! Licensed under Apache License 2.0
//!
//! ## Features
//!
//! - **Gang Scheduling**: All resources allocated together or not at all
//! - **Topology Awareness**: NVLink/NVSwitch/NUMA-aware placement
//! - **Preemption & Backfill**: Priority-based with backfill optimization
//! - **Quotas & Fairness**: Per-user and per-project resource limits
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Zenith Scheduler                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
//! │  │   gRPC API  │  │  REST API   │  │   Node Registry     │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────┘  │
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │              Scheduling Engine                          ││
//! │  │  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐    ││
//! │  │  │ Job Queue   │  │ Topology    │  │ Gang Scheduler│    ││
//! │  │  │ (Priority)  │  │ Matcher     │  │               │    ││
//! │  │  └─────────────┘  └─────────────┘  └───────────────┘    ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  │              State Manager (Persistence)                ││
//! │  └─────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//! ```

#![warn(missing_docs)]

pub mod agent;
pub mod api;
pub mod config;
pub mod job;
pub mod node;
pub mod scheduler;
pub mod state;

// Re-exports
pub use config::SchedulerConfig;
pub use job::{Job, JobDescriptor, JobState};
pub use node::{Node, NodeRegistry};
pub use scheduler::Scheduler;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Scheduler errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Job-related errors
    #[error("Job error: {0}")]
    Job(String),
    
    /// Node-related errors
    #[error("Node error: {0}")]
    Node(String),
    
    /// Scheduling errors
    #[error("Scheduling error: {0}")]
    Scheduling(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),
}
