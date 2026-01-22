use std::sync::Once;

use prompt_faster::core::diversity_analyzer::{DefaultDiversityAnalyzer, DiversityAnalyzer};
use prompt_faster::domain::models::{DiversityConfig, DiversityMetrics};
use prompt_faster::infra::db::pool::{create_pool, init_global_db_pool};
use prompt_faster::infra::db::repositories::DiversityBaselineRepo;
use sqlx::SqlitePool;

static INIT: Once = Once::new();

async fn setup_pool() -> SqlitePool {
    let pool = create_pool("sqlite::memory:").await.expect("create pool");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS diversity_baselines (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL UNIQUE,
            metrics_json TEXT NOT NULL,
            recorded_at INTEGER NOT NULL,
            iteration INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create diversity_baselines");

    INIT.call_once(|| {
        init_global_db_pool(pool.clone());
    });

    pool
}

#[tokio::test]
async fn baseline_insert_if_absent_is_idempotent() {
    let pool = setup_pool().await;
    let metrics = DiversityMetrics {
        lexical_diversity: 0.2,
        structural_diversity: 0.3,
        semantic_diversity: 0.0,
        overall_score: 0.25,
    };

    DiversityBaselineRepo::insert_if_absent(&pool, "task-1", &metrics, 1)
        .await
        .expect("insert baseline");
    DiversityBaselineRepo::insert_if_absent(&pool, "task-1", &metrics, 2)
        .await
        .expect("second insert baseline");

    let baseline = DiversityBaselineRepo::get_by_task_id(&pool, "task-1")
        .await
        .expect("get baseline")
        .expect("baseline exists");
    assert_eq!(baseline.metrics.overall_score, 0.25);
    assert_eq!(baseline.iteration, 1);
    assert!(baseline.recorded_at.contains('T'));
}

#[tokio::test]
async fn baseline_upsert_updates_metrics() {
    let pool = setup_pool().await;
    let metrics_v1 = DiversityMetrics {
        lexical_diversity: 0.1,
        structural_diversity: 0.1,
        semantic_diversity: 0.0,
        overall_score: 0.1,
    };
    let metrics_v2 = DiversityMetrics {
        lexical_diversity: 0.6,
        structural_diversity: 0.7,
        semantic_diversity: 0.0,
        overall_score: 0.65,
    };

    let _ = DiversityBaselineRepo::upsert(&pool, "task-2", &metrics_v1, 1)
        .await
        .expect("upsert v1");
    let updated = DiversityBaselineRepo::upsert(&pool, "task-2", &metrics_v2, 2)
        .await
        .expect("upsert v2");
    assert_eq!(updated.metrics.overall_score, 0.65);
    assert_eq!(updated.iteration, 2);

    let baseline = DiversityBaselineRepo::get_by_task_id(&pool, "task-2")
        .await
        .expect("get baseline")
        .expect("baseline exists");
    assert_eq!(baseline.metrics.structural_diversity, 0.7);
}

#[test]
fn analyzer_returns_zero_for_single_output() {
    let analyzer = DefaultDiversityAnalyzer::new(DiversityConfig::default());
    let outputs = vec!["only one".to_string()];
    let analysis = analyzer.analyze(&outputs, None, None);
    assert_eq!(analysis.metrics.overall_score, 0.0);
}
