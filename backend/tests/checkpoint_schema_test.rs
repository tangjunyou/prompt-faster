use prompt_faster::infra::db::pool::create_pool;

#[tokio::test]
async fn test_checkpoints_table_exists() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("运行 migrations 失败");

    let row = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='checkpoints'")
        .fetch_optional(&pool)
        .await
        .expect("查询 sqlite_master 失败");

    assert!(row.is_some(), "checkpoints 表应存在");
}
