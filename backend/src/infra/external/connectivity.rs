//! 网络连接检测与缓存

use std::sync::OnceLock;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::Mutex;

use crate::domain::models::{ConnectivityResponse, ConnectivityStatus};
use crate::domain::types::unix_ms_to_iso8601;
use crate::shared::time::now_millis;

const CACHE_TTL: Duration = Duration::from_secs(30);
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

pub const OFFLINE_AVAILABLE_FEATURES: &[&str] = &[
    "view_history",
    "manage_test_sets",
    "view_checkpoints",
];
pub const OFFLINE_RESTRICTED_FEATURES: &[&str] = &[
    "api_connection_test",
    "run_optimization",
];

#[derive(Debug, Clone)]
struct ConnectivityCache {
    status: ConnectivityStatus,
    last_checked_at: i64,
    message: Option<String>,
}

impl Default for ConnectivityCache {
    fn default() -> Self {
        Self {
            status: ConnectivityStatus::Online,
            last_checked_at: 0,
            message: None,
        }
    }
}

static CONNECTIVITY_CACHE: OnceLock<Mutex<ConnectivityCache>> = OnceLock::new();
static CONNECTIVITY_PROBE_URL: OnceLock<String> = OnceLock::new();
static CONNECTIVITY_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn init_connectivity_probe(url: impl Into<String>) {
    let _ = CONNECTIVITY_PROBE_URL.set(url.into());
}

fn client() -> Client {
    CONNECTIVITY_CLIENT
        .get_or_init(|| {
            Client::builder()
                .timeout(PROBE_TIMEOUT)
                .build()
                .unwrap_or_else(|_| Client::new())
        })
        .clone()
}

fn build_response(cache: &ConnectivityCache) -> ConnectivityResponse {
    let (available, restricted) = features_for_status(&cache.status);
    ConnectivityResponse {
        status: cache.status.clone(),
        last_checked_at: unix_ms_to_iso8601(cache.last_checked_at),
        message: cache.message.clone(),
        available_features: available,
        restricted_features: restricted,
    }
}

fn features_for_status(status: &ConnectivityStatus) -> (Vec<String>, Vec<String>) {
    match status {
        ConnectivityStatus::Online => (Vec::new(), Vec::new()),
        ConnectivityStatus::Limited => (
            OFFLINE_AVAILABLE_FEATURES.iter().map(|s| s.to_string()).collect(),
            OFFLINE_RESTRICTED_FEATURES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ),
        ConnectivityStatus::Offline => (
            OFFLINE_AVAILABLE_FEATURES.iter().map(|s| s.to_string()).collect(),
            OFFLINE_RESTRICTED_FEATURES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ),
    }
}

pub async fn check_connectivity() -> ConnectivityResponse {
    let cache = CONNECTIVITY_CACHE.get_or_init(|| Mutex::new(ConnectivityCache::default()));
    let now = now_millis();

    {
        let guard = cache.lock().await;
        if now.saturating_sub(guard.last_checked_at) < CACHE_TTL.as_millis() as i64 {
            return build_response(&guard);
        }
    }

    let (status, message) = match CONNECTIVITY_PROBE_URL.get() {
        Some(url) => active_probe(url).await,
        None => (
            ConnectivityStatus::Online,
            Some("未配置主动探测地址，仅返回缓存状态".to_string()),
        ),
    };

    let mut guard = cache.lock().await;
    guard.status = status;
    guard.message = message;
    guard.last_checked_at = now;
    build_response(&guard)
}

pub async fn check_connectivity_status() -> ConnectivityStatus {
    check_connectivity().await.status
}

pub async fn record_connectivity_success() {
    let cache = CONNECTIVITY_CACHE.get_or_init(|| Mutex::new(ConnectivityCache::default()));
    let mut guard = cache.lock().await;
    guard.status = ConnectivityStatus::Online;
    guard.message = Some("被动探测：最近请求成功".to_string());
    guard.last_checked_at = now_millis();
}

pub async fn record_connectivity_failure(status: ConnectivityStatus, message: String) {
    let cache = CONNECTIVITY_CACHE.get_or_init(|| Mutex::new(ConnectivityCache::default()));
    let mut guard = cache.lock().await;
    guard.status = status;
    guard.message = Some(message);
    guard.last_checked_at = now_millis();
}

async fn active_probe(url: &str) -> (ConnectivityStatus, Option<String>) {
    let response = client().get(url).send().await;
    match response {
        Ok(resp) if resp.status().is_success() => {
            (ConnectivityStatus::Online, Some("主动探测成功".to_string()))
        }
        Ok(resp) => (
            ConnectivityStatus::Limited,
            Some(format!("主动探测返回 HTTP {}", resp.status())),
        ),
        Err(err) => (
            ConnectivityStatus::Offline,
            Some(format!("主动探测失败: {}", err)),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn record_failure_sets_offline() {
        record_connectivity_failure(ConnectivityStatus::Offline, "offline".to_string()).await;
        let status = check_connectivity_status().await;
        assert!(matches!(status, ConnectivityStatus::Offline));
    }

    #[tokio::test]
    async fn record_success_overrides_failure() {
        record_connectivity_failure(ConnectivityStatus::Offline, "offline".to_string()).await;
        record_connectivity_success().await;
        let status = check_connectivity_status().await;
        assert!(matches!(status, ConnectivityStatus::Online));
    }
}
