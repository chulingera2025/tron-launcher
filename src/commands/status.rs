use crate::core::{HealthChecker, ProcessManager};
use crate::error::Result;

pub async fn execute(verbose: bool) -> Result<()> {
    let pid = ProcessManager::read_pid()?;

    match pid {
        None => {
            println!("状态: 未运行");
            println!("提示: 运行 'tronctl init' 初始化节点");
            Ok(())
        }
        Some(pid) => {
            let checker = HealthChecker::new();
            let health = checker.check(pid).await?;

            println!("状态: 运行中");
            println!("PID: {}", pid);
            println!(
                "进程存活: {}",
                if health.process_alive { "✓" } else { "✗" }
            );
            println!(
                "RPC 响应: {}",
                if health.rpc_responding { "✓" } else { "✗" }
            );

            if health.rpc_responding {
                println!("当前区块: {}", health.current_block);

                if verbose {
                    println!("\n检查区块同步状态...");
                    let syncing = checker.check_block_syncing().await?;
                    println!("区块同步: {}", if syncing { "✓" } else { "✗" });
                }
            }

            Ok(())
        }
    }
}
