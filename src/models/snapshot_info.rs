use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SnapshotServer {
    pub url: String,
    pub latency: Duration,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub date: String,
    pub size_gb: u64,
    pub md5: String,
    pub download_url: String,
}
