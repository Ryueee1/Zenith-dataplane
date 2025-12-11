#!/usr/bin/env python3
"""
Zenith Benchmark Dataset Generator

Generates synthetic datasets for reproducible benchmarking.
Author: Wahyu Ardiansyah
"""

import os
import sys
import json
import argparse
import time
from pathlib import Path
from typing import Optional
import numpy as np

# Try imports
try:
    import pyarrow as pa
    import pyarrow.parquet as pq
except ImportError:
    print("ERROR: pyarrow not installed. Run: pip install pyarrow")
    sys.exit(1)


class DatasetGenerator:
    """Generate synthetic datasets for benchmarking."""
    
    def __init__(self, output_dir: str = "./bench/data"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
    def generate_parquet_dataset(
        self,
        name: str = "parquet_benchmark",
        num_rows: int = 1_000_000,
        num_columns: int = 10,
        include_binary: bool = True,
        binary_size: int = 1024,
    ) -> Path:
        """
        Generate a Parquet dataset for benchmarking.
        
        Args:
            name: Dataset name
            num_rows: Number of rows
            num_columns: Number of numeric columns
            include_binary: Include binary blob column
            binary_size: Size of binary blobs in bytes
            
        Returns:
            Path to generated dataset
        """
        print(f"\n[Parquet Generator] Creating {name}...")
        print(f"  Rows: {num_rows:,}")
        print(f"  Columns: {num_columns}")
        
        output_path = self.output_dir / f"{name}.parquet"
        
        # Generate in batches to manage memory
        batch_size = 100_000
        num_batches = (num_rows + batch_size - 1) // batch_size
        
        writer = None
        rows_written = 0
        
        start_time = time.time()
        
        for batch_idx in range(num_batches):
            current_batch_size = min(batch_size, num_rows - rows_written)
            
            # Generate numeric columns
            data = {
                f"col_{i}": np.random.randn(current_batch_size).astype(np.float32)
                for i in range(num_columns)
            }
            
            # Add ID column
            data["id"] = np.arange(rows_written, rows_written + current_batch_size)
            
            # Add label column
            data["label"] = np.random.randint(0, 1000, current_batch_size)
            
            # Add binary blob if requested
            if include_binary:
                data["blob"] = [
                    np.random.bytes(binary_size) 
                    for _ in range(current_batch_size)
                ]
            
            table = pa.table(data)
            
            if writer is None:
                writer = pq.ParquetWriter(output_path, table.schema)
            
            writer.write_table(table)
            rows_written += current_batch_size
            
            # Progress
            progress = (batch_idx + 1) / num_batches * 100
            print(f"\r  Progress: {progress:.1f}% ({rows_written:,} rows)", end="")
        
        if writer:
            writer.close()
        
        elapsed = time.time() - start_time
        file_size = output_path.stat().st_size / (1024 * 1024)  # MB
        
        print(f"\n  Generated: {output_path}")
        print(f"  Size: {file_size:.2f} MB")
        print(f"  Time: {elapsed:.2f}s")
        print(f"  Throughput: {num_rows / elapsed:,.0f} rows/sec")
        
        return output_path
    
    def generate_arrow_ipc_dataset(
        self,
        name: str = "arrow_benchmark",
        num_rows: int = 1_000_000,
        num_columns: int = 10,
    ) -> Path:
        """Generate an Arrow IPC dataset for zero-copy benchmarking."""
        print(f"\n[Arrow IPC Generator] Creating {name}...")
        
        output_path = self.output_dir / f"{name}.arrow"
        
        # Generate data
        data = {
            f"col_{i}": np.random.randn(num_rows).astype(np.float32)
            for i in range(num_columns)
        }
        data["id"] = np.arange(num_rows)
        data["label"] = np.random.randint(0, 1000, num_rows)
        
        table = pa.table(data)
        
        start_time = time.time()
        
        with pa.ipc.new_file(str(output_path), table.schema) as writer:
            # Write in batches
            batch_size = 100_000
            for i in range(0, num_rows, batch_size):
                end = min(i + batch_size, num_rows)
                batch = table.slice(i, end - i)
                writer.write_batch(batch.to_batches()[0])
        
        elapsed = time.time() - start_time
        file_size = output_path.stat().st_size / (1024 * 1024)
        
        print(f"  Generated: {output_path}")
        print(f"  Size: {file_size:.2f} MB")
        print(f"  Time: {elapsed:.2f}s")
        
        return output_path
    
    def generate_csv_dataset(
        self,
        name: str = "csv_benchmark",
        num_rows: int = 100_000,
        num_columns: int = 10,
    ) -> Path:
        """Generate a CSV dataset for comparison."""
        print(f"\n[CSV Generator] Creating {name}...")
        
        output_path = self.output_dir / f"{name}.csv"
        
        # Use pandas for CSV writing
        try:
            import pandas as pd
        except ImportError:
            print("  Skipping CSV (pandas not installed)")
            return None
        
        data = {
            f"col_{i}": np.random.randn(num_rows).astype(np.float32)
            for i in range(num_columns)
        }
        data["id"] = np.arange(num_rows)
        data["label"] = np.random.randint(0, 1000, num_rows)
        
        df = pd.DataFrame(data)
        
        start_time = time.time()
        df.to_csv(output_path, index=False)
        elapsed = time.time() - start_time
        
        file_size = output_path.stat().st_size / (1024 * 1024)
        print(f"  Generated: {output_path}")
        print(f"  Size: {file_size:.2f} MB")
        print(f"  Time: {elapsed:.2f}s")
        
        return output_path
    
    def generate_image_like_dataset(
        self,
        name: str = "imagenet_synthetic",
        num_samples: int = 10_000,
        image_size: tuple = (224, 224, 3),
    ) -> Path:
        """Generate synthetic image-like data stored as Parquet."""
        print(f"\n[Image-like Generator] Creating {name}...")
        print(f"  Samples: {num_samples:,}")
        print(f"  Image size: {image_size}")
        
        output_path = self.output_dir / f"{name}.parquet"
        
        batch_size = 1000
        num_batches = (num_samples + batch_size - 1) // batch_size
        
        writer = None
        samples_written = 0
        
        start_time = time.time()
        
        for batch_idx in range(num_batches):
            current_batch_size = min(batch_size, num_samples - samples_written)
            
            # Generate image-like tensors (flattened)
            flat_size = image_size[0] * image_size[1] * image_size[2]
            images = [
                np.random.randint(0, 256, flat_size, dtype=np.uint8).tobytes()
                for _ in range(current_batch_size)
            ]
            
            data = {
                "id": np.arange(samples_written, samples_written + current_batch_size),
                "image_data": images,
                "label": np.random.randint(0, 1000, current_batch_size),
                "height": [image_size[0]] * current_batch_size,
                "width": [image_size[1]] * current_batch_size,
                "channels": [image_size[2]] * current_batch_size,
            }
            
            table = pa.table(data)
            
            if writer is None:
                writer = pq.ParquetWriter(output_path, table.schema)
            
            writer.write_table(table)
            samples_written += current_batch_size
            
            progress = (batch_idx + 1) / num_batches * 100
            print(f"\r  Progress: {progress:.1f}%", end="")
        
        if writer:
            writer.close()
        
        elapsed = time.time() - start_time
        file_size = output_path.stat().st_size / (1024 * 1024)
        
        print(f"\n  Generated: {output_path}")
        print(f"  Size: {file_size:.2f} MB")
        print(f"  Time: {elapsed:.2f}s")
        
        return output_path
    
    def generate_all(self, scale: str = "small"):
        """Generate all benchmark datasets."""
        
        scales = {
            "tiny": {
                "parquet_rows": 10_000,
                "arrow_rows": 10_000,
                "csv_rows": 5_000,
                "image_samples": 1_000,
            },
            "small": {
                "parquet_rows": 100_000,
                "arrow_rows": 100_000,
                "csv_rows": 50_000,
                "image_samples": 10_000,
            },
            "medium": {
                "parquet_rows": 1_000_000,
                "arrow_rows": 1_000_000,
                "csv_rows": 100_000,
                "image_samples": 50_000,
            },
            "large": {
                "parquet_rows": 10_000_000,
                "arrow_rows": 10_000_000,
                "csv_rows": 1_000_000,
                "image_samples": 100_000,
            },
        }
        
        if scale not in scales:
            print(f"Unknown scale: {scale}. Available: {list(scales.keys())}")
            return
        
        config = scales[scale]
        
        print("=" * 60)
        print(f"Generating {scale.upper()} benchmark datasets")
        print("=" * 60)
        
        results = {}
        
        # Parquet
        path = self.generate_parquet_dataset(
            name=f"benchmark_parquet_{scale}",
            num_rows=config["parquet_rows"],
        )
        results["parquet"] = str(path)
        
        # Arrow IPC
        path = self.generate_arrow_ipc_dataset(
            name=f"benchmark_arrow_{scale}",
            num_rows=config["arrow_rows"],
        )
        results["arrow"] = str(path)
        
        # CSV
        path = self.generate_csv_dataset(
            name=f"benchmark_csv_{scale}",
            num_rows=config["csv_rows"],
        )
        if path:
            results["csv"] = str(path)
        
        # Image-like
        path = self.generate_image_like_dataset(
            name=f"benchmark_imagenet_{scale}",
            num_samples=config["image_samples"],
        )
        results["imagenet"] = str(path)
        
        # Save manifest
        manifest_path = self.output_dir / f"manifest_{scale}.json"
        with open(manifest_path, "w") as f:
            json.dump({
                "scale": scale,
                "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ"),
                "datasets": results,
            }, f, indent=2)
        
        print("\n" + "=" * 60)
        print("Dataset generation complete!")
        print(f"Manifest: {manifest_path}")
        print("=" * 60)
        
        return results


def main():
    parser = argparse.ArgumentParser(
        description="Generate synthetic datasets for Zenith benchmarking"
    )
    parser.add_argument(
        "--scale",
        choices=["tiny", "small", "medium", "large"],
        default="small",
        help="Dataset scale (default: small)",
    )
    parser.add_argument(
        "--output-dir",
        default="./bench/data",
        help="Output directory (default: ./bench/data)",
    )
    
    args = parser.parse_args()
    
    generator = DatasetGenerator(output_dir=args.output_dir)
    generator.generate_all(scale=args.scale)


if __name__ == "__main__":
    main()
