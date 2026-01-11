use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronCtlConfig {
    pub java_path: PathBuf,
    pub jvm_min_heap: String,
    pub jvm_max_heap: String,
    pub fullnode_jar: PathBuf,
    pub node_config: PathBuf,
    pub data_dir: PathBuf,
    pub log_file: PathBuf,
    pub snapshot_type: String,
}

impl Default for TronCtlConfig {
    fn default() -> Self {
        Self {
            java_path: PathBuf::from("/usr/bin/java"),
            jvm_min_heap: crate::constants::DEFAULT_JVM_MIN_HEAP.to_string(),
            jvm_max_heap: crate::constants::DEFAULT_JVM_MAX_HEAP.to_string(),
            fullnode_jar: PathBuf::from(crate::constants::DATA_DIR).join("FullNode.jar"),
            node_config: PathBuf::from(crate::constants::CONFIG_DIR).join(crate::constants::NODE_CONFIG),
            data_dir: PathBuf::from(crate::constants::DATA_DIR).join("data"),
            log_file: PathBuf::from(crate::constants::LOG_DIR).join("fullnode.log"),
            snapshot_type: "none".to_string(),
        }
    }
}
