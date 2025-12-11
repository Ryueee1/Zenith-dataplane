//! SIMD-Accelerated Processing Layer
//!
//! Provides vectorized operations for data preprocessing.
//! Uses stable Rust with manual vectorization hints.

/// SIMD feature detection result
#[derive(Debug, Clone, Copy)]
pub struct SimdFeatures {
    pub avx2: bool,
    pub avx512: bool,
    pub neon: bool,
    pub sse4: bool,
}

impl SimdFeatures {
    /// Detect available SIMD features
    #[cfg(target_arch = "x86_64")]
    pub fn detect() -> Self {
        Self {
            avx2: std::arch::is_x86_feature_detected!("avx2"),
            avx512: std::arch::is_x86_feature_detected!("avx512f"),
            sse4: std::arch::is_x86_feature_detected!("sse4.1"),
            neon: false,
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    pub fn detect() -> Self {
        Self {
            avx2: false,
            avx512: false,
            sse4: false,
            neon: cfg!(target_arch = "aarch64"),
        }
    }
    
    /// Get best available SIMD width (elements per operation)
    pub fn best_width(&self) -> usize {
        if self.avx512 { 16 }
        else if self.avx2 || self.neon { 8 }
        else if self.sse4 { 4 }
        else { 1 }
    }
}

/// SIMD-accelerated operations using stable Rust
pub struct SimdOps {
    features: SimdFeatures,
}

impl SimdOps {
    /// Create new SIMD operations handler
    pub fn new() -> Self {
        let features = SimdFeatures::detect();
        Self { features }
    }
    
    /// Get detected features
    pub fn features(&self) -> SimdFeatures {
        self.features
    }
    
    /// Normalize a slice of f32 values in-place
    /// Formula: (x - mean) / std
    #[inline]
    pub fn normalize_inplace(&self, data: &mut [f32], mean: f32, std: f32) {
        let inv_std = 1.0 / std;
        
        // Process in chunks for better vectorization
        for chunk in data.chunks_mut(8) {
            for x in chunk.iter_mut() {
                *x = (*x - mean) * inv_std;
            }
        }
    }
    
    /// Compute sum of f32 slice
    #[inline]
    pub fn sum(&self, data: &[f32]) -> f32 {
        // Unroll manually for better vectorization
        let mut acc = [0.0f32; 8];
        let chunks = data.len() / 8;
        
        for i in 0..chunks {
            let base = i * 8;
            for j in 0..8 {
                acc[j] += data[base + j];
            }
        }
        
        let mut result: f32 = acc.iter().sum();
        
        // Handle remainder
        for val in data.iter().skip(chunks * 8) {
            result += val;
        }
        
        result
    }
    
    /// Compute mean of f32 slice
    #[inline]
    pub fn mean(&self, data: &[f32]) -> f32 {
        if data.is_empty() { return 0.0; }
        self.sum(data) / data.len() as f32
    }
    
    /// Compute variance of f32 slice
    #[inline]
    pub fn variance(&self, data: &[f32], mean: f32) -> f32 {
        if data.is_empty() { return 0.0; }
        
        let mut sum_sq = 0.0f32;
        for &x in data {
            let diff = x - mean;
            sum_sq += diff * diff;
        }
        
        sum_sq / data.len() as f32
    }
    
    /// Standard deviation
    #[inline]
    pub fn std(&self, data: &[f32], mean: f32) -> f32 {
        self.variance(data, mean).sqrt()
    }
    
    /// Element-wise multiply and accumulate (FMA)
    #[inline]
    pub fn fma(&self, a: &[f32], b: &[f32], c: &[f32], result: &mut [f32]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(b.len(), c.len());
        assert_eq!(c.len(), result.len());
        
        for i in 0..a.len() {
            result[i] = a[i].mul_add(b[i], c[i]);
        }
    }
    
    /// ReLU activation: max(0, x)
    #[inline]
    pub fn relu_inplace(&self, data: &mut [f32]) {
        for x in data.iter_mut() {
            *x = x.max(0.0);
        }
    }
    
    /// Sigmoid activation: 1 / (1 + exp(-x))
    #[inline]
    pub fn sigmoid_inplace(&self, data: &mut [f32]) {
        for x in data.iter_mut() {
            *x = 1.0 / (1.0 + (-*x).exp());
        }
    }
    
    /// Softmax (per-row for 2D data)
    pub fn softmax(&self, data: &mut [f32], row_size: usize) {
        if data.is_empty() || row_size == 0 { return; }
        
        let num_rows = data.len() / row_size;
        
        for row in 0..num_rows {
            let offset = row * row_size;
            let row_data = &mut data[offset..offset + row_size];
            
            // Find max for numerical stability
            let max_val = row_data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            
            // exp(x - max) and sum
            let mut sum = 0.0f32;
            for x in row_data.iter_mut() {
                *x = (*x - max_val).exp();
                sum += *x;
            }
            
            // Normalize
            let inv_sum = 1.0 / sum;
            for x in row_data.iter_mut() {
                *x *= inv_sum;
            }
        }
    }
    
    /// Batch matrix-vector multiply (simplified)
    /// For each batch: result = matrix @ vector
    #[inline]
    pub fn batch_matvec(&self, 
        matrices: &[f32], 
        vectors: &[f32], 
        results: &mut [f32],
        batch_size: usize,
        m: usize, 
        n: usize
    ) {
        for b in 0..batch_size {
            let mat_offset = b * m * n;
            let vec_offset = b * n;
            let res_offset = b * m;
            
            for i in 0..m {
                let mut sum = 0.0f32;
                for j in 0..n {
                    sum += matrices[mat_offset + i * n + j] * vectors[vec_offset + j];
                }
                results[res_offset + i] = sum;
            }
        }
    }
}

impl Default for SimdOps {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_features() {
        let features = SimdFeatures::detect();
        println!("SIMD features: {:?}", features);
        assert!(features.best_width() >= 1);
    }
    
    #[test]
    fn test_simd_normalize() {
        let simd = SimdOps::new();
        
        let mut data: Vec<f32> = (0..16).map(|x| x as f32).collect();
        let mean = simd.mean(&data);
        let std = simd.std(&data, mean);
        
        simd.normalize_inplace(&mut data, mean, std);
        
        let new_mean = simd.mean(&data);
        assert!(new_mean.abs() < 0.01, "Mean should be ~0, got {}", new_mean);
    }
    
    #[test]
    fn test_simd_sum() {
        let simd = SimdOps::new();
        let data: Vec<f32> = (1..=100).map(|x| x as f32).collect();
        
        let sum = simd.sum(&data);
        let expected = 5050.0;
        
        assert!((sum - expected).abs() < 0.01);
    }
    
    #[test]
    fn test_simd_relu() {
        let simd = SimdOps::new();
        let mut data = vec![-2.0, -1.0, 0.0, 1.0, 2.0, -3.0, 4.0, -5.0];
        
        simd.relu_inplace(&mut data);
        
        assert_eq!(data, vec![0.0, 0.0, 0.0, 1.0, 2.0, 0.0, 4.0, 0.0]);
    }
    
    #[test]
    fn test_softmax() {
        let simd = SimdOps::new();
        let mut data = vec![1.0, 2.0, 3.0, 4.0];
        
        simd.softmax(&mut data, 4);
        
        // Sum should be 1
        let sum: f32 = data.iter().sum();
        assert!((sum - 1.0).abs() < 0.0001);
        
        // Values should be increasing
        assert!(data[0] < data[1]);
        assert!(data[1] < data[2]);
        assert!(data[2] < data[3]);
    }
}
