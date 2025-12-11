"""
Zenith Python FFI Bindings
High-performance data plane client for Python
"""
import ctypes
import os
from typing import Optional, Dict, Any
from pathlib import Path


class ZenithError(Exception):
    """Base exception for Zenith errors"""
    ERROR_CODES = {
        0: "Success",
        -1: "Null pointer error",
        -2: "Buffer full",
        -3: "Plugin load error",
        -4: "FFI conversion error",
    }
    
    def __init__(self, code: int, message: str = ""):
        self.code = code
        self.message = message or self.ERROR_CODES.get(code, f"Unknown error: {code}")
        super().__init__(self.message)


class Stats:
    """Engine statistics"""
    def __init__(self, buffer_len: int, plugin_count: int, events_processed: int):
        self.buffer_len = buffer_len
        self.plugin_count = plugin_count
        self.events_processed = events_processed
    
    def __repr__(self):
        return f"Stats(buffer_len={self.buffer_len}, plugin_count={self.plugin_count}, events_processed={self.events_processed})"


class _CStats(ctypes.Structure):
    _fields_ = [
        ("buffer_len", ctypes.c_size_t),
        ("plugin_count", ctypes.c_size_t),
        ("events_processed", ctypes.c_uint64),
    ]


class ZenithClient:
    """
    Zenith Data Plane Client
    
    Example:
        >>> with ZenithClient(buffer_size=1024) as client:
        ...     client.load_plugin("filter.wasm")
        ...     stats = client.get_stats()
        ...     print(f"Loaded {stats.plugin_count} plugins")
    """
    
    def __init__(self, buffer_size: int = 1024, lib_path: Optional[str] = None):
        """
        Initialize Zenith client
        
        Args:
            buffer_size: Ring buffer size
            lib_path: Path to libzenith_core.so (auto-detected if None)
        """
        if lib_path is None:
            # Auto-detect library path
            base_path = Path(__file__).parent.parent.parent / "core" / "target" / "release"
            lib_path = str(base_path / "libzenith_core.so")
        
        self._lib = ctypes.CDLL(lib_path)
        self._setup_functions()
        
        self._engine_ptr = self._lib.zenith_init(ctypes.c_uint32(buffer_size))
        if not self._engine_ptr:
            raise ZenithError(-1, "Failed to initialize Zenith engine")
        
        self._closed = False
    
    def _setup_functions(self):
        """Setup C function signatures"""
        # zenith_init
        self._lib.zenith_init.argtypes = [ctypes.c_uint32]
        self._lib.zenith_init.restype = ctypes.c_void_p
        
        # zenith_free
        self._lib.zenith_free.argtypes = [ctypes.c_void_p]
        self._lib.zenith_free.restype = None
        
        # zenith_load_plugin
        self._lib.zenith_load_plugin.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_uint8),
            ctypes.c_size_t
        ]
        self._lib.zenith_load_plugin.restype = ctypes.c_int32
        
        # zenith_get_stats
        self._lib.zenith_get_stats.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(_CStats)
        ]
        self._lib.zenith_get_stats.restype = ctypes.c_int32
    
    def load_plugin(self, wasm_path: str) -> None:
        """
        Load a WASM plugin
        
        Args:
            wasm_path: Path to .wasm file
            
        Raises:
            ZenithError: If plugin loading fails
        """
        if self._closed:
            raise ZenithError(-1, "Client is closed")
        
        with open(wasm_path, 'rb') as f:
            wasm_bytes = f.read()
        
        if not wasm_bytes:
            raise ValueError("Empty WASM file")
        
        c_bytes = (ctypes.c_uint8 * len(wasm_bytes)).from_buffer_copy(wasm_bytes)
        ret = self._lib.zenith_load_plugin(
            self._engine_ptr,
            c_bytes,
            len(wasm_bytes)
        )
        
        if ret != 0:
            raise ZenithError(ret, f"Failed to load plugin: {wasm_path}")
    
    def get_stats(self) -> Stats:
        """
        Get engine statistics
        
        Returns:
            Stats object with current metrics
        """
        if self._closed:
            raise ZenithError(-1, "Client is closed")
        
        c_stats = _CStats()
        ret = self._lib.zenith_get_stats(self._engine_ptr, ctypes.byref(c_stats))
        
        if ret != 0:
            raise ZenithError(ret, "Failed to get stats")
        
        return Stats(
            buffer_len=c_stats.buffer_len,
            plugin_count=c_stats.plugin_count,
            events_processed=c_stats.events_processed
        )
    
    def close(self) -> None:
        """Free engine resources"""
        if not self._closed:
            self._lib.zenith_free(self._engine_ptr)
            self._closed = True
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
    
    def __del__(self):
        self.close()


__all__ = ['ZenithClient', 'ZenithError', 'Stats']
