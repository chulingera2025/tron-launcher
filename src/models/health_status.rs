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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus {
            process_alive: true,
            rpc_responding: true,
            block_syncing: true,
            current_block: 1000,
            previous_block: 999,
        };

        assert!(status.process_alive);
        assert!(status.rpc_responding);
        assert!(status.block_syncing);
        assert_eq!(status.current_block, 1000);
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus {
            process_alive: true,
            rpc_responding: false,
            block_syncing: false,
            current_block: 500,
            previous_block: 500,
        };

        assert!(!status.rpc_responding);
        assert!(!status.block_syncing);
        assert_eq!(status.current_block, status.previous_block);
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus {
            process_alive: true,
            rpc_responding: true,
            block_syncing: false,
            current_block: 12345,
            previous_block: 12344,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("12345"));
        assert!(json.contains("12344"));
    }

    #[test]
    fn test_block_info_deserialization() {
        let json = r#"{
            "block_header": {
                "raw_data": {
                    "number": 99999,
                    "timestamp": 1704985200000
                }
            }
        }"#;

        let block_info: BlockInfo = serde_json::from_str(json).unwrap();
        assert_eq!(block_info.block_header.raw_data.number, 99999);
        assert_eq!(block_info.block_header.raw_data.timestamp, 1704985200000);
    }

    #[test]
    fn test_block_raw_data_clone() {
        let raw1 = BlockRawData {
            number: 123,
            timestamp: 456,
        };

        let raw2 = raw1.clone();
        assert_eq!(raw2.number, 123);
        assert_eq!(raw2.timestamp, 456);
    }

    #[test]
    fn test_health_status_clone() {
        let status1 = HealthStatus {
            process_alive: true,
            rpc_responding: false,
            block_syncing: true,
            current_block: 777,
            previous_block: 776,
        };

        let status2 = status1.clone();
        assert_eq!(status2.current_block, 777);
        assert_eq!(status2.previous_block, 776);
        assert_eq!(status2.process_alive, true);
    }

    #[test]
    fn test_block_header_nested_structure() {
        let header = BlockHeader {
            raw_data: BlockRawData {
                number: 555,
                timestamp: 666,
            },
        };

        let cloned = header.clone();
        assert_eq!(cloned.raw_data.number, 555);
        assert_eq!(cloned.raw_data.timestamp, 666);
    }
}
