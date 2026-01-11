use crate::constants::{APP_CONFIG, CONFIG_DIR, DATA_DIR, LOG_DIR, NODE_CONFIG};
use crate::core::{Downloader, EnvironmentChecker, SnapshotManager};
use crate::error::Result;
use crate::models::TronCtlConfig;
use crate::utils::fs;
use std::path::{Path, PathBuf};
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
        info!("正在流式下载并解压，请耐心等待...");

        let data_dir = PathBuf::from(DATA_DIR).join("data");
        fs::ensure_dir_exists(&data_dir).await?;

        // 流式下载并解压，无需中间文件
        downloader
            .download_and_extract_tgz(&metadata.download_url, &data_dir, Some(&metadata.md5))
            .await?;

        info!("快照下载并解压完成");
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
    info!("下载默认配置文件...");

    let config_path = PathBuf::from(CONFIG_DIR).join(NODE_CONFIG);

    if config_path.exists() {
        warn!("配置文件已存在，跳过生成: {:?}", config_path);
        return Ok(());
    }

    // 从 GitHub 下载官方配置文件
    let config_url = "https://raw.githubusercontent.com/tronprotocol/java-tron/master/framework/src/main/resources/config.conf";

    let client = reqwest::Client::new();
    let response = client.get(config_url).send().await?;

    if !response.status().is_success() {
        return Err(crate::error::TronCtlError::DownloadFailed(format!(
            "下载配置文件失败: HTTP {}",
            response.status()
        )));
    }

    let config_content = response.text().await?;
    tokio::fs::write(&config_path, config_content).await?;

    info!("配置文件已下载: {:?}", config_path);

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
