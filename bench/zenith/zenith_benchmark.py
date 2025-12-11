#!/usr/bin/env python3
"""
Zenith DataLoader Benchmark

Measures performance of Zenith's high-performance data loading.
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

# Add SDK to path
sdk_path = Path(__file__).parents[2] / "sdk-python"
sys.path.insert(0, str(sdk_path))

import numpy as np

try:
    import pyarrow.parquet as pq
    import pyarrow as pa
except ImportError:
    print("ERROR: PyArrow not installed")
    sys.exit(1)


class ZenithBenchmarkRunner:
    """Run and measure Zenith DataLoader performance."""
    
    def __init__(
        self,
        num_workers: int = 4,
        batch_size: int = 32,
    ):
        self.num_workers = num_workers
        self.batch_size = batch_size
        self.latencies: List[float] = []
        
    def benchmark_zenith_engine(
        self,
        dataset_path: str,
        duration_seconds: float = 60.0,
    ) -> Dict:
        """Benchmark Zenith Engine loading."""
        
        print(f"\n[Zenith Engine Benchmark]")
        print(f"  Dataset: {dataset_path}")
        print(f"  Batch Size: {self.batch_size}")
        
        # Try to import Zenith
        try:
            import zenith
            print(f"  Zenith Version: {zenith.__version__}")
            zenith.info()
        except ImportError as e:
            print(f"  Zenith import failed: {e}")
            print("  Falling back to PyArrow-based loading")
            return self.benchmark_pyarrow_direct(dataset_path, duration_seconds)
        
        self.latencies = []
        total_samples = 0
        total_batches = 0
        
        start_time = time.perf_counter()
        end_time = start_time + duration_seconds
        
        # Warmup
        print("  Warming up...")
        try:
            data = zenith.load(dataset_path)
            print(f"  Loaded {data.num_rows:,} rows")
        except Exception as e:
            print(f"  Zenith load failed: {e}")
            return self.benchmark_pyarrow_direct(dataset_path, duration_seconds)
        
        print(f"  Starting benchmark...")
        
        epoch = 0
        while time.perf_counter() < end_time:
            epoch += 1
            batch_count = 0
            
            # Iterate through data in batches
            num_rows = data.num_rows
            for i in range(0, num_rows, self.batch_size):
                batch_start = time.perf_counter()
                
                # Slice batch
                end_idx = min(i + self.batch_size, num_rows)
                batch = data.slice(i, end_idx - i)
                
                # Convert to numpy (simulating tensor creation)
                for col in batch.column_names:
                    if col not in ['blob', 'image_data']:
                        arr = batch.column(col).to_numpy()
                
                batch_end = time.perf_counter()
                batch_latency = (batch_end - batch_start) * 1000  # ms
                
                self.latencies.append(batch_latency)
                total_samples += end_idx - i
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
        results["loader"] = "zenith_engine"
        
        return results
    
    def benchmark_pyarrow_direct(
        self,
        dataset_path: str,
        duration_seconds: float = 60.0,
    ) -> Dict:
        """Benchmark direct PyArrow loading (Zenith core path)."""
        
        print(f"\n[PyArrow Direct Benchmark (Zenith Core Path)]")
        print(f"  Dataset: {dataset_path}")
        print(f"  Batch Size: {self.batch_size}")
        
        self.latencies = []
        total_samples = 0
        total_batches = 0
        
        # Read table once
        table = pq.read_table(dataset_path)
        num_rows = table.num_rows
        print(f"  Dataset Size: {num_rows:,} rows")
        
        start_time = time.perf_counter()
        end_time = start_time + duration_seconds
        
        print(f"  Starting benchmark...")
        
        epoch = 0
        while time.perf_counter() < end_time:
            epoch += 1
            batch_count = 0
            
            for i in range(0, num_rows, self.batch_size):
                batch_start = time.perf_counter()
                
                # Slice batch (zero-copy in Arrow!)
                end_idx = min(i + self.batch_size, num_rows)
                batch = table.slice(i, end_idx - i)
                
                # Convert to numpy arrays (simulating tensor creation)
                for col in batch.column_names:
                    if col not in ['blob', 'image_data']:
                        arr = batch.column(col).to_numpy()
                
                batch_end = time.perf_counter()
                batch_latency = (batch_end - batch_start) * 1000  # ms
                
                self.latencies.append(batch_latency)
                total_samples += end_idx - i
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
        results["loader"] = "pyarrow_direct"
        
        return results
    
    def benchmark_batch_iterator(
        self,
        dataset_path: str,
        duration_seconds: float = 60.0,
    ) -> Dict:
        """Benchmark batch iterator (streaming pattern)."""
        
        print(f"\n[Batch Iterator Benchmark (Streaming)]")
        print(f"  Dataset: {dataset_path}")
        print(f"  Batch Size: {self.batch_size}")
        
        self.latencies = []
        total_samples = 0
        total_batches = 0
        
        pf = pq.ParquetFile(dataset_path)
        print(f"  Dataset Rows: {pf.metadata.num_rows:,}")
        
        start_time = time.perf_counter()
        end_time = start_time + duration_seconds
        
        print(f"  Starting benchmark...")
        
        epoch = 0
        while time.perf_counter() < end_time:
            epoch += 1
            batch_count = 0
            
            for batch in pf.iter_batches(batch_size=self.batch_size):
                batch_start = time.perf_counter()
                
                # Process batch
                for col in batch.schema.names:
                    if col not in ['blob', 'image_data']:
                        arr = batch.column(col).to_numpy()
                
                batch_end = time.perf_counter()
                batch_latency = (batch_end - batch_start) * 1000
                
                self.latencies.append(batch_latency)
                total_samples += batch.num_rows
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
        results["loader"] = "batch_iterator"
        
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
        description="Zenith DataLoader Benchmark"
    )
    parser.add_argument("--dataset", help="Path to dataset file")
    parser.add_argument("--duration", type=float, default=60, help="Duration in seconds")
    parser.add_argument("--workers", type=int, default=4, help="Number of workers")
    parser.add_argument("--batch-size", type=int, default=32, help="Batch size")
    parser.add_argument("--output", help="Output JSON file")
    parser.add_argument(
        "--mode",
        choices=["engine", "direct", "iterator", "all"],
        default="all",
        help="Benchmark mode",
    )
    
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
    runner = ZenithBenchmarkRunner(
        num_workers=args.workers,
        batch_size=args.batch_size,
    )
    
    all_results = {}
    
    if args.mode in ["engine", "all"]:
        results = runner.benchmark_zenith_engine(
            dataset_path,
            duration_seconds=args.duration,
        )
        all_results["zenith_engine"] = results
    
    if args.mode in ["direct", "all"]:
        results = runner.benchmark_pyarrow_direct(
            dataset_path,
            duration_seconds=args.duration,
        )
        all_results["pyarrow_direct"] = results
    
    if args.mode in ["iterator", "all"]:
        results = runner.benchmark_batch_iterator(
            dataset_path,
            duration_seconds=args.duration,
        )
        all_results["batch_iterator"] = results
    
    # Find best result
    best_mode = max(all_results.keys(), key=lambda k: all_results[k]["throughput"])
    best_results = all_results[best_mode]
    
    # Print summary
    print("\n" + "=" * 60)
    print("ZENITH BENCHMARK RESULTS")
    print("=" * 60)
    
    for mode, results in all_results.items():
        print(f"\n{mode}:")
        print(f"  Throughput: {results['throughput']:,.0f} samples/sec")
        print(f"  Latency p50: {results['latency_p50_ms']:.3f} ms")
        print(f"  Latency p99: {results['latency_p99_ms']:.3f} ms")
    
    print("\n" + "=" * 60)
    print(f"BEST: {best_mode} @ {best_results['throughput']:,.0f} samples/sec")
    print("=" * 60)
    
    # Prepare output
    output_data = {
        "benchmark": "zenith",
        "dataset": dataset_path,
        "modes": all_results,
        "best_mode": best_mode,
        **best_results,
    }
    
    # Save results
    if args.output:
        with open(args.output, "w") as f:
            json.dump(output_data, f, indent=2)
        print(f"\nResults saved to: {args.output}")
    
    return output_data


if __name__ == "__main__":
    main()
