use crate::core::ProcessManager;
use crate::error::Result;
use tracing::info;

pub async fn execute(daemon: bool) -> Result<()> {
    info!("重启 Tron FullNode...");

    // 停止节点
    if let Err(e) = ProcessManager::stop(false) {
        // 如果节点本来就未运行，忽略错误
        info!("停止节点: {}", e);
    }

    // 等待2秒确保进程完全退出
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 启动节点
    super::start::execute(daemon).await
}
