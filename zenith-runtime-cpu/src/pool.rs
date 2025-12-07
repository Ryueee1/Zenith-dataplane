//! Memory Pool Implementation
//!
//! High-performance memory pool with slab allocation.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::Mutex;

use crate::Result;

/// Memory pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Size of each slab (power of 2)
    pub slab_size: usize,
    /// Initial number of slabs
    pub initial_slabs: usize,
    /// Maximum number of slabs
    pub max_slabs: usize,
    /// Alignment requirement
    pub alignment: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            slab_size: 4096,
            initial_slabs: 16,
            max_slabs: 1024,
            alignment: 64, // Cache line aligned
        }
    }
}

/// A slab of memory
struct Slab {
    ptr: NonNull<u8>,
    layout: Layout,
    in_use: bool,
}

impl Slab {
    fn new(size: usize, align: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, align).ok()?;
        
        let ptr = unsafe { alloc(layout) };
        let ptr = NonNull::new(ptr)?;
        
        Some(Self {
            ptr,
            layout,
            in_use: false,
        })
    }
    
    fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for Slab {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

/// Thread-safe memory pool
pub struct MemoryPool {
    config: PoolConfig,
    slabs: Mutex<Vec<Slab>>,
    allocated: AtomicUsize,
    high_water_mark: AtomicUsize,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(config: PoolConfig) -> Result<Self> {
        let mut slabs = Vec::with_capacity(config.max_slabs);
        
        // Pre-allocate initial slabs
        for _ in 0..config.initial_slabs {
            if let Some(slab) = Slab::new(config.slab_size, config.alignment) {
                slabs.push(slab);
            }
        }
        
        Ok(Self {
            config,
            slabs: Mutex::new(slabs),
            allocated: AtomicUsize::new(0),
            high_water_mark: AtomicUsize::new(0),
        })
    }
    
    /// Allocate a buffer from the pool
    pub fn allocate(&self) -> Option<PoolBuffer> {
        let mut slabs = self.slabs.lock();
        
        // Find a free slab
        for (idx, slab) in slabs.iter_mut().enumerate() {
            if !slab.in_use {
                slab.in_use = true;
                self.allocated.fetch_add(1, Ordering::Relaxed);
                
                // Update high water mark
                let current = self.allocated.load(Ordering::Relaxed);
                let mut hwm = self.high_water_mark.load(Ordering::Relaxed);
                while current > hwm {
                    match self.high_water_mark.compare_exchange_weak(
                        hwm, current, Ordering::SeqCst, Ordering::Relaxed
                    ) {
                        Ok(_) => break,
                        Err(h) => hwm = h,
                    }
                }
                
                return Some(PoolBuffer {
                    ptr: slab.as_ptr(),
                    size: self.config.slab_size,
                    pool_idx: idx,
                });
            }
        }
        
        // No free slab, try to allocate new one
        if slabs.len() < self.config.max_slabs {
            if let Some(mut slab) = Slab::new(self.config.slab_size, self.config.alignment) {
                slab.in_use = true;
                let ptr = slab.as_ptr();
                let idx = slabs.len();
                slabs.push(slab);
                
                self.allocated.fetch_add(1, Ordering::Relaxed);
                
                return Some(PoolBuffer {
                    ptr,
                    size: self.config.slab_size,
                    pool_idx: idx,
                });
            }
        }
        
        None
    }
    
    /// Return a buffer to the pool
    pub fn deallocate(&self, buffer: PoolBuffer) {
        let mut slabs = self.slabs.lock();
        
        if buffer.pool_idx < slabs.len() {
            slabs[buffer.pool_idx].in_use = false;
            self.allocated.fetch_sub(1, Ordering::Relaxed);
        }
    }
    
    /// Get current allocation count
    pub fn allocated_count(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }
    
    /// Get high water mark
    pub fn high_water_mark(&self) -> usize {
        self.high_water_mark.load(Ordering::Relaxed)
    }
    
    /// Get total capacity
    pub fn capacity(&self) -> usize {
        self.slabs.lock().len()
    }
    
    /// Get statistics
    pub fn stats(&self) -> PoolStats {
        let slabs = self.slabs.lock();
        PoolStats {
            total_slabs: slabs.len(),
            allocated_slabs: self.allocated.load(Ordering::Relaxed),
            slab_size: self.config.slab_size,
            total_memory: slabs.len() * self.config.slab_size,
            high_water_mark: self.high_water_mark.load(Ordering::Relaxed),
        }
    }
}

/// A buffer from the pool
pub struct PoolBuffer {
    ptr: *mut u8,
    size: usize,
    pool_idx: usize,
}

impl PoolBuffer {
    /// Get the buffer as a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }
    
    /// Get the buffer as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.size) }
    }
    
    /// Get buffer size
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get raw pointer
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }
}

// Safety: PoolBuffer is safe to send between threads
unsafe impl Send for PoolBuffer {}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total number of slabs
    pub total_slabs: usize,
    /// Currently allocated slabs
    pub allocated_slabs: usize,
    /// Size of each slab
    pub slab_size: usize,
    /// Total memory in bytes
    pub total_memory: usize,
    /// Maximum concurrent allocations
    pub high_water_mark: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pool_creation() {
        let config = PoolConfig {
            slab_size: 1024,
            initial_slabs: 4,
            max_slabs: 16,
            alignment: 64,
        };
        
        let pool = MemoryPool::new(config).unwrap();
        assert_eq!(pool.capacity(), 4);
        assert_eq!(pool.allocated_count(), 0);
    }
    
    #[test]
    fn test_pool_allocate_deallocate() {
        let config = PoolConfig {
            slab_size: 1024,
            initial_slabs: 4,
            max_slabs: 16,
            alignment: 64,
        };
        
        let pool = MemoryPool::new(config).unwrap();
        
        let buf1 = pool.allocate().unwrap();
        assert_eq!(buf1.size(), 1024);
        assert_eq!(pool.allocated_count(), 1);
        
        let buf2 = pool.allocate().unwrap();
        assert_eq!(pool.allocated_count(), 2);
        
        pool.deallocate(buf1);
        assert_eq!(pool.allocated_count(), 1);
        
        pool.deallocate(buf2);
        assert_eq!(pool.allocated_count(), 0);
    }
    
    #[test]
    fn test_pool_write_read() {
        let config = PoolConfig::default();
        let pool = MemoryPool::new(config).unwrap();
        
        let mut buf = pool.allocate().unwrap();
        
        // Write data
        let data = b"Hello, World!";
        buf.as_mut_slice()[..data.len()].copy_from_slice(data);
        
        // Read back
        assert_eq!(&buf.as_slice()[..data.len()], data);
        
        pool.deallocate(buf);
    }
    
    #[test]
    fn test_pool_stats() {
        let config = PoolConfig {
            slab_size: 1024,
            initial_slabs: 4,
            max_slabs: 16,
            alignment: 64,
        };
        
        let pool = MemoryPool::new(config).unwrap();
        
        let _buf1 = pool.allocate().unwrap();
        let _buf2 = pool.allocate().unwrap();
        
        let stats = pool.stats();
        assert_eq!(stats.allocated_slabs, 2);
        assert_eq!(stats.high_water_mark, 2);
        assert_eq!(stats.slab_size, 1024);
    }
}
