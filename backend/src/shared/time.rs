//! 时间戳工具函数
//! 统一使用 Unix 毫秒时间戳 (AR3)

/// 获取当前 Unix 毫秒时间戳
pub fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_millis_returns_positive() {
        let ts = now_millis();
        assert!(ts > 0);
    }

    #[test]
    fn test_now_millis_is_reasonable() {
        let ts = now_millis();
        // 2024-01-01 00:00:00 UTC = 1704067200000
        assert!(ts > 1704067200000);
    }
}
