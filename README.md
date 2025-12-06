# Zenith Data Plane: High-Performance Real-Time Engine

[![Zenith CI](https://github.com/vibeswithkk/Zenith-dataplane/actions/workflows/ci.yml/badge.svg)](https://github.com/vibeswithkk/Zenith-dataplane/actions)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**Zenith** is a next-generation data plane designed for ultra-low latency (< 100µs) workloads. It bridges the gap between high-level prototyping (Python/Go) and bare-metal performance (Rust).

## 🚀 Key Features

*   **Zero-Copy Architecture**: Uses [Apache Arrow](https://arrow.apache.org/) in shared memory to eliminate serialization overhead.
*   **Multi-Language Support**: First-class SDKs for **Python** and **Go**.
*   **Secure Plugin System**: Run user-defined logic safely using **WebAssembly (WASM)** sandboxing (via Wasmtime).
*   **High Throughput**: Validated at **6,000,000 events/second** on a single node.
*   **Lock-Free Design**: Utilizes Ring Buffer patterns to minimize contention and GC pauses.

## 📦 Components

| Component | Description | Status |
| :--- | :--- | :--- |
| **Core Engine** | Rust-based runtime managing ring buffers and plugins. | ✅ Stable (MVP) |
| **Runtime Lib** | Application runner with Plugin Hot-Reloading. | ✅ Beta |
| **CLI Tool** | Standalone daemon to run Zenith services. | ✅ Stable |
| **Dashboard** | Web-based monitoring UI. | ✅ Beta |
| **Python SDK** | `pyarrow` compatible bindings for zero-copy publish. | ✅ Stable |
| **Go SDK** | CGO bindings for high-performance ingestion. | 🚧 Beta |
| **WASM Host** | Secure runtime for basic filter/transform plugins. | ✅ Stable |

## 🛠️ Installation

### Prerequisites
*   Rust 1.75+
*   Python 3.10+
*   Go 1.20+ (Optional)

### Building form Source

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/vibeswithkk/Zenith-dataplane.git
    cd Zenith-dataplane
    ```

2.  **Build the Core Engine**:
    ```bash
    cd core
    cargo build --release
    ```
    This produces `libzenith_core.so` (or `.dylib` on macOS).

3.  **Build WASM Plugins**:
    ```bash
    rustup target add wasm32-wasip1
    cd plugins/simple_filter
    cargo build --target wasm32-wasip1 --release
    ```

## ⚡ Quick Start (Python)

Run the included demo application to see Zenith in action:

```bash
# Setup Environment
python3 -m venv .venv
source .venv/bin/activate
pip install pyarrow numpy

# Run Demo (Publishes events -> Core -> WASM Filter)
python3 demo_app.py
```

**Expected Output:**
```
Zenith Core Engine: Consumer thread started.
Initializing Zenith Client...
Loaded plugin: ./filter.wasm (583 bytes)
Publishing 10 batches...
Done. 10 batches published in 2.62ms
```

## 📖 Documentation

*   [**Architecture Overview**](docs/ARCHITECTURE.md)
*   [**Research & Analysis**](docs/RESEARCH_ANALYSIS.md)
*   [**API Specification**](docs/API_SPEC.md)
*   [**Benchmark Report**](docs/BENCHMARK_REPORT.md)

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to submit Pull Requests.

## 📄 License

This project is licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) for details.

---
*Built with ❤️ by the Zenith Team.*
