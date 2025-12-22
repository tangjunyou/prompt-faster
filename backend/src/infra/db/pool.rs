//! SQLx 连接池配置

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::str::FromStr;

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

    let options = SqliteConnectOptions::from_str(&database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}
