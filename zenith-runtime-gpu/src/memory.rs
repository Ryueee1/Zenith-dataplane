//! Memory Manager - ZeRO-style offload patterns


/// Memory tier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTier {
    /// GPU VRAM (fastest)
    GpuVram,
    /// CPU RAM (pinned/pageable)
    CpuRam,
    /// NVMe storage (slowest)
    Nvme,
}

/// Memory placement decision
pub struct MemoryPlacement {
    /// Tier to place data
    pub tier: MemoryTier,
    /// Estimated access latency (microseconds)
    pub latency_us: u64,
    /// Estimated bandwidth (GB/s)
    pub bandwidth_gbps: f64,
}

/// ZeRO-style memory manager
pub struct MemoryManager {
    /// GPU memory limit
    gpu_memory_limit: u64,
    /// CPU memory limit
    cpu_memory_limit: u64,
    /// Current GPU usage
    gpu_memory_used: u64,
    /// Current CPU usage
    cpu_memory_used: u64,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(gpu_memory_limit: u64, cpu_memory_limit: u64) -> Self {
        Self {
            gpu_memory_limit,
            cpu_memory_limit,
            gpu_memory_used: 0,
            cpu_memory_used: 0,
        }
    }
    
    /// Decide where to place data
    pub fn decide_placement(&self, size: u64, priority: u32) -> MemoryPlacement {
        // High priority data goes to GPU if possible
        if priority > 5 && self.gpu_memory_used + size <= self.gpu_memory_limit {
            return MemoryPlacement {
                tier: MemoryTier::GpuVram,
                latency_us: 1,
                bandwidth_gbps: 2000.0, // HBM3
            };
        }
        
        // Medium priority goes to CPU
        if self.cpu_memory_used + size <= self.cpu_memory_limit {
            return MemoryPlacement {
                tier: MemoryTier::CpuRam,
                latency_us: 100,
                bandwidth_gbps: 100.0, // DDR5
            };
        }
        
        // Low priority or overflow goes to NVMe
        MemoryPlacement {
            tier: MemoryTier::Nvme,
            latency_us: 10000,
            bandwidth_gbps: 7.0, // NVMe SSD
        }
    }
    
    /// Available GPU memory
    pub fn available_gpu_memory(&self) -> u64 {
        self.gpu_memory_limit.saturating_sub(self.gpu_memory_used)
    }
    
    /// Available CPU memory
    pub fn available_cpu_memory(&self) -> u64 {
        self.cpu_memory_limit.saturating_sub(self.cpu_memory_used)
    }
}
