import unittest
import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent))

from zenith import ZenithClient, ZenithError


class TestZenithClient(unittest.TestCase):
    
    def test_client_creation(self):
        """Test client can be created"""
        client = ZenithClient(buffer_size=1024)
        self.assertIsNotNone(client)
        client.close()
    
    def test_context_manager(self):
        """Test context manager protocol"""
        with ZenithClient(buffer_size=512) as client:
            self.assertFalse(client._closed)
        # Should be closed after context
        self.assertTrue(client._closed)
    
    def test_get_stats(self):
        """Test getting statistics"""
        with ZenithClient() as client:
            stats = client.get_stats()
            self.assertIsNotNone(stats)
            self.assertGreaterEqual(stats.buffer_len, 0)
            self.assertGreaterEqual(stats.plugin_count, 0)
    
    def test_close_idempotent(self):
        """Test close can be called multiple times"""
        client = ZenithClient()
        client.close()
        client.close()  # Should not raise
    
    def test_operations_after_close(self):
        """Test operations fail after close"""
        client = ZenithClient()
        client.close()
        
        with self.assertRaises(ZenithError):
            client.get_stats()


if __name__ == '__main__':
    unittest.main()
