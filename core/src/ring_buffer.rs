use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use crate::error::{Result, ZenithError};
use crate::event::ZenithEvent;

pub struct ZenithRingBuffer {
    queue: Arc<ArrayQueue<ZenithEvent>>,
}

impl ZenithRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    pub fn push(&self, event: ZenithEvent) -> Result<()> {
        self.queue.push(event).map_err(|_| ZenithError::BufferFull)
    }

    pub fn pop(&self) -> Option<ZenithEvent> {
        self.queue.pop()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Clone for ZenithRingBuffer {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::ZenithEvent;
    use arrow::array::Int32Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;
    
    /// Create a test event for ring buffer tests
    fn create_test_event(source_id: u32, seq_no: u64) -> ZenithEvent {
        let schema = Arc::new(Schema::new(vec![
            Field::new("value", DataType::Int32, false),
        ]));
        let values = Int32Array::from(vec![1, 2, 3]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(values)]).unwrap();
        ZenithEvent::new(source_id, seq_no, batch)
    }
    
    #[test]
    fn test_ring_buffer_creation() {
        let buffer = ZenithRingBuffer::new(10);
        
        // New buffer should be empty
        assert!(buffer.is_empty(), "New buffer should be empty");
        assert_eq!(buffer.len(), 0, "New buffer length should be 0");
    }
    
    #[test]
    fn test_ring_buffer_push_pop() {
        let buffer = ZenithRingBuffer::new(10);
        
        // Push an event
        let event = create_test_event(1, 100);
        let result = buffer.push(event);
        assert!(result.is_ok(), "Push to empty buffer should succeed");
        
        // Buffer should not be empty after push
        assert!(!buffer.is_empty(), "Buffer should not be empty after push");
        assert_eq!(buffer.len(), 1, "Buffer length should be 1 after push");
        
        // Pop the event
        let popped = buffer.pop();
        assert!(popped.is_some(), "Pop should return Some after push");
        
        // Verify event data
        let popped_event = popped.unwrap();
        assert_eq!(popped_event.header.source_id, 1);
        assert_eq!(popped_event.header.seq_no, 100);
        
        // Buffer should be empty after pop
        assert!(buffer.is_empty(), "Buffer should be empty after pop");
        assert_eq!(buffer.len(), 0, "Buffer length should be 0 after pop");
    }
    
    #[test]
    fn test_ring_buffer_pop_empty() {
        let buffer = ZenithRingBuffer::new(10);
        
        // Pop from empty buffer should return None
        let result = buffer.pop();
        assert!(result.is_none(), "Pop from empty buffer should return None");
    }
    
    #[test]
    fn test_ring_buffer_len_accuracy() {
        let buffer = ZenithRingBuffer::new(10);
        
        // Test len returns correct value, not 0 or 1 constant
        assert_eq!(buffer.len(), 0);
        
        buffer.push(create_test_event(1, 1)).unwrap();
        assert_eq!(buffer.len(), 1);
        
        buffer.push(create_test_event(1, 2)).unwrap();
        assert_eq!(buffer.len(), 2);
        
        buffer.push(create_test_event(1, 3)).unwrap();
        assert_eq!(buffer.len(), 3);
        
        buffer.pop();
        assert_eq!(buffer.len(), 2);
        
        buffer.pop();
        assert_eq!(buffer.len(), 1);
        
        buffer.pop();
        assert_eq!(buffer.len(), 0);
    }
    
    #[test]
    fn test_ring_buffer_is_empty_accuracy() {
        let buffer = ZenithRingBuffer::new(10);
        
        // Test is_empty returns correct value, not constant true/false
        assert!(buffer.is_empty(), "Empty buffer is_empty should be true");
        
        buffer.push(create_test_event(1, 1)).unwrap();
        assert!(!buffer.is_empty(), "Non-empty buffer is_empty should be false");
        
        buffer.push(create_test_event(1, 2)).unwrap();
        assert!(!buffer.is_empty(), "Buffer with 2 items is_empty should be false");
        
        buffer.pop();
        assert!(!buffer.is_empty(), "Buffer with 1 item is_empty should be false");
        
        buffer.pop();
        assert!(buffer.is_empty(), "Empty buffer after pops is_empty should be true");
    }
    
    #[test]
    fn test_ring_buffer_push_returns_ok() {
        let buffer = ZenithRingBuffer::new(2);
        
        // First push should succeed
        let result1 = buffer.push(create_test_event(1, 1));
        assert!(result1.is_ok(), "First push should return Ok");
        
        // Second push should succeed
        let result2 = buffer.push(create_test_event(1, 2));
        assert!(result2.is_ok(), "Second push should return Ok");
        
        // Third push to full buffer should fail
        let result3 = buffer.push(create_test_event(1, 3));
        assert!(result3.is_err(), "Push to full buffer should return Err");
    }
    
    #[test]
    fn test_ring_buffer_clone() {
        let buffer = ZenithRingBuffer::new(10);
        buffer.push(create_test_event(1, 1)).unwrap();
        
        // Clone shares the same underlying queue
        let cloned = buffer.clone();
        assert_eq!(cloned.len(), 1);
        assert!(!cloned.is_empty());
        
        // Pop from clone affects original (shared queue)
        cloned.pop();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }
    
    #[test]
    fn test_ring_buffer_fifo_order() {
        let buffer = ZenithRingBuffer::new(10);
        
        // Push events with different seq_no
        buffer.push(create_test_event(1, 100)).unwrap();
        buffer.push(create_test_event(2, 200)).unwrap();
        buffer.push(create_test_event(3, 300)).unwrap();
        
        // Pop should return in FIFO order
        let first = buffer.pop().unwrap();
        assert_eq!(first.header.seq_no, 100, "First pop should have seq_no 100");
        
        let second = buffer.pop().unwrap();
        assert_eq!(second.header.seq_no, 200, "Second pop should have seq_no 200");
        
        let third = buffer.pop().unwrap();
        assert_eq!(third.header.seq_no, 300, "Third pop should have seq_no 300");
    }
}
