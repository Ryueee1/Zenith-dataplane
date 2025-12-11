"""
Integration tests for Zenith Data Plane
Tests end-to-end functionality
"""
import unittest
import sys
import os
import time
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../sdk-python'))

from zenith import ZenithClient, ZenithError


class TestZenithIntegration(unittest.TestCase):
    """Integration test suite"""
    
    def setUp(self):
        """Setup test client"""
        self.client = ZenithClient(buffer_size=1024)
    
    def tearDown(self):
        """Cleanup"""
        self.client.close()
    
    def test_client_lifecycle(self):
        """Test client initialization and cleanup"""
        # Should be initialized
        self.assertFalse(self.client._closed)
        
        # Get stats
        stats = self.client.get_stats()
        self.assertIsNotNone(stats)
        self.assertGreaterEqual(stats.buffer_len, 0)
        
        # Close
        self.client.close()
        self.assertTrue(self.client._closed)
    
    def test_plugin_loading(self):
        """Test WASM plugin loading"""
        plugin_path = os.path.join(
            os.path.dirname(__file__),
            '../filter.wasm'
        )
        
        if os.path.exists(plugin_path):
            # Load plugin
            self.client.load_plugin(plugin_path)
            
            # Verify it's loaded
            stats = self.client.get_stats()
            self.assertGreater(stats.plugin_count, 0)
        else:
            self.skipTest("filter.wasm not found")
    
    def test_multiple_clients(self):
        """Test multiple concurrent clients"""
        clients = []
        
        try:
            # Create multiple clients
            for i in range(5):
                client = ZenithClient(buffer_size=512)
                clients.append(client)
                
                # Each should work independently
                stats = client.get_stats()
                self.assertIsNotNone(stats)
        finally:
            # Cleanup
            for client in clients:
                client.close()
    
    def test_error_handling(self):
        """Test error conditions"""
        # Close client
        self.client.close()
        
        # Operations after close should fail
        with self.assertRaises(ZenithError):
            self.client.get_stats()


class TestZenithPerformance(unittest.TestCase):
    """Performance-oriented tests"""
    
    def test_rapid_init_close(self):
        """Test rapid client creation/destruction"""
        start = time.time()
        
        for i in range(10):
            with ZenithClient(buffer_size=256) as client:
                stats = client.get_stats()
                self.assertIsNotNone(stats)
        
        elapsed = time.time() - start
        self.assertLess(elapsed, 5.0, "Too slow for rapid init/close")


if __name__ == '__main__':
    unittest.main(verbosity=2)
