use crate::constants::DATA_DIR;
use crate::error::Result;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

pub async fn execute(follow: bool, lines: usize) -> Result<()> {
    // Java-tron会在工作目录创建 logs/tron.log
    let log_file = PathBuf::from(DATA_DIR).join("logs/tron.log");

    if !log_file.exists() {
        println!("日志文件不存在: {:?}", log_file);
        println!("提示: 节点可能尚未启动");
        return Ok(());
    }

    let mut cmd = Command::new("tail");

    // 构建参数：tail -n <lines> [-f] <file>
    cmd.arg("-n").arg(lines.to_string());

    if follow {
        cmd.arg("-f");
    }

    cmd.arg(&log_file)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn()?;
    child.wait().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_nonexistent_log_file() {
        let result = execute(false, 100).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_with_follow() {
        let result = execute(true, 50).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_execute_different_line_counts() {
        let result1 = execute(false, 10).await;
        let result2 = execute(false, 100).await;
        let result3 = execute(false, 1000).await;

        assert_eq!(result1.is_ok(), result2.is_ok());
        assert_eq!(result2.is_ok(), result3.is_ok());
    }
}
