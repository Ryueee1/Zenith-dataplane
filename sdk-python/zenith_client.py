import ctypes
import pyarrow as pa
from pyarrow.cffi import ffi as arrow_ffi

class ZenithSDK:
    def __init__(self, lib_path="./core/target/release/libzenith_core.so"):
        self.lib = ctypes.CDLL(lib_path)
        
        # void* zenith_init(uint32_t buffer_size)
        self.lib.zenith_init.argtypes = [ctypes.c_uint32]
        self.lib.zenith_init.restype = ctypes.c_void_p
        
        # int32_t zenith_publish(void* engine, void* array, void* schema, u32, u64)
        # We use c_void_p because we just pass the address
        self.lib.zenith_publish.argtypes = [
            ctypes.c_void_p, 
            ctypes.c_void_p, 
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_uint64
        ]
        self.lib.zenith_publish.restype = ctypes.c_int32

        self.engine_ptr = self.lib.zenith_init(1024)
        if not self.engine_ptr:
            raise RuntimeError("Failed to initialize Zenith Engine")
        print(f"Zenith Engine init success at {hex(self.engine_ptr)}")

    def publish(self, record_batch: pa.RecordBatch, source_id: int, seq_no: int):
        # 1. Convert to StructArray (RecordBatch is logical, StructArray is physical layout for FFI usually)
        # However, for simplicity here, we assume single-record-batch export pattern
        # PyArrow allows exporting RecordBatch directly if we treat it as an array (StructArray)
        struct_array = record_batch.to_struct_array()
        
        # 2. Allocate C structs using cffi (standard way with pyarrow)
        c_schema = arrow_ffi.new("struct ArrowSchema*")
        c_array = arrow_ffi.new("struct ArrowArray*")
        
        # 3. Get integers for the addresses
        c_schema_addr = int(arrow_ffi.cast("uintptr_t", c_schema))
        c_array_addr = int(arrow_ffi.cast("uintptr_t", c_array))

        # 4. Export
        struct_array._export_to_c(c_array_addr, c_schema_addr)

        # 5. Pass to Rust (Cast integer -> c_void_p)
        # Rust takes ownership!
        ret = self.lib.zenith_publish(
            self.engine_ptr, 
            ctypes.c_void_p(c_array_addr), 
            ctypes.c_void_p(c_schema_addr),
            source_id, 
            seq_no
        )

        if ret != 0:
            raise RuntimeError(f"Publish failed with code {ret}")

    def load_plugin(self, wasm_path: str):
        with open(wasm_path, 'rb') as f:
            wasm_bytes = f.read()
        
        # zenith_load_plugin(engine, bytes_ptr, len)
        self.lib.zenith_load_plugin.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p, # bytes
            ctypes.c_size_t
        ]
        self.lib.zenith_load_plugin.restype = ctypes.c_int32
        
        ret = self.lib.zenith_load_plugin(
            self.engine_ptr,
            wasm_bytes,
            len(wasm_bytes)
        )
        if ret != 0:
            raise RuntimeError(f"Failed to load plugin from {wasm_path}")
        print(f"Loaded plugin: {wasm_path} ({len(wasm_bytes)} bytes)")

    def close(self):
        if self.engine_ptr:
            self.lib.zenith_free(self.engine_ptr)
            self.engine_ptr = None
