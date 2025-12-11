//! Kernel Manager - Runtime kernel selection
//!
//! Selects the optimal kernel implementation at runtime.

use serde::{Deserialize, Serialize};

/// Available kernel backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelBackend {
    /// Native CUDA kernels
    Cuda,
    /// OpenAI Triton kernels
    Triton,
    /// Apache TVM generated kernels
    Tvm,
    /// CPU fallback
    Cpu,
}

/// Kernel selection criteria
#[derive(Debug, Clone)]
pub struct KernelCriteria {
    /// Operation type
    pub op_type: String,
    /// Input shapes
    pub input_shapes: Vec<Vec<usize>>,
    /// Data type (float32, float16, bfloat16, int8)
    pub dtype: String,
    /// Available backends
    pub available_backends: Vec<KernelBackend>,
}

/// Kernel manager for runtime selection
pub struct KernelManager {
    /// Available backends
    available_backends: Vec<KernelBackend>,
    /// Benchmark cache
    benchmark_cache: std::collections::HashMap<String, KernelBackend>,
}

impl KernelManager {
    /// Create a new kernel manager
    pub fn new() -> Self {
        Self {
            available_backends: Self::detect_backends(),
            benchmark_cache: std::collections::HashMap::new(),
        }
    }
    
    /// Detect available kernel backends
    fn detect_backends() -> Vec<KernelBackend> {
        let mut backends = vec![];
        
        // Always have CPU fallback
        backends.push(KernelBackend::Cpu);
        
        // Check for CUDA
        if std::path::Path::new("/usr/local/cuda").exists() {
            backends.push(KernelBackend::Cuda);
        }
        
        // Triton and TVM would need specific detection
        
        backends
    }
    
    /// Select the best kernel for an operation
    pub fn select(&self, criteria: &KernelCriteria) -> KernelBackend {
        // Check cache first
        let cache_key = format!(
            "{}_{:?}_{}",
            criteria.op_type,
            criteria.input_shapes,
            criteria.dtype
        );
        
        if let Some(&backend) = self.benchmark_cache.get(&cache_key) {
            return backend;
        }
        
        // Priority order: CUDA > Triton > TVM > CPU
        for backend in [
            KernelBackend::Cuda,
            KernelBackend::Triton,
            KernelBackend::Tvm,
            KernelBackend::Cpu,
        ] {
            if self.available_backends.contains(&backend) &&
               criteria.available_backends.contains(&backend) {
                return backend;
            }
        }
        
        KernelBackend::Cpu
    }
}

impl Default for KernelManager {
    fn default() -> Self {
        Self::new()
    }
}
