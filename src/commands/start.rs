use crate::constants::{APP_CONFIG, CONFIG_DIR};
use crate::core::ProcessManager;
use crate::error::Result;
use crate::models::TronCtlConfig;
use std::path::PathBuf;
use tracing::info;

pub async fn execute(daemon: bool) -> Result<()> {
    let config = load_config()?;

    let pid = ProcessManager::start(&config).await?;

    if daemon {
        info!("节点已在后台运行 (PID: {})", pid);
        info!("使用 'tronctl status' 查看状态");
        info!("使用 'tronctl logs -f' 查看日志");
    } else {
        info!("节点正在运行... (按 Ctrl+C 停止)");

        tokio::signal::ctrl_c().await?;

        info!("\n收到中断信号，停止节点...");
        ProcessManager::stop(false)?;
    }

    Ok(())
}

fn load_config() -> Result<TronCtlConfig> {
    let config_path = PathBuf::from(CONFIG_DIR).join(APP_CONFIG);

    let content = std::fs::read_to_string(config_path)?;
    let config: TronCtlConfig = toml::from_str(&content)?;

    Ok(config)
}
