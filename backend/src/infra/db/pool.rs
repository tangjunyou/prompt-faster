//! SQLx 连接池配置

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// 创建 SQLite 连接池
pub async fn create_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // 启用 WAL 模式和 FULL synchronous (NFR6)
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA synchronous=FULL")
        .execute(&pool)
        .await?;

    Ok(pool)
}
