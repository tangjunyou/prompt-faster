use reqwest::Client;
use sqlx::SqlitePool;

/// 应用状态
///
/// 包含共享的数据库连接池和 HTTP 客户端
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池
    pub db: SqlitePool,
    /// HTTP 客户端（复用连接池，提高性能）
    pub http_client: Client,
}
