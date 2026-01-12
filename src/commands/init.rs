use crate::constants::{APP_CONFIG, CONFIG_DIR, DATA_DIR, LOG_DIR, NODE_CONFIG};
use crate::core::{Downloader, EnvironmentChecker, SnapshotManager};
use crate::error::Result;
use crate::models::TronCtlConfig;
use crate::utils::fs;
use dialoguer::{Confirm, Select};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub async fn execute(
    snapshot_type: Option<String>,
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

    // 5. 交互式选择快照
    let snapshot_choice = if let Some(s) = snapshot_type {
        s
    } else {
        // 交互式询问
        println!("\n📦 快照下载配置");
        println!("快照可以加速节点同步，但需要较大的下载空间和时间");

        if !Confirm::new()
            .with_prompt("是否需要下载快照？")
            .default(false)
            .interact()?
        {
            "none".to_string()
        } else {
            let items = vec![
                "Lite 快照 (53 GB) - 推荐：快速同步，适合大多数场景",
                "Full 快照 (2937 GB) - 完整数据，适合归档节点",
            ];

            let selection = Select::new()
                .with_prompt("选择快照类型")
                .items(&items)
                .default(0)
                .interact()?;

            match selection {
                0 => "lite".to_string(),
                1 => "full".to_string(),
                _ => "none".to_string(),
            }
        }
    };

    // 6. 下载快照（如果需要）
    if snapshot_choice != "none" {
        let snapshot_mgr = SnapshotManager::new();

        info!("选择快照服务器...");
        let server = snapshot_mgr.select_fastest_server().await?;

        info!("获取最新快照元数据...");
        let metadata = snapshot_mgr
            .get_latest_snapshot(&server, &snapshot_choice)
            .await?;

        info!("下载快照: {} ({} GB)", metadata.date, metadata.size_gb);

        // 询问是否需要 MD5 校验
        let verify_md5 = Confirm::new()
            .with_prompt(
                "是否启用 MD5 校验？\n  \
                启用: 下载完整文件后校验，更安全但需要更多磁盘空间\n  \
                禁用: 流式下载解压，节省磁盘空间但无法验证完整性\n  \
                选择",
            )
            .default(false)
            .interact()?;

        let data_dir = PathBuf::from(DATA_DIR).join("data");
        fs::ensure_dir_exists(&data_dir).await?;

        if verify_md5 {
            info!("使用 MD5 校验模式（完整下载后解压）");
            info!("正在下载快照到临时文件...");

            let temp_file = PathBuf::from("/tmp").join(format!("tron-snapshot-{}.tgz", metadata.date));

            // 完整下载并校验
            downloader
                .download_with_progress(&metadata.download_url, &temp_file, Some(&metadata.md5))
                .await?;

            info!("MD5 校验通过，开始解压...");

            // 解压
            extract_snapshot_file(&temp_file, &data_dir).await?;

            // 删除临时文件
            tokio::fs::remove_file(&temp_file).await?;
            info!("临时文件已清理");
        } else {
            info!("使用流式解压模式（无 MD5 校验）");
            info!("正在流式下载并解压，请耐心等待...");

            // 流式下载并解压
            downloader
                .download_and_extract_tgz(&metadata.download_url, &data_dir, None)
                .await?;
        }

        info!("快照下载并解压完成");
    }

    // 7. 保存配置
    save_config(&snapshot_choice)?;

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

async fn extract_snapshot_file(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use std::path::Component;
    use tar::Archive;

    info!("开始解压快照文件...");

    // 规范化目标路径
    let dest_dir_canonical = dest_dir
        .canonicalize()
        .map_err(|e| crate::error::TronCtlError::Other(anyhow::anyhow!("无效的目标路径: {}", e)))?;

    // 在独立线程中进行解压（阻塞操作）
    let archive_path = archive_path.to_path_buf();
    let extract_task = tokio::task::spawn_blocking(move || {
        let file = File::open(&archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        // 安全解压：验证每个文件的路径
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            // 1. 检查路径是否包含 .. 组件
            for component in path.components() {
                if matches!(component, Component::ParentDir) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("检测到路径遍历攻击（包含 ..）: {:?}", path),
                    ));
                }
            }

            // 2. 检查是否为绝对路径
            if path.is_absolute() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("拒绝解压绝对路径: {:?}", path),
                ));
            }

            // 3. 构造完整路径并验证
            let full_path = dest_dir_canonical.join(&path);

            // 4. 验证解压路径确实在目标目录内
            let path_to_check = if full_path.exists() {
                full_path.canonicalize()?
            } else if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
                let parent_canonical = parent.canonicalize()?;
                if let Some(file_name) = full_path.file_name() {
                    parent_canonical.join(file_name)
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "无效的文件路径",
                    ));
                }
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "无效的文件路径",
                ));
            };

            // 确保路径在目标目录内
            if !path_to_check.starts_with(&dest_dir_canonical) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("路径在目标目录外: {:?} -> {:?}", path, path_to_check),
                ));
            }

            // 安全解压
            entry.unpack(&full_path)?;
        }

        Ok::<_, std::io::Error>(())
    });

    extract_task
        .await
        .map_err(|e| crate::error::TronCtlError::Other(anyhow::anyhow!("解压任务失败: {}", e)))??;

    info!("快照解压完成");
    Ok(())
}
