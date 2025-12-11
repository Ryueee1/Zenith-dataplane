//! Thread management and CPU pinning

use crate::{Error, Result};
use std::thread::JoinHandle;
use tracing::{debug, warn};

/// Thread pinning configuration
#[derive(Debug, Clone)]
pub struct ThreadConfig {
    /// Name prefix for threads
    pub name_prefix: String,
    /// CPU cores to pin to (empty = no pinning)
    pub pinned_cores: Vec<usize>,
    /// Stack size in bytes (0 = default)
    pub stack_size: usize,
    /// Priority (0 = normal)
    pub priority: i32,
}

impl Default for ThreadConfig {
    fn default() -> Self {
        Self {
            name_prefix: "zenith-worker".to_string(),
            pinned_cores: vec![],
            stack_size: 0,
            priority: 0,
        }
    }
}

/// Thread pool with CPU affinity support
pub struct PinnedThreadPool {
    handles: Vec<JoinHandle<()>>,
    config: ThreadConfig,
}

impl PinnedThreadPool {
    /// Create a new pinned thread pool
    pub fn new(config: ThreadConfig) -> Self {
        Self {
            handles: Vec::new(),
            config,
        }
    }
    
    /// Spawn a thread with optional CPU pinning
    pub fn spawn<F>(&mut self, core_id: Option<usize>, f: F) -> Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        let thread_name = format!(
            "{}-{}",
            self.config.name_prefix,
            self.handles.len()
        );
        
        let mut builder = std::thread::Builder::new()
            .name(thread_name.clone());
        
        if self.config.stack_size > 0 {
            builder = builder.stack_size(self.config.stack_size);
        }
        
        let handle = builder.spawn(move || {
            // Pin to core if specified
            if let Some(core) = core_id {
                if let Err(e) = pin_to_core(core) {
                    warn!("Failed to pin thread to core {}: {}", core, e);
                } else {
                    debug!("Thread {} pinned to core {}", thread_name, core);
                }
            }
            
            f();
        }).map_err(|e| Error::Affinity(e.to_string()))?;
        
        self.handles.push(handle);
        Ok(())
    }
    
    /// Wait for all threads to complete
    pub fn join_all(self) -> Vec<std::thread::Result<()>> {
        self.handles.into_iter()
            .map(|h| h.join())
            .collect()
    }
}

/// Pin the current thread to a specific CPU core
pub fn pin_to_core(core_id: usize) -> Result<()> {
    let core_ids = core_affinity::get_core_ids()
        .ok_or_else(|| Error::Affinity("Failed to get core IDs".into()))?;
    
    if core_id >= core_ids.len() {
        return Err(Error::Affinity(format!(
            "Core ID {} is out of range (max: {})",
            core_id,
            core_ids.len() - 1
        )));
    }
    
    core_affinity::set_for_current(core_ids[core_id]);
    
    Ok(())
}

/// Pin the current thread to a set of CPU cores
pub fn pin_to_cores(core_ids: &[usize]) -> Result<()> {
    if core_ids.is_empty() {
        return Ok(());
    }
    
    // Pin to the first core in the set
    // (Full cpuset support would require platform-specific code)
    pin_to_core(core_ids[0])
}

/// Get the number of available CPU cores
pub fn available_cores() -> usize {
    core_affinity::get_core_ids()
        .map(|ids| ids.len())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        })
}

/// Get the current thread's assigned core
pub fn current_core() -> Option<usize> {
    // This is platform-specific and may require sched_getcpu on Linux
    #[cfg(target_os = "linux")]
    {
        let cpu = unsafe { libc::sched_getcpu() };
        if cpu >= 0 {
            Some(cpu as usize)
        } else {
            None
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Set the current thread's scheduling priority
#[cfg(target_os = "linux")]
pub fn set_thread_priority(priority: i32) -> Result<()> {
    use libc::{sched_param, sched_setscheduler, SCHED_FIFO};
    
    let param = sched_param {
        sched_priority: priority,
    };
    
    let result = unsafe {
        sched_setscheduler(0, SCHED_FIFO, &param)
    };
    
    if result != 0 {
        return Err(Error::Affinity(format!(
            "Failed to set thread priority: {}",
            std::io::Error::last_os_error()
        )));
    }
    
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn set_thread_priority(_priority: i32) -> Result<()> {
    Ok(()) // No-op on non-Linux
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    #[test]
    fn test_available_cores() {
        let cores = available_cores();
        assert!(cores >= 1);
    }
    
    #[test]
    fn test_thread_pool() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let mut pool = PinnedThreadPool::new(ThreadConfig::default());
        
        pool.spawn(None, move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }).unwrap();
        
        let results = pool.join_all();
        assert!(results.iter().all(|r| r.is_ok()));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
