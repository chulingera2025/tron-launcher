use crate::constants::PID_FILE;
use crate::error::{Result, TronCtlError};
use crate::models::TronCtlConfig;
use fs2::FileExt;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::fs;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{error, info, warn};

pub struct ProcessManager;

impl ProcessManager {
    /// 启动 FullNode 进程
    pub async fn start(config: &TronCtlConfig) -> Result<i32> {
        use std::io::{Seek, Write};

        let pid_path = Path::new(PID_FILE);

        // 确保目录存在
        if let Some(parent) = pid_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 打开/创建 PID 文件并获取排他锁（在整个启动过程中持有锁）
        let mut pid_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(pid_path)?;

        // 尝试获取排他锁（非阻塞），防止多个实例同时启动
        pid_file.try_lock_exclusive().map_err(|_| {
            TronCtlError::Other(anyhow::anyhow!("无法获取 PID 文件锁，可能有其他实例正在启动或运行"))
        })?;

        // 持有锁的情况下，检查是否已有进程在运行
        if let Ok(content) = fs::read_to_string(pid_path)
            && let Ok(existing_pid) = content.trim().parse::<i32>()
                && Self::is_process_alive(existing_pid) {
                    // 释放锁（通过 drop）
                    drop(pid_file);
                    return Err(TronCtlError::NodeAlreadyRunning(existing_pid));
                }

        info!("启动 Tron FullNode...");

        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_file)?;

        let child = Command::new(&config.java_path)
            .arg(format!("-Xms{}", config.jvm_min_heap))
            .arg(format!("-Xmx{}", config.jvm_max_heap))
            .arg("-jar")
            .arg(&config.fullnode_jar)
            .arg("-c")
            .arg(&config.node_config)
            .arg("-d")
            .arg(&config.data_dir)
            .stdout(Stdio::from(log_file.try_clone()?))
            .stderr(Stdio::from(log_file))
            .spawn()?;

        let pid = child
            .id()
            .ok_or_else(|| TronCtlError::ProcessStartFailed("无法获取进程 PID".to_string()))?
            as i32;

        // 清空文件并写入新 PID（仍在锁保护下）
        pid_file.set_len(0)?;
        pid_file.seek(std::io::SeekFrom::Start(0))?;
        write!(pid_file, "{}", pid)?;
        pid_file.flush()?;

        // 文件关闭时自动释放锁
        drop(pid_file);

        info!("FullNode 已启动, PID: {}", pid);
        Ok(pid)
    }

    /// 停止 FullNode 进程
    pub fn stop(force: bool) -> Result<()> {
        let pid = Self::read_pid()?
            .ok_or(TronCtlError::NodeNotRunning)?;

        if !Self::is_process_alive(pid) {
            Self::remove_pid_file()?;
            return Err(TronCtlError::NodeNotRunning);
        }

        info!("停止 Tron FullNode (PID: {})...", pid);

        if force {
            signal::kill(Pid::from_raw(pid), Signal::SIGKILL)
                .map_err(|e| TronCtlError::ProcessStartFailed(format!("强制终止失败: {}", e)))?;
            Self::remove_pid_file()?;
            info!("FullNode 已强制停止");
            return Ok(());
        }

        signal::kill(Pid::from_raw(pid), Signal::SIGTERM)
            .map_err(|e| TronCtlError::ProcessStartFailed(format!("发送信号失败: {}", e)))?;

        // 等待进程退出（最多30秒）
        for i in 0..30 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if !Self::is_process_alive(pid) {
                Self::remove_pid_file()?;
                info!("FullNode 已停止");
                return Ok(());
            }
            if i % 5 == 0 {
                warn!("等待进程退出... ({}/30)", i);
            }
        }

        // 超时，强制杀死
        error!("进程未在30秒内退出，强制终止");
        signal::kill(Pid::from_raw(pid), Signal::SIGKILL)
            .map_err(|e| TronCtlError::ProcessStartFailed(format!("强制终止失败: {}", e)))?;

        Self::remove_pid_file()?;
        info!("FullNode 已强制停止");
        Ok(())
    }

    /// 读取 PID 文件
    pub fn read_pid() -> Result<Option<i32>> {
        let pid_path = Path::new(PID_FILE);

        if !pid_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(pid_path)?;
        let pid = content
            .trim()
            .parse::<i32>()
            .map_err(|_| TronCtlError::ConfigError("无效的 PID 文件".to_string()))?;

        Ok(Some(pid))
    }

    /// 删除 PID 文件
    fn remove_pid_file() -> Result<()> {
        let pid_path = Path::new(PID_FILE);
        if pid_path.exists() {
            fs::remove_file(pid_path)?;
        }
        Ok(())
    }

    /// 检查进程是否存活
    pub fn is_process_alive(pid: i32) -> bool {
        signal::kill(Pid::from_raw(pid), None).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_process_alive() {
        // 测试当前进程
        let current_pid = std::process::id() as i32;
        assert!(ProcessManager::is_process_alive(current_pid));

        // 测试不存在的进程
        assert!(!ProcessManager::is_process_alive(999999));
    }

    #[test]
    fn test_read_pid_nonexistent() {
        let result = ProcessManager::read_pid();
        assert!(result.is_ok());
    }

    #[test]
    fn test_stop_when_no_node_running() {
        let result = ProcessManager::stop(false);
        assert!(result.is_err());
    }

    #[test]
    fn test_stop_force_when_no_node_running() {
        let result = ProcessManager::stop(true);
        assert!(result.is_err());
    }
}
