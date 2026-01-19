use prompt_faster::infra::db::pool::create_pool;
use sqlx::Row;

#[tokio::test]
async fn test_checkpoints_table_exists() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("运行 migrations 失败");

    let row =
        sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='checkpoints'")
            .fetch_optional(&pool)
            .await
            .expect("查询 sqlite_master 失败");

    assert!(row.is_some(), "checkpoints 表应存在");
}

#[tokio::test]
async fn test_checkpoints_table_has_rollback_columns() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("运行 migrations 失败");

    let rows = sqlx::query("PRAGMA table_info(checkpoints)")
        .fetch_all(&pool)
        .await
        .expect("查询 checkpoints 表结构失败");

    let mut columns = Vec::new();
    for row in rows {
        let name: String = row.try_get("name").expect("读取 checkpoints 列名失败");
        columns.push(name);
    }

    assert!(columns.contains(&"archived_at".to_string()));
    assert!(columns.contains(&"archive_reason".to_string()));
    assert!(columns.contains(&"pass_rate_summary".to_string()));
}
