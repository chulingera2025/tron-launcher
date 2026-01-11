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
pub const GITHUB_API_RELEASES: &str = "https://api.github.com/repos/tronprotocol/java-tron/releases";

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
