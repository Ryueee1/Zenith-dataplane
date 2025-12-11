use arrow::record_batch::RecordBatch;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct EventHeader {
    pub source_id: u32,
    pub seq_no: u64,
    pub timestamp_ns: u64,
    pub flags: u32,
}

impl EventHeader {
    pub fn new(source_id: u32, seq_no: u64) -> Self {
        let start = SystemTime::now();
        let timestamp_ns = start
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            source_id,
            seq_no,
            timestamp_ns,
            flags: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZenithEvent {
    pub header: EventHeader,
    pub payload: Option<RecordBatch>, // Option to allow header-only heatbeats
}

impl ZenithEvent {
    pub fn new(source_id: u32, seq_no: u64, payload: RecordBatch) -> Self {
        Self {
            header: EventHeader::new(source_id, seq_no),
            payload: Some(payload),
        }
    }
}
