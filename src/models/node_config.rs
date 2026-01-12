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
            node_config: PathBuf::from(crate::constants::CONFIG_DIR)
                .join(crate::constants::NODE_CONFIG),
            data_dir: PathBuf::from(crate::constants::DATA_DIR).join("data"),
            log_file: PathBuf::from(crate::constants::LOG_DIR).join("fullnode.log"),
            snapshot_type: "none".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TronCtlConfig::default();

        assert_eq!(config.java_path, PathBuf::from("/usr/bin/java"));
        assert_eq!(config.jvm_min_heap, "8g");
        assert_eq!(config.jvm_max_heap, "12g");
        assert_eq!(config.snapshot_type, "none");
        assert!(
            config
                .fullnode_jar
                .to_string_lossy()
                .contains("FullNode.jar")
        );
        assert!(config.data_dir.to_string_lossy().contains("data"));
    }

    #[test]
    fn test_config_serialization() {
        let config = TronCtlConfig::default();
        let serialized = toml::to_string(&config).unwrap();

        assert!(serialized.contains("java_path"));
        assert!(serialized.contains("jvm_min_heap"));
        assert!(serialized.contains("snapshot_type"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            java_path = "/usr/bin/java"
            jvm_min_heap = "8g"
            jvm_max_heap = "12g"
            fullnode_jar = "/var/lib/tronctl/FullNode.jar"
            node_config = "/etc/tronctl/tron.conf"
            data_dir = "/var/lib/tronctl/data"
            log_file = "/var/log/tronctl/fullnode.log"
            snapshot_type = "lite"
        "#;

        let config: TronCtlConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.snapshot_type, "lite");
        assert_eq!(config.jvm_min_heap, "8g");
    }

    #[test]
    fn test_config_clone() {
        let config1 = TronCtlConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.java_path, config2.java_path);
        assert_eq!(config1.snapshot_type, config2.snapshot_type);
    }
}
