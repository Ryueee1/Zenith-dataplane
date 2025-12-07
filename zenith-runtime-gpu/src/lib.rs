//! # Zenith GPU Runtime
//!
//! High-performance GPU optimization runtime for AI/ML workloads.
//!
//! Copyright 2025 Wahyu Ardiansyah and Zenith AI Contributors
//! Licensed under Apache License 2.0
//!
//! ## Features
//!
//! - **Device Discovery**: Automatic GPU topology detection (NVLink, NVSwitch, PCIe)
//! - **Kernel Manager**: Runtime selection between CUDA/Triton/TVM/CPU kernels
//! - **Memory Optimization**: ZeRO-style offload patterns (GPU ↔ CPU ↔ NVMe)
//! - **Dynamic Precision**: Per-layer precision switching (FP32/FP16/BF16/FP8)
//! - **Collective Communication**: NCCL integration with RDMA fallback
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Zenith GPU Runtime                        │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │              Device Discovery & Topology                 ││
//! │  │  ┌───────────┐ ┌───────────┐ ┌───────────────────────┐  ││
//! │  │  │ GPU Enum  │ │ NVLink    │ │ PCIe/NUMA Affinity    │  ││
//! │  │  └───────────┘ └───────────┘ └───────────────────────┘  ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │              Kernel Manager                              ││
//! │  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────┐  ││
//! │  │  │ CUDA      │ │ Triton    │ │ TVM       │ │ CPU     │  ││
//! │  │  └───────────┘ └───────────┘ └───────────┘ └─────────┘  ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │              Memory Manager (ZeRO-style)                 ││
//! │  │  ┌─────────────┐ ┌─────────────┐ ┌───────────────────┐  ││
//! │  │  │ GPU VRAM    │ │ Host RAM    │ │ NVMe Offload      │  ││
//! │  │  └─────────────┘ └─────────────┘ └───────────────────┘  ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │              Collective Communication                    ││
//! │  │  ┌─────────────┐ ┌─────────────┐ ┌───────────────────┐  ││
//! │  │  │ NCCL        │ │ RDMA        │ │ TCP Fallback      │  ││
//! │  │  └─────────────┘ └─────────────┘ └───────────────────┘  ││
//! │  └─────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Note
//!
//! Full GPU functionality requires:
//! - NVIDIA GPU with CUDA support
//! - CUDA Toolkit 11.8+ or 12.x
//! - NCCL library for multi-GPU communication
//!
//! Without GPU hardware, this crate provides abstraction layers and CPU fallbacks.

#![warn(missing_docs)]

pub mod device;
pub mod kernel;
pub mod memory;
pub mod collective;
pub mod config;
pub mod nvml;
pub mod cuda;
pub mod tensorrt;
pub mod multigpu;

// Re-exports
pub use config::GpuRuntimeConfig;
pub use device::{GpuDevice, GpuTopology};
pub use nvml::NvmlManager;
pub use cuda::{CudaRuntime, CudaError, DeviceProperties};
pub use tensorrt::{TrtEngine, TrtContext, Precision as TrtPrecision};
pub use multigpu::{MultiGpuComm, GpuTopology as MultiGpuTopology, MultiGpuStrategy};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// GPU Runtime errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Device errors
    #[error("Device error: {0}")]
    Device(String),
    
    /// Kernel errors
    #[error("Kernel error: {0}")]
    Kernel(String),
    
    /// Memory errors
    #[error("Memory error: {0}")]
    Memory(String),
    
    /// NCCL/Collective errors
    #[error("Collective error: {0}")]
    Collective(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// GPU/NVML errors
    #[error("GPU error: {0}")]
    Gpu(String),
}
