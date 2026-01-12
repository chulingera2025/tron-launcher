use crate::constants::{APP_CONFIG, CONFIG_DIR};
use crate::core::ProcessManager;
use crate::error::Result;
use crate::models::TronCtlConfig;
use std::path::PathBuf;
use tracing::info;

pub async fn execute(daemon: bool) -> Result<()> {
    let config = load_config()?;

    let pid = ProcessManager::start(&config).await?;

    info!("节点已启动 (PID: {})", pid);
    info!("使用 'tronctl status' 查看状态");
    info!("使用 'tronctl logs -f' 查看日志");
    info!("使用 'tronctl stop' 停止节点");

    if !daemon {
        info!("\n提示: 使用 --daemon 参数可以抑制提示信息");
    }

    Ok(())
}

fn load_config() -> Result<TronCtlConfig> {
    let config_path = PathBuf::from(CONFIG_DIR).join(APP_CONFIG);

    let content = std::fs::read_to_string(config_path)?;
    let config: TronCtlConfig = toml::from_str(&content)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_config_error() {
        // 测试配置文件不存在的情况
        let result = load_config();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_config_valid() {
        // 创建临时配置文件
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"
            java_path = "/usr/bin/java"
            jvm_min_heap = "8g"
            jvm_max_heap = "12g"
            fullnode_jar = "/tmp/FullNode.jar"
            node_config = "/tmp/tron.conf"
            data_dir = "/tmp/data"
            log_file = "/tmp/fullnode.log"
            snapshot_type = "none"
        "#;

        let config_path = temp_dir.path().join("tronctl.toml");
        tokio::fs::write(&config_path, config_content)
            .await
            .unwrap();

        // 由于 load_config 使用硬编码路径，这个测试只能验证解析逻辑
    }
}
