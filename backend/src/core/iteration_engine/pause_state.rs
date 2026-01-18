//! 暂停状态管理
//!
//! 本模块实现最小暂停持久化与恢复机制。
//! 注意：这是临时实现，Epic 7 完成后将替换为完整 Checkpoint 机制。

use crate::domain::types::{IterationArtifacts, RunControlState, UserGuidance};
use crate::shared::ws::{
    EVT_ITERATION_PAUSED, EVT_ITERATION_RESUMED, IterationPausedPayload, IterationResumedPayload,
    WsMessage, chrono_timestamp,
};
use crate::shared::ws_bus::global_ws_bus;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, Notify};
use tracing::{info, warn};

/// 暂停状态快照（用于持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PauseStateSnapshot {
    /// 任务 ID
    pub task_id: String,
    /// 暂停时间（ISO 8601）
    pub paused_at: String,
    /// 触发暂停的 correlationId（AR2）
    pub correlation_id: String,
    /// 用户 ID（用于审计）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// 暂停时的运行控制状态
    pub run_control_state: RunControlState,
    /// 暂停时的迭代轮次
    pub iteration: u32,
    /// 暂停时所处的阶段
    pub stage: String,
    /// 最小上下文快照（用于重启恢复）
    #[serde(default)]
    pub context_snapshot: Value,
}

/// 暂停控制器（每个任务一个实例）
#[derive(Debug)]
pub struct PauseController {
    /// 任务 ID
    task_id: String,
    /// 暂停请求标志（原子布尔值，支持并发安全检查）
    pause_requested: AtomicBool,
    /// 终止请求标志（原子布尔值，支持并发安全检查）
    stop_requested: AtomicBool,
    /// 当前是否处于暂停状态
    is_paused: AtomicBool,
    /// 继续通知器（用于 await 暂停恢复）
    resume_notify: Notify,
    /// 暂停状态快照（用于持久化）
    snapshot: Mutex<Option<PauseStateSnapshot>>,
    /// 运行中配置变更（仅存内存）
    max_iterations_override: Mutex<Option<u32>>,
    /// 最后一次操作的 correlationId（用于幂等性检查）
    last_correlation_id: Mutex<Option<String>>,
    /// 最后一次操作的用户 ID
    last_user_id: Mutex<Option<String>>,
}

struct GuidanceLogFields {
    task_id: String,
    correlation_id: String,
    user_id: String,
    action: String,
    artifact_type: String,
    edit_action: String,
    prev_state: String,
    new_state: String,
    prev_guidance_state: String,
    new_guidance_state: String,
    guidance_preview: String,
    iteration_state: String,
    timestamp: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PauseStateError {
    #[error("暂停状态持久化失败: {0}")]
    Persist(String),
    #[error("暂停状态恢复失败: {0}")]
    Restore(String),
}

/// artifacts 存储在 context_snapshot 中的键名
const ARTIFACTS_KEY: &str = "artifacts";

impl PauseController {
    /// 创建新的暂停控制器
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            pause_requested: AtomicBool::new(false),
            stop_requested: AtomicBool::new(false),
            is_paused: AtomicBool::new(false),
            resume_notify: Notify::new(),
            snapshot: Mutex::new(None),
            max_iterations_override: Mutex::new(None),
            last_correlation_id: Mutex::new(None),
            last_user_id: Mutex::new(None),
        }
    }

    /// 获取任务 ID
    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    /// 请求暂停（幂等）
    ///
    /// 返回 `true` 表示暂停请求已被接受（首次或状态变更）
    /// 返回 `false` 表示已经在暂停状态或已请求暂停（幂等）
    pub async fn request_pause(&self, correlation_id: &str, user_id: &str) -> bool {
        // 幂等性检查：如果已经暂停或已请求暂停，不重复处理
        if self.is_paused.load(Ordering::SeqCst) {
            info!(
                task_id = %self.task_id,
                correlation_id = %correlation_id,
                "暂停请求被忽略：任务已处于暂停状态（幂等）"
            );
            return false;
        }

        let was_requested = self.pause_requested.swap(true, Ordering::SeqCst);
        if was_requested {
            info!(
                task_id = %self.task_id,
                correlation_id = %correlation_id,
                "暂停请求被忽略：已有暂停请求待处理（幂等）"
            );
            return false;
        }

        // 记录 correlationId / user_id
        *self.last_correlation_id.lock().await = Some(correlation_id.to_string());
        *self.last_user_id.lock().await = Some(user_id.to_string());

        info!(
            task_id = %self.task_id,
            correlation_id = %correlation_id,
            "暂停请求已接受，将在下一个安全点生效"
        );
        true
    }

    /// 请求终止（幂等）
    ///
    /// 返回 `true` 表示终止请求已被接受（首次或状态变更）
    /// 返回 `false` 表示已存在终止请求（幂等）
    pub async fn request_stop(&self, correlation_id: &str, user_id: &str) -> bool {
        let was_requested = self.stop_requested.swap(true, Ordering::SeqCst);
        if was_requested {
            info!(
                task_id = %self.task_id,
                correlation_id = %correlation_id,
                "终止请求被忽略：已有终止请求待处理（幂等）"
            );
            return false;
        }

        *self.last_correlation_id.lock().await = Some(correlation_id.to_string());
        *self.last_user_id.lock().await = Some(user_id.to_string());

        if self.is_paused.load(Ordering::SeqCst) {
            self.is_paused.store(false, Ordering::SeqCst);
            self.pause_requested.store(false, Ordering::SeqCst);
            self.resume_notify.notify_waiters();
        }

        info!(
            task_id = %self.task_id,
            correlation_id = %correlation_id,
            user_id = %user_id,
            "终止请求已记录"
        );

        true
    }

    /// 是否已请求终止
    pub fn is_stop_requested(&self) -> bool {
        self.stop_requested.load(Ordering::SeqCst)
    }

    /// 清理终止请求标志（用于任务结束后复用控制器）
    pub fn clear_stop_requested(&self) {
        self.stop_requested.store(false, Ordering::SeqCst);
    }

    /// 设置运行中最大迭代轮数（用于增量更新）
    pub async fn set_max_iterations_override(&self, max_iterations: u32) {
        *self.max_iterations_override.lock().await = Some(max_iterations);
    }

    /// 获取运行中最大迭代轮数（若无则返回 None）
    pub async fn get_max_iterations_override(&self) -> Option<u32> {
        *self.max_iterations_override.lock().await
    }

    /// 请求继续（幂等）
    ///
    /// 返回 `true` 表示继续请求已被接受
    /// 返回 `false` 表示未处于暂停状态（幂等）
    pub async fn request_resume(&self, correlation_id: &str, user_id: &str) -> bool {
        if !self.is_paused.load(Ordering::SeqCst) {
            info!(
                task_id = %self.task_id,
                correlation_id = %correlation_id,
                "继续请求被忽略：任务未处于暂停状态（幂等）"
            );
            return false;
        }

        // 记录 correlationId / user_id
        *self.last_correlation_id.lock().await = Some(correlation_id.to_string());
        *self.last_user_id.lock().await = Some(user_id.to_string());

        // 清除暂停状态
        self.is_paused.store(false, Ordering::SeqCst);
        self.pause_requested.store(false, Ordering::SeqCst);

        // 先清理落盘快照，避免新的连接误判为已暂停
        if let Err(err) = clear_snapshot_file(&self.task_id) {
            warn!(task_id = %self.task_id, error = %err, "清理暂停快照文件失败");
        }

        // 通知等待的任务继续执行
        self.resume_notify.notify_waiters();

        // 推送 resumed 事件（不阻塞）
        emit_resumed_event(&self.task_id, correlation_id);

        info!(
            task_id = %self.task_id,
            correlation_id = %correlation_id,
            "继续请求已接受，任务将恢复执行"
        );
        true
    }

    /// 检查是否有暂停请求
    pub fn is_pause_requested(&self) -> bool {
        self.pause_requested.load(Ordering::SeqCst)
    }

    /// 检查是否处于暂停状态
    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    /// 在安全点执行暂停（Layer 完成后调用）
    ///
    /// 如果有暂停请求，设置暂停状态并保存快照
    pub async fn checkpoint_pause(
        &self,
        iteration: u32,
        stage: &str,
        correlation_id: Option<&str>,
        context_snapshot: Value,
    ) -> Result<bool, PauseStateError> {
        if !self.pause_requested.load(Ordering::SeqCst) {
            return Ok(false);
        }

        // 设置暂停状态
        self.is_paused.store(true, Ordering::SeqCst);

        let last_correlation_id = self.last_correlation_id.lock().await.clone();
        let correlation_id = correlation_id
            .map(|s| s.to_string())
            .or(last_correlation_id)
            .unwrap_or_else(|| format!("system-{}", chrono_timestamp()));
        let user_id = self.last_user_id.lock().await.clone();

        // 创建快照
        let snapshot = PauseStateSnapshot {
            task_id: self.task_id.clone(),
            paused_at: chrono_timestamp(),
            correlation_id: correlation_id.clone(),
            user_id: user_id.clone(),
            run_control_state: RunControlState::Paused,
            iteration,
            stage: stage.to_string(),
            context_snapshot,
        };

        *self.snapshot.lock().await = Some(snapshot.clone());
        persist_snapshot(&snapshot)?;

        // 推送 paused 事件（不阻塞）
        emit_paused_event(&snapshot);

        info!(
            task_id = %self.task_id,
            iteration = iteration,
            stage = %stage,
            correlation_id = %snapshot.correlation_id,
            user_id = %user_id.clone().unwrap_or_else(|| "unknown".to_string()),
            prev_state = ?RunControlState::Running,
            new_state = ?RunControlState::Paused,
            "任务已在安全点暂停"
        );

        Ok(true)
    }

    /// 等待继续信号
    pub async fn wait_for_resume(&self) {
        if self.is_paused.load(Ordering::SeqCst) {
            info!(task_id = %self.task_id, "等待继续信号...");
            self.resume_notify.notified().await;
            info!(task_id = %self.task_id, "收到继续信号，恢复执行");
        }
    }

    /// 获取暂停状态快照
    pub async fn get_snapshot(&self) -> Option<PauseStateSnapshot> {
        self.snapshot.lock().await.clone()
    }

    /// 获取最后一次操作的 correlationId
    pub async fn get_last_correlation_id(&self) -> Option<String> {
        self.last_correlation_id.lock().await.clone()
    }

    /// 获取当前产物（从 context_snapshot 中提取）
    pub async fn get_artifacts(&self) -> Option<IterationArtifacts> {
        let snapshot = self.snapshot.lock().await;
        let snapshot = snapshot.as_ref()?;
        let artifacts_value = snapshot.context_snapshot.get(ARTIFACTS_KEY)?;
        serde_json::from_value(artifacts_value.clone()).ok()
    }

    /// 更新产物（存储到 context_snapshot 中）
    ///
    /// 仅在 Paused 状态下允许更新
    /// 返回 `Ok(updated_artifacts)` 表示更新成功
    /// 返回 `Err(reason)` 表示更新失败
    pub async fn update_artifacts(
        &self,
        updated: &IterationArtifacts,
        correlation_id: &str,
        user_id: &str,
    ) -> Result<IterationArtifacts, PauseStateError> {
        // 状态校验：仅 Paused 状态允许编辑
        if !self.is_paused.load(Ordering::SeqCst) {
            return Err(PauseStateError::Persist(
                "任务未处于暂停状态，无法编辑产物".to_string(),
            ));
        }

        let mut snapshot_guard = self.snapshot.lock().await;
        let snapshot = snapshot_guard
            .as_mut()
            .ok_or_else(|| PauseStateError::Persist("暂停快照不存在".to_string()))?;

        // 获取当前产物用于验证
        let current_artifacts: Option<IterationArtifacts> = snapshot
            .context_snapshot
            .get(ARTIFACTS_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        if current_artifacts.is_none() {
            return Err(PauseStateError::Persist(
                "当前暂停快照不存在可编辑产物".to_string(),
            ));
        }

        // 验证更新合法性（禁止新增 ID）
        if let Some(ref current) = current_artifacts {
            current
                .validate_update(updated)
                .map_err(PauseStateError::Persist)?;
        }

        // 验证内容长度限制
        updated
            .validate_content_length()
            .map_err(PauseStateError::Persist)?;

        // 应用更新
        let new_artifacts = current_artifacts
            .as_ref()
            .expect("current_artifacts checked above")
            .apply_update(updated);

        // 更新 context_snapshot
        let artifacts_json = serde_json::to_value(&new_artifacts)
            .map_err(|e| PauseStateError::Persist(e.to_string()))?;

        if let Value::Object(ref mut map) = snapshot.context_snapshot {
            map.insert(ARTIFACTS_KEY.to_string(), artifacts_json);
        } else {
            let mut map = serde_json::Map::new();
            map.insert(ARTIFACTS_KEY.to_string(), artifacts_json);
            snapshot.context_snapshot = Value::Object(map);
        }

        // 更新 correlationId
        snapshot.correlation_id = correlation_id.to_string();

        // 持久化
        persist_snapshot(snapshot)?;

        // 记录操作日志（不回显 prompt 原文）
        let timestamp = chrono_timestamp();
        let prev_patterns = current_artifacts
            .as_ref()
            .map(|a| a.patterns.len())
            .unwrap_or(0);
        let prev_prompts = current_artifacts
            .as_ref()
            .map(|a| a.candidate_prompts.len())
            .unwrap_or(0);
        info!(
            task_id = %self.task_id,
            correlation_id = %correlation_id,
            user_id = %user_id,
            artifact_type = "iteration_artifacts",
            edit_action = "update",
            prev_state = %format!("patterns={prev_patterns},prompts={prev_prompts}"),
            new_state = %format!("patterns={},prompts={}", new_artifacts.patterns.len(), new_artifacts.candidate_prompts.len()),
            iteration_state = %snapshot.stage,
            timestamp = %timestamp,
            "产物已更新"
        );

        Ok(new_artifacts)
    }

    /// 更新用户引导（存储到 context_snapshot.artifacts.user_guidance 中）
    ///
    /// 仅在 Paused 状态下允许更新
    /// 遵循 Last One Wins 策略：多次发送仅保留最后一次
    /// 返回 `Ok(guidance)` 表示更新成功
    /// 返回 `Err(reason)` 表示更新失败
    pub async fn update_guidance(
        &self,
        content: &str,
        correlation_id: &str,
        user_id: &str,
    ) -> Result<UserGuidance, PauseStateError> {
        // 状态校验：仅 Paused 状态允许发送引导
        if !self.is_paused.load(Ordering::SeqCst) {
            return Err(PauseStateError::Persist(
                "任务未处于暂停状态，无法发送引导".to_string(),
            ));
        }

        // 创建引导并验证
        let guidance = UserGuidance::new(content);
        guidance.validate().map_err(PauseStateError::Persist)?;

        let mut snapshot_guard = self.snapshot.lock().await;
        let snapshot = snapshot_guard
            .as_mut()
            .ok_or_else(|| PauseStateError::Persist("暂停快照不存在".to_string()))?;

        // 获取当前产物
        let mut current_artifacts: IterationArtifacts = snapshot
            .context_snapshot
            .get(ARTIFACTS_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        // 记录旧引导状态用于日志
        let prev_guidance_id = current_artifacts
            .user_guidance
            .as_ref()
            .map(|g| g.id.clone());
        let prev_guidance_status = current_artifacts
            .user_guidance
            .as_ref()
            .map(|g| format!("{:?}", g.status).to_lowercase())
            .unwrap_or_else(|| "none".to_string());

        // Last One Wins：直接覆盖
        current_artifacts.user_guidance = Some(guidance.clone());
        current_artifacts.updated_at = chrono_timestamp();

        // 更新 context_snapshot
        let artifacts_json = serde_json::to_value(&current_artifacts)
            .map_err(|e| PauseStateError::Persist(e.to_string()))?;

        if let Value::Object(ref mut map) = snapshot.context_snapshot {
            map.insert(ARTIFACTS_KEY.to_string(), artifacts_json);
        } else {
            let mut map = serde_json::Map::new();
            map.insert(ARTIFACTS_KEY.to_string(), artifacts_json);
            snapshot.context_snapshot = Value::Object(map);
        }

        // 更新 correlationId
        snapshot.correlation_id = correlation_id.to_string();

        // 持久化
        persist_snapshot(snapshot)?;

        // 记录操作日志（仅记录引导内容的前 50 字符）
        let timestamp = chrono_timestamp();
        let log_fields = build_guidance_log_fields(
            &self.task_id,
            correlation_id,
            user_id,
            &prev_guidance_status,
            &guidance,
            &snapshot.stage,
            &timestamp,
            prev_guidance_id.is_some(),
        );
        info!(
            task_id = %log_fields.task_id,
            correlation_id = %log_fields.correlation_id,
            user_id = %log_fields.user_id,
            action = %log_fields.action,
            artifact_type = %log_fields.artifact_type,
            edit_action = %log_fields.edit_action,
            prev_state = %log_fields.prev_state,
            new_state = %log_fields.new_state,
            prev_guidance_state = %log_fields.prev_guidance_state,
            new_guidance_state = %log_fields.new_guidance_state,
            guidance_preview = %log_fields.guidance_preview,
            iteration_state = %log_fields.iteration_state,
            timestamp = %log_fields.timestamp,
            "引导已更新"
        );

        Ok(guidance)
    }

    /// 获取当前用户引导
    pub async fn get_guidance(&self) -> Option<UserGuidance> {
        let snapshot = self.snapshot.lock().await;
        let snapshot = snapshot.as_ref()?;
        let artifacts: IterationArtifacts = snapshot
            .context_snapshot
            .get(ARTIFACTS_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())?;
        artifacts.user_guidance
    }

    /// 清除用户引导（迭代结束后调用）
    pub async fn clear_guidance(&self) -> Result<(), PauseStateError> {
        let mut snapshot_guard = self.snapshot.lock().await;
        let snapshot = match snapshot_guard.as_mut() {
            Some(s) => s,
            None => return Ok(()), // 无快照时静默返回
        };

        // 获取当前产物
        let mut current_artifacts: IterationArtifacts = snapshot
            .context_snapshot
            .get(ARTIFACTS_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        if current_artifacts.user_guidance.is_none() {
            return Ok(()); // 无引导时静默返回
        }

        // 清除引导
        let cleared_id = current_artifacts
            .user_guidance
            .as_ref()
            .map(|g| g.id.clone());
        current_artifacts.user_guidance = None;
        current_artifacts.updated_at = chrono_timestamp();

        // 更新 context_snapshot
        let artifacts_json = serde_json::to_value(&current_artifacts)
            .map_err(|e| PauseStateError::Persist(e.to_string()))?;

        if let Value::Object(ref mut map) = snapshot.context_snapshot {
            map.insert(ARTIFACTS_KEY.to_string(), artifacts_json);
        }

        // 持久化
        persist_snapshot(snapshot)?;

        info!(
            task_id = %self.task_id,
            cleared_guidance_id = %cleared_id.as_deref().unwrap_or("none"),
            "引导已清除（迭代结束）"
        );

        Ok(())
    }

    /// 恢复已持久化的暂停状态
    pub async fn restore_snapshot(&self, snapshot: PauseStateSnapshot) {
        self.is_paused.store(true, Ordering::SeqCst);
        self.pause_requested.store(false, Ordering::SeqCst);
        let cid = snapshot.correlation_id.clone();
        let uid = snapshot.user_id.clone();
        let task_id = snapshot.task_id.clone();
        let snap = Some(snapshot);

        *self.snapshot.lock().await = snap;
        *self.last_correlation_id.lock().await = Some(cid);
        *self.last_user_id.lock().await = uid;
        info!(task_id = %task_id, "已恢复暂停快照");
    }

    /// 清理暂停快照（应用编辑后调用）
    pub async fn clear_snapshot(&self) {
        *self.snapshot.lock().await = None;
        if let Err(err) = clear_snapshot_file(&self.task_id) {
            warn!(task_id = %self.task_id, error = %err, "清理暂停快照文件失败");
        }
    }

    /// 重置控制器状态（用于任务完成或取消）
    pub async fn reset(&self) {
        self.pause_requested.store(false, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        *self.snapshot.lock().await = None;
        *self.last_correlation_id.lock().await = None;
        *self.last_user_id.lock().await = None;
        let _ = clear_snapshot_file(&self.task_id);
    }
}

#[allow(clippy::too_many_arguments)]
fn build_guidance_log_fields(
    task_id: &str,
    correlation_id: &str,
    user_id: &str,
    prev_guidance_state: &str,
    guidance: &UserGuidance,
    iteration_state: &str,
    timestamp: &str,
    has_prev_guidance: bool,
) -> GuidanceLogFields {
    GuidanceLogFields {
        task_id: task_id.to_string(),
        correlation_id: correlation_id.to_string(),
        user_id: user_id.to_string(),
        action: "guidance_send".to_string(),
        artifact_type: "user_guidance".to_string(),
        edit_action: if has_prev_guidance {
            "replace".to_string()
        } else {
            "create".to_string()
        },
        prev_state: format!("{:?}", RunControlState::Paused),
        new_state: format!("{:?}", RunControlState::Paused),
        prev_guidance_state: prev_guidance_state.to_string(),
        new_guidance_state: format!("{:?}", guidance.status).to_lowercase(),
        guidance_preview: guidance.content_preview(),
        iteration_state: iteration_state.to_string(),
        timestamp: timestamp.to_string(),
    }
}

/// 暂停控制器注册表（全局管理所有任务的暂停控制器）
#[derive(Debug, Default)]
pub struct PauseControllerRegistry {
    controllers: Mutex<HashMap<String, Arc<PauseController>>>,
}

impl PauseControllerRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取或创建任务的暂停控制器
    pub async fn get_or_create(&self, task_id: &str) -> Arc<PauseController> {
        let mut controllers = self.controllers.lock().await;
        if let Some(controller) = controllers.get(task_id) {
            return controller.clone();
        }

        let controller = Arc::new(PauseController::new(task_id));
        if let Ok(Some(snapshot)) = load_snapshot(task_id) {
            controller.restore_snapshot(snapshot).await;
        }
        controllers.insert(task_id.to_string(), controller.clone());
        controller
    }

    /// 获取任务的暂停控制器（如果存在）
    pub async fn get(&self, task_id: &str) -> Option<Arc<PauseController>> {
        self.controllers.lock().await.get(task_id).cloned()
    }

    /// 移除任务的暂停控制器
    pub async fn remove(&self, task_id: &str) -> Option<Arc<PauseController>> {
        self.controllers.lock().await.remove(task_id)
    }

    /// 获取所有处于暂停状态的任务快照
    pub async fn get_all_paused_snapshots(&self) -> Vec<PauseStateSnapshot> {
        let controllers = self.controllers.lock().await;
        let controller_list: Vec<Arc<PauseController>> = controllers.values().cloned().collect();
        drop(controllers);

        let mut snapshots = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        for controller in controller_list {
            if controller.is_paused() {
                if let Some(snapshot) = controller.get_snapshot().await {
                    seen.insert(snapshot.task_id.clone());
                    snapshots.push(snapshot);
                }
            }
        }

        if let Ok(entries) = std::fs::read_dir(pause_state_dir()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                let bytes = match std::fs::read(&path) {
                    Ok(bytes) => bytes,
                    Err(_) => continue,
                };
                let snapshot: PauseStateSnapshot = match serde_json::from_slice(&bytes) {
                    Ok(snapshot) => snapshot,
                    Err(_) => continue,
                };
                if seen.insert(snapshot.task_id.clone()) {
                    let _ = self.get_or_create(&snapshot.task_id).await;
                    snapshots.push(snapshot);
                }
            }
        }

        snapshots
    }
}

static PAUSE_REGISTRY: OnceLock<Arc<PauseControllerRegistry>> = OnceLock::new();

/// 获取全局暂停控制器注册表
pub fn global_pause_registry() -> Arc<PauseControllerRegistry> {
    PAUSE_REGISTRY
        .get_or_init(|| Arc::new(PauseControllerRegistry::new()))
        .clone()
}

static PAUSE_STATE_DIR: OnceLock<PathBuf> = OnceLock::new();

fn pause_state_dir() -> PathBuf {
    PAUSE_STATE_DIR
        .get_or_init(|| {
            if let Ok(dir) = std::env::var("PAUSE_STATE_DIR") {
                return PathBuf::from(dir);
            }
            if cfg!(test) {
                return std::env::temp_dir().join("prompt_faster_pause_state");
            }
            PathBuf::from("data/pause_state")
        })
        .clone()
}

fn snapshot_path(task_id: &str) -> PathBuf {
    let sanitized = task_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    pause_state_dir().join(format!("{sanitized}.json"))
}

fn persist_snapshot(snapshot: &PauseStateSnapshot) -> Result<(), PauseStateError> {
    let dir = pause_state_dir();
    if let Err(err) = std::fs::create_dir_all(&dir) {
        return Err(PauseStateError::Persist(err.to_string()));
    }
    let path = snapshot_path(&snapshot.task_id);
    let json =
        serde_json::to_vec_pretty(snapshot).map_err(|e| PauseStateError::Persist(e.to_string()))?;
    std::fs::write(path, json).map_err(|e| PauseStateError::Persist(e.to_string()))?;
    Ok(())
}

fn clear_snapshot_file(task_id: &str) -> Result<(), PauseStateError> {
    let path = snapshot_path(task_id);
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| PauseStateError::Persist(e.to_string()))?;
    }
    Ok(())
}

fn load_snapshot(task_id: &str) -> Result<Option<PauseStateSnapshot>, PauseStateError> {
    let path = snapshot_path(task_id);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path).map_err(|e| PauseStateError::Restore(e.to_string()))?;
    let snapshot =
        serde_json::from_slice(&bytes).map_err(|e| PauseStateError::Restore(e.to_string()))?;
    Ok(Some(snapshot))
}

fn emit_paused_event(snapshot: &PauseStateSnapshot) {
    let payload = IterationPausedPayload {
        task_id: snapshot.task_id.clone(),
        paused_at: snapshot.paused_at.clone(),
        stage: snapshot.stage.clone(),
        iteration: snapshot.iteration,
    };
    let msg = WsMessage::new(
        EVT_ITERATION_PAUSED,
        payload,
        snapshot.correlation_id.clone(),
    );
    if let Ok(text) = serde_json::to_string(&msg) {
        global_ws_bus().publish(text);
    }
}

fn emit_resumed_event(task_id: &str, correlation_id: &str) {
    let payload = IterationResumedPayload {
        task_id: task_id.to_string(),
        resumed_at: chrono_timestamp(),
    };
    let msg = WsMessage::new(EVT_ITERATION_RESUMED, payload, correlation_id.to_string());
    if let Ok(text) = serde_json::to_string(&msg) {
        global_ws_bus().publish(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::optimization_task_config::OPTIMIZATION_TASK_CONFIG_MAX_INITIAL_PROMPT_BYTES;
    use crate::domain::types::{ArtifactSource, CandidatePrompt, PatternHypothesis};

    #[tokio::test]
    async fn pause_controller_idempotent_pause() {
        let controller = PauseController::new("task-1");

        // 第一次暂停请求应该成功
        assert!(controller.request_pause("cid-1", "user-1").await);

        // 第二次暂停请求应该返回 false（幂等）
        assert!(!controller.request_pause("cid-2", "user-1").await);
    }

    #[tokio::test]
    async fn pause_controller_idempotent_resume() {
        let controller = PauseController::new("task-1");

        // 未暂停时继续请求应该返回 false
        assert!(!controller.request_resume("cid-1", "user-1").await);

        // 设置暂停状态
        controller.request_pause("cid-1", "user-1").await;
        let _ = controller
            .checkpoint_pause(1, "test", Some("cid-1"), serde_json::json!({}))
            .await
            .expect("checkpoint pause");

        // 第一次继续请求应该成功
        assert!(controller.request_resume("cid-2", "user-1").await);

        // 第二次继续请求应该返回 false（幂等）
        assert!(!controller.request_resume("cid-3", "user-1").await);
    }

    #[tokio::test]
    async fn pause_controller_checkpoint_creates_snapshot() {
        let controller = PauseController::new("task-1");

        // 无暂停请求时 checkpoint 应该返回 false
        assert!(
            !controller
                .checkpoint_pause(1, "running_tests", None, serde_json::json!({}))
                .await
                .expect("checkpoint pause")
        );

        // 有暂停请求时 checkpoint 应该返回 true 并创建快照
        controller.request_pause("cid-1", "user-1").await;
        assert!(
            controller
                .checkpoint_pause(2, "reflecting", Some("cid-1"), serde_json::json!({"k":"v"}))
                .await
                .expect("checkpoint pause")
        );

        let snapshot = controller.get_snapshot().await.unwrap();
        assert_eq!(snapshot.task_id, "task-1");
        assert_eq!(snapshot.iteration, 2);
        assert_eq!(snapshot.stage, "reflecting");
        assert_eq!(snapshot.run_control_state, RunControlState::Paused);
        assert_eq!(snapshot.correlation_id, "cid-1");
        assert_eq!(snapshot.user_id.as_deref(), Some("user-1"));
    }

    #[tokio::test]
    async fn registry_get_or_create() {
        let registry = PauseControllerRegistry::new();

        let c1 = registry.get_or_create("task-1").await;
        let c2 = registry.get_or_create("task-1").await;

        // 应该返回同一个实例
        assert!(Arc::ptr_eq(&c1, &c2));
    }

    #[tokio::test]
    async fn registry_remove() {
        let registry = PauseControllerRegistry::new();

        registry.get_or_create("task-1").await;
        assert!(registry.get("task-1").await.is_some());

        registry.remove("task-1").await;
        assert!(registry.get("task-1").await.is_none());
    }

    #[tokio::test]
    async fn update_artifacts_requires_paused_state() {
        let controller = PauseController::new("task-1");
        let artifacts = IterationArtifacts::default();

        let err = controller
            .update_artifacts(&artifacts, "cid-1", "user-1")
            .await
            .expect_err("should reject when not paused");

        assert!(err.to_string().contains("暂停状态"));
    }

    #[tokio::test]
    async fn update_artifacts_rejects_missing_snapshot_artifacts() {
        let controller = PauseController::new("task-1");
        controller.request_pause("cid-1", "user-1").await;
        let _ = controller
            .checkpoint_pause(1, "test", Some("cid-1"), serde_json::json!({}))
            .await
            .expect("checkpoint pause");

        let err = controller
            .update_artifacts(&IterationArtifacts::default(), "cid-1", "user-1")
            .await
            .expect_err("should reject missing artifacts");

        assert!(err.to_string().contains("不存在可编辑产物"));
    }

    #[tokio::test]
    async fn update_artifacts_rejects_overlong_content() {
        let controller = PauseController::new("task-1");
        controller.request_pause("cid-1", "user-1").await;

        let base = IterationArtifacts {
            patterns: vec![PatternHypothesis {
                id: "p1".to_string(),
                pattern: "ok".to_string(),
                source: ArtifactSource::System,
                confidence: None,
            }],
            candidate_prompts: vec![CandidatePrompt {
                id: "c1".to_string(),
                content: "ok".to_string(),
                source: ArtifactSource::System,
                score: None,
                is_best: false,
            }],
            user_guidance: None,
            updated_at: chrono_timestamp(),
        };

        let _ = controller
            .checkpoint_pause(
                1,
                "test",
                Some("cid-1"),
                serde_json::json!({ "artifacts": base }),
            )
            .await
            .expect("checkpoint pause");

        let too_long = "a".repeat(OPTIMIZATION_TASK_CONFIG_MAX_INITIAL_PROMPT_BYTES + 1);
        let updated = IterationArtifacts {
            patterns: vec![PatternHypothesis {
                id: "p1".to_string(),
                pattern: too_long.clone(),
                source: ArtifactSource::System,
                confidence: None,
            }],
            candidate_prompts: vec![CandidatePrompt {
                id: "c1".to_string(),
                content: "ok".to_string(),
                source: ArtifactSource::System,
                score: None,
                is_best: false,
            }],
            user_guidance: None,
            updated_at: chrono_timestamp(),
        };

        let err = controller
            .update_artifacts(&updated, "cid-1", "user-1")
            .await
            .expect_err("should reject overlong content");

        assert!(err.to_string().contains("过长"));
    }

    #[tokio::test]
    async fn update_guidance_requires_paused_state() {
        let controller = PauseController::new("task-guidance-1");

        let result = controller
            .update_guidance("测试引导", "cid-1", "user-1")
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未处于暂停状态"));
    }

    #[tokio::test]
    async fn update_guidance_validates_content() {
        let controller = PauseController::new("task-guidance-2");
        controller.request_pause("cid-1", "user-1").await;

        let _ = controller
            .checkpoint_pause(
                1,
                "test",
                Some("cid-1"),
                serde_json::json!({ "artifacts": IterationArtifacts::default() }),
            )
            .await
            .expect("checkpoint pause");

        // 空内容应该被拒绝
        let result = controller.update_guidance("   ", "cid-1", "user-1").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不能为空"));

        // 超长内容应该被拒绝
        let too_long = "a".repeat(UserGuidance::MAX_CONTENT_LENGTH + 1);
        let result = controller
            .update_guidance(&too_long, "cid-1", "user-1")
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("最大长度"));
    }

    #[tokio::test]
    async fn update_guidance_last_one_wins() {
        let controller = PauseController::new("task-guidance-3");
        controller.request_pause("cid-1", "user-1").await;

        let _ = controller
            .checkpoint_pause(
                1,
                "test",
                Some("cid-1"),
                serde_json::json!({ "artifacts": IterationArtifacts::default() }),
            )
            .await
            .expect("checkpoint pause");

        // 第一次发送
        let g1 = controller
            .update_guidance("第一次引导", "cid-1", "user-1")
            .await
            .expect("first guidance");

        // 第二次发送（应该覆盖）
        let g2 = controller
            .update_guidance("第二次引导", "cid-2", "user-1")
            .await
            .expect("second guidance");

        assert_ne!(g1.id, g2.id);

        // 获取当前引导应该是第二次的
        let current = controller
            .get_guidance()
            .await
            .expect("should have guidance");
        assert_eq!(current.id, g2.id);
        assert_eq!(current.content, "第二次引导");
    }

    #[tokio::test]
    async fn clear_guidance_removes_guidance() {
        let controller = PauseController::new("task-guidance-4");
        controller.request_pause("cid-1", "user-1").await;

        let _ = controller
            .checkpoint_pause(
                1,
                "test",
                Some("cid-1"),
                serde_json::json!({ "artifacts": IterationArtifacts::default() }),
            )
            .await
            .expect("checkpoint pause");

        // 发送引导
        let _ = controller
            .update_guidance("测试引导", "cid-1", "user-1")
            .await
            .expect("guidance");

        assert!(controller.get_guidance().await.is_some());

        // 清除引导
        controller.clear_guidance().await.expect("clear guidance");

        assert!(controller.get_guidance().await.is_none());
    }

    #[test]
    fn build_guidance_log_fields_captures_required_values() {
        let guidance = UserGuidance::new("测试引导内容");
        let fields = build_guidance_log_fields(
            "task-guidance-log",
            "cid-log",
            "user-log",
            "none",
            &guidance,
            "test-stage",
            "2024-01-01T00:00:00Z",
            false,
        );

        assert_eq!(fields.task_id, "task-guidance-log");
        assert_eq!(fields.correlation_id, "cid-log");
        assert_eq!(fields.user_id, "user-log");
        assert_eq!(fields.action, "guidance_send");
        assert_eq!(fields.artifact_type, "user_guidance");
        assert_eq!(fields.edit_action, "create");
        assert_eq!(fields.prev_state, "Paused");
        assert_eq!(fields.new_state, "Paused");
        assert_eq!(fields.prev_guidance_state, "none");
        assert_eq!(fields.new_guidance_state, "pending");
        assert_eq!(fields.iteration_state, "test-stage");
        assert_eq!(fields.timestamp, "2024-01-01T00:00:00Z");
        assert_eq!(fields.guidance_preview, "测试引导内容");
    }
}
