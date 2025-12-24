use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::shared::time::now_millis;

#[derive(Clone)]
pub struct LoginAttemptStore {
    attempts: Arc<RwLock<HashMap<String, AttemptRecord>>>,
    max_failures: u32,
    cooldown_ms: i64,
}

#[derive(Clone, Debug)]
struct AttemptRecord {
    failure_count: u32,
    cooldown_until_ms: i64,
    expires_at_ms: i64,
}

impl LoginAttemptStore {
    pub fn new(max_failures: u32, cooldown_seconds: u64) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_failures,
            cooldown_ms: (cooldown_seconds * 1000) as i64,
        }
    }

    pub fn default() -> Self {
        Self::new(5, 60)
    }

    pub fn make_key(ip: &str, username: &str) -> String {
        format!("{}|{}", ip, username)
    }

    pub async fn is_blocked(&self, key: &str) -> bool {
        let now = now_millis();
        let attempts = self.attempts.read().await;

        match attempts.get(key) {
            Some(record) if record.expires_at_ms > now => record.cooldown_until_ms > now,
            _ => false,
        }
    }

    pub async fn record_failure(&self, key: String) {
        let now = now_millis();
        let mut attempts = self.attempts.write().await;

        let record = attempts.entry(key).or_insert(AttemptRecord {
            failure_count: 0,
            cooldown_until_ms: 0,
            expires_at_ms: now + self.cooldown_ms,
        });

        if record.expires_at_ms <= now {
            record.failure_count = 0;
            record.cooldown_until_ms = 0;
        }

        record.failure_count += 1;

        if record.failure_count >= self.max_failures {
            record.cooldown_until_ms = now + self.cooldown_ms;
        }

        record.expires_at_ms = std::cmp::max(record.cooldown_until_ms, now + self.cooldown_ms);
    }

    pub async fn reset(&self, key: &str) {
        let mut attempts = self.attempts.write().await;
        attempts.remove(key);
    }

    pub async fn cleanup_expired(&self) -> usize {
        let now = now_millis();
        let mut attempts = self.attempts.write().await;

        let before = attempts.len();
        attempts.retain(|_, record| record.expires_at_ms > now);
        before - attempts.len()
    }
}
