use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::api::middleware::{LoginAttemptStore, SessionStore};
use crate::infra::external::api_key_manager::ApiKeyManager;
use crate::shared::config::AppConfig;

/// 应用状态
///
/// 包含共享的数据库连接池、HTTP 客户端、API Key 管理器和会话存储
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池
    pub db: SqlitePool,
    /// HTTP 客户端（复用连接池，提高性能）
    pub http_client: Client,
    /// 应用配置
    pub config: Arc<AppConfig>,
    /// API Key 管理器（加解密）
    pub api_key_manager: Arc<ApiKeyManager>,
    /// 会话存储（内存）
    pub session_store: SessionStore,
    /// 登录尝试存储（内存）
    pub login_attempt_store: LoginAttemptStore,
}
