//! GPU Device Discovery and Topology

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// GPU device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    /// Device index
    pub index: u32,
    /// Device name
    pub name: String,
    /// UUID
    pub uuid: String,
    /// Compute capability major
    pub compute_major: u32,
    /// Compute capability minor
    pub compute_minor: u32,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Free memory in bytes
    pub free_memory: u64,
    /// Number of SMs
    pub sm_count: u32,
    /// Current utilization (0-100)
    pub utilization: u32,
    /// Temperature (Celsius)
    pub temperature: u32,
    /// Power usage (Watts)
    pub power_usage: u32,
    /// Power limit (Watts)
    pub power_limit: u32,
    /// PCIe generation
    pub pcie_gen: u32,
    /// PCIe width
    pub pcie_width: u32,
}

/// NVLink connection between two GPUs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvLinkConnection {
    /// Source device index
    pub source: u32,
    /// Target device index
    pub target: u32,
    /// Number of NVLinks
    pub link_count: u32,
    /// Bandwidth in GB/s
    pub bandwidth_gbps: u32,
}

/// GPU topology for a system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuTopology {
    /// All GPUs in the system
    pub devices: Vec<GpuDevice>,
    /// NVLink connections
    pub nvlink_connections: Vec<NvLinkConnection>,
    /// NVSwitch present
    pub nvswitch_present: bool,
    /// NUMA affinity map (GPU index -> NUMA node)
    pub numa_affinity: HashMap<u32, u32>,
}

impl GpuTopology {
    /// Discover GPU topology on the system
    pub fn discover() -> Result<Self> {
        info!("Discovering GPU topology...");
        
        // Check for NVIDIA driver
        if !Self::check_nvidia_driver() {
            warn!("NVIDIA driver not found, returning empty topology");
            return Ok(Self::empty());
        }
        
        // In production, this would call NVML or CUDA APIs
        // For now, return mock data or empty
        
        if let Some(mock) = Self::discover_mock() {
            info!("Using mock GPU topology for development");
            return Ok(mock);
        }
        
        Ok(Self::empty())
    }
    
    /// Check if NVIDIA driver is available
    fn check_nvidia_driver() -> bool {
        std::path::Path::new("/dev/nvidia0").exists() ||
        std::process::Command::new("nvidia-smi")
            .arg("--query-gpu=name")
            .arg("--format=csv,noheader")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    /// Create empty topology
    fn empty() -> Self {
        Self {
            devices: vec![],
            nvlink_connections: vec![],
            nvswitch_present: false,
            numa_affinity: HashMap::new(),
        }
    }
    
    /// Create mock topology for development
    fn discover_mock() -> Option<Self> {
        // Only create mock if explicitly requested via env var
        if std::env::var("ZENITH_MOCK_GPUS").is_ok() {
            let devices: Vec<GpuDevice> = (0..4)
                .map(|i| GpuDevice {
                    index: i,
                    name: "NVIDIA A100-SXM4-80GB (Mock)".to_string(),
                    uuid: format!("GPU-MOCK-{:08x}", i),
                    compute_major: 8,
                    compute_minor: 0,
                    total_memory: 80 * 1024 * 1024 * 1024,
                    free_memory: 78 * 1024 * 1024 * 1024,
                    sm_count: 108,
                    utilization: 0,
                    temperature: 40,
                    power_usage: 100,
                    power_limit: 400,
                    pcie_gen: 4,
                    pcie_width: 16,
                })
                .collect();
            
            // NVLink connections (all-to-all for mock)
            let mut nvlink_connections = vec![];
            for i in 0..4 {
                for j in (i + 1)..4 {
                    nvlink_connections.push(NvLinkConnection {
                        source: i,
                        target: j,
                        link_count: 12,
                        bandwidth_gbps: 600,
                    });
                }
            }
            
            Some(Self {
                devices,
                nvlink_connections,
                nvswitch_present: true,
                numa_affinity: HashMap::from([
                    (0, 0), (1, 0), (2, 1), (3, 1)
                ]),
            })
        } else {
            None
        }
    }
    
    /// Get number of GPUs
    pub fn gpu_count(&self) -> usize {
        self.devices.len()
    }
    
    /// Check if NVLink is available between two GPUs
    pub fn has_nvlink(&self, gpu1: u32, gpu2: u32) -> bool {
        self.nvlink_connections.iter().any(|c| {
            (c.source == gpu1 && c.target == gpu2) ||
            (c.source == gpu2 && c.target == gpu1)
        })
    }
    
    /// Get GPUs on a specific NUMA node
    pub fn gpus_on_numa(&self, numa_node: u32) -> Vec<u32> {
        self.numa_affinity.iter()
            .filter(|(_, &node)| node == numa_node)
            .map(|(&gpu, _)| gpu)
            .collect()
    }
    
    /// Get total GPU memory in bytes
    pub fn total_memory(&self) -> u64 {
        self.devices.iter().map(|d| d.total_memory).sum()
    }
    
    /// Get total free GPU memory in bytes
    pub fn free_memory(&self) -> u64 {
        self.devices.iter().map(|d| d.free_memory).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_topology() {
        let topo = GpuTopology::empty();
        assert_eq!(topo.gpu_count(), 0);
    }
}
