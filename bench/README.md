# Zenith Benchmark Suite

> **Reproducible Performance Benchmarks for ML Data Loading**
> 
> Author: Wahyu Ardiansyah
> Last Updated: 2024-12-09

---

## Overview

This benchmark suite provides reproducible performance measurements comparing Zenith against industry-standard data loaders:

- **PyTorch DataLoader** (baseline)
- **NVIDIA DALI** (GPU-accelerated)
- **WebDataset** (large-scale streaming)
- **Zenith** (our implementation)

All benchmarks are designed to be reproducible across different hardware configurations.

---

## Quick Start

```bash
# 1. Setup environment
./setup_benchmark.sh

# 2. Generate synthetic datasets
python generate_datasets.py

# 3. Run all benchmarks
./run_benchmarks.sh

# 4. View results
cat reports/benchmark_report.md
```

---

## Hardware Requirements

### Minimum
- CPU: 8 cores
- RAM: 32GB
- Storage: 100GB SSD
- GPU: NVIDIA GPU with CUDA 11.8+

### Recommended
- CPU: 32+ cores
- RAM: 128GB
- Storage: 500GB NVMe SSD
- GPU: NVIDIA A100/H100

---

## Benchmark Workloads

### Workload 1: ImageNet-like (Small Files)
- **Description**: Many small JPEG files (~100KB each)
- **Purpose**: Tests I/O overhead and file opening latency
- **Size**: 1M images, ~100GB total

### Workload 2: Parquet/Columnar (Large Blocks)
- **Description**: Large columnar files with binary blobs
- **Purpose**: Tests block reads and zero-copy efficiency
- **Size**: 10GB per file, 100GB total

### Workload 3: WebDataset TAR Shards
- **Description**: Sharded TAR archives for streaming
- **Purpose**: Tests sequential streaming and prefetch
- **Size**: 1000 shards x 100MB = 100GB

---

## Metrics Collected

| Metric | Description | Unit |
|--------|-------------|------|
| `throughput` | End-to-end samples/sec | samples/s |
| `latency_mean` | Average batch load time | ms |
| `latency_p50` | Median batch load time | ms |
| `latency_p95` | 95th percentile latency | ms |
| `latency_p99` | 99th percentile latency | ms |
| `gpu_util` | GPU utilization during training | % |
| `cpu_util` | CPU utilization | % |
| `memory_peak` | Peak memory usage | GB |
| `io_bandwidth` | Disk I/O throughput | MB/s |

---

## Directory Structure

```
bench/
├── README.md              # This file
├── setup_benchmark.sh     # Environment setup script
├── run_benchmarks.sh      # Main benchmark runner
├── generate_datasets.py   # Synthetic dataset generator
├── configs/
│   ├── imagenet.yaml     # ImageNet workload config
│   ├── parquet.yaml      # Parquet workload config
│   └── webdataset.yaml   # WebDataset workload config
├── baselines/
│   ├── pytorch_baseline.py
│   ├── dali_baseline.py
│   └── webdataset_baseline.py
├── zenith/
│   └── zenith_benchmark.py
├── reports/
│   └── .gitkeep
└── results/
    └── .gitkeep
```

---

## Running Benchmarks

### Individual Benchmark

```bash
# PyTorch baseline
python baselines/pytorch_baseline.py --config configs/imagenet.yaml

# DALI baseline (requires NVIDIA GPU)
python baselines/dali_baseline.py --config configs/imagenet.yaml

# Zenith
python zenith/zenith_benchmark.py --config configs/imagenet.yaml
```

### Full Suite

```bash
./run_benchmarks.sh --all
```

### Custom Configuration

```bash
./run_benchmarks.sh --workers 8 --batch-size 64 --duration 300
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ZENITH_BENCHMARK_DATA` | Dataset directory | `./data` |
| `ZENITH_BENCHMARK_RESULTS` | Results directory | `./results` |
| `ZENITH_NUM_WORKERS` | Number of workers | `4` |
| `ZENITH_BATCH_SIZE` | Batch size | `32` |

---

## Reproducibility

All benchmarks include:

1. **Hardware specification logging** (CPU, GPU, RAM, storage)
2. **Software version logging** (Python, PyTorch, CUDA, etc.)
3. **Configuration snapshots** (YAML configs saved with results)
4. **Raw measurement logs** (CSV files with all data points)
5. **Statistical summaries** (mean, std, percentiles)

Results are saved in `results/<timestamp>/` with full provenance.

---

## Success Criteria (MVP v0.1)

Zenith must demonstrate:

- [x] ≥20% throughput improvement OR
- [x] ≥20% GPU idle time reduction

...in at least one canonical workload compared to PyTorch DataLoader.

---

## References

- [NVIDIA DALI Documentation](https://docs.nvidia.com/deeplearning/dali/)
- [WebDataset GitHub](https://github.com/webdataset/webdataset)
- [PyTorch DataLoader Best Practices](https://pytorch.org/tutorials/)
- [Arrow IPC Specification](https://arrow.apache.org/docs/format/IPC.html)

---

## Contact

For questions or issues, please open a GitHub issue or contact:
- **Author**: Wahyu Ardiansyah
- **Repository**: https://github.com/vibeswithkk/Zenith-dataplane
