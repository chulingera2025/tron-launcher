use crate::constants::LOG_DIR;
use crate::error::Result;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

pub async fn execute(follow: bool, lines: usize) -> Result<()> {
    let log_file = PathBuf::from(LOG_DIR).join("fullnode.log");

    if !log_file.exists() {
        println!("日志文件不存在: {:?}", log_file);
        println!("提示: 节点可能尚未启动");
        return Ok(());
    }

    let mut cmd = Command::new("tail");

    if follow {
        cmd.arg("-f");
    }

    cmd.arg("-n")
        .arg(lines.to_string())
        .arg(&log_file)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn()?;
    child.wait().await?;

    Ok(())
}
