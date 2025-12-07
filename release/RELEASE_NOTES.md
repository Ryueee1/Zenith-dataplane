# Zenith v0.1.1 Release Notes

## 🎉 Zenith v0.1.1 - Phase 1-3 Complete

We're excited to announce Zenith v0.1.1, featuring major improvements across all core components!

### 🚀 Highlights

- **43M+ ops/sec** ring buffer performance (industry-leading)
- **Full io_uring** async I/O implementation
- **Kubernetes-ready** with complete Helm chart
- **Production hardening** with health checks and circuit breaker
- **41 unit tests + 6 integration tests** all passing

---

## ⬇️ Download

| Platform | Architecture | File | Size |
|----------|--------------|------|------|
| Linux | x86_64 | `zenith-v0.1.1-linux-x86_64.tar.gz` | 873 KB |

### Installation

```bash
# Download and extract
tar -xzvf zenith-v0.1.1-linux-x86_64.tar.gz

# Run the scheduler
./zenith-scheduler --help
```

---

## 🆕 What's New

### Phase 1: Core Runtime Enhancements

- ✨ **Prometheus Metrics Export** - `/metrics` and `/health` endpoints
- ✨ **Scheduler REST API** - Full CRUD for jobs
- ✨ **Scheduler gRPC API** - High-performance RPC
- ✨ **Node Agent** - GPU discovery and heartbeats

### Phase 2: Advanced Implementation

- ✨ **io_uring Engine** - Linux kernel async I/O
- ✨ **Memory Pool** - Zero-copy slab allocation
- ✨ **NVML Manager** - GPU monitoring
- ✨ **State Persistence** - Durable job storage

### Phase 3: Production Hardening

- ✨ **Health Checks** - K8s liveness/readiness probes
- ✨ **Circuit Breaker** - Fault tolerance pattern
- ✨ **Helm Chart** - Complete Kubernetes deployment
- ✨ **CI/CD Pipeline** - GitHub Actions workflow
- ✨ **OpenAPI Spec** - REST API documentation

---

## 📈 Performance Benchmarks

| Component | Metric | Result |
|-----------|--------|--------|
| Ring Buffer (SPSC) | Throughput | **43.16 M ops/sec** |
| Memory Pool | 1000 stress iterations | **32.69 ms** |
| Async File I/O | 1 MB read/write | **< 5 ms** |
| Telemetry | 10K events | **191 µs** |
| CPU Thread Pinning | Affinity | **Success** |

---

## 🧪 Testing

```
Unit Tests:       41 passed ✅
Integration Tests: 6 passed ✅
Doc Tests:        2 passed ✅
```

### New Test Coverage

- `pool::tests` - Memory pool operations
- `uring::tests` - io_uring functionality
- `nvml::tests` - GPU management
- `health::tests` - Health check system
- `circuit_breaker::tests` - Fault tolerance

---

## 📦 Installation Methods

### From Binary (Recommended)

```bash
curl -LO https://github.com/vibeswithkk/Zenith-dataplane/releases/download/v0.1.1/zenith-v0.1.1-linux-x86_64.tar.gz
tar -xzvf zenith-v0.1.1-linux-x86_64.tar.gz
./zenith-scheduler
```

### From Source

```bash
git clone https://github.com/vibeswithkk/Zenith-dataplane.git
cd Zenith-dataplane
cargo build --release
```

### Using Helm (Kubernetes)

```bash
helm install zenith ./deploy/helm/zenith --namespace zenith --create-namespace
```

### Using Docker

```bash
docker build -t zenith/scheduler .
docker run -p 8080:8080 -p 50051:50051 zenith/scheduler
```

---

## 📋 Requirements

### Minimum
- Linux kernel 5.1+ (for io_uring)
- 4 CPU cores, 8 GB RAM
- Rust 1.75+ (if building from source)

### Recommended
- 8+ cores, 32 GB RAM
- NVIDIA GPU with driver (for GPU features)

---

## 🔗 Links

- 📚 [Documentation](https://github.com/vibeswithkk/Zenith-dataplane/blob/main/README.md)
- 🗺️ [Roadmap](https://github.com/vibeswithkk/Zenith-dataplane/blob/main/ROADMAP.md)
- 📝 [Changelog](https://github.com/vibeswithkk/Zenith-dataplane/blob/main/CHANGELOG.md)
- 🐛 [Issues](https://github.com/vibeswithkk/Zenith-dataplane/issues)

---

## 🤝 Contributing

We welcome contributions! See our [roadmap](ROADMAP.md) for priority areas.

---

## 💰 Sponsorship

Interested in sponsoring development? See [ROADMAP.md](ROADMAP.md#-sponsorship-opportunities) for opportunities.

---

## 📄 License

Apache License 2.0

---

**Full Changelog**: https://github.com/vibeswithkk/Zenith-dataplane/compare/v0.1.0...v0.1.1
