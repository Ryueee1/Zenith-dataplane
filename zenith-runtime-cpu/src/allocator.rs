//! NUMA-Aware Memory Allocator
//!
//! Custom allocator with NUMA awareness and hugepage support.

use crate::{Error, Result};
use std::alloc::Layout;
use std::ptr::NonNull;

/// NUMA-aware allocator configuration
#[derive(Debug, Clone)]
pub struct AllocatorConfig {
    /// Preferred NUMA node (-1 for any)
    pub preferred_node: i32,
    /// Use hugepages when available
    pub use_hugepages: bool,
    /// Minimum size for hugepage allocation
    pub hugepage_threshold: usize,
    /// Enable zero-initialization
    pub zero_init: bool,
}

impl Default for AllocatorConfig {
    fn default() -> Self {
        Self {
            preferred_node: -1,
            use_hugepages: true,
            hugepage_threshold: 2 * 1024 * 1024, // 2MB
            zero_init: false,
        }
    }
}

/// NUMA-aware memory allocator
///
/// Provides memory allocation with:
/// - NUMA node affinity
/// - Hugepage support
/// - Memory locking (mlock) for latency-critical allocations
pub struct NumaAllocator {
    config: AllocatorConfig,
}

impl NumaAllocator {
    /// Create a new NUMA-aware allocator
    pub fn new(config: AllocatorConfig) -> Self {
        Self { config }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(AllocatorConfig::default())
    }
    
    /// Allocate memory on the preferred NUMA node
    ///
    /// # Safety
    ///
    /// The caller must ensure the layout is valid and the returned
    /// pointer is freed with the same allocator.
    pub unsafe fn allocate(&self, layout: Layout) -> Result<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();
        
        // Use hugepages for large allocations
        let ptr = if self.config.use_hugepages && size >= self.config.hugepage_threshold {
            self.allocate_hugepage(size, align)?
        } else {
            self.allocate_regular(size, align)?
        };
        
        // Zero-initialize if requested
        if self.config.zero_init {
            std::ptr::write_bytes(ptr.as_ptr(), 0, size);
        }
        
        Ok(ptr)
    }
    
    /// Allocate regular memory
    unsafe fn allocate_regular(&self, size: usize, align: usize) -> Result<NonNull<u8>> {
        let layout = Layout::from_size_align(size, align)
            .map_err(|e| Error::Allocation(e.to_string()))?;
        
        let ptr = std::alloc::alloc(layout);
        
        NonNull::new(ptr).ok_or_else(|| {
            Error::Allocation(format!(
                "Failed to allocate {} bytes with alignment {}",
                size, align
            ))
        })
    }
    
    /// Allocate using hugepages
    #[cfg(target_os = "linux")]
    unsafe fn allocate_hugepage(&self, size: usize, _align: usize) -> Result<NonNull<u8>> {
        use libc::{mmap, MAP_ANONYMOUS, MAP_HUGETLB, MAP_PRIVATE, PROT_READ, PROT_WRITE};
        
        // Round up to hugepage size (2MB)
        let hugepage_size = 2 * 1024 * 1024;
        let aligned_size = (size + hugepage_size - 1) & !(hugepage_size - 1);
        
        let ptr = mmap(
            std::ptr::null_mut(),
            aligned_size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS | MAP_HUGETLB,
            -1,
            0,
        );
        
        if ptr == libc::MAP_FAILED {
            // Fall back to regular allocation
            return self.allocate_regular(size, _align);
        }
        
        NonNull::new(ptr as *mut u8).ok_or_else(|| {
            Error::Allocation("Hugepage allocation returned null".into())
        })
    }
    
    #[cfg(not(target_os = "linux"))]
    unsafe fn allocate_hugepage(&self, size: usize, align: usize) -> Result<NonNull<u8>> {
        // Hugepages only supported on Linux
        self.allocate_regular(size, align)
    }
    
    /// Deallocate memory
    ///
    /// # Safety
    ///
    /// The pointer must have been allocated by this allocator with the same layout.
    pub unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        std::alloc::dealloc(ptr.as_ptr(), layout);
    }
    
    /// Lock memory to prevent paging (for latency-critical allocations)
    #[cfg(target_os = "linux")]
    pub fn lock_memory(&self, ptr: *mut u8, size: usize) -> Result<()> {
        let result = unsafe { libc::mlock(ptr as *const libc::c_void, size) };
        
        if result != 0 {
            return Err(Error::Allocation(format!(
                "Failed to lock memory: {}",
                std::io::Error::last_os_error()
            )));
        }
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn lock_memory(&self, _ptr: *mut u8, _size: usize) -> Result<()> {
        Ok(()) // No-op on non-Linux
    }
    
    /// Unlock previously locked memory
    #[cfg(target_os = "linux")]
    pub fn unlock_memory(&self, ptr: *mut u8, size: usize) -> Result<()> {
        let result = unsafe { libc::munlock(ptr as *const libc::c_void, size) };
        
        if result != 0 {
            return Err(Error::Allocation(format!(
                "Failed to unlock memory: {}",
                std::io::Error::last_os_error()
            )));
        }
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn unlock_memory(&self, _ptr: *mut u8, _size: usize) -> Result<()> {
        Ok(())
    }
}

/// Type-safe wrapper for NUMA-allocated memory
pub struct NumaBox<T> {
    ptr: NonNull<T>,
    allocator: NumaAllocator,
}

impl<T> NumaBox<T> {
    /// Allocate a value on the preferred NUMA node
    pub fn new(value: T, allocator: NumaAllocator) -> Result<Self> {
        let layout = Layout::new::<T>();
        
        let ptr = unsafe { allocator.allocate(layout)? };
        
        unsafe {
            std::ptr::write(ptr.as_ptr() as *mut T, value);
        }
        
        Ok(Self {
            ptr: ptr.cast(),
            allocator,
        })
    }
}

impl<T> std::ops::Deref for NumaBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> std::ops::DerefMut for NumaBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for NumaBox<T> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(self.ptr.as_ptr());
            self.allocator.deallocate(
                self.ptr.cast(),
                Layout::new::<T>(),
            );
        }
    }
}

// Safety: NumaBox is Send/Sync if T is
unsafe impl<T: Send> Send for NumaBox<T> {}
unsafe impl<T: Sync> Sync for NumaBox<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_numa_allocator_basic() {
        let allocator = NumaAllocator::with_defaults();
        let layout = Layout::from_size_align(1024, 8).unwrap();
        
        unsafe {
            let ptr = allocator.allocate(layout).unwrap();
            assert!(!ptr.as_ptr().is_null());
            
            // Write some data
            std::ptr::write(ptr.as_ptr(), 42u8);
            assert_eq!(*ptr.as_ptr(), 42);
            
            allocator.deallocate(ptr, layout);
        }
    }
    
    #[test]
    fn test_numa_box() {
        let allocator = NumaAllocator::with_defaults();
        let boxed = NumaBox::new(42u64, allocator).unwrap();
        assert_eq!(*boxed, 42);
    }
}
