use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub process_alive: bool,
    pub rpc_responding: bool,
    pub block_syncing: bool,
    pub current_block: u64,
    pub previous_block: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockInfo {
    #[serde(rename = "block_header")]
    pub block_header: BlockHeader,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockHeader {
    pub raw_data: BlockRawData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockRawData {
    pub number: u64,
    pub timestamp: i64,
}
