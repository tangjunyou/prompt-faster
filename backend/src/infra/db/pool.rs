//! SQLx 连接池配置

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration;

fn normalize_sqlite_url(database_url: &str) -> String {
    if database_url.starts_with("sqlite::") {
        return database_url.to_string();
    }

    if database_url.starts_with("sqlite:") && !database_url.starts_with("sqlite://") {
        return format!("sqlite://{}", &database_url["sqlite:".len()..]);
    }

    database_url.to_string()
}

/// 创建 SQLite 连接池
pub async fn create_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let database_url = normalize_sqlite_url(database_url);
    let is_in_memory = database_url.contains(":memory:");

    let options = SqliteConnectOptions::from_str(&database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(30))
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        // sqlite::memory: is per-connection; use a single connection to avoid tests becoming flaky.
        .max_connections(if is_in_memory { 1 } else { 5 })
        .connect_with(options)
        .await?;

    Ok(pool)
}

static GLOBAL_DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

/// 初始化全局数据库连接池（用于核心流程访问数据库）
pub fn init_global_db_pool(pool: SqlitePool) {
    let _ = GLOBAL_DB_POOL.set(pool);
}

/// 获取全局数据库连接池（如未初始化则返回 None）
pub fn global_db_pool() -> Option<SqlitePool> {
    GLOBAL_DB_POOL.get().cloned()
}
