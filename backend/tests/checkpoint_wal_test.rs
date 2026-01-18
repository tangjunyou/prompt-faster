use prompt_faster::infra::db::pool::create_pool;
use uuid::Uuid;

#[tokio::test]
async fn sqlite_wal_mode_enabled() {
    let db_path = std::env::temp_dir().join(format!("checkpoint_wal_{}.db", Uuid::new_v4()));
    let db_url = format!("sqlite:{}", db_path.display());
    let pool = create_pool(&db_url).await.expect("创建数据库失败");

    let mode: String = sqlx::query_scalar("PRAGMA journal_mode;")
        .fetch_one(&pool)
        .await
        .expect("查询 journal_mode 失败");

    assert_eq!(mode.to_lowercase(), "wal");

    let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous;")
        .fetch_one(&pool)
        .await
        .expect("查询 synchronous 失败");
    assert_eq!(synchronous, 2);
}
