use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    NotInitialized,
    Stopped,
    Running { pid: i32 },
    Unhealthy { pid: i32, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub status: NodeStatus,
    pub block_height: Option<u64>,
    pub last_block_time: Option<i64>,
    pub sync_progress: Option<f64>,
}
