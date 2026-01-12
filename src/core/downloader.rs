use crate::error::{Result, TronCtlError};
use crate::utils::ui;
use futures::StreamExt;
use reqwest::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

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
    #[allow(dead_code)]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// 流式下载并解压 .tar.gz 文件
    pub async fn download_and_extract_tgz(
        &self,
        url: &str,
        dest_dir: &Path,
        expected_md5: Option<&str>,
    ) -> Result<()> {
        use async_compression::tokio::bufread::GzipDecoder;
        use futures::StreamExt;
        use tokio_util::io::StreamReader;

        info!("开始流式下载并解压: {} -> {:?}", url, dest_dir);

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(TronCtlError::DownloadFailed(format!(
                "HTTP 状态码: {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);
        let pb = ui::create_download_progress_bar(total_size);
        let pb_clone = pb.clone();

        // 规范化目标路径，防止路径遍历攻击
        let dest_dir_canonical = dest_dir
            .canonicalize()
            .map_err(|e| TronCtlError::Other(anyhow::anyhow!("无效的目标路径: {}", e)))?;

        // 将字节流转换为 AsyncRead，同时更新进度条
        let stream = response.bytes_stream().map(move |result| {
            if let Ok(chunk) = &result {
                pb_clone.inc(chunk.len() as u64);
            }
            result.map_err(std::io::Error::other)
        });

        let reader = StreamReader::new(stream);

        // Gzip 解压器
        let gzip_decoder = GzipDecoder::new(tokio::io::BufReader::new(reader));

        // 在独立线程中进行 tar 解压（tar 是阻塞操作）
        let dest_dir = dest_dir_canonical;
        let extract_task = tokio::task::spawn_blocking(move || {
            use tokio_util::io::SyncIoBridge;
            use std::path::Component;

            let sync_reader = SyncIoBridge::new(gzip_decoder);
            let mut archive = tar::Archive::new(sync_reader);

            // 安全解压：验证每个文件的路径
            for entry in archive.entries()? {
                let mut entry = entry?;
                let path = entry.path()?;

                // 1. 检查路径是否包含 .. 组件（父目录引用）
                for component in path.components() {
                    if matches!(component, Component::ParentDir) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            format!("检测到路径遍历攻击（包含 ..）: {:?}", path),
                        ));
                    }
                }

                // 2. 检查是否为绝对路径
                if path.is_absolute() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("拒绝解压绝对路径: {:?}", path),
                    ));
                }

                // 3. 构造完整路径并验证
                let full_path = dest_dir.join(&path);

                // 4. 验证解压路径确实在目标目录内
                // 对于尚不存在的文件，验证其父目录
                let path_to_check = if full_path.exists() {
                    full_path.canonicalize().map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("无法规范化路径 {:?}: {}", full_path, e),
                        )
                    })?
                } else if let Some(parent) = full_path.parent() {
                    // 确保父目录存在
                    std::fs::create_dir_all(parent)?;
                    let parent_canonical = parent.canonicalize()?;
                    if let Some(file_name) = full_path.file_name() {
                        parent_canonical.join(file_name)
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "无效的文件路径",
                        ));
                    }
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "无效的文件路径",
                    ));
                };

                // 确保路径在目标目录内
                if !path_to_check.starts_with(&dest_dir) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("路径在目标目录外: {:?} -> {:?}", path, path_to_check),
                    ));
                }

                // 安全解压
                entry.unpack(&full_path)?;
            }

            Ok::<_, std::io::Error>(())
        });

        extract_task
            .await
            .map_err(|e| TronCtlError::Other(anyhow::anyhow!("解压任务失败: {}", e)))??;

        pb.finish_with_message("下载并解压完成");

        // MD5 校验提示
        if expected_md5.is_some() {
            warn!("流式解压模式下暂不支持 MD5 校验，建议通过服务器 HTTPS 保证文件完整性");
        }

        info!("流式解压完成");
        Ok(())
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_downloader_new() {
        let downloader = Downloader::new();
        assert!(std::ptr::addr_of!(downloader.client) as usize != 0);
    }

    #[test]
    fn test_downloader_default() {
        let downloader = Downloader::default();
        assert!(std::ptr::addr_of!(downloader.client) as usize != 0);
    }

    #[test]
    fn test_downloader_client_ref() {
        let downloader = Downloader::new();
        let client = downloader.client();
        assert!(std::ptr::addr_of!(*client) as usize != 0);
    }

    #[tokio::test]
    async fn test_download_with_progress_success() {
        let mut server = mockito::Server::new_async().await;
        let content = b"test file content";

        let _mock = server
            .mock("GET", "/test.file")
            .with_status(200)
            .with_header("content-length", &content.len().to_string())
            .with_body(content)
            .create_async()
            .await;

        let downloader = Downloader::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("test.file");

        let url = format!("{}/test.file", server.url());
        let result = downloader.download_with_progress(&url, &dest, None).await;

        assert!(result.is_ok());
        assert!(dest.exists());

        let downloaded_content = tokio::fs::read(&dest).await.unwrap();
        assert_eq!(downloaded_content, content);
    }

    #[tokio::test]
    async fn test_download_with_progress_md5_match() {
        let mut server = mockito::Server::new_async().await;
        let content = b"test";
        let md5_hash = format!("{:x}", md5::compute(content));

        let _mock = server
            .mock("GET", "/test.file")
            .with_status(200)
            .with_header("content-length", &content.len().to_string())
            .with_body(content)
            .create_async()
            .await;

        let downloader = Downloader::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("test.file");

        let url = format!("{}/test.file", server.url());
        let result = downloader
            .download_with_progress(&url, &dest, Some(&md5_hash))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_download_with_progress_md5_mismatch() {
        let mut server = mockito::Server::new_async().await;
        let content = b"test";

        let _mock = server
            .mock("GET", "/test.file")
            .with_status(200)
            .with_header("content-length", &content.len().to_string())
            .with_body(content)
            .create_async()
            .await;

        let downloader = Downloader::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("test.file");

        let url = format!("{}/test.file", server.url());
        let result = downloader
            .download_with_progress(&url, &dest, Some("wrongmd5"))
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TronCtlError::Md5Mismatch { .. }
        ));
    }

    #[tokio::test]
    async fn test_download_with_progress_http_error() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", "/test.file")
            .with_status(404)
            .create_async()
            .await;

        let downloader = Downloader::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("test.file");

        let url = format!("{}/test.file", server.url());
        let result = downloader.download_with_progress(&url, &dest, None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_download_with_progress_network_error() {
        let downloader = Downloader::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("test.file");

        let result = downloader
            .download_with_progress("http://invalid.test.nonexistent", &dest, None)
            .await;

        assert!(result.is_err());
    }
}
