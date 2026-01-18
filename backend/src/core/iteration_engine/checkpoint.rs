//! Checkpoint 核心逻辑

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::OnceLock;

use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tracing::{info, warn};

use crate::core::iteration_engine::pause_state::global_pause_registry;
use crate::domain::models::{CheckpointCreateRequest, CheckpointEntity, LineageType};
use crate::domain::types::{
    ArtifactSource, CandidatePrompt, EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT,
    EXT_PREV_ITERATION_STATE, EXT_USER_GUIDANCE, IterationArtifacts, OptimizationContext,
    PatternHypothesis, RunControlState, UserGuidance,
};
use crate::infra::db::pool::global_db_pool;
use crate::infra::db::repositories::{CheckpointRepo, CheckpointRepoError};
use crate::shared::time::now_millis;

pub const CHECKPOINT_MODULE_REGISTERED: bool = true;

const CHECKPOINT_CACHE_LIMIT_DEFAULT: usize = 10;
const CHECKPOINT_CACHE_LIMIT_KEY: &str = "checkpoint.cache_limit";
const CHECKPOINT_MEMORY_ALERT_THRESHOLD_KEY: &str = "checkpoint.memory_alert_threshold";
const IDLE_AUTOSAVE_INTERVAL_MS: i64 = 5 * 60 * 1000;
const IDLE_AUTOSAVE_TICK_SECONDS: u64 = 60;

#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("Checkpoint 数据库未初始化")]
    DatabaseNotInitialized,
    #[error("Checkpoint 仓库错误: {0}")]
    Repository(#[from] CheckpointRepoError),
}

/// 初始化 Checkpoint 缓存的默认配置（从 AppConfig 注入）
pub fn init_checkpoint_cache_defaults(cache_limit: usize, alert_threshold: usize) {
    let defaults = CheckpointCacheDefaults {
        cache_limit: cache_limit.max(1),
        alert_threshold: alert_threshold.max(1),
    };
    let _ = CHECKPOINT_CACHE_DEFAULTS.set(defaults);
}

/// 获取 Checkpoint 缓存默认配置（用于监控/对外展示）
pub fn checkpoint_cache_defaults() -> (usize, usize) {
    let defaults = CHECKPOINT_CACHE_DEFAULTS
        .get()
        .copied()
        .unwrap_or(CheckpointCacheDefaults {
            cache_limit: CHECKPOINT_CACHE_LIMIT_DEFAULT,
            alert_threshold: CHECKPOINT_CACHE_LIMIT_DEFAULT,
        });
    (defaults.cache_limit, defaults.alert_threshold)
}

/// 记录用户介入/状态变更，用于重置空闲保存计时器
pub async fn reset_idle_autosave_timer(task_id: &str) {
    let state = IDLE_AUTOSAVE_STATE.get_or_init(|| Mutex::new(IdleAutoSaveState::default()));
    let mut guard = state.lock().await;
    guard
        .last_saved_at
        .insert(task_id.to_string(), now_millis());
}

#[derive(Default)]
struct CheckpointCache {
    entries: HashMap<String, VecDeque<CheckpointEntity>>,
}

static CHECKPOINT_CACHE: OnceLock<Mutex<CheckpointCache>> = OnceLock::new();
static IDLE_AUTOSAVE_STATE: OnceLock<Mutex<IdleAutoSaveState>> = OnceLock::new();
static IDLE_AUTOSAVE_STARTED: OnceLock<()> = OnceLock::new();
static CHECKPOINT_CACHE_DEFAULTS: OnceLock<CheckpointCacheDefaults> = OnceLock::new();

#[derive(Default)]
struct IdleAutoSaveState {
    last_saved_at: HashMap<String, i64>,
    last_context: HashMap<String, OptimizationContext>,
}

#[derive(Debug, Clone, Copy)]
struct CheckpointCacheDefaults {
    cache_limit: usize,
    alert_threshold: usize,
}

#[derive(Debug, Clone)]
pub struct CheckpointCacheMetrics {
    pub total_tasks: usize,
    pub total_cached: usize,
}

/// 计算 Checkpoint checksum（SHA-256）
pub fn compute_checksum(req: &CheckpointCreateRequest) -> String {
    let mut hasher = Sha256::new();
    hasher.update(req.task_id.as_bytes());
    hasher.update(req.iteration.to_le_bytes());
    hasher.update(stable_json_string(&req.state).as_bytes());
    hasher.update(stable_json_string(&req.run_control_state).as_bytes());
    hasher.update(req.prompt.as_bytes());
    hasher.update(stable_json_string(&req.rule_system).as_bytes());
    if let Some(ref artifacts) = req.artifacts {
        hasher.update(stable_json_string(artifacts).as_bytes());
    }
    if let Some(ref guidance) = req.user_guidance {
        hasher.update(stable_json_string(guidance).as_bytes());
    }
    hasher.update(req.branch_id.as_bytes());
    if let Some(ref parent_id) = req.parent_id {
        hasher.update(parent_id.as_bytes());
    }
    hasher.update(stable_json_string(&req.lineage_type).as_bytes());
    if let Some(ref desc) = req.branch_description {
        hasher.update(desc.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// 校验 Checkpoint 完整性
pub fn verify_checksum(checkpoint: &CheckpointEntity) -> bool {
    let req = CheckpointCreateRequest {
        task_id: checkpoint.task_id.clone(),
        iteration: checkpoint.iteration,
        state: checkpoint.state,
        run_control_state: checkpoint.run_control_state,
        prompt: checkpoint.prompt.clone(),
        rule_system: checkpoint.rule_system.clone(),
        artifacts: checkpoint.artifacts.clone(),
        user_guidance: checkpoint.user_guidance.clone(),
        branch_id: checkpoint.branch_id.clone(),
        parent_id: checkpoint.parent_id.clone(),
        lineage_type: checkpoint.lineage_type.clone(),
        branch_description: checkpoint.branch_description.clone(),
    };
    compute_checksum(&req) == checkpoint.checksum
}

/// 保存 Checkpoint（包含兼容 pause_state 的用户介入数据）
pub async fn save_checkpoint(
    ctx: &OptimizationContext,
    user_id: &str,
    correlation_id: &str,
) -> Result<CheckpointEntity, CheckpointError> {
    let pool = global_db_pool().ok_or(CheckpointError::DatabaseNotInitialized)?;
    let registry = global_pause_registry();
    let controller = registry.get_or_create(&ctx.task_id).await;

    let pause_artifacts = controller.get_artifacts().await;
    let pause_guidance = controller.get_guidance().await;
    let guidance_from_ctx = read_user_guidance_from_context(ctx);
    let user_guidance = pause_guidance.or(guidance_from_ctx);

    let artifacts =
        pause_artifacts.or_else(|| Some(build_iteration_artifacts(ctx, user_guidance.clone())));

    let req = CheckpointCreateRequest {
        task_id: ctx.task_id.clone(),
        iteration: ctx.iteration,
        state: ctx.state,
        run_control_state: ctx.run_control_state,
        prompt: ctx.current_prompt.clone(),
        rule_system: ctx.rule_system.clone(),
        artifacts,
        user_guidance,
        branch_id: ctx.task_id.clone(),
        parent_id: None,
        lineage_type: LineageType::Automatic,
        branch_description: None,
    };

    let checksum = compute_checksum(&req);
    let entity = CheckpointEntity {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: req.task_id,
        iteration: req.iteration,
        state: req.state,
        run_control_state: req.run_control_state,
        prompt: req.prompt,
        rule_system: req.rule_system,
        artifacts: req.artifacts,
        user_guidance: req.user_guidance,
        branch_id: req.branch_id,
        parent_id: req.parent_id,
        lineage_type: req.lineage_type,
        branch_description: req.branch_description,
        checksum,
        created_at: now_millis(),
    };

    let checkpoint = CheckpointRepo::create_checkpoint(&pool, entity.clone()).await?;
    let evicted = cache_checkpoint(ctx, &checkpoint).await;
    record_idle_context(ctx, checkpoint.created_at).await;

    let ctx_user_id = read_optional_string(ctx, "user_id");
    let ctx_correlation_id = read_optional_string(ctx, "correlation_id");
    let safe_user_id = if user_id.trim().is_empty() {
        ctx_user_id.as_deref().unwrap_or("system")
    } else {
        user_id
    };
    let safe_correlation_id = if correlation_id.trim().is_empty() {
        ctx_correlation_id
            .or(controller.get_last_correlation_id().await)
            .unwrap_or_else(|| format!("checkpoint-{}", ctx.task_id))
    } else {
        correlation_id.to_string()
    };

    let new_state = iteration_state_label(checkpoint.state);
    let prev_state =
        read_optional_string(ctx, EXT_PREV_ITERATION_STATE).unwrap_or_else(|| new_state.clone());

    info!(
        correlation_id = %safe_correlation_id,
        user_id = %safe_user_id,
        task_id = %checkpoint.task_id,
        action = "checkpoint_saved",
        prev_state = %prev_state,
        new_state = %new_state,
        iteration_state = %new_state,
        timestamp = checkpoint.created_at,
        "Checkpoint 已保存"
    );

    if let Some(evicted) = evicted {
        info!(
            task_id = %checkpoint.task_id,
            evicted_checkpoint_id = %evicted.id,
            "Checkpoint 内存缓存淘汰旧记录"
        );
    }

    Ok(checkpoint)
}

/// 启动空闲自动保存任务（仅启动一次）
pub fn start_idle_autosave_task() {
    if IDLE_AUTOSAVE_STARTED.set(()).is_err() {
        return;
    }

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(IDLE_AUTOSAVE_TICK_SECONDS)).await;
            if let Err(err) = run_idle_autosave().await {
                tracing::error!(
                    correlation_id = "idle-auto-save",
                    user_id = "system",
                    action = "checkpoint_idle_save_failed",
                    prev_state = "N/A",
                    new_state = "N/A",
                    iteration_state = "N/A",
                    timestamp = now_millis(),
                    error = %err,
                    "空闲自动保存失败（已降级）"
                );
            }
        }
    });
}

/// 获取内存缓存指标（用于监控）
pub async fn checkpoint_cache_metrics() -> CheckpointCacheMetrics {
    let cache = CHECKPOINT_CACHE.get_or_init(|| Mutex::new(CheckpointCache::default()));
    let guard = cache.lock().await;
    let total_tasks = guard.entries.len();
    let total_cached = guard.entries.values().map(|queue| queue.len()).sum();
    CheckpointCacheMetrics {
        total_tasks,
        total_cached,
    }
}

fn read_user_guidance_from_context(ctx: &OptimizationContext) -> Option<UserGuidance> {
    ctx.extensions
        .get(EXT_USER_GUIDANCE)
        .and_then(|value| serde_json::from_value::<UserGuidance>(value.clone()).ok())
}

fn read_optional_string(ctx: &OptimizationContext, key: &str) -> Option<String> {
    ctx.extensions
        .get(key)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

fn read_optional_usize(ctx: &OptimizationContext, key: &str) -> Option<usize> {
    ctx.extensions
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
}

fn build_iteration_artifacts(
    ctx: &OptimizationContext,
    user_guidance: Option<UserGuidance>,
) -> IterationArtifacts {
    let patterns: Vec<PatternHypothesis> = ctx
        .rule_system
        .rules
        .iter()
        .map(|rule| PatternHypothesis {
            id: rule.id.clone(),
            pattern: rule.description.clone(),
            source: ArtifactSource::System,
            confidence: Some(rule.verification_score),
        })
        .collect();

    let mut candidate_prompts: Vec<CandidatePrompt> = Vec::new();
    if !ctx.current_prompt.trim().is_empty() {
        candidate_prompts.push(CandidatePrompt {
            id: "current".to_string(),
            content: ctx.current_prompt.clone(),
            source: ArtifactSource::System,
            score: None,
            is_best: false,
        });
    }

    let best_prompt = read_optional_string(ctx, EXT_BEST_CANDIDATE_PROMPT);
    let best_index = read_optional_usize(ctx, EXT_BEST_CANDIDATE_INDEX);

    if let Some(best_prompt) = best_prompt {
        if candidate_prompts
            .first()
            .map(|p| p.content == best_prompt)
            .unwrap_or(false)
        {
            if let Some(current) = candidate_prompts.first_mut() {
                current.is_best = true;
            }
        } else {
            let id = best_index
                .map(|idx| format!("candidate:{idx}"))
                .unwrap_or_else(|| "best".to_string());
            candidate_prompts.push(CandidatePrompt {
                id,
                content: best_prompt,
                source: ArtifactSource::System,
                score: None,
                is_best: true,
            });
        }
    } else if let Some(current) = candidate_prompts.first_mut() {
        current.is_best = true;
    }

    IterationArtifacts {
        patterns,
        candidate_prompts,
        user_guidance,
        updated_at: crate::shared::ws::chrono_timestamp(),
    }
}

fn iteration_state_label(state: crate::domain::models::IterationState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn stable_json_string<T: Serialize>(value: &T) -> String {
    let value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
    let normalized = canonicalize_json(&value);
    serde_json::to_string(&normalized).unwrap_or_default()
}

fn canonicalize_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted: BTreeMap<String, serde_json::Value> = BTreeMap::new();
            for (key, val) in map {
                sorted.insert(key.clone(), canonicalize_json(val));
            }
            serde_json::Value::Object(sorted.into_iter().collect())
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_json).collect())
        }
        other => other.clone(),
    }
}

async fn cache_checkpoint(
    ctx: &OptimizationContext,
    checkpoint: &CheckpointEntity,
) -> Option<CheckpointEntity> {
    let limit = checkpoint_cache_limit(ctx);
    let alert_threshold = checkpoint_alert_threshold(ctx, limit);

    let cache = CHECKPOINT_CACHE.get_or_init(|| Mutex::new(CheckpointCache::default()));
    let mut guard = cache.lock().await;
    let queue = guard
        .entries
        .entry(checkpoint.task_id.clone())
        .or_insert_with(VecDeque::new);
    queue.push_back(checkpoint.clone());

    if queue.len() >= alert_threshold {
        warn!(
            task_id = %checkpoint.task_id,
            cache_len = queue.len(),
            threshold = alert_threshold,
            "Checkpoint 内存缓存接近阈值"
        );
    }

    if queue.len() > limit {
        return queue.pop_front();
    }

    None
}

async fn record_idle_context(ctx: &OptimizationContext, saved_at: i64) {
    let state = IDLE_AUTOSAVE_STATE.get_or_init(|| Mutex::new(IdleAutoSaveState::default()));
    let mut guard = state.lock().await;
    guard.last_context.insert(ctx.task_id.clone(), ctx.clone());
    guard.last_saved_at.insert(ctx.task_id.clone(), saved_at);
}

async fn run_idle_autosave() -> Result<(), CheckpointError> {
    let state = IDLE_AUTOSAVE_STATE.get_or_init(|| Mutex::new(IdleAutoSaveState::default()));
    let now = now_millis();
    let snapshots: Vec<(String, OptimizationContext, i64)> = {
        let guard = state.lock().await;
        guard
            .last_context
            .iter()
            .map(|(task_id, ctx)| {
                let last_saved = guard.last_saved_at.get(task_id).copied().unwrap_or(0);
                (task_id.clone(), ctx.clone(), last_saved)
            })
            .collect()
    };

    for (task_id, ctx, last_saved) in snapshots {
        if ctx.run_control_state == RunControlState::Running {
            continue;
        }
        if now.saturating_sub(last_saved) < IDLE_AUTOSAVE_INTERVAL_MS {
            continue;
        }

        if let Err(err) = save_checkpoint(&ctx, "system", "idle-auto-save").await {
            tracing::error!(
                correlation_id = "idle-auto-save",
                user_id = "system",
                task_id = %task_id,
                action = "checkpoint_idle_save_failed",
                prev_state = ?ctx.state,
                new_state = ?ctx.state,
                iteration_state = ?ctx.state,
                timestamp = now_millis(),
                error = %err,
                "空闲自动保存失败（已降级）"
            );
        }
    }

    Ok(())
}

fn checkpoint_cache_limit(ctx: &OptimizationContext) -> usize {
    let defaults = CHECKPOINT_CACHE_DEFAULTS
        .get()
        .copied()
        .unwrap_or(CheckpointCacheDefaults {
            cache_limit: CHECKPOINT_CACHE_LIMIT_DEFAULT,
            alert_threshold: CHECKPOINT_CACHE_LIMIT_DEFAULT,
        });
    read_optional_usize(ctx, CHECKPOINT_CACHE_LIMIT_KEY)
        .unwrap_or(defaults.cache_limit)
        .max(1)
}

fn checkpoint_alert_threshold(ctx: &OptimizationContext, _limit: usize) -> usize {
    let defaults = CHECKPOINT_CACHE_DEFAULTS
        .get()
        .copied()
        .unwrap_or(CheckpointCacheDefaults {
            cache_limit: CHECKPOINT_CACHE_LIMIT_DEFAULT,
            alert_threshold: CHECKPOINT_CACHE_LIMIT_DEFAULT,
        });
    read_optional_usize(ctx, CHECKPOINT_MEMORY_ALERT_THRESHOLD_KEY)
        .unwrap_or(defaults.alert_threshold)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{IterationState, RuleSystem};
    use crate::domain::types::{ExecutionTargetConfig, OptimizationConfig};
    use crate::infra::db::pool::{create_pool, init_global_db_pool};
    use crate::infra::db::repositories::CheckpointRepo;
    use sqlx::SqlitePool;
    use std::collections::HashMap;
    use std::sync::OnceLock;

    static TEST_POOL: OnceLock<SqlitePool> = OnceLock::new();
    static TEST_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

    async fn setup_test_pool() -> SqlitePool {
        if let Some(pool) = TEST_POOL.get() {
            return pool.clone();
        }

        let db_path = std::env::temp_dir().join(format!(
            "checkpoint_idle_autosave_{}.db",
            std::process::id()
        ));
        let db_url = format!("sqlite:{}", db_path.display());
        let pool = create_pool(&db_url).await.expect("创建测试数据库失败");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("运行 migrations 失败");
        init_global_db_pool(pool.clone());
        let _ = TEST_POOL.set(pool.clone());
        pool
    }

    async fn reset_test_state(pool: &SqlitePool) {
        if let Some(cache) = CHECKPOINT_CACHE.get() {
            let mut guard = cache.lock().await;
            guard.entries.clear();
        }
        if let Some(state) = IDLE_AUTOSAVE_STATE.get() {
            let mut guard = state.lock().await;
            guard.last_saved_at.clear();
            guard.last_context.clear();
        }

        sqlx::query("DELETE FROM checkpoints")
            .execute(pool)
            .await
            .expect("清理 checkpoints 失败");
        sqlx::query("DELETE FROM optimization_tasks")
            .execute(pool)
            .await
            .expect("清理 optimization_tasks 失败");
        sqlx::query("DELETE FROM workspaces")
            .execute(pool)
            .await
            .expect("清理 workspaces 失败");
        sqlx::query("DELETE FROM users")
            .execute(pool)
            .await
            .expect("清理 users 失败");
    }

    async fn seed_user_workspace_task(
        db: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        task_id: &str,
    ) {
        let now = now_millis();
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(user_id)
        .bind(user_id)
        .bind("hashed")
        .bind(now)
        .bind(now)
        .execute(db)
        .await
        .expect("insert user");

        sqlx::query(
            r#"
            INSERT INTO workspaces (id, user_id, name, description, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(workspace_id)
        .bind(user_id)
        .bind("ws-test")
        .bind(None::<String>)
        .bind(now)
        .bind(now)
        .execute(db)
        .await
        .expect("insert workspace");

        sqlx::query(
            r#"
            INSERT INTO optimization_tasks
              (id, workspace_id, name, description, goal, execution_target_type, task_mode, status, config_json, created_at, updated_at)
            VALUES
              (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10)
            "#,
        )
        .bind(task_id)
        .bind(workspace_id)
        .bind("ws-task")
        .bind(None::<String>)
        .bind("goal")
        .bind("generic")
        .bind("fixed")
        .bind("draft")
        .bind(now)
        .bind(now)
        .execute(db)
        .await
        .expect("insert task");
    }

    fn build_context(task_id: &str, run_control_state: RunControlState) -> OptimizationContext {
        OptimizationContext {
            task_id: task_id.to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: "prompt".to_string(),
            rule_system: RuleSystem {
                rules: vec![],
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 1,
            },
            iteration: 1,
            state: IterationState::Completed,
            run_control_state,
            test_cases: vec![],
            config: OptimizationConfig::default(),
            checkpoints: vec![],
            extensions: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn idle_autosave_saves_when_idle_and_overdue() {
        let _guard = TEST_LOCK
            .get_or_init(|| tokio::sync::Mutex::new(()))
            .lock()
            .await;
        let pool = setup_test_pool().await;
        reset_test_state(&pool).await;

        let task_id = "task-idle-save";
        seed_user_workspace_task(&pool, "user-idle", "workspace-idle", task_id).await;

        let ctx = build_context(task_id, RunControlState::Idle);
        record_idle_context(&ctx, now_millis() - IDLE_AUTOSAVE_INTERVAL_MS - 1).await;

        run_idle_autosave().await.expect("运行空闲保存失败");

        let list = CheckpointRepo::list_checkpoints_by_task(&pool, task_id, 10)
            .await
            .expect("查询 checkpoint 失败");
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn idle_autosave_skips_when_running() {
        let _guard = TEST_LOCK
            .get_or_init(|| tokio::sync::Mutex::new(()))
            .lock()
            .await;
        let pool = setup_test_pool().await;
        reset_test_state(&pool).await;

        let task_id = "task-running";
        seed_user_workspace_task(&pool, "user-running", "workspace-running", task_id).await;

        let ctx = build_context(task_id, RunControlState::Running);
        record_idle_context(&ctx, now_millis() - IDLE_AUTOSAVE_INTERVAL_MS - 1).await;

        run_idle_autosave().await.expect("运行空闲保存失败");

        let list = CheckpointRepo::list_checkpoints_by_task(&pool, task_id, 10)
            .await
            .expect("查询 checkpoint 失败");
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn idle_autosave_resets_after_interaction() {
        let _guard = TEST_LOCK
            .get_or_init(|| tokio::sync::Mutex::new(()))
            .lock()
            .await;
        let pool = setup_test_pool().await;
        reset_test_state(&pool).await;

        let task_id = "task-reset";
        seed_user_workspace_task(&pool, "user-reset", "workspace-reset", task_id).await;

        let ctx = build_context(task_id, RunControlState::Idle);
        record_idle_context(&ctx, now_millis() - IDLE_AUTOSAVE_INTERVAL_MS - 1).await;
        reset_idle_autosave_timer(task_id).await;

        run_idle_autosave().await.expect("运行空闲保存失败");

        let list = CheckpointRepo::list_checkpoints_by_task(&pool, task_id, 10)
            .await
            .expect("查询 checkpoint 失败");
        assert_eq!(list.len(), 0);
    }
}
