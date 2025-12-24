//! 会话管理模块
//! 提供内存会话存储和会话管理功能

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use zeroize::Zeroizing;

use crate::shared::time::now_millis;

/// 会话数据
#[derive(Debug, Clone)]
pub struct SessionData {
    /// 用户 ID
    pub user_id: String,
    /// 过期时间戳（Unix 毫秒）
    pub expires_at: i64,
    /// 解锁上下文（用于 API Key 加解密的派生材料）
    /// 注意：登出/过期时必须清除此字段
    pub unlock_context: Option<UnlockContext>,
}

/// 解锁上下文
/// 用于存放可再派生材料（用户密码的内存副本）
///
/// Code Review Fix (Story 1.6):
/// - 使用 zeroize 进行内存清零
/// - Drop 时自动清除密码内存
#[derive(Clone)]
pub struct UnlockContext {
    /// 用户密码（仅存在内存中，登出时清除）
    /// 使用 zeroize 确保 Drop 时内存清零
    password: Arc<Zeroizing<Vec<u8>>>,
}

impl std::fmt::Debug for UnlockContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnlockContext")
            .field("password", &"<redacted>")
            .finish()
    }
}

impl UnlockContext {
    /// 创建解锁上下文
    pub fn new(password: String) -> Self {
        Self {
            password: Arc::new(Zeroizing::new(password.into_bytes())),
        }
    }

    /// 获取密码引用（用于派生加密密钥）
    pub fn password_bytes(&self) -> &[u8] {
        self.password.as_ref().as_slice()
    }

    pub fn password_str(&self) -> Option<&str> {
        std::str::from_utf8(self.password_bytes()).ok()
    }
}

/// 会话存储
/// 使用 Arc<RwLock<HashMap>> 实现并发安全的内存存储
#[derive(Clone)]
pub struct SessionStore {
    /// 会话映射: session_token -> SessionData
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    /// 会话过期时间（毫秒）
    session_ttl_ms: i64,
}

impl SessionStore {
    /// 创建新的会话存储
    ///
    /// # 参数
    /// - `session_ttl_hours`: 会话过期时间（小时），默认 24 小时
    pub fn new(session_ttl_hours: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_ttl_ms: (session_ttl_hours * 60 * 60 * 1000) as i64,
        }
    }

    /// 创建新会话
    ///
    /// # 参数
    /// - `user_id`: 用户 ID
    /// - `unlock_context`: 可选的解锁上下文
    ///
    /// # 返回
    /// 新创建的会话 token（UUID v4 格式，≥ 120 bits 随机性）
    pub async fn create_session(
        &self,
        user_id: String,
        unlock_context: Option<UnlockContext>,
    ) -> String {
        let session_token = uuid::Uuid::new_v4().to_string();
        let expires_at = now_millis() + self.session_ttl_ms;

        let session_data = SessionData {
            user_id,
            expires_at,
            unlock_context,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_token.clone(), session_data);

        session_token
    }

    /// 验证会话并获取会话数据
    ///
    /// # 参数
    /// - `session_token`: 会话 token
    ///
    /// # 返回
    /// - 有效会话返回 `Some(SessionData)`
    /// - 无效或过期会话返回 `None`
    pub async fn validate_session(&self, session_token: &str) -> Option<SessionData> {
        let sessions = self.sessions.read().await;

        if let Some(session) = sessions.get(session_token) {
            let now = now_millis();
            if session.expires_at > now {
                return Some(session.clone());
            }
        }

        None
    }

    /// 移除会话（登出）
    ///
    /// # 注意
    /// 移除会话时会清除 unlock_context
    pub async fn remove_session(&self, session_token: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_token).is_some()
    }

    /// 清理过期会话
    ///
    /// 定期调用此方法以防止内存泄漏
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let now = now_millis();
        let mut sessions = self.sessions.write().await;

        let before_count = sessions.len();
        sessions.retain(|_, session| session.expires_at > now);
        let after_count = sessions.len();

        before_count - after_count
    }

    /// 获取当前活跃会话数量
    pub async fn active_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new(24)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_validate_session() {
        let store = SessionStore::new(1); // 1 小时过期

        let token = store.create_session("user123".to_string(), None).await;

        let session = store.validate_session(&token).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().user_id, "user123");
    }

    #[tokio::test]
    async fn test_invalid_token_returns_none() {
        let store = SessionStore::default();

        let session = store.validate_session("invalid_token").await;
        assert!(session.is_none());
    }

    #[tokio::test]
    async fn test_remove_session() {
        let store = SessionStore::default();

        let token = store.create_session("user123".to_string(), None).await;

        assert!(store.validate_session(&token).await.is_some());

        let removed = store.remove_session(&token).await;
        assert!(removed);

        assert!(store.validate_session(&token).await.is_none());
    }

    #[tokio::test]
    async fn test_session_with_unlock_context() {
        let store = SessionStore::default();

        let unlock_ctx = UnlockContext::new("user_password".to_string());
        let token = store
            .create_session("user123".to_string(), Some(unlock_ctx))
            .await;

        let session = store.validate_session(&token).await.unwrap();
        assert!(session.unlock_context.is_some());
        assert_eq!(
            session.unlock_context.unwrap().password_str(),
            Some("user_password")
        );
    }
}
