//! GPU Runtime Configuration

use serde::{Deserialize, Serialize};

/// GPU Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuRuntimeConfig {
    /// Enable automatic kernel selection
    pub auto_kernel_select: bool,
    /// Enable memory offload to CPU
    pub enable_cpu_offload: bool,
    /// Enable memory offload to NVMe
    pub enable_nvme_offload: bool,
    /// NVMe offload path
    pub nvme_offload_path: Option<String>,
    /// Default precision
    pub default_precision: String,
    /// Enable dynamic precision switching
    pub dynamic_precision: bool,
    /// GPU memory limit per device (0 = no limit)
    pub gpu_memory_limit: u64,
    /// NCCL socket interface
    pub nccl_socket_ifname: Option<String>,
    /// Enable profiling
    pub profiling_enabled: bool,
}

impl Default for GpuRuntimeConfig {
    fn default() -> Self {
        Self {
            auto_kernel_select: true,
            enable_cpu_offload: true,
            enable_nvme_offload: false,
            nvme_offload_path: None,
            default_precision: "float16".to_string(),
            dynamic_precision: false,
            gpu_memory_limit: 0,
            nccl_socket_ifname: None,
            profiling_enabled: false,
        }
    }
}
