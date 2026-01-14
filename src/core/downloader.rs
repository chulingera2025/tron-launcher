use crate::error::{Result, TronCtlError};
use crate::utils::ui;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tracing::{debug, info, warn};

/// 断点续传进度记录
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadProgress {
    url: String,
    total_size: u64,
    chunk_size: u64,
    chunks: Vec<ChunkProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChunkProgress {
    index: usize,
    start: u64,
    end: u64,
    downloaded: u64,
    status: ChunkStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum ChunkStatus {
    Pending,
    Downloading,
    Completed,
}

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                // 不设置全局超时，允许大文件长时间下载
                // 只设置连接超时，防止一直连不上服务器
                .connect_timeout(std::time::Duration::from_secs(60))
                .user_agent("tronctl/0.1.0")
                .build()
                .expect("Failed to build HTTP client"),
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

    /// 流式下载大文件并显示进度（自动选择单线程或多线程）
    pub async fn download_with_progress(
        &self,
        url: &str,
        dest: &Path,
        expected_md5: Option<&str>,
    ) -> Result<()> {
        debug!("开始下载: {} -> {:?}", url, dest);

        // 发送 HEAD 请求检查文件信息（reqwest 会自动跟随重定向）
        let head_response = self.client.head(url).send().await?;

        if !head_response.status().is_success() {
            return Err(TronCtlError::DownloadFailed(format!(
                "HEAD 请求失败，状态码: {}",
                head_response.status()
            )));
        }

        let total_size = head_response
            .content_length()
            .ok_or_else(|| TronCtlError::DownloadFailed("无法获取文件大小".to_string()))?;

        // 检查最终响应是否支持 Range 请求（重定向后的实际文件服务器）
        let supports_range = head_response
            .headers()
            .get("accept-ranges")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "bytes")
            .unwrap_or(false);

        // 如果支持 Range 且文件 > 10MB，使用多线程下载
        if supports_range && total_size > 10 * 1024 * 1024 {
            debug!("使用多线程下载，文件大小: {} bytes", total_size);
            self.download_multithreaded(url, dest, total_size, expected_md5)
                .await
        } else {
            debug!("使用单线程下载");
            self.download_single_thread(url, dest, expected_md5).await
        }
    }

    /// 单线程下载（支持断点续传）
    async fn download_single_thread(
        &self,
        url: &str,
        dest: &Path,
        expected_md5: Option<&str>,
    ) -> Result<()> {
        // 检查是否支持断点续传，决定起始位置
        let resume_pos = if dest.exists() {
            let metadata = tokio::fs::metadata(dest).await?;
            let existing_size = metadata.len();

            let head_response = self.client.head(url).send().await?;
            if !head_response.status().is_success() {
                return Err(TronCtlError::DownloadFailed(format!(
                    "HEAD 请求失败，状态码: {}",
                    head_response.status()
                )));
            }

            let total = head_response.content_length().unwrap_or(0);
            if existing_size > 0 && existing_size < total {
                info!("检测到未完成的下载 ({} bytes)，继续下载...", existing_size);
                Some(existing_size)
            } else {
                None
            }
        } else {
            None
        };

        // 发送 GET 请求获取响应（从断点续传位置或从头开始）
        let response = if let Some(pos) = resume_pos {
            let range = format!("bytes={}-", pos);
            self.client.get(url).header("Range", range).send().await?
        } else {
            self.client.get(url).send().await?
        };

        if !response.status().is_success() && response.status().as_u16() != 206 {
            return Err(TronCtlError::DownloadFailed(format!(
                "HTTP 状态码: {}",
                response.status()
            )));
        }

        // 从 GET 响应中获取可靠的 content_length
        let total_size = response.content_length().unwrap_or(0);

        let pb = ui::create_download_progress_bar(total_size);
        if let Some(pos) = resume_pos {
            pb.set_position(pos);
        }

        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(false)
            .open(dest)
            .await?;

        if resume_pos.is_some() {
            file.seek(std::io::SeekFrom::End(0)).await?;
        }

        let mut stream = response.bytes_stream();
        let mut all_bytes = if resume_pos.is_some() && expected_md5.is_some() {
            tokio::fs::read(dest).await?
        } else {
            Vec::new()
        };

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;

            if expected_md5.is_some() {
                all_bytes.extend_from_slice(&chunk);
            }

            pb.inc(chunk.len() as u64);
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

    /// 获取进度文件路径
    fn progress_file(dest: &Path) -> PathBuf {
        dest.with_extension("progress")
    }

    /// 获取分块文件路径
    fn chunk_file(dest: &Path, index: usize) -> PathBuf {
        let mut chunk_path = dest.to_path_buf();
        chunk_path.set_extension(format!("part{}", index));
        chunk_path
    }

    /// 加载下载进度
    async fn load_progress(dest: &Path) -> Result<Option<DownloadProgress>> {
        let progress_path = Self::progress_file(dest);
        if !progress_path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&progress_path).await?;
        let progress: DownloadProgress = serde_json::from_str(&content)
            .map_err(|e| TronCtlError::Other(anyhow::anyhow!("解析进度文件失败: {}", e)))?;

        Ok(Some(progress))
    }

    /// 保存下载进度
    async fn save_progress(dest: &Path, progress: &DownloadProgress) -> Result<()> {
        let progress_path = Self::progress_file(dest);
        let content = serde_json::to_string_pretty(progress)
            .map_err(|e| TronCtlError::Other(anyhow::anyhow!("序列化进度失败: {}", e)))?;
        tokio::fs::write(&progress_path, content).await?;
        Ok(())
    }

    /// 删除进度文件
    async fn remove_progress(dest: &Path) -> Result<()> {
        let progress_path = Self::progress_file(dest);
        if progress_path.exists() {
            tokio::fs::remove_file(&progress_path).await?;
        }
        Ok(())
    }

    /// 检查分块文件是否已下载完成
    async fn is_chunk_complete(dest: &Path, chunk: &ChunkProgress) -> Result<bool> {
        let chunk_path = Self::chunk_file(dest, chunk.index);
        if !chunk_path.exists() {
            return Ok(false);
        }

        let metadata = tokio::fs::metadata(&chunk_path).await?;
        let expected_size = chunk.end - chunk.start + 1;
        Ok(metadata.len() == expected_size)
    }

    /// 合并所有分块文件
    async fn merge_chunks(dest: &Path, progress: &DownloadProgress) -> Result<()> {
        let mut dest_file = File::create(dest).await?;

        for chunk in &progress.chunks {
            let chunk_path = Self::chunk_file(dest, chunk.index);
            if chunk_path.exists() {
                let mut chunk_file = File::open(&chunk_path).await?;
                let mut buffer = vec![0u8; 8192];
                loop {
                    let n = chunk_file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    dest_file.write_all(&buffer[..n]).await?;
                }
            }
        }

        dest_file.flush().await?;
        Ok(())
    }

    /// 多线程分块下载（支持断点续传）
    async fn download_multithreaded(
        &self,
        url: &str,
        dest: &Path,
        total_size: u64,
        expected_md5: Option<&str>,
    ) -> Result<()> {
        use std::sync::Arc;

        let num_threads = num_cpus::get();
        let chunk_size = total_size / num_threads as u64;

        // 尝试加载之前的下载进度
        let mut progress = if let Some(saved) = Self::load_progress(dest).await? {
            if saved.url == url && saved.total_size == total_size {
                info!("检测到未完成的下载，继续下载...");
                saved
            } else {
                info!("下载源或文件大小已变化，重新开始下载");
                Self::create_initial_progress(url, total_size, num_threads, chunk_size)
            }
        } else {
            Self::create_initial_progress(url, total_size, num_threads, chunk_size)
        };

        let pb = Arc::new(ui::create_download_progress_bar(total_size));

        // 计算已下载的字节数
        let mut downloaded_bytes: u64 = 0;
        for chunk in &progress.chunks {
            if chunk.status == ChunkStatus::Completed {
                downloaded_bytes += chunk.end - chunk.start + 1;
            }
        }
        pb.set_position(downloaded_bytes);

        // 保存初始进度，确保可以断点续传
        Self::save_progress(dest, &progress).await?;

        let mut tasks = Vec::new();

        for i in 0..num_threads {
            let chunk = &progress.chunks[i];

            // 跳过已完成的分块
            if chunk.status == ChunkStatus::Completed {
                let is_complete = Self::is_chunk_complete(dest, chunk).await?;
                if is_complete {
                    continue;
                }
            }

            let client = self.client.clone();
            let url = url.to_string();
            let dest = dest.to_path_buf();
            let pb = Arc::clone(&pb);
            let chunk = chunk.clone();

            let task = tokio::spawn(async move {
                let chunk_path = Self::chunk_file(&dest, chunk.index);

                // 检查是否已有部分下载
                let start_pos = if chunk_path.exists() {
                    let metadata = tokio::fs::metadata(&chunk_path).await?;
                    metadata.len()
                } else {
                    0
                };

                let range_start = chunk.start + start_pos;
                let range = format!("bytes={}-{}", range_start, chunk.end);

                let response = client
                    .get(&url)
                    .header("Range", range)
                    .send()
                    .await
                    .map_err(|e| TronCtlError::DownloadFailed(format!("分块下载失败: {}", e)))?;

                if !response.status().is_success() && response.status().as_u16() != 206 {
                    return Err(TronCtlError::DownloadFailed(format!(
                        "分块下载失败，状态码: {}",
                        response.status()
                    )));
                }

                let mut chunk_file = File::options()
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(&chunk_path)
                    .await?;

                if start_pos > 0 {
                    chunk_file.seek(std::io::SeekFrom::End(0)).await?;
                }

                let mut stream = response.bytes_stream();

                while let Some(chunk_result) = stream.next().await {
                    let data = chunk_result.map_err(|e| {
                        TronCtlError::DownloadFailed(format!("读取数据块失败: {}", e))
                    })?;

                    chunk_file.write_all(&data).await?;
                    pb.inc(data.len() as u64);
                }

                chunk_file.flush().await?;

                Ok::<(), TronCtlError>(())
            });

            tasks.push((i, task));
        }

        // 等待所有分块下载完成
        for (index, task) in tasks {
            match task.await {
                Ok(Ok(())) => {
                    progress.chunks[index].status = ChunkStatus::Completed;
                    Self::save_progress(dest, &progress).await?;
                }
                Ok(Err(e)) => {
                    progress.chunks[index].status = ChunkStatus::Pending;
                    Self::save_progress(dest, &progress).await?;
                    return Err(TronCtlError::DownloadFailed(format!(
                        "下载分块 {} 失败: {}\n\n进度已保存，请重新运行命令继续下载",
                        index, e
                    )));
                }
                Err(e) => {
                    progress.chunks[index].status = ChunkStatus::Pending;
                    Self::save_progress(dest, &progress).await?;
                    return Err(TronCtlError::DownloadFailed(format!(
                        "下载分块 {} 失败: {}\n\n进度已保存，请重新运行命令继续下载",
                        index, e
                    )));
                }
            }
        }

        pb.finish_with_message("下载完成");

        // 合并所有分块文件
        info!("合并分块文件...");
        Self::merge_chunks(dest, &progress).await?;

        // 清理分块文件
        for chunk in &progress.chunks {
            let chunk_path = Self::chunk_file(dest, chunk.index);
            if chunk_path.exists() {
                tokio::fs::remove_file(&chunk_path).await.ok();
            }
        }

        // 删除进度文件
        Self::remove_progress(dest).await?;

        // MD5 校验
        if let Some(expected) = expected_md5 {
            let file_content = tokio::fs::read(dest).await?;
            let digest = md5::compute(&file_content);
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

    /// 创建初始下载进度
    fn create_initial_progress(
        url: &str,
        total_size: u64,
        num_threads: usize,
        chunk_size: u64,
    ) -> DownloadProgress {
        let mut chunks = Vec::new();

        for i in 0..num_threads {
            let start = i as u64 * chunk_size;
            let end = if i == num_threads - 1 {
                total_size - 1
            } else {
                (i as u64 + 1) * chunk_size - 1
            };

            chunks.push(ChunkProgress {
                index: i,
                start,
                end,
                downloaded: 0,
                status: ChunkStatus::Pending,
            });
        }

        DownloadProgress {
            url: url.to_string(),
            total_size,
            chunk_size,
            chunks,
        }
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
            use std::path::Component;
            use tokio_util::io::SyncIoBridge;

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
                let path_to_check = if full_path.exists() {
                    full_path.canonicalize().map_err(|e| {
                        std::io::Error::other(format!("无法规范化路径 {:?}: {}", full_path, e))
                    })?
                } else {
                    // 确保父目录存在
                    if let Some(parent) = full_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    // 对于不存在的文件，验证其父目录在目标目录内
                    if let Some(parent) = full_path.parent() {
                        parent.canonicalize().map_err(|e| {
                            std::io::Error::other(format!("无法规范化父目录: {}", e))
                        })?
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "无效的文件路径",
                        ));
                    }
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

        // Mock HEAD 请求（不支持 Range，走单线程下载）
        let _head_mock = server
            .mock("HEAD", "/test.file")
            .with_status(200)
            .with_header("content-length", &content.len().to_string())
            .create_async()
            .await;

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

        // Mock HEAD 请求
        let _head_mock = server
            .mock("HEAD", "/test.file")
            .with_status(200)
            .with_header("content-length", &content.len().to_string())
            .create_async()
            .await;

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

        // Mock HEAD 请求
        let _head_mock = server
            .mock("HEAD", "/test.file")
            .with_status(200)
            .with_header("content-length", &content.len().to_string())
            .create_async()
            .await;

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

        // Mock HEAD 请求返回 404
        let _head_mock = server
            .mock("HEAD", "/test.file")
            .with_status(404)
            .create_async()
            .await;

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
