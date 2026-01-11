use crate::error::{Result, TronCtlError};
use crate::utils::ui;
use futures::StreamExt;
use reqwest::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(3600))
                .user_agent("tronctl/0.1.0")
                .build()
                .unwrap(),
        }
    }

    /// 从 GitHub Releases 下载 FullNode.jar
    pub async fn download_fullnode(&self, version: Option<String>, dest: &Path) -> Result<()> {
        let release_url = if let Some(ver) = version {
            format!(
                "https://github.com/{}/releases/download/{}/FullNode.jar",
                crate::constants::GITHUB_REPO,
                ver
            )
        } else {
            let latest = self.get_latest_release().await?;
            info!("使用最新版本: {}", latest);
            format!(
                "https://github.com/{}/releases/download/{}/FullNode.jar",
                crate::constants::GITHUB_REPO,
                latest
            )
        };

        info!("下载 FullNode.jar: {}", release_url);

        self.download_with_progress(&release_url, dest, None).await
    }

    /// 获取最新 Release 版本
    async fn get_latest_release(&self) -> Result<String> {
        #[derive(serde::Deserialize)]
        struct Release {
            tag_name: String,
        }

        let resp = self
            .client
            .get(crate::constants::GITHUB_API_RELEASES)
            .header("User-Agent", "tronctl/0.1.0")
            .send()
            .await?;

        let releases: Vec<Release> = resp.json().await?;

        releases
            .first()
            .map(|r| r.tag_name.clone())
            .ok_or_else(|| TronCtlError::DownloadFailed("未找到可用版本".to_string()))
    }

    /// 流式下载大文件并显示进度
    pub async fn download_with_progress(
        &self,
        url: &str,
        dest: &Path,
        expected_md5: Option<&str>,
    ) -> Result<()> {
        debug!("开始下载: {} -> {:?}", url, dest);

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(TronCtlError::DownloadFailed(format!(
                "HTTP 状态码: {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);

        let pb = ui::create_download_progress_bar(total_size);

        let mut file = File::create(dest).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        let mut all_bytes = Vec::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;

            if expected_md5.is_some() {
                all_bytes.extend_from_slice(&chunk);
            }

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("下载完成");
        file.flush().await?;

        // MD5 校验
        if let Some(expected) = expected_md5 {
            let digest = md5::compute(&all_bytes);
            let actual = format!("{:x}", digest);
            if actual != expected {
                return Err(TronCtlError::Md5Mismatch {
                    expected: expected.to_string(),
                    actual,
                });
            }
            info!("MD5 校验通过");
        }

        Ok(())
    }

    /// 获取 HTTP 客户端引用（供其他模块使用）
    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}
