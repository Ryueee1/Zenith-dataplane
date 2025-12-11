//! # Zenith Protocol Definitions
//!
//! Protobuf and gRPC definitions for Zenith infrastructure.
//!
//! Copyright 2025 Wahyu Ardiansyah and Zenith AI Contributors

pub mod v1 {
    //! Version 1 of the Zenith protocol
    
    // In production, this would include generated code:
    // tonic::include_proto!("zenith.v1");
    
    // For now, export placeholder types
    pub use super::types::*;
}

pub mod types {
    //! Common types (placeholder until proto compilation)
    
    use serde::{Deserialize, Serialize};
    
    /// Timestamp
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Timestamp {
        pub seconds: i64,
        pub nanos: i32,
    }
    
    impl Timestamp {
        pub fn now() -> Self {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            Self {
                seconds: now.as_secs() as i64,
                nanos: now.subsec_nanos() as i32,
            }
        }
    }
}
