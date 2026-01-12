pub const DATA_DIR: &str = "/var/lib/tronctl";
pub const CONFIG_DIR: &str = "/etc/tronctl";
pub const LOG_DIR: &str = "/var/log/tronctl";
pub const PID_FILE: &str = "/run/tronctl/tronctl.pid";

pub const NODE_CONFIG: &str = "tron.conf";
pub const APP_CONFIG: &str = "tronctl.toml";

pub const REQUIRED_JAVA_VERSION: &str = "1.8";

pub const RECOMMENDED_MEMORY_GB: u64 = 32;
pub const RECOMMENDED_DISK_GB: u64 = 2560;

pub const GITHUB_REPO: &str = "tronprotocol/java-tron";
pub const GITHUB_API_RELEASES: &str =
    "https://api.github.com/repos/tronprotocol/java-tron/releases";

pub const SNAPSHOT_SERVERS: &[&str] = &[
    "http://34.143.247.77",
    "http://34.86.86.229",
    "http://35.247.128.170",
];

pub const DEFAULT_JVM_MIN_HEAP: &str = "8g";
pub const DEFAULT_JVM_MAX_HEAP: &str = "12g";

pub const RPC_ENDPOINT: &str = "http://127.0.0.1:8090/wallet/getnowblock";
pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 5;
pub const BLOCK_HEIGHT_CHECK_COUNT: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_paths_absolute() {
        assert!(DATA_DIR.starts_with('/'));
        assert!(CONFIG_DIR.starts_with('/'));
        assert!(LOG_DIR.starts_with('/'));
        assert!(PID_FILE.starts_with('/'));
    }

    #[test]
    fn test_config_filenames() {
        assert!(NODE_CONFIG.ends_with(".conf"));
        assert!(APP_CONFIG.ends_with(".toml"));
        assert!(!NODE_CONFIG.is_empty());
        assert!(!APP_CONFIG.is_empty());
    }

    #[test]
    fn test_java_version_format() {
        assert!(!REQUIRED_JAVA_VERSION.is_empty());
        assert_eq!(REQUIRED_JAVA_VERSION, "1.8");
    }

    #[test]
    fn test_memory_disk_requirements() {
        assert!(RECOMMENDED_MEMORY_GB > 0);
        assert!(RECOMMENDED_DISK_GB > 0);
        assert!(RECOMMENDED_DISK_GB > RECOMMENDED_MEMORY_GB);
    }

    #[test]
    fn test_github_repo_format() {
        assert!(!GITHUB_REPO.is_empty());
        assert!(GITHUB_REPO.contains('/'));
        assert!(!GITHUB_REPO.starts_with('/'));
    }

    #[test]
    fn test_github_api_url() {
        assert!(GITHUB_API_RELEASES.starts_with("https://"));
        assert!(GITHUB_API_RELEASES.contains("api.github.com"));
    }

    #[test]
    fn test_snapshot_servers_valid() {
        assert!(!SNAPSHOT_SERVERS.is_empty());
        assert_eq!(SNAPSHOT_SERVERS.len(), 3);

        for server in SNAPSHOT_SERVERS {
            assert!(server.starts_with("http://") || server.starts_with("https://"));
        }
    }

    #[test]
    fn test_jvm_heap_format() {
        assert!(DEFAULT_JVM_MIN_HEAP.ends_with('g') || DEFAULT_JVM_MIN_HEAP.ends_with('m'));
        assert!(DEFAULT_JVM_MAX_HEAP.ends_with('g') || DEFAULT_JVM_MAX_HEAP.ends_with('m'));
        assert!(!DEFAULT_JVM_MIN_HEAP.is_empty());
        assert!(!DEFAULT_JVM_MAX_HEAP.is_empty());
    }

    #[test]
    fn test_rpc_endpoint_format() {
        assert!(RPC_ENDPOINT.starts_with("http://"));
        assert!(RPC_ENDPOINT.contains("127.0.0.1"));
        assert!(RPC_ENDPOINT.contains(":8090"));
    }

    #[test]
    fn test_health_check_params() {
        assert!(HEALTH_CHECK_INTERVAL_SECS > 0);
        assert!(BLOCK_HEIGHT_CHECK_COUNT > 1);
        assert!(BLOCK_HEIGHT_CHECK_COUNT < 10);
    }
}
