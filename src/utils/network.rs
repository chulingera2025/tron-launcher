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
