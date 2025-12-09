#!/usr/bin/env python3
"""
PyTorch DataLoader Baseline Benchmark

Measures performance of standard PyTorch DataLoader for comparison.
Author: Wahyu Ardiansyah
"""

import os
import sys
import json
import time
import argparse
from pathlib import Path
from typing import Dict, List, Optional
import statistics

import numpy as np

try:
    import torch
    from torch.utils.data import Dataset, DataLoader, IterableDataset
except ImportError:
    print("ERROR: PyTorch not installed")
    sys.exit(1)

try:
    import pyarrow.parquet as pq
    import pyarrow as pa
except ImportError:
    print("ERROR: PyArrow not installed")
    sys.exit(1)


class ParquetDataset(Dataset):
    """Map-style dataset for Parquet files."""
    
    def __init__(self, path: str):
        self.table = pq.read_table(path)
        self.num_rows = self.table.num_rows
        
    def __len__(self):
        return self.num_rows
    
    def __getitem__(self, idx):
        row = {
            col: self.table.column(col)[idx].as_py()
            for col in self.table.column_names
            if col not in ['blob', 'image_data']  # Skip binary for speed
        }
        
        # Convert to tensor
        features = []
        label = row.get('label', 0)
        
        for key, val in row.items():
            if key not in ['id', 'label'] and isinstance(val, (int, float)):
                features.append(float(val))
        
        if features:
            x = torch.tensor(features, dtype=torch.float32)
        else:
            x = torch.zeros(10)
        
        y = torch.tensor(label, dtype=torch.long)
        
        return x, y


class ParquetIterableDataset(IterableDataset):
    """Iterable dataset for streaming Parquet files."""
    
    def __init__(self, path: str, batch_size: int = 1000):
        self.path = path
        self.batch_size = batch_size
        
    def __iter__(self):
        pf = pq.ParquetFile(self.path)
        
        for batch in pf.iter_batches(batch_size=self.batch_size):
            for i in range(batch.num_rows):
                row = {
                    col: batch.column(col)[i].as_py()
                    for col in batch.schema.names
                    if col not in ['blob', 'image_data']
                }
                
                features = []
                label = row.get('label', 0)
                
                for key, val in row.items():
                    if key not in ['id', 'label'] and isinstance(val, (int, float)):
                        features.append(float(val))
                
                if features:
                    x = torch.tensor(features, dtype=torch.float32)
                else:
                    x = torch.zeros(10)
                
                y = torch.tensor(label, dtype=torch.long)
                
                yield x, y


class BenchmarkRunner:
    """Run and measure DataLoader performance."""
    
    def __init__(
        self,
        num_workers: int = 4,
        batch_size: int = 32,
        pin_memory: bool = True,
        prefetch_factor: int = 2,
    ):
        self.num_workers = num_workers
        self.batch_size = batch_size
        self.pin_memory = pin_memory
        self.prefetch_factor = prefetch_factor
        self.latencies: List[float] = []
        
    def benchmark_map_style(
        self,
        dataset_path: str,
        duration_seconds: float = 60.0,
    ) -> Dict:
        """Benchmark map-style dataset loading."""
        
        print(f"\n[Map-Style Benchmark]")
        print(f"  Dataset: {dataset_path}")
        print(f"  Workers: {self.num_workers}")
        print(f"  Batch Size: {self.batch_size}")
        
        dataset = ParquetDataset(dataset_path)
        print(f"  Dataset Size: {len(dataset):,} samples")
        
        loader = DataLoader(
            dataset,
            batch_size=self.batch_size,
            shuffle=True,
            num_workers=self.num_workers,
            pin_memory=self.pin_memory,
            prefetch_factor=self.prefetch_factor if self.num_workers > 0 else None,
            persistent_workers=self.num_workers > 0,
        )
        
        return self._run_benchmark(loader, duration_seconds)
    
    def benchmark_iterable_style(
        self,
        dataset_path: str,
        duration_seconds: float = 60.0,
    ) -> Dict:
        """Benchmark iterable-style dataset loading."""
        
        print(f"\n[Iterable-Style Benchmark]")
        print(f"  Dataset: {dataset_path}")
        
        dataset = ParquetIterableDataset(dataset_path, batch_size=1000)
        
        loader = DataLoader(
            dataset,
            batch_size=self.batch_size,
            num_workers=self.num_workers,
            pin_memory=self.pin_memory,
        )
        
        return self._run_benchmark(loader, duration_seconds)
    
    def _run_benchmark(
        self,
        loader: DataLoader,
        duration_seconds: float,
    ) -> Dict:
        """Core benchmark loop."""
        
        self.latencies = []
        total_samples = 0
        total_batches = 0
        
        start_time = time.perf_counter()
        end_time = start_time + duration_seconds
        
        # Warmup (3 batches)
        warmup_count = 0
        for batch in loader:
            warmup_count += 1
            if warmup_count >= 3:
                break
        
        print(f"  Warmup complete, starting benchmark...")
        
        # Main benchmark loop
        epoch = 0
        while time.perf_counter() < end_time:
            epoch += 1
            batch_count = 0
            
            for batch in loader:
                batch_start = time.perf_counter()
                
                if isinstance(batch, (tuple, list)):
                    x, y = batch
                    batch_samples = x.shape[0]
                else:
                    batch_samples = self.batch_size
                
                # Simulate minimal processing (move to device if GPU)
                if torch.cuda.is_available():
                    if isinstance(batch, (tuple, list)):
                        x = x.cuda(non_blocking=True)
                        y = y.cuda(non_blocking=True)
                
                batch_end = time.perf_counter()
                batch_latency = (batch_end - batch_start) * 1000  # ms
                
                self.latencies.append(batch_latency)
                total_samples += batch_samples
                total_batches += 1
                batch_count += 1
                
                if time.perf_counter() >= end_time:
                    break
            
            print(f"\r  Epoch {epoch}: {batch_count} batches, {total_samples:,} samples total", end="")
        
        actual_duration = time.perf_counter() - start_time
        
        print(f"\n  Benchmark complete!")
        
        results = self._compute_statistics(total_samples, actual_duration)
        results["epochs"] = epoch
        results["total_batches"] = total_batches
        
        return results
    
    def _compute_statistics(self, total_samples: int, duration: float) -> Dict:
        """Compute benchmark statistics."""
        
        throughput = total_samples / duration
        
        latencies = self.latencies
        if not latencies:
            latencies = [0]
        
        sorted_latencies = sorted(latencies)
        n = len(sorted_latencies)
        
        return {
            "throughput": throughput,
            "total_samples": total_samples,
            "duration_seconds": duration,
            "latency_mean_ms": statistics.mean(latencies),
            "latency_std_ms": statistics.stdev(latencies) if len(latencies) > 1 else 0,
            "latency_min_ms": min(latencies),
            "latency_max_ms": max(latencies),
            "latency_p50_ms": sorted_latencies[int(n * 0.50)],
            "latency_p95_ms": sorted_latencies[int(n * 0.95)],
            "latency_p99_ms": sorted_latencies[int(n * 0.99)] if n >= 100 else sorted_latencies[-1],
            "num_batches": len(latencies),
            "batch_size": self.batch_size,
            "num_workers": self.num_workers,
        }


def find_dataset(data_dir: str) -> Optional[str]:
    """Find a parquet dataset in the data directory."""
    data_path = Path(data_dir)
    
    for pattern in ["*.parquet", "benchmark_*.parquet"]:
        files = list(data_path.glob(pattern))
        if files:
            return str(files[0])
    
    return None


def main():
    parser = argparse.ArgumentParser(
        description="PyTorch DataLoader Baseline Benchmark"
    )
    parser.add_argument("--dataset", help="Path to dataset file")
    parser.add_argument("--duration", type=float, default=60, help="Duration in seconds")
    parser.add_argument("--workers", type=int, default=4, help="Number of workers")
    parser.add_argument("--batch-size", type=int, default=32, help="Batch size")
    parser.add_argument("--output", help="Output JSON file")
    
    args = parser.parse_args()
    
    # Find dataset
    dataset_path = args.dataset
    if not dataset_path:
        data_dir = os.environ.get("ZENITH_BENCHMARK_DATA", "./bench/data")
        dataset_path = find_dataset(data_dir)
    
    if not dataset_path or not Path(dataset_path).exists():
        print(f"ERROR: Dataset not found. Run generate_datasets.py first.")
        sys.exit(1)
    
    # Run benchmark
    runner = BenchmarkRunner(
        num_workers=args.workers,
        batch_size=args.batch_size,
    )
    
    results = runner.benchmark_map_style(
        dataset_path,
        duration_seconds=args.duration,
    )
    
    # Add metadata
    results["benchmark"] = "pytorch_dataloader"
    results["dataset"] = dataset_path
    results["pytorch_version"] = torch.__version__
    results["cuda_available"] = torch.cuda.is_available()
    
    # Print summary
    print("\n" + "=" * 50)
    print("RESULTS SUMMARY")
    print("=" * 50)
    print(f"Throughput: {results['throughput']:,.0f} samples/sec")
    print(f"Latency p50: {results['latency_p50_ms']:.2f} ms")
    print(f"Latency p99: {results['latency_p99_ms']:.2f} ms")
    print("=" * 50)
    
    # Save results
    if args.output:
        with open(args.output, "w") as f:
            json.dump(results, f, indent=2)
        print(f"\nResults saved to: {args.output}")
    
    return results


if __name__ == "__main__":
    main()
