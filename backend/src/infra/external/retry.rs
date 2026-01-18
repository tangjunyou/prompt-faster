//! 通用重试机制

use std::time::Duration;

use tokio::time::sleep;

/// 重试策略
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub use_exponential_backoff: bool,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            use_exponential_backoff: true,
            max_delay: Duration::from_secs(30),
        }
    }
}

/// 带重试的异步操作执行器
pub async fn with_retry<F, Fut, T, E, R, L>(
    policy: &RetryPolicy,
    correlation_id: &str,
    operation_name: &str,
    mut operation: F,
    should_retry: R,
    error_type: L,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    R: Fn(&E) -> bool,
    L: Fn(&E) -> &'static str,
{
    let mut last_error = None;

    for attempt in 0..=policy.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_error = Some(err);

                let err_ref = last_error.as_ref().unwrap();
                if attempt < policy.max_retries && should_retry(err_ref) {
                    let delay = calculate_delay(policy, attempt);
                    tracing::info!(
                        correlation_id = %correlation_id,
                        operation = %operation_name,
                        retry_count = attempt + 1,
                        max_retries = policy.max_retries,
                        delay_ms = delay.as_millis(),
                        error_type = %error_type(err_ref),
                        error = %err_ref,
                        "操作失败，准备重试"
                    );
                    sleep(delay).await;
                } else {
                    tracing::error!(
                        correlation_id = %correlation_id,
                        operation = %operation_name,
                        retry_count = attempt + 1,
                        max_retries = policy.max_retries,
                        error_type = %error_type(err_ref),
                        error = %err_ref,
                        "操作失败，已停止重试"
                    );
                    break;
                }
            }
        }
    }

    Err(last_error.expect("retry should capture last error"))
}

fn calculate_delay(policy: &RetryPolicy, attempt: u32) -> Duration {
    if policy.use_exponential_backoff {
        let delay = policy.base_delay * 2u32.pow(attempt);
        std::cmp::min(delay, policy.max_delay)
    } else {
        policy.base_delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn with_retry_succeeds_after_retries() {
        let policy = RetryPolicy {
            max_retries: 2,
            base_delay: Duration::from_millis(1),
            use_exponential_backoff: false,
            max_delay: Duration::from_secs(1),
        };
        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let result = with_retry(
            &policy,
            "cid-1",
            "retry_test",
            || async {
                let current = attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if current < 3 { Err("failed") } else { Ok("ok") }
            },
            |_| true,
            |_| "test_error",
        )
        .await;

        assert_eq!(result.unwrap(), "ok");
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn with_retry_exhausts() {
        let policy = RetryPolicy {
            max_retries: 1,
            base_delay: Duration::from_millis(1),
            use_exponential_backoff: true,
            max_delay: Duration::from_secs(1),
        };
        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let result = with_retry(
            &policy,
            "cid-1",
            "retry_test",
            || async {
                attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err::<(), _>("failed")
            },
            |_| true,
            |_| "test_error",
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
