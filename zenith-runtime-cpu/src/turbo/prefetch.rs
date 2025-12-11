//! Async Prefetch Pipeline
//!
//! Zero-latency data loading with async prefetching.

use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::{Mutex, Condvar};
use std::thread::{self, JoinHandle};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Prefetch buffer containing prepared batch data
pub struct PrefetchBuffer {
    /// Raw data buffer
    pub data: Vec<u8>,
    /// Number of samples in this buffer
    pub num_samples: usize,
    /// Byte offset for each sample
    pub offsets: Vec<usize>,
    /// Is this buffer ready for consumption
    pub ready: bool,
}

impl PrefetchBuffer {
    /// Create empty buffer with capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            num_samples: 0,
            offsets: Vec::new(),
            ready: false,
        }
    }
    
    /// Reset buffer for reuse
    pub fn reset(&mut self) {
        self.data.clear();
        self.num_samples = 0;
        self.offsets.clear();
        self.ready = false;
    }
}

/// Prefetch pipeline configuration
#[derive(Debug, Clone)]
pub struct PrefetchConfig {
    /// Number of prefetch buffers
    pub num_buffers: usize,
    /// Size of each buffer in bytes
    pub buffer_size: usize,
    /// Number of worker threads
    pub num_workers: usize,
    /// Enable pinned memory for GPU
    pub pinned_memory: bool,
}

impl Default for PrefetchConfig {
    fn default() -> Self {
        Self {
            num_buffers: 4,
            buffer_size: 64 * 1024 * 1024, // 64MB
            num_workers: 2,
            pinned_memory: false,
        }
    }
}

/// Thread-safe prefetch queue
pub struct PrefetchQueue {
    ready_buffers: Mutex<VecDeque<PrefetchBuffer>>,
    free_buffers: Mutex<VecDeque<PrefetchBuffer>>,
    not_empty: Condvar,
    not_full: Condvar,
    shutdown: AtomicBool,
    stats: PrefetchStats,
}

/// Prefetch statistics
#[derive(Debug, Default)]
pub struct PrefetchStats {
    pub buffers_produced: AtomicUsize,
    pub buffers_consumed: AtomicUsize,
    pub bytes_prefetched: AtomicUsize,
    pub queue_full_waits: AtomicUsize,
    pub queue_empty_waits: AtomicUsize,
}

impl PrefetchQueue {
    /// Create new prefetch queue
    pub fn new(config: &PrefetchConfig) -> Self {
        let mut free_buffers = VecDeque::new();
        for _ in 0..config.num_buffers {
            free_buffers.push_back(PrefetchBuffer::new(config.buffer_size));
        }
        
        Self {
            ready_buffers: Mutex::new(VecDeque::new()),
            free_buffers: Mutex::new(free_buffers),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            shutdown: AtomicBool::new(false),
            stats: PrefetchStats::default(),
        }
    }
    
    /// Get a free buffer for filling
    pub fn get_free_buffer(&self) -> Option<PrefetchBuffer> {
        let mut free = self.free_buffers.lock();
        
        while free.is_empty() && !self.shutdown.load(Ordering::Relaxed) {
            self.stats.queue_full_waits.fetch_add(1, Ordering::Relaxed);
            self.not_full.wait(&mut free);
        }
        
        if self.shutdown.load(Ordering::Relaxed) {
            return None;
        }
        
        free.pop_front()
    }
    
    /// Submit a filled buffer to the ready queue
    pub fn submit_buffer(&self, mut buffer: PrefetchBuffer) {
        buffer.ready = true;
        
        let mut ready = self.ready_buffers.lock();
        self.stats.buffers_produced.fetch_add(1, Ordering::Relaxed);
        self.stats.bytes_prefetched.fetch_add(buffer.data.len(), Ordering::Relaxed);
        
        ready.push_back(buffer);
        self.not_empty.notify_one();
    }
    
    /// Get a ready buffer for consumption
    pub fn get_ready_buffer(&self) -> Option<PrefetchBuffer> {
        let mut ready = self.ready_buffers.lock();
        
        while ready.is_empty() && !self.shutdown.load(Ordering::Relaxed) {
            self.stats.queue_empty_waits.fetch_add(1, Ordering::Relaxed);
            self.not_empty.wait(&mut ready);
        }
        
        if self.shutdown.load(Ordering::Relaxed) && ready.is_empty() {
            return None;
        }
        
        let buffer = ready.pop_front();
        if buffer.is_some() {
            self.stats.buffers_consumed.fetch_add(1, Ordering::Relaxed);
        }
        buffer
    }
    
    /// Return a consumed buffer to the free pool
    pub fn return_buffer(&self, mut buffer: PrefetchBuffer) {
        buffer.reset();
        
        let mut free = self.free_buffers.lock();
        free.push_back(buffer);
        self.not_full.notify_one();
    }
    
    /// Shutdown the queue
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }
    
    /// Check if shutdown
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }
    
    /// Get current queue depth
    pub fn queue_depth(&self) -> usize {
        self.ready_buffers.lock().len()
    }
    
    /// Get statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            self.stats.buffers_produced.load(Ordering::Relaxed),
            self.stats.buffers_consumed.load(Ordering::Relaxed),
            self.stats.bytes_prefetched.load(Ordering::Relaxed),
        )
    }
}

/// Prefetch pipeline managing async data loading
pub struct PrefetchPipeline {
    config: PrefetchConfig,
    queue: Arc<PrefetchQueue>,
    workers: Vec<JoinHandle<()>>,
    running: AtomicBool,
}

impl PrefetchPipeline {
    /// Create new pipeline
    pub fn new(config: PrefetchConfig) -> Self {
        let queue = Arc::new(PrefetchQueue::new(&config));
        
        Self {
            config,
            queue,
            workers: Vec::new(),
            running: AtomicBool::new(false),
        }
    }
    
    /// Start prefetching with custom data loader function
    pub fn start<F>(&mut self, loader: F)
    where
        F: Fn(&mut PrefetchBuffer) -> bool + Send + Sync + 'static,
    {
        self.running.store(true, Ordering::SeqCst);
        
        let loader = Arc::new(loader);
        
        for worker_id in 0..self.config.num_workers {
            let queue = Arc::clone(&self.queue);
            let loader = Arc::clone(&loader);
            
            let handle = thread::spawn(move || {
                tracing::debug!("Prefetch worker {} started", worker_id);
                
                while !queue.is_shutdown() {
                    if let Some(mut buffer) = queue.get_free_buffer() {
                        // Load data into buffer
                        let success = loader(&mut buffer);
                        
                        if success {
                            queue.submit_buffer(buffer);
                        } else {
                            // End of data or error, return buffer and shutdown
                            queue.return_buffer(buffer);
                            break;
                        }
                    }
                }
                
                tracing::debug!("Prefetch worker {} stopped", worker_id);
            });
            
            self.workers.push(handle);
        }
    }
    
    /// Get next batch of data
    pub fn next(&self) -> Option<PrefetchBuffer> {
        self.queue.get_ready_buffer()
    }
    
    /// Return consumed buffer
    pub fn recycle(&self, buffer: PrefetchBuffer) {
        self.queue.return_buffer(buffer);
    }
    
    /// Stop the pipeline
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.queue.shutdown();
        
        for handle in self.workers.drain(..) {
            let _ = handle.join();
        }
    }
    
    /// Get queue depth
    pub fn queue_depth(&self) -> usize {
        self.queue.queue_depth()
    }
    
    /// Get statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        self.queue.stats()
    }
}

impl Drop for PrefetchPipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    
    #[test]
    fn test_prefetch_buffer() {
        let mut buffer = PrefetchBuffer::new(1024);
        buffer.data.extend_from_slice(b"test data");
        buffer.num_samples = 1;
        buffer.ready = true;
        
        assert!(buffer.ready);
        assert_eq!(buffer.num_samples, 1);
        
        buffer.reset();
        assert!(!buffer.ready);
        assert_eq!(buffer.num_samples, 0);
    }
    
    #[test]
    fn test_prefetch_queue() {
        let config = PrefetchConfig {
            num_buffers: 2,
            buffer_size: 1024,
            ..Default::default()
        };
        
        let queue = PrefetchQueue::new(&config);
        
        // Get free buffer
        let mut buffer = queue.get_free_buffer().unwrap();
        buffer.data.extend_from_slice(b"test");
        buffer.num_samples = 10;
        
        // Submit it
        queue.submit_buffer(buffer);
        
        // Get ready buffer
        let ready = queue.get_ready_buffer().unwrap();
        assert_eq!(ready.num_samples, 10);
        
        // Return it
        queue.return_buffer(ready);
        
        let (produced, consumed, _) = queue.stats();
        assert_eq!(produced, 1);
        assert_eq!(consumed, 1);
    }
    
    #[test]
    fn test_prefetch_pipeline() {
        let config = PrefetchConfig {
            num_buffers: 4,
            buffer_size: 1024,
            num_workers: 1,
            ..Default::default()
        };
        
        let mut pipeline = PrefetchPipeline::new(config);
        
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        
        pipeline.start(move |buffer| {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count >= 5 {
                return false; // Stop after 5 batches
            }
            
            buffer.data.extend_from_slice(&[count as u8; 100]);
            buffer.num_samples = 10;
            true
        });
        
        // Consume some buffers
        for _ in 0..3 {
            if let Some(buffer) = pipeline.next() {
                assert_eq!(buffer.num_samples, 10);
                pipeline.recycle(buffer);
            }
        }
        
        pipeline.stop();
        
        let (produced, consumed, _bytes) = pipeline.stats();
        assert!(produced >= 3);
        assert!(consumed >= 3);
    }
}
