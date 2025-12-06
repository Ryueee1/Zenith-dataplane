# Zenith Python SDK

The official Python client for the Zenith Data Plane. This library provides a high-performance, zero-copy interface to the Rust core engine.

## Installation

```bash
# Requires the core library to be built first
cd ../core && cargo build --release
python3 -m pip install -r requirements.txt
```

## Usage

```python
import pyarrow as pa
from zenith_client import ZenithSDK

# Initialize the engine (loading the shared library)
sdk = ZenithSDK(lib_path="../core/target/release/libzenith_core.so")

# Create your Arrow data
batch = pa.RecordBatch.from_arrays(
    [pa.array([1, 2, 3])], 
    names=['data']
)

# Publish (Zero-Copy)
sdk.publish(batch, source_id=1, seq_no=100)
```

## Architecture
This SDK uses `ctypes` to load the compiled Rust library (`.so`) and `pyarrow.cffi` to export Arrow memory regions directly to the engine without serialization.
