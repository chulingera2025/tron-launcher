use thiserror::Error;

#[derive(Error, Debug)]
pub enum TronCtlError {
    #[error("权限不足: 需要 root 权限")]
    InsufficientPermissions,

    #[error("Java 版本不兼容: 需要 Java {required}, 当前 {current}")]
    IncompatibleJavaVersion { required: String, current: String },

    #[error("内存不足: 推荐 {recommended}GB, 当前 {current}GB")]
    InsufficientMemory { recommended: u64, current: u64 },

    #[error("磁盘空间不足: 推荐 {recommended}GB, 当前 {current}GB")]
    InsufficientDisk { recommended: u64, current: u64 },

    #[error("节点未初始化: 请先运行 'tronctl init'")]
    NodeNotInitialized,

    #[error("节点已在运行中: PID {0}")]
    NodeAlreadyRunning(i32),

    #[error("节点未运行")]
    NodeNotRunning,

    #[error("下载失败: {0}")]
    DownloadFailed(String),

    #[error("MD5 校验失败: 期望 {expected}, 实际 {actual}")]
    Md5Mismatch { expected: String, actual: String },

    #[error("进程启动失败: {0}")]
    ProcessStartFailed(String),

    #[error("RPC 调用失败: {0}")]
    RpcCallFailed(String),

    #[error("配置文件错误: {0}")]
    ConfigError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP 错误: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON 解析错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("TOML 解析错误: {0}")]
    TomlDeError(#[from] toml::de::Error),

    #[error("TOML 序列化错误: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("其他错误: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, TronCtlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_permissions_error() {
        let err = TronCtlError::InsufficientPermissions;
        assert_eq!(err.to_string(), "权限不足: 需要 root 权限");
    }

    #[test]
    fn test_incompatible_java_version_error() {
        let err = TronCtlError::IncompatibleJavaVersion {
            required: "1.8".to_string(),
            current: "11".to_string(),
        };
        assert!(err.to_string().contains("1.8"));
        assert!(err.to_string().contains("11"));
    }

    #[test]
    fn test_insufficient_memory_error() {
        let err = TronCtlError::InsufficientMemory {
            recommended: 32,
            current: 16,
        };
        assert!(err.to_string().contains("32"));
        assert!(err.to_string().contains("16"));
    }

    #[test]
    fn test_node_already_running_error() {
        let err = TronCtlError::NodeAlreadyRunning(12345);
        assert!(err.to_string().contains("12345"));
    }

    #[test]
    fn test_md5_mismatch_error() {
        let err = TronCtlError::Md5Mismatch {
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        assert!(err.to_string().contains("abc123"));
        assert!(err.to_string().contains("def456"));
    }

    #[test]
    fn test_download_failed_error() {
        let err = TronCtlError::DownloadFailed("网络超时".to_string());
        assert!(err.to_string().contains("网络超时"));
    }

    #[test]
    fn test_config_error() {
        let err = TronCtlError::ConfigError("配置无效".to_string());
        assert!(err.to_string().contains("配置无效"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: TronCtlError = io_err.into();
        assert!(err.to_string().contains("IO 错误"));
    }

    #[test]
    fn test_result_type() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(TronCtlError::NodeNotRunning);
        assert!(err.is_err());
    }
}
