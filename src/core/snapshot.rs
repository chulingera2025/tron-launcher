use crate::error::{Result, TronCtlError};
use crate::models::snapshot_info::{SnapshotMetadata, SnapshotServer};
use crate::utils::network;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, info};

pub struct SnapshotManager {
    client: Client,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .user_agent("tronctl/0.1.0")
                .build()
                .unwrap(),
        }
    }

    /// 自动选择最快的快照服务器
    pub async fn select_fastest_server(&self) -> Result<SnapshotServer> {
        info!("测试快照服务器延迟...");

        let mut servers = Vec::new();

        for &url in crate::constants::SNAPSHOT_SERVERS {
            let latency = network::measure_latency(&self.client, url, Duration::from_secs(5)).await;

            let server = SnapshotServer {
                url: url.to_string(),
                latency: latency.unwrap_or(Duration::from_secs(999)),
                available: latency.is_some(),
            };

            info!(
                "  {} - {}ms {}",
                server.url,
                server.latency.as_millis(),
                if server.available { "可用" } else { "不可用" }
            );

            servers.push(server);
        }

        servers.sort_by_key(|s| s.latency);

        servers
            .into_iter()
            .find(|s| s.available)
            .ok_or_else(|| TronCtlError::DownloadFailed("无可用快照服务器".to_string()))
    }

    /// 获取最新的快照元数据
    pub async fn get_latest_snapshot(
        &self,
        server: &SnapshotServer,
        snapshot_type: &str,
    ) -> Result<SnapshotMetadata> {
        debug!("查找最新快照: 类型={}", snapshot_type);

        let (size_gb, filename_prefix) = match snapshot_type {
            "lite" => (53, "LiteFullNode_output-directory"),
            "full" => (2937, "FullNode_output-directory"),
            _ => {
                return Err(TronCtlError::ConfigError(format!(
                    "无效的快照类型: {}",
                    snapshot_type
                )))
            }
        };

        // 从今天开始向前尝试7天
        for days_ago in 0..7 {
            let date = chrono::Utc::now() - chrono::Duration::days(days_ago);
            let date_str = date.format("%Y%m%d").to_string();

            // 构造快照 URL，格式如：backup20260109/FullNode_output-directory.tgz
            let snapshot_url = format!(
                "{}/backup{}/{}.tgz",
                server.url, date_str, filename_prefix
            );
            let md5_url = format!("{}.md5sum", snapshot_url);

            debug!("尝试快照: {}", snapshot_url);

            if network::check_url_exists(&self.client, &snapshot_url).await {
                let md5 = self.fetch_md5(&md5_url).await.unwrap_or_default();

                info!("找到快照: {} ({} GB)", date_str, size_gb);

                return Ok(SnapshotMetadata {
                    date: date_str,
                    size_gb,
                    md5,
                    download_url: snapshot_url,
                });
            }
        }

        Err(TronCtlError::DownloadFailed(format!(
            "未找到可用的 {} 快照",
            snapshot_type
        )))
    }

    async fn fetch_md5(&self, url: &str) -> Result<String> {
        let resp = self.client.get(url).send().await?;
        let text = resp.text().await?;

        Ok(text.split_whitespace().next().unwrap_or("").to_string())
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_manager_new() {
        let manager = SnapshotManager::new();
        assert!(std::ptr::addr_of!(manager.client) as usize != 0);
    }

    #[test]
    fn test_snapshot_manager_default() {
        let manager = SnapshotManager::default();
        assert!(std::ptr::addr_of!(manager.client) as usize != 0);
    }

    #[tokio::test]
    async fn test_get_latest_snapshot_invalid_type() {
        let manager = SnapshotManager::new();
        let server = SnapshotServer {
            url: "http://test.com".to_string(),
            latency: Duration::from_millis(100),
            available: true,
        };

        let result = manager.get_latest_snapshot(&server, "invalid").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TronCtlError::ConfigError(_)));
    }

    #[tokio::test]
    async fn test_fetch_md5_success() {
        let mut server = mockito::Server::new_async().await;
        let md5_content = "abc123def456  filename.tgz\n";

        let _mock = server
            .mock("GET", "/test.md5sum")
            .with_status(200)
            .with_body(md5_content)
            .create_async()
            .await;

        let manager = SnapshotManager::new();
        let url = format!("{}/test.md5sum", server.url());
        let md5 = manager.fetch_md5(&url).await.unwrap();

        assert_eq!(md5, "abc123def456");
    }

    #[tokio::test]
    async fn test_fetch_md5_empty() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", "/test.md5sum")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let manager = SnapshotManager::new();
        let url = format!("{}/test.md5sum", server.url());
        let md5 = manager.fetch_md5(&url).await.unwrap();

        assert_eq!(md5, "");
    }

    #[tokio::test]
    async fn test_fetch_md5_network_error() {
        let manager = SnapshotManager::new();
        let result = manager
            .fetch_md5("http://invalid.test.nonexistent/test.md5sum")
            .await;

        assert!(result.is_err());
    }
}
