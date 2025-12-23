use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::infra::external::api_key_manager::ApiKeyManager;

/// 应用状态
///
/// 包含共享的数据库连接池、HTTP 客户端和 API Key 管理器
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池
    pub db: SqlitePool,
    /// HTTP 客户端（复用连接池，提高性能）
    pub http_client: Client,
    /// API Key 管理器（加解密）
    /// TODO(Story-1.6): 替换为用户登录密码派生
    pub api_key_manager: Arc<ApiKeyManager>,
}
