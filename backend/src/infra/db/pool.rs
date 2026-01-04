//! SQLx 连接池配置

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use std::str::FromStr;
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

    let options = SqliteConnectOptions::from_str(&database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(30))
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}
