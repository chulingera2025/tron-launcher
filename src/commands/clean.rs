use crate::constants::{CONFIG_DIR, DATA_DIR, LOG_DIR, PID_FILE};
use crate::core::ProcessManager;
use crate::error::Result;
use dialoguer::Confirm;
use std::path::Path;
use tracing::{info, warn};

pub async fn execute(skip_confirm: bool) -> Result<()> {
    // 检查节点是否在运行
    if let Some(pid) = ProcessManager::read_pid()?
        && ProcessManager::is_process_alive(pid)
    {
        warn!("检测到节点正在运行 (PID: {})", pid);
        warn!("请先使用 'tronctl stop' 停止节点后再清理");
        return Ok(());
    }

    println!("\n警告: 此操作将删除 tronctl 产生的所有文件");
    println!("包括:");
    println!("  - 配置文件: {}", CONFIG_DIR);
    println!("  - JAR 文件: {}", DATA_DIR);
    println!("  - 日志文件: {}", LOG_DIR);
    println!("  - PID 文件: {}", PID_FILE);

    // 总体确认
    let confirmed = if skip_confirm {
        true
    } else {
        Confirm::new()
            .with_prompt("确定要继续吗？")
            .default(false)
            .interact()?
    };

    if !confirmed {
        info!("已取消清理操作");
        return Ok(());
    }

    // 询问是否清理区块链数据
    let clean_blockchain_data = if skip_confirm {
        true
    } else {
        println!("\n区块链数据位于: {}/data", DATA_DIR);
        println!("此数据可能非常大（数百 GB 到数 TB）");

        Confirm::new()
            .with_prompt("是否同时清理区块链数据？")
            .default(false)
            .interact()?
    };

    info!("开始清理...");

    // 清理配置目录
    clean_directory(CONFIG_DIR, "配置").await?;

    // 清理日志目录
    clean_directory(LOG_DIR, "日志").await?;

    // 清理 PID 文件
    clean_file(PID_FILE, "PID 文件").await?;

    // 清理数据目录
    if clean_blockchain_data {
        // 清理整个数据目录（包括区块链数据）
        clean_directory(DATA_DIR, "数据（包括区块链数据）").await?;
    } else {
        // 仅清理 FullNode.jar，保留区块链数据
        let jar_path = Path::new(DATA_DIR).join("FullNode.jar");
        clean_file(jar_path.to_str().unwrap_or(""), "FullNode.jar").await?;

        info!("已保留区块链数据目录: {}/data", DATA_DIR);
    }

    info!("清理完成!");

    if !clean_blockchain_data {
        println!("\n提示: 区块链数据已保留，如需完全清理请重新运行并选择清理数据");
    }

    Ok(())
}

async fn clean_directory(path: &str, description: &str) -> Result<()> {
    let dir_path = Path::new(path);

    if !dir_path.exists() {
        info!("跳过 {} 目录（不存在）: {}", description, path);
        return Ok(());
    }

    tokio::fs::remove_dir_all(dir_path).await?;
    info!("已删除 {} 目录: {}", description, path);

    Ok(())
}

async fn clean_file(path: &str, description: &str) -> Result<()> {
    let file_path = Path::new(path);

    if !file_path.exists() {
        info!("跳过 {}（不存在）: {}", description, path);
        return Ok(());
    }

    tokio::fs::remove_file(file_path).await?;
    info!("已删除 {}: {}", description, path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_clean_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_dir");
        tokio::fs::create_dir(&test_path).await.unwrap();
        tokio::fs::write(test_path.join("file.txt"), "test")
            .await
            .unwrap();

        let result = clean_directory(test_path.to_str().unwrap(), "测试").await;
        assert!(result.is_ok());
        assert!(!test_path.exists());
    }

    #[tokio::test]
    async fn test_clean_directory_not_exists() {
        let result = clean_directory("/nonexistent/path", "测试").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clean_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, "test").await.unwrap();

        let result = clean_file(test_file.to_str().unwrap(), "测试文件").await;
        assert!(result.is_ok());
        assert!(!test_file.exists());
    }

    #[tokio::test]
    async fn test_clean_file_not_exists() {
        let result = clean_file("/nonexistent/file.txt", "测试文件").await;
        assert!(result.is_ok());
    }
}
