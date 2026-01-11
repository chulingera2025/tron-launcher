use crate::constants::{APP_CONFIG, CONFIG_DIR, DATA_DIR, LOG_DIR, NODE_CONFIG};
use crate::core::{Downloader, EnvironmentChecker, SnapshotManager};
use crate::error::Result;
use crate::models::TronCtlConfig;
use crate::utils::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{info, warn};

pub async fn execute(
    snapshot_type: String,
    version: Option<String>,
    skip_checks: bool,
) -> Result<()> {
    info!("开始初始化 Tron FullNode...");

    // 1. 环境检查
    if !skip_checks {
        EnvironmentChecker::check_all()?;
    } else {
        warn!("跳过环境检查");
    }

    // 2. 创建目录
    create_directories().await?;

    // 3. 下载 FullNode.jar
    let downloader = Downloader::new();
    let fullnode_jar = PathBuf::from(DATA_DIR).join("FullNode.jar");

    downloader
        .download_fullnode(version, &fullnode_jar)
        .await?;

    // 4. 生成默认配置文件
    generate_default_config().await?;

    // 5. 下载快照（如果需要）
    if snapshot_type != "none" {
        let snapshot_mgr = SnapshotManager::new();

        info!("选择快照服务器...");
        let server = snapshot_mgr.select_fastest_server().await?;

        info!("获取最新快照元数据...");
        let metadata = snapshot_mgr.get_latest_snapshot(&server, &snapshot_type).await?;

        info!("下载快照: {} ({} GB)", metadata.date, metadata.size_gb);
        info!("这可能需要较长时间，请耐心等待...");

        let snapshot_file = PathBuf::from("/tmp/tron-snapshot.tgz");
        downloader
            .download_with_progress(&metadata.download_url, &snapshot_file, Some(&metadata.md5))
            .await?;

        extract_snapshot(&snapshot_file).await?;
    }

    // 6. 保存配置
    save_config(&snapshot_type)?;

    info!("✓ 初始化完成!");
    info!("运行 'tronctl start' 启动节点");

    Ok(())
}

async fn create_directories() -> Result<()> {
    info!("创建目录...");

    for dir in &[DATA_DIR, CONFIG_DIR, LOG_DIR] {
        fs::ensure_dir_exists(Path::new(dir)).await?;
    }

    if let Some(parent) = Path::new(crate::constants::PID_FILE).parent() {
        fs::ensure_dir_exists(parent).await?;
    }

    Ok(())
}

async fn generate_default_config() -> Result<()> {
    info!("生成默认配置文件...");

    let config_path = PathBuf::from(CONFIG_DIR).join(NODE_CONFIG);

    if config_path.exists() {
        warn!("配置文件已存在，跳过生成: {:?}", config_path);
        return Ok(());
    }

    // 使用官方默认配置的简化版本
    // 实际应该从 GitHub 下载，这里简化处理
    let default_config = r#"net {
  type = mainnet
}

storage {
  db.version = 2,
  db.engine = "LEVELDB",
  db.directory = "database",
  index.directory = "index",
  transHistory.switch = "on"
}

node.discovery = {
  enable = true
  persist = true
}

node {
  p2p {
    version = 11111
  }

  active = [
    "39.107.80.135:18888",
    "47.254.16.55:18888",
    "47.254.18.49:18888"
  ]

  listen.port = 18888

  http {
    fullNodePort = 8090
    solidityPort = 8091
  }

  rpc {
    port = 50051
  }
}
"#;

    tokio::fs::write(&config_path, default_config).await?;
    info!("配置文件已生成: {:?}", config_path);

    Ok(())
}

async fn extract_snapshot(snapshot_file: &Path) -> Result<()> {
    info!("解压快照数据...");

    let data_dir = PathBuf::from(DATA_DIR).join("data");
    fs::ensure_dir_exists(&data_dir).await?;

    let output = Command::new("tar")
        .arg("-xzf")
        .arg(snapshot_file)
        .arg("-C")
        .arg(&data_dir)
        .arg("--strip-components=1")
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::error::TronCtlError::Other(anyhow::anyhow!(
            "解压失败: {}",
            stderr
        )));
    }

    tokio::fs::remove_file(snapshot_file).await?;
    info!("快照解压完成");

    Ok(())
}

fn save_config(snapshot_type: &str) -> Result<()> {
    let config = TronCtlConfig {
        snapshot_type: snapshot_type.to_string(),
        ..Default::default()
    };

    let toml = toml::to_string(&config)?;
    let config_path = PathBuf::from(CONFIG_DIR).join(APP_CONFIG);

    std::fs::write(&config_path, toml)?;
    info!("配置已保存: {:?}", config_path);

    Ok(())
}
