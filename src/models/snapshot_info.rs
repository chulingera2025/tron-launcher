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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_server_creation() {
        let server = SnapshotServer {
            url: "http://example.com".to_string(),
            latency: Duration::from_millis(100),
            available: true,
        };

        assert_eq!(server.url, "http://example.com");
        assert_eq!(server.latency, Duration::from_millis(100));
        assert!(server.available);
    }

    #[test]
    fn test_snapshot_server_unavailable() {
        let server = SnapshotServer {
            url: "http://slow.com".to_string(),
            latency: Duration::from_secs(999),
            available: false,
        };

        assert!(!server.available);
        assert_eq!(server.latency.as_secs(), 999);
    }

    #[test]
    fn test_snapshot_metadata_serialization() {
        let metadata = SnapshotMetadata {
            date: "20260109".to_string(),
            size_gb: 53,
            md5: "abc123".to_string(),
            download_url: "http://example.com/snapshot.tgz".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("20260109"));
        assert!(json.contains("53"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn test_snapshot_metadata_deserialization() {
        let json = r#"{
            "date": "20260109",
            "size_gb": 2937,
            "md5": "def456",
            "download_url": "http://example.com/full.tgz"
        }"#;

        let metadata: SnapshotMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.date, "20260109");
        assert_eq!(metadata.size_gb, 2937);
        assert_eq!(metadata.md5, "def456");
    }

    #[test]
    fn test_snapshot_metadata_clone() {
        let meta1 = SnapshotMetadata {
            date: "20260109".to_string(),
            size_gb: 53,
            md5: "test".to_string(),
            download_url: "http://test.com".to_string(),
        };

        let meta2 = meta1.clone();
        assert_eq!(meta1.date, meta2.date);
        assert_eq!(meta1.size_gb, meta2.size_gb);
    }
}
