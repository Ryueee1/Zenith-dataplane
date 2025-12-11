import pyarrow as pa
import time
import numpy as np

def generate_ticks(batch_size=1000):
    """
    Generates a synthetic Arrow RecordBatch representing market ticks.
    Schema: {symbol: string, price: float64, size: uint32, ts_ns: uint64}
    """
    symbols = np.random.choice(['AAPL', 'GOOG', 'MSFT', 'AMZN'], batch_size)
    prices = np.random.uniform(100.0, 2000.0, batch_size)
    sizes = np.random.randint(1, 100, batch_size)
    timestamps = np.full(batch_size, time.time_ns())

    batch = pa.RecordBatch.from_arrays(
        [
            pa.array(symbols),
            pa.array(prices),
            pa.array(sizes, type=pa.uint32()),
            pa.array(timestamps, type=pa.uint64())
        ],
        names=['symbol', 'price', 'size', 'ts_ns']
    )
    return batch

if __name__ == "__main__":
    start = time.time()
    total_events = 0
    duration = 5.0 # Run for 5 seconds

    print("Starting Tick Generator Benchmark...")
    while time.time() - start < duration:
        batch = generate_ticks(5000)
        # In a real scenario, this batch would be written to the shared ring buffer.
        # Here we just measure generation throughput.
        total_events += 5000
    
    end = time.time()
    rate = total_events / (end - start)
    print(f"Generated {total_events} events in {end-start:.2f}s")
    print(f"Throughput: {rate:,.2f} events/sec")
