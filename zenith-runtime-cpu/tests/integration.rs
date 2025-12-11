//! Integration Tests - Real World Validation
//!
//! These tests validate that components work with real I/O and system resources.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Test 1: Ring Buffer - Real concurrent throughput
#[test]
fn integration_ring_buffer_throughput() {
    use zenith_runtime_cpu::buffer::{SpscRingBuffer, RingBuffer};
    
    let buffer = Arc::new(SpscRingBuffer::<u64>::new(65536));
    let buffer_producer = Arc::clone(&buffer);
    let buffer_consumer = Arc::clone(&buffer);
    
    const ITEMS: u64 = 1_000_000;
    
    let start = Instant::now();
    
    // Producer thread
    let producer = thread::spawn(move || {
        for i in 0..ITEMS {
            while buffer_producer.try_push(i).is_err() {
                std::hint::spin_loop();
            }
        }
    });
    
    // Consumer thread
    let consumer = thread::spawn(move || {
        let mut received = 0u64;
        let mut sum = 0u64;
        
        while received < ITEMS {
            if let Some(value) = buffer_consumer.try_pop() {
                sum += value;
                received += 1;
            } else {
                std::hint::spin_loop();
            }
        }
        
        sum
    });
    
    producer.join().unwrap();
    let sum = consumer.join().unwrap();
    
    let elapsed = start.elapsed();
    let ops_per_sec = ITEMS as f64 / elapsed.as_secs_f64();
    
    // Verify correctness
    let expected = ITEMS * (ITEMS - 1) / 2;
    assert_eq!(sum, expected, "Data integrity check failed");
    
    // Performance check (should be > 1M ops/sec)
    println!("[RING BUFFER] {} items in {:?}", ITEMS, elapsed);
    println!("[RING BUFFER] Throughput: {:.2} M ops/sec", ops_per_sec / 1_000_000.0);
    assert!(ops_per_sec > 1_000_000.0, "Performance below threshold");
}

/// Test 2: Memory Pool - Real allocation patterns
#[test]
fn integration_memory_pool_stress() {
    use zenith_runtime_cpu::pool::{MemoryPool, PoolConfig};
    
    let config = PoolConfig {
        slab_size: 4096,
        initial_slabs: 16,
        max_slabs: 256,
        alignment: 64,
    };
    
    let pool = MemoryPool::new(config).unwrap();
    
    let start = Instant::now();
    
    // Stress test: allocate, use, deallocate
    for iteration in 0..1000 {
        let mut buffers = Vec::new();
        
        // Allocate multiple buffers
        for _ in 0..16 {
            if let Some(mut buf) = pool.allocate() {
                // Write pattern
                let pattern = (iteration % 256) as u8;
                buf.as_mut_slice().fill(pattern);
                buffers.push(buf);
            }
        }
        
        // Verify pattern
        for buf in &buffers {
            let pattern = (iteration % 256) as u8;
            assert!(buf.as_slice().iter().all(|&b| b == pattern));
        }
        
        // Deallocate
        for buf in buffers {
            pool.deallocate(buf);
        }
    }
    
    let elapsed = start.elapsed();
    let stats = pool.stats();
    
    println!("[MEMORY POOL] 1000 iterations in {:?}", elapsed);
    println!("[MEMORY POOL] High water mark: {}", stats.high_water_mark);
    println!("[MEMORY POOL] Total memory: {} KB", stats.total_memory / 1024);
    
    assert_eq!(pool.allocated_count(), 0, "Memory leak detected");
}

/// Test 3: NUMA Topology Discovery - Real system info
#[test]
fn integration_numa_discovery() {
    use zenith_runtime_cpu::numa::NumaTopology;
    
    let topology = NumaTopology::discover().expect("NUMA discovery failed");
    
    let total_mem = topology.total_memory();
    let num_nodes = topology.num_nodes();
    let num_cpus = topology.num_cpus();
    
    println!("[NUMA] Discovered {} NUMA nodes", num_nodes);
    println!("[NUMA] Total CPUs: {}", num_cpus);
    println!("[NUMA] Total physical memory: {} GB", total_mem / (1024 * 1024 * 1024));
    
    for node in topology.nodes() {
        println!("[NUMA] Node {}: {} CPUs, {} GB memory", 
            node.node_id, 
            node.cpu_cores.len(),
            node.total_memory / (1024 * 1024 * 1024)
        );
    }
    
    // Should detect at least 1 NUMA node and CPUs
    assert!(num_nodes >= 1, "No NUMA nodes detected");
    assert!(num_cpus > 0, "No CPUs detected");
    // Memory may be 0 on some virtualized systems, so we just log it
}

/// Test 4: Telemetry - Real metrics collection
#[test]
fn integration_telemetry_metrics() {
    use zenith_runtime_cpu::telemetry::TelemetryCollector;
    use std::thread::sleep;
    
    let collector = TelemetryCollector::new(100);
    collector.start();
    
    // Simulate workload
    let start = Instant::now();
    for _ in 0..10000 {
        collector.record_event(1024);
        collector.record_latency(50);
    }
    let elapsed = start.elapsed();
    
    // Wait a bit for metrics to settle
    sleep(Duration::from_millis(10));
    
    let snapshot = collector.snapshot();
    
    println!("[TELEMETRY] Events: {}", snapshot.events_processed);
    println!("[TELEMETRY] Bytes: {} KB", snapshot.bytes_processed / 1024);
    println!("[TELEMETRY] Avg latency: {} µs", snapshot.avg_latency_us);
    println!("[TELEMETRY] Recording took: {:?}", elapsed);
    
    assert_eq!(snapshot.events_processed, 10000);
    assert_eq!(snapshot.bytes_processed, 10000 * 1024);
    assert_eq!(snapshot.avg_latency_us, 50);
    
    collector.stop();
}

/// Test 5: File I/O - Real file operations
#[test]
fn integration_file_io() {
    use zenith_runtime_cpu::io::standard::{AsyncFileReader, AsyncFileWriter};
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_data.bin");
    
    // Create test data
    let data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
    
    // Write using standard async I/O
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        // Write
        let mut writer = AsyncFileWriter::create(test_file.to_str().unwrap()).await.unwrap();
        writer.write_all(&data).await.unwrap();
        writer.flush().await.unwrap();
        
        // Read back
        let mut reader = AsyncFileReader::open(test_file.to_str().unwrap()).await.unwrap();
        let read_data = reader.read_all().await.unwrap();
        
        assert_eq!(read_data.len(), data.len());
        assert_eq!(read_data, data);
        
        println!("[FILE I/O] Wrote and read {} KB successfully", data.len() / 1024);
    });
}

/// Test 6: Thread Pinning - Real CPU affinity
#[test]
fn integration_thread_affinity() {
    use zenith_runtime_cpu::thread::available_cores;
    
    let num_cores = available_cores();
    
    println!("[CPU] Available cores: {}", num_cores);
    assert!(num_cores > 0, "No CPU cores detected");
    
    // Try to get core IDs and pin
    if let Some(core_ids) = core_affinity::get_core_ids() {
        println!("[CPU] Core IDs: {:?}", core_ids);
        if let Some(first_core) = core_ids.first() {
            match core_affinity::set_for_current(*first_core) {
                true => println!("[CPU] Successfully pinned to core {:?}", first_core),
                false => println!("[CPU] Could not pin (may need privileges)"),
            }
        }
    }
}
