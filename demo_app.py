import time
import pyarrow as pa
import sys
import os

# Ensure sdk-python is in path
sys.path.append(os.path.join(os.getcwd(), 'sdk-python'))

from zenith_client import ZenithSDK

def main():
    print("Initializing Zenith Client...")
    
    # Needs to point to the .so built
    sdk = ZenithSDK(lib_path="./core/target/release/libzenith_core.so")
    
    # Create Data
    data = [
        pa.array([1, 2, 3, 4]),
        pa.array(["foo", "bar", "baz", "qux"]),
        pa.array([True, False, True, False])
    ]
    batch = pa.RecordBatch.from_arrays(data, names=['col1', 'col2', 'col3'])
    
    print("Publishing 10 batches...")
    start = time.time()
    for i in range(10):
        sdk.publish(batch, source_id=1, seq_no=i)
    end = time.time()
    
    print(f"Done. 10 batches published in {(end-start)*1000:.2f}ms")
    sdk.close()

if __name__ == "__main__":
    main()
