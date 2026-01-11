use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    NotInitialized,
    Stopped,
    Running { pid: i32 },
    Unhealthy { pid: i32, reason: String },
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub status: NodeStatus,
    pub block_height: Option<u64>,
    pub last_block_time: Option<i64>,
    pub sync_progress: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_status_not_initialized() {
        let status = NodeStatus::NotInitialized;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("NotInitialized"));
    }

    #[test]
    fn test_node_status_stopped() {
        let status = NodeStatus::Stopped;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Stopped"));
    }

    #[test]
    fn test_node_status_running() {
        let status = NodeStatus::Running { pid: 12345 };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("12345"));
    }

    #[test]
    fn test_node_status_unhealthy() {
        let status = NodeStatus::Unhealthy {
            pid: 12345,
            reason: "RPC not responding".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("12345"));
        assert!(json.contains("RPC not responding"));
    }

    #[test]
    fn test_node_state_full() {
        let state = NodeState {
            status: NodeStatus::Running { pid: 999 },
            block_height: Some(12345678),
            last_block_time: Some(1704985200),
            sync_progress: Some(0.95),
        };

        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("12345678"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_node_state_partial() {
        let state = NodeState {
            status: NodeStatus::Stopped,
            block_height: None,
            last_block_time: None,
            sync_progress: None,
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: NodeState = serde_json::from_str(&json).unwrap();
        assert!(deserialized.block_height.is_none());
        assert!(deserialized.sync_progress.is_none());
    }

    #[test]
    fn test_node_state_clone() {
        let state1 = NodeState {
            status: NodeStatus::Running { pid: 111 },
            block_height: Some(100),
            last_block_time: Some(200),
            sync_progress: Some(0.5),
        };

        let state2 = state1.clone();
        assert_eq!(state2.block_height, Some(100));
        assert_eq!(state2.last_block_time, Some(200));
    }

    #[test]
    fn test_node_status_deserialization() {
        let json = r#"{"Running":{"pid":5555}}"#;
        let status: NodeStatus = serde_json::from_str(json).unwrap();

        match status {
            NodeStatus::Running { pid } => assert_eq!(pid, 5555),
            _ => panic!("Expected Running status"),
        }
    }
}
