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
