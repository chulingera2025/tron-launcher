use reqwest::Client;
use std::time::{Duration, Instant};

pub async fn measure_latency(client: &Client, url: &str, timeout: Duration) -> Option<Duration> {
    let start = Instant::now();

    match tokio::time::timeout(timeout, client.head(url).send()).await {
        Ok(Ok(resp)) if resp.status().is_success() => Some(start.elapsed()),
        _ => None,
    }
}

pub async fn check_url_exists(client: &Client, url: &str) -> bool {
    client
        .head(url)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_measure_latency_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("HEAD", "/")
            .with_status(200)
            .create_async()
            .await;

        let client = Client::new();
        let url = server.url();
        let latency = measure_latency(&client, &url, Duration::from_secs(5)).await;

        mock.assert_async().await;
        assert!(latency.is_some());
    }

    #[tokio::test]
    async fn test_measure_latency_failure() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("HEAD", "/")
            .with_status(500)
            .create_async()
            .await;

        let client = Client::new();
        let url = server.url();
        let latency = measure_latency(&client, &url, Duration::from_secs(5)).await;

        mock.assert_async().await;
        assert!(latency.is_none());
    }

    #[tokio::test]
    async fn test_measure_latency_invalid_url() {
        let client = Client::new();
        let url = "http://invalid.local.test.nonexistent";
        let latency = measure_latency(&client, url, Duration::from_secs(1)).await;

        assert!(latency.is_none());
    }

    #[tokio::test]
    async fn test_check_url_exists_true() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("HEAD", "/")
            .with_status(200)
            .create_async()
            .await;

        let client = Client::new();
        let url = server.url();
        let exists = check_url_exists(&client, &url).await;

        mock.assert_async().await;
        assert!(exists);
    }

    #[tokio::test]
    async fn test_check_url_exists_false() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("HEAD", "/")
            .with_status(404)
            .create_async()
            .await;

        let client = Client::new();
        let url = server.url();
        let exists = check_url_exists(&client, &url).await;

        mock.assert_async().await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_check_url_exists_network_error() {
        let client = Client::new();
        let url = "http://invalid.local.test.nonexistent";
        let exists = check_url_exists(&client, url).await;

        assert!(!exists);
    }
}
