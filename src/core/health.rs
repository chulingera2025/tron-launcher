use crate::constants::{BLOCK_HEIGHT_CHECK_COUNT, HEALTH_CHECK_INTERVAL_SECS, RPC_ENDPOINT};
use crate::core::ProcessManager;
use crate::error::{Result, TronCtlError};
use crate::models::health_status::{BlockInfo, HealthStatus};
use reqwest::Client;
use tracing::{debug, info};

pub struct HealthChecker {
    client: Client,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    /// 检查节点健康状态
    pub async fn check(&self, pid: i32) -> Result<HealthStatus> {
        let process_alive = ProcessManager::is_process_alive(pid);

        if !process_alive {
            return Ok(HealthStatus {
                process_alive: false,
                rpc_responding: false,
                block_syncing: false,
                current_block: 0,
                previous_block: 0,
            });
        }

        let (rpc_responding, current_block) = match self.get_current_block().await {
            Ok(block) => (true, block),
            Err(_) => (false, 0),
        };

        Ok(HealthStatus {
            process_alive,
            rpc_responding,
            block_syncing: false,
            current_block,
            previous_block: 0,
        })
    }

    /// 检查区块是否在增长
    pub async fn check_block_syncing(&self) -> Result<bool> {
        let mut heights = Vec::new();

        for _ in 0..BLOCK_HEIGHT_CHECK_COUNT {
            let height = self.get_current_block().await?;
            heights.push(height);

            if heights.len() > 1 {
                tokio::time::sleep(std::time::Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS))
                    .await;
            }
        }

        let is_syncing = heights.windows(2).all(|w| w[1] > w[0]);

        if is_syncing && !heights.is_empty() {
            info!(
                "区块正在同步: {} -> {}",
                heights[0],
                heights[heights.len() - 1]
            );
        }

        Ok(is_syncing)
    }

    /// 获取当前区块高度
    async fn get_current_block(&self) -> Result<u64> {
        debug!("查询当前区块: {}", RPC_ENDPOINT);

        let resp = self.client.get(RPC_ENDPOINT).send().await?;

        if !resp.status().is_success() {
            return Err(TronCtlError::RpcCallFailed(format!(
                "HTTP {}",
                resp.status()
            )));
        }

        let block: BlockInfo = resp.json().await?;
        Ok(block.block_header.raw_data.number)
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}
