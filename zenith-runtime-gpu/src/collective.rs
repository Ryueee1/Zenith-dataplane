//! Collective Communication - NCCL integration

/// Collective operation types
#[derive(Debug, Clone, Copy)]
pub enum CollectiveOp {
    /// All-reduce (sum)
    AllReduce,
    /// Broadcast
    Broadcast,
    /// All-gather
    AllGather,
    /// Reduce-scatter
    ReduceScatter,
    /// Point-to-point send
    Send,
    /// Point-to-point receive
    Recv,
}

/// NCCL communicator handle (placeholder)
pub struct NcclCommunicator {
    /// World size
    pub world_size: usize,
    /// Local rank
    pub rank: usize,
    /// Unique ID
    pub unique_id: String,
}

impl NcclCommunicator {
    /// Create a new communicator
    pub fn new(world_size: usize, rank: usize) -> Self {
        Self {
            world_size,
            rank,
            unique_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    
    /// Check if this is the root rank
    pub fn is_root(&self) -> bool {
        self.rank == 0
    }
}
