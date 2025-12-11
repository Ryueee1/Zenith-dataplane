# Zenith Data Plane: Research & Analysis Report

**Author:** Wahyu Ardiansyah  
**Date:** December 7, 2025  
**Project:** Zenith-dataplane  
**Classification:** Internal Technical Architecture

---

## 1. Executive Summary

This document outlines the technical research and architectural analysis for "Zenith," a high-performance, low-latency data plane designed for financial (HFT) and telemetry workloads. The analysis confirms the feasibility of the proposed blueprint, specifically the combination of **Rust** for the core engine, **Apache Arrow** for zero-copy memory layouts, and **Wasmtime** for secure, sandboxed user-defined logic.

## 2. Technical Feasibility Analysis

### 2.1 Core Architecture: The "Zero-Copy" Promise
The central requirement of Zenith is to minimize latency (< 10 µs in-memory path). Our research validates that traditional serialization (Protobuf/JSON) creates unacceptable overhead. The solution is to use **Apache Arrow** IPC format in shared memory.

*   **Findings:** Apache Arrow provides a standardized columnar memory layout. By mapping this layout directly into a shared memory region (via `mmap` or shared heap), producer and consumer processes (potentially in different languages like Python and Rust) can read the same data bytes without CPU-intensive copying or parsing.
*   **Rust Integration:** The `arrow-rs` crate (v53+) offers mature support for "Zero-Copy" slicing. We can expose pointers to Arrow `RecordBatch` structures directly to consumers.

### 2.2 In-Memory Ring Buffer: The Disruptor Pattern
To handle high throughput without Garbage Collection (GC) pauses (a common issue in Java/Go), we utilize the LMAX Disruptor pattern.

*   **Mechanism:** A pre-allocated ring buffer where producers claim slots using atomic sequence numbers (CAS). This eliminates lock contention.
*   **Implementation:** In Rust, we will implement a multi-producer single-consumer (MPSC) or multi-producer multi-consumer (MPMC) ring buffer using `AtomicU64` for cursors, ensuring thread safety without Mutexes.

### 2.3 Extension Mechanism: WebAssembly (WASM)
The blueprint calls for "safe user-defined functions." Native plugins (.so/.dll) are unsafe (segfaults/security risks).

*   **Selected Technology:** **Wasmtime**. It is a Bytecode Alliance project, highly optimized for Rust.
*   **Component Model:** We will utilize the Wasm Component Model (WASI Preview 2) to define typed interfaces (WIT) for the plugins. This allows plugins to receive "views" of Arrow data.
*   **Performance:** JIT compilation in Wasmtime ensures near-native performance. The primary cost is the boundary crossing, which we minimize by passing pointers/indexes rather than copying large buffers.

### 2.4 Networking: Optional DPDK
For "Enterprise" grade, bypassing the Linux kernel network stack is necessary for microsecond-level packet processing.

*   **Strategy:** We will design the `NetworkSource` trait. The default implementation will use `tokio` (epoll) for ease of use. The high-performance implementation will bind to DPDK PMD (Poll Mode Drivers).

## 3. Architecture Blueprint Validation

The proposed architecture in the user prompt is sound and aligns with modern high-performance systems (e.g., Redpanda, ScyllaDB patterns).

**Critical Path to Success:**
1.  **Memory Management:** Strict ownership management in Rust to prevent "use-after-free" when multiple consumers (or WASM) access shared Arrow buffers.
2.  **FFI Boundary:** Ensuring Python/Go SDKs strictly adhere to the Arrow C Data Interface to prevent segmentation faults.

## 4. Implementation Roadmap (Adjusted)

Based on this analysis, the implementation will proceed as follows:
1.  **Core:** Rust crate with `arrow`, `wasmtime`, and a custom RingBuffer.
2.  **SDKs:** Python `pyo3` binding wrapping the core logic.
3.  **Verification:** Micro-benchmarks validating the "zero-copy" claim (throughput vs. payload size).

---
*End of Verification Report by Wahyu Ardiansyah*
