# Story 7.2: 断点恢复与异常处理

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 7-2-breakpoint-recovery-and-exception-handling

## Epic 7 开工门槛（必须先满足）

> ⚠️ **重要**：本 Story 继承 Epic 7 开工门槛，7.1 已完成门槛验证。
> 跟踪文件：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`

- [x] **P1 Checkpoint ↔ pause_state 收敛设计完成并评审通过**（7.1 已完成）
- [x] **P2 iterations ↔ checkpoints 历史口径统一策略明确**（7.1 已完成）
- [x] **P3 恢复/回滚测试矩阵与验证脚本就绪**（已签收）
- [x] **P4 Epic 7 规划复核会议完成并更新实现路径**（7.1 已完成）


## Key Decisions (MVP)

- **恢复时机**：应用启动时自动检测未完成任务，用户确认后从最近 Checkpoint 恢复。
- **恢复策略**：从 `checkpoints` 表加载最近有效 Checkpoint，校验 checksum 后恢复 `OptimizationContext`。
- **收敛策略（Phase 2）**：恢复优先 Checkpoint；若仅存在 pause_state，则生成“补偿 Checkpoint”后再恢复。
- **异常分类**：区分可恢复异常（网络超时、API 错误）与不可恢复异常（数据损坏、配置缺失）。
- **重试机制**：API 调用失败时自动重试最多 3 次（NFR8），使用指数退避策略。
- **超时控制**：单次 API 调用超时阈值 ≤ 60 秒（NFR23）。
- **离线支持**：网络离线时本地功能（查看历史、配置管理）正常可用（NFR26）。
- **AR2 遵循**：所有恢复操作需记录 correlationId，支持全链路追踪。
- **日志规范**：遵循 A2 日志字段齐全要求（correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp）。
- **任务中止状态**：恢复放弃使用 `OptimizationTaskStatus::Terminated`（避免新增 Aborted 状态）。

## Story

As a Prompt 优化用户,
I want 在异常中断后恢复到断点状态,
so that 不会因为意外中断而丢失优化进度。

## Acceptance Criteria

1. **Given** 系统异常中断（如崩溃、断电、网络中断）
   **When** 用户重新启动应用
   **Then** 自动检测未完成的任务（状态为 Running/Paused 且有有效 Checkpoint）
   **And** 提示用户"检测到上次未完成的任务，是否恢复？"
   **And** 显示任务名称、最近 Checkpoint 时间、迭代轮次

2. **Given** 用户选择恢复
   **When** 执行恢复操作
   **Then** 从最近的 Checkpoint 恢复状态
   **And** 校验 Checkpoint checksum 确保数据完整性（NFR7）
   **And** 恢复 OptimizationContext 包含：iteration、state、run_control_state、current_prompt、rule_system、artifacts、user_guidance
   **And** 提供恢复率统计能力，支撑 NFR5（断点恢复率 100%）

3. **Given** 用户选择不恢复
   **When** 点击"放弃恢复"
   **Then** 标记任务为 `Terminated` 状态（OptimizationTaskStatus）
   **And** 保留 Checkpoint 历史记录（不删除）
   **And** 用户可以后续从历史 Checkpoint 手动恢复（Story 7.3 承接）

4. **Given** API 调用失败（网络错误或超时）
   **When** 检测到可重试错误
   **Then** 自动重试最多 3 次（NFR8）
   **And** 使用指数退避策略（1s → 2s → 4s）
   **And** 每次重试记录 tracing 日志（包含 retry_count、error_type）

5. **Given** API 调用
   **When** 执行 HTTP 请求
   **Then** 单次调用超时阈值 ≤ 60 秒（NFR23）
   **And** 超时后触发重试机制
   **And** 最终失败后记录详细错误信息并通知用户

6. **Given** 网络完全离线
   **When** 用户尝试操作
   **Then** 本地功能正常可用：查看历史记录、管理测试集、查看 Checkpoint（NFR26）
   **And** 需要网络的功能（API 测试、执行优化）显示提示"当前离线，部分功能受限"
   **And** 网络恢复后自动检测并提示用户

7. **Given** Checkpoint 恢复失败（checksum 校验失败或数据损坏）
   **When** 检测到不可恢复错误
   **Then** 尝试回退到上一个有效 Checkpoint
   **And** 如果所有 Checkpoint 均无效，提示用户"无法恢复，建议重新开始任务"
   **And** 记录错误日志并提供诊断信息

8. **Given** 恢复操作完成
   **When** OptimizationContext 已加载
   **Then** 更新前端状态显示恢复后的迭代进度
   **And** 用户可以选择"继续迭代"或"暂停查看"
   **And** 记录恢复成功日志（包含 A2 必填字段）

## Tasks / Subtasks

- [x] 后端：恢复服务模块（AC: 1,2,3,7）
  - [x] 在 `backend/src/core/iteration_engine/` 创建 `recovery.rs` 模块
  - [x] 实现 `detect_unfinished_tasks(user_id: &str) -> Result<Vec<UnfinishedTask>>`（调用 Repo 方法）
  - [x] 实现 `recover_from_checkpoint(checkpoint_id: &str, user_id: &str, correlation_id: &str) -> Result<OptimizationContext>`
  - [x] 实现 `abort_task(task_id: &str, user_id: &str, correlation_id: &str) -> Result<()>`（状态置为 `Terminated`）
  - [x] 集成 checksum 校验（复用 `checkpoint.rs` 中的 `verify_checksum`）
  - [x] 实现 Checkpoint 回退策略（当前失败时尝试上一个）
  - [x] 实现 pause_state 回退与“补偿 Checkpoint”（仅当 Checkpoint 缺失时）
  - [x] 在 `backend/src/core/iteration_engine/mod.rs` 注册 `recovery` 模块

- [x] 后端：Repo 支持（AC: 1,2,3）
  - [x] 在 `backend/src/infra/db/repositories/optimization_task_repo.rs` 新增 `find_unfinished_with_checkpoints(user_id: &str) -> Result<Vec<UnfinishedTask>, OptimizationTaskRepoError>`
  - [x] 在 `backend/src/infra/db/repositories/optimization_task_repo.rs` 新增 `update_status(task_id: &str, status: OptimizationTaskStatus) -> Result<(), OptimizationTaskRepoError>`（用于置为 Terminated）
  - [x] 将“未完成任务检测 SQL”落到 Repo 内部，recovery.rs 只调用 Repo

- [x] 后端：重试机制实现（AC: 4,5）
  - [x] 在 `backend/src/infra/external/` 创建 `retry.rs` 模块
  - [x] 实现 `RetryPolicy` 结构体（max_retries=3, base_delay=1s, exponential_backoff=true）
  - [x] 实现 `with_retry<F, T>(policy: &RetryPolicy, operation: F) -> Result<T>`
  - [x] 在 `llm_client.rs` 集成重试机制
  - [x] 配置 HTTP 客户端超时阈值 60 秒（NFR23）
  - [x] 在 `backend/src/infra/external/mod.rs` 注册 `retry` 模块

- [x] 后端：离线检测与降级（AC: 6）
  - [x] 在 `backend/src/infra/external/` 创建 `connectivity.rs` 模块
  - [x] 实现 `check_connectivity() -> ConnectivityStatus`（优先被动错误分类；必要时主动探测 `/api/v1/health` 作为确认）
  - [x] 实现离线状态缓存（避免频繁探测），并明确缓存失效后的刷新策略
  - [x] 定义离线可用功能列表与受限功能列表
  - [x] 在 API 层添加离线状态检查中间件

- [x] 后端：恢复 API 端点（AC: 1,2,3,8）
  - [x] 在 `backend/src/api/routes/` 创建 `recovery.rs` 路由
  - [x] 实现 `GET /api/v1/recovery/unfinished-tasks` 获取未完成任务列表
  - [x] 实现 `POST /api/v1/recovery/tasks/{task_id}/recover` 执行恢复操作
  - [x] 实现 `POST /api/v1/recovery/tasks/{task_id}/abort` 放弃恢复
  - [x] 实现 `GET /api/v1/connectivity` 获取网络状态
  - [x] 添加权限校验（仅任务所有者可操作：从 session/token 获取 current_user_id，查询 task.owner_user_id，对比失败返回 403）
  - [x] 在 `backend/src/api/routes/mod.rs` 导出 `recovery` 模块
  - [x] 在 `backend/src/main.rs` 挂载路由

- [x] 后端：日志与监控（AC: 4,7,8）
  - [x] 恢复操作记录 tracing 日志（A2 必填字段）
  - [x] 重试操作记录 INFO 级别日志（含 retry_count/error_type/delay）
  - [x] 恢复失败记录 ERROR 级别日志（含诊断信息）
  - [x] 实现恢复成功率指标落库与查询接口（success_count / attempt_count，按 task_id 维度统计）

- [x] 后端：数据结构定义（AC: 1,2）
  - [x] 在 `backend/src/domain/models/` 创建 `recovery.rs` 模型
  - [x] 定义 `UnfinishedTask` 结构体（task_id, task_name, checkpoint_id, last_checkpoint_at, iteration, state, run_control_state）
  - [x] 定义 `RecoveryRequest` DTO
  - [x] 定义 `RecoveryResponse` DTO
  - [x] 定义 `ConnectivityStatus` 枚举（Online, Offline, Limited）
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增 DTO

- [x] 前端：恢复提示 UI（AC: 1,2,3）
  - [x] 在 `frontend/src/features/checkpoint-recovery/components/` 创建 `RecoveryPrompt.tsx`
  - [x] 实现未完成任务检测 hook `useUnfinishedTasks.ts`
  - [x] 实现恢复确认对话框（显示任务信息、Checkpoint 时间）
  - [x] 实现"恢复"和"放弃"按钮处理逻辑
  - [x] 应用启动时自动调用检测（建议在 `AppLayout.tsx` 的 `useEffect` 触发）

- [x] 前端：离线状态 UI（AC: 6）
  - [x] 创建 `useConnectivity.ts` hook 监听网络状态
  - [x] 创建 `OfflineBanner.tsx` 组件（显示离线提示）
  - [x] 在 `AppLayout.tsx` 集成离线状态显示
  - [x] 离线时禁用需要网络的功能按钮

- [x] 前端：服务层封装（AC: 1-8）
  - [x] 在 `frontend/src/features/checkpoint-recovery/services/` 创建 `recoveryService.ts`
  - [x] 封装恢复相关 API 调用
  - [x] 创建 `useRecovery.ts` TanStack Query hooks

- [x] 测试与回归（AC: 1-8）
  - [x] 后端单测：未完成任务检测
  - [x] 后端单测：Checkpoint 恢复与 checksum 校验
  - [x] 后端单测：重试机制（成功/失败场景）
  - [x] 后端单测：超时控制
  - [x] 后端单测：离线检测
  - [x] 集成测试：完整恢复流程（模拟异常中断）
  - [x] 集成测试：断网场景下 Checkpoint 保存，网络恢复后可恢复
  - [x] 集成测试：跨版本恢复兼容性（旧版本 Checkpoint 可恢复）
  - [x] 集成测试：恢复失败回退策略
  - [x] 集成测试：API 重试与降级
  - [x] 前端测试：恢复提示 UI 交互
  - [x] 回归命令：`cd backend && cargo test`；`cd frontend && npx vitest --run && npm run build`
  - [x] 生成类型：`cd backend && cargo run --bin gen-types` 并提交产物

### Hard Gate Checklist

> 必填：跨 Story 硬门禁清单（若不适用请标注 N/A 并说明原因）。

- [x] correlationId 全链路透传（HTTP/WS/日志）
- [x] A2 日志字段齐全（correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp）
- [x] 新增/变更类型已运行 gen-types 并提交生成产物
- [x] 状态一致性与幂等性已校验（如 RunControlState / IterationState）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review] 补齐 OptimizationTaskRepo 的未完成任务查询与状态更新方法
- [x] [AI-Review] 明确恢复流程中的 pause_state 补偿 Checkpoint 生成逻辑
- [x] [AI-Review] 补齐 NFR5 的断网恢复与跨版本恢复集成测试
- [x] [AI-Review] 明确离线检测策略（被动错误分类 + 主动健康检查）

## Dev Notes

### Developer Context (Read This First)

- **现状基线（Story 7.1 已完成）**：
  - Checkpoint 自动保存机制已实现（`backend/src/core/iteration_engine/checkpoint.rs`）
  - `CheckpointEntity` 包含完整状态：iteration、state、run_control_state、prompt、rule_system、artifacts、user_guidance、checksum
  - `CheckpointRepo` 提供 CRUD 操作
  - checksum 校验函数 `verify_checksum` 已实现
  - WAL 模式 + FULL synchronous 已配置
  - `optimization_tasks` 表有 `status` 字段标识任务状态

- **Epic 7 全景（便于对齐业务价值与范围）**：
  - 7.1 Checkpoint 自动保存（已完成，FR52）
  - **7.2 断点恢复与异常处理（本 Story，FR53）**
  - 7.3 历史 Checkpoint 回滚（FR54）
  - 7.4 完整迭代历史记录（FR55）

- **业务价值（为什么做）**：确保用户在任何异常情况下（崩溃、断电、网络中断）都能恢复到已保存的状态，实现 100% 断点恢复率（NFR5），提升用户信任感和系统可靠性。

- **依赖关系**：
  - 依赖 Story 7.1 提供的 Checkpoint 机制
  - 依赖 `CheckpointEntity` 和 `verify_checksum`
  - 依赖 `OptimizationContext` 上下文结构
  - 依赖 `optimization_tasks` 表的任务状态
  - 依赖 `OptimizationTaskRepo` 获取任务配置与测试集，用于重建 OptimizationContext

- **范围边界（必须遵守）**：
  - 本 Story 实现自动恢复和重试机制
  - 手动回滚由 Story 7.3 承接
  - 历史记录统一展示由 Story 7.4 承接
  - 不修改 Checkpoint 保存逻辑（7.1 已稳定）
  - 前端 UI 保持最小可用，复杂交互后续迭代

### 与 Story 7.1 的关系

| 功能 | Story 7.1 | Story 7.2（本 Story） |
| --- | --- | --- |
| Checkpoint 保存 | ✅ 已实现 | 复用 |
| checksum 校验 | ✅ 已实现 | 复用 |
| Checkpoint 加载 | `get_checkpoint_by_id` | 扩展恢复逻辑 |
| 未完成任务检测 | - | 新增 |
| 恢复 API | - | 新增 |
| 重试机制 | - | 新增 |
| 离线检测 | - | 新增 |

### Database Schema

无需新增表，复用 Story 7.1 的 `checkpoints` 表和 `optimization_tasks` 表。

**未完成任务检测 SQL**（仅供参考，需放入 Repo 层实现）：

```sql
-- 检测未完成任务（有有效 Checkpoint 且状态为 running/paused）
SELECT t.id, t.name, t.status, c.created_at as last_checkpoint_at, c.iteration, c.state
FROM optimization_tasks t
INNER JOIN (
    SELECT task_id, MAX(created_at) as max_created_at
    FROM checkpoints
    GROUP BY task_id
) latest ON t.id = latest.task_id
INNER JOIN checkpoints c ON c.task_id = latest.task_id AND c.created_at = latest.max_created_at
WHERE t.user_id = ? AND t.status IN ('running', 'paused')
ORDER BY c.created_at DESC;
```

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/models/recovery.rs

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 未完成任务信息
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct UnfinishedTask {
    pub task_id: String,
    pub task_name: String,
    pub last_checkpoint_at: String,  // ISO 8601
    pub iteration: u32,
    pub state: String,               // IterationState 字符串
    pub run_control_state: String,   // RunControlState 字符串
}

/// 未完成任务列表响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct UnfinishedTasksResponse {
    pub tasks: Vec<UnfinishedTask>,
    pub total: u32,
}

/// 恢复请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryRequest {
    pub checkpoint_id: Option<String>,  // 可选指定 Checkpoint，默认使用最近的
}

/// 恢复响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct RecoveryResponse {
    pub success: bool,
    pub task_id: String,
    pub checkpoint_id: String,
    pub iteration: u32,
    pub state: String,
    pub message: String,
}

/// 网络连接状态
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub enum ConnectivityStatus {
    Online,
    Offline,
    Limited,  // 部分服务可用
}

/// 网络状态响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct ConnectivityResponse {
    pub status: ConnectivityStatus,
    pub last_checked_at: String,  // ISO 8601
    pub message: Option<String>,
}
```

### Repository Signatures (Required)

```rust
/// 位置：backend/src/infra/db/repositories/optimization_task_repo.rs
impl OptimizationTaskRepo {
    pub async fn find_unfinished_with_checkpoints(
        pool: &SqlitePool,
        user_id: &str,
    ) -> Result<Vec<UnfinishedTask>, OptimizationTaskRepoError>;

    pub async fn update_status(
        pool: &SqlitePool,
        task_id: &str,
        status: OptimizationTaskStatus,
    ) -> Result<(), OptimizationTaskRepoError>;
}
```

### Recovery Core Module

```rust
/// 位置：backend/src/core/iteration_engine/recovery.rs

use crate::domain::models::checkpoint::CheckpointEntity;
use crate::domain::models::recovery::{UnfinishedTask, RecoveryResponse};
use crate::domain::models::OptimizationTaskStatus;
use crate::domain::types::{ExecutionTargetConfig, OptimizationContext};
use crate::core::iteration_engine::checkpoint::verify_checksum;
use crate::infra::db::repositories::checkpoint_repo::CheckpointRepo;
use crate::infra::db::repositories::optimization_task_repo::OptimizationTaskRepo;
use sqlx::SqlitePool;

/// 检测未完成任务
pub async fn detect_unfinished_tasks(
    user_id: &str,
    pool: &SqlitePool,
    checkpoint_repo: &CheckpointRepo,
) -> Result<Vec<UnfinishedTask>, RecoveryError> {
    // 查询状态为 running/paused 且有 Checkpoint 的任务
    let tasks = OptimizationTaskRepo::find_unfinished_with_checkpoints(pool, user_id).await?;
    Ok(tasks)
}

/// 从 Checkpoint 恢复
pub async fn recover_from_checkpoint(
    checkpoint_id: &str,
    user_id: &str,
    correlation_id: &str,
    pool: &SqlitePool,
    checkpoint_repo: &CheckpointRepo,
) -> Result<OptimizationContext, RecoveryError> {
    // 1. 加载 Checkpoint
    let checkpoint = match checkpoint_repo.get_checkpoint_by_id(checkpoint_id).await? {
        Some(c) => c,
        None => {
            // 如果 Checkpoint 不存在但 pause_state 存在：生成“补偿 Checkpoint”后再继续恢复
            // （仅保留单读原则：恢复路径以 Checkpoint 为准）
            return Err(RecoveryError::CheckpointNotFound);
        }
    };

    // 2. 校验 checksum
    if !verify_checksum(&checkpoint) {
        tracing::warn!(
            checkpoint_id = %checkpoint_id,
            correlation_id = %correlation_id,
            "Checkpoint checksum 校验失败，尝试回退"
        );
        // 尝试回退到上一个 Checkpoint
        return recover_from_previous_checkpoint(
            &checkpoint.task_id,
            checkpoint_id,
            user_id,
            correlation_id,
            checkpoint_repo,
            pool,
        ).await;
    }

    // 3. 重建 OptimizationContext
    let context = rebuild_optimization_context(&checkpoint, pool).await?;

    // 4. 记录恢复日志
    tracing::info!(
        user_id = %user_id,
        task_id = %checkpoint.task_id,
        checkpoint_id = %checkpoint_id,
        correlation_id = %correlation_id,
        prev_state = ?checkpoint.state,
        new_state = ?checkpoint.state,
        iteration_state = ?checkpoint.state,
        iteration = checkpoint.iteration,
        action = "checkpoint_recovered",
        timestamp = %chrono::Utc::now().timestamp_millis(),
        "Checkpoint 恢复成功"
    );

    Ok(context)
}

/// 回退到上一个 Checkpoint
async fn recover_from_previous_checkpoint(
    task_id: &str,
    failed_checkpoint_id: &str,
    user_id: &str,
    correlation_id: &str,
    checkpoint_repo: &CheckpointRepo,
    pool: &SqlitePool,
) -> Result<OptimizationContext, RecoveryError> {
    // 获取该任务的所有 Checkpoint，按时间降序
    let checkpoints = checkpoint_repo
        .list_checkpoints_by_task(task_id, 10)
        .await?;

    // 跳过失败的 Checkpoint，尝试下一个
    for checkpoint in checkpoints.iter().filter(|c| c.id != failed_checkpoint_id) {
        if verify_checksum(checkpoint) {
            tracing::info!(
                checkpoint_id = %checkpoint.id,
                correlation_id = %correlation_id,
                "回退到有效 Checkpoint"
            );
            return rebuild_optimization_context(checkpoint, pool).await;
        }
    }

    Err(RecoveryError::NoValidCheckpoint)
}

/// 从 CheckpointEntity 重建 OptimizationContext
async fn rebuild_optimization_context(
    checkpoint: &CheckpointEntity,
    pool: &SqlitePool,
) -> Result<OptimizationContext, RecoveryError> {
    // 必须补齐 OptimizationContext 的所有必填字段：
    // - execution_target_config / test_cases / config / checkpoints / extensions 等
    // 建议从 task 配置、测试集与 checkpoint_repo 复原（使用现有 Repo 获取）
    Ok(OptimizationContext {
        task_id: checkpoint.task_id.clone(),
        iteration: checkpoint.iteration,
        state: checkpoint.state.clone(),
        run_control_state: checkpoint.run_control_state.clone(),
        current_prompt: checkpoint.prompt.clone(),
        rule_system: checkpoint.rule_system.clone(),
        execution_target_config: /* from task config */ ExecutionTargetConfig::default(),
        test_cases: /* from test_set repo */ vec![],
        config: /* from task config */ Default::default(),
        checkpoints: /* from checkpoint_repo */ vec![],
        extensions: Default::default(),
    })
}

/// 放弃恢复，标记任务为中止
pub async fn abort_task(
    task_id: &str,
    user_id: &str,
    correlation_id: &str,
    pool: &SqlitePool,
) -> Result<(), RecoveryError> {
    OptimizationTaskRepo::update_status(pool, task_id, OptimizationTaskStatus::Terminated).await?;

    tracing::info!(
        user_id = %user_id,
        task_id = %task_id,
        correlation_id = %correlation_id,
        action = "task_aborted",
        timestamp = %chrono::Utc::now().timestamp_millis(),
        "任务已中止"
    );

    Ok(())
}
```

### Retry Module

```rust
/// 位置：backend/src/infra/external/retry.rs

use std::time::Duration;
use tokio::time::sleep;

/// 重试策略
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub use_exponential_backoff: bool,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            use_exponential_backoff: true,
            max_delay: Duration::from_secs(30),
        }
    }
}

/// 带重试的异步操作执行器
pub async fn with_retry<F, Fut, T, E>(
    policy: &RetryPolicy,
    correlation_id: &str,
    operation_name: &str,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;
    
    for attempt in 0..=policy.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                
                if attempt < policy.max_retries {
                    let delay = calculate_delay(policy, attempt);
                    
                    tracing::info!(
                        correlation_id = %correlation_id,
                        operation = %operation_name,
                        attempt = attempt + 1,
                        max_retries = policy.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %last_error.as_ref().unwrap(),
                        "操作失败，准备重试"
                    );
                    
                    sleep(delay).await;
                }
            }
        }
    }
    
    tracing::error!(
        correlation_id = %correlation_id,
        operation = %operation_name,
        max_retries = policy.max_retries,
        error = %last_error.as_ref().unwrap(),
        "操作最终失败，已用尽重试次数"
    );
    
    Err(last_error.unwrap())
}

fn calculate_delay(policy: &RetryPolicy, attempt: u32) -> Duration {
    if policy.use_exponential_backoff {
        let delay = policy.base_delay * 2u32.pow(attempt);
        std::cmp::min(delay, policy.max_delay)
    } else {
        policy.base_delay
    }
}
```

### Suggested API Endpoints

```
# 获取未完成任务列表
GET /api/v1/recovery/unfinished-tasks
Response: ApiResponse<UnfinishedTasksResponse>
权限校验：仅返回当前用户的任务

# 执行恢复操作
POST /api/v1/recovery/tasks/{task_id}/recover
Body: RecoveryRequest (可选指定 checkpoint_id)
Response: ApiResponse<RecoveryResponse>
权限校验：仅任务所有者可操作

# 放弃恢复
POST /api/v1/recovery/tasks/{task_id}/abort
Response: ApiResponse<{ success: bool, message: string }>
权限校验：仅任务所有者可操作

# 获取网络连接状态
GET /api/v1/connectivity
Response: ApiResponse<ConnectivityResponse>
```

### Dev Agent Guardrails（避免常见踩坑）

- **checksum 校验必须执行**：恢复前必须调用 `verify_checksum`，校验失败时执行回退策略。
- **重试使用指数退避**：避免雪崩效应，延迟序列为 1s → 2s → 4s。
- **超时必须配置**：HTTP 客户端超时阈值 60 秒（NFR23）。
- **离线状态缓存**：避免频繁网络探测，缓存有效期 30 秒。
- **pause_state 回退**：若仅存在 pause_state，先生成补偿 Checkpoint 再恢复（Phase 2 收敛要求）。
- **不要忘记 correlationId**：所有操作必须携带 correlationId（AR2）。
- **日志字段齐全**：必须包含 correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp。
- **权限校验严格**：只有任务所有者可以恢复或中止任务。
- **复用 7.1 代码**：checksum 校验、Checkpoint 加载复用 `checkpoint.rs` 现有实现。
- **类型生成**：新增类型后运行 `cargo run --bin gen-types` 并提交产物。
- **中止状态对齐**：放弃恢复时使用 `OptimizationTaskStatus::Terminated`。
- **HTTP 客户端复用**：统一通过 `create_http_client()` 获取（已包含 60s 超时）。

### Technical Requirements（必须满足）

- Checkpoint checksum 校验使用 SHA-256（复用 7.1 实现）
- API 调用重试最多 3 次（NFR8）
- 单次 API 调用超时阈值 ≤ 60 秒（NFR23）
- 离线时本地功能 100% 可用（NFR26）
- 断点恢复率目标 100%（NFR5）
- API 响应使用 `ApiResponse<T>` 统一结构
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/core/iteration_engine/recovery.rs`：恢复核心逻辑
  - `backend/src/infra/external/retry.rs`：重试机制
  - `backend/src/infra/external/connectivity.rs`：离线检测
  - `backend/src/api/routes/recovery.rs`：API 路由
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **错误处理**：后端 `thiserror` + `anyhow`
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types`

### Library / Framework Requirements (Version Snapshot)

- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`
- tokio：异步运行时 + 定时器（sleep/timeout）
- reqwest：HTTP 客户端（配置超时）
- chrono：时间戳处理

### File Structure Requirements（落点约束）

**后端**：
- 恢复模块：`backend/src/core/iteration_engine/recovery.rs`（新增）
- 模块导出：`backend/src/core/iteration_engine/mod.rs`（更新）
- 重试模块：`backend/src/infra/external/retry.rs`（新增）
- 离线检测：`backend/src/infra/external/connectivity.rs`（新增）
- 模块导出：`backend/src/infra/external/mod.rs`（更新）
- 恢复模型：`backend/src/domain/models/recovery.rs`（新增）
- 模型导出：`backend/src/domain/models/mod.rs`（更新）
- 恢复 API：`backend/src/api/routes/recovery.rs`（新增）
- 路由导出：`backend/src/api/routes/mod.rs`（更新）
- 主路由挂载：`backend/src/main.rs`（更新）
- 类型生成：`backend/src/bin/gen-types.rs`（更新）

**前端**：
- 恢复提示：`frontend/src/features/checkpoint-recovery/components/RecoveryPrompt.tsx`（新增）
- 离线横幅：`frontend/src/features/checkpoint-recovery/components/OfflineBanner.tsx`（新增）
- 恢复服务：`frontend/src/features/checkpoint-recovery/services/recoveryService.ts`（新增）
- 恢复 Hooks：`frontend/src/features/checkpoint-recovery/hooks/useRecovery.ts`（新增）
- 离线 Hooks：`frontend/src/features/checkpoint-recovery/hooks/useConnectivity.ts`（新增）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 未完成任务检测 | 正确识别 running/paused 且有 Checkpoint 的任务 |
| 后端单测 | Checkpoint 恢复 | checksum 校验通过时正确恢复；校验失败时执行回退 |
| 后端单测 | 重试机制 | 重试成功场景；重试耗尽场景；指数退避延迟正确 |
| 后端单测 | 超时控制 | 60 秒超时触发 |
| 后端单测 | 离线检测 | 正确检测在线/离线状态 |
| 后端单测 | 任务中止 | 正确更新状态为 Terminated |
| 集成测试 | 完整恢复流程 | 模拟异常中断后恢复 |
| 集成测试 | 断网恢复 | 断网后可继续保存，网络恢复后可恢复 |
| 集成测试 | 跨版本恢复 | 旧版本 Checkpoint 可恢复 |
| 集成测试 | 回退策略 | 当前 Checkpoint 失败时回退到上一个 |
| 集成测试 | API 权限 | 非任务所有者返回 403 |
| 前端测试 | 恢复提示 UI | 对话框显示/恢复/放弃交互 |
| 前端测试 | 离线状态 | 横幅显示/功能禁用 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 复用 `backend/src/core/iteration_engine/checkpoint.rs` 的 `verify_checksum`
- 复用 `backend/src/infra/db/repositories/checkpoint_repo.rs` 的 Checkpoint 查询
- 遵循 Repository 模式访问数据库
- 重试机制可被其他模块（如 LLM 调用）复用

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 7 / Story 7.2）
- PRD 可靠性：`docs/project-planning-artifacts/prd.md#能力区域 8: 可靠性与恢复`
- 架构（Checkpoint）：`docs/project-planning-artifacts/architecture.md#8. 可靠性与恢复`
- Epic 7 规划复核：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`
- Story 7.1（前序）：`docs/implementation-artifacts/7-1-checkpoint-auto-save.md`
- Checkpoint 实现：`backend/src/core/iteration_engine/checkpoint.rs`
- Checkpoint Repo：`backend/src/infra/db/repositories/checkpoint_repo.rs`

## Git Intelligence Summary

- Story 7.1 关键提交：
  - Checkpoint 自动保存实现
  - checksum 计算与校验
  - 空闲自动保存计时器
  - API 权限校验
- 最近提交关注 clippy warnings 和测试稳定性

## Latest Tech Information (Web/Registry Snapshot)

- 版本以本地依赖快照为准：`frontend/package.json` 与 `backend/Cargo.toml`
- 关键关注点：
  - reqwest HTTP 客户端超时配置
  - tokio::time::timeout 用于超时控制

## Project Context Reference

- 以 `docs/project-planning-artifacts/*.md`、`docs/developer-guides/*` 与现有代码为准

## Story Completion Status

- Status set to `ready-for-dev`
- Completion note: Ultimate context engine analysis completed - comprehensive developer guide created

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

- `cd backend && cargo run --bin gen-types`
- `cd backend && cargo test`
- `cd frontend && npx vitest --run`（MSW 对 `/api/v1/connectivity` 与 `/api/v1/recovery/unfinished-tasks` 有未匹配 handler 的告警，但用例通过）
- `cd frontend && npm run build`

### Completion Notes List

- ✅ 完成恢复核心：未完成任务检测、checksum 校验回退、pause_state 补偿 Checkpoint、任务中止与恢复成功率统计。
- ✅ 补齐 Repo 能力、重试机制、离线检测缓存与中间件、恢复/网络 API 与 OpenAPI 描述。
- ✅ 前端新增恢复提示、离线横幅、网络状态 Hook 与禁用逻辑，完成服务层与 Query hooks 集成。
- ✅ 增补单测/集成测试（恢复流程、回退策略、断网与跨版本恢复、API 重试、前端交互），并完成全量回归与类型生成。

### File List

- backend/migrations/011_add_recovery_metrics.sql
- backend/src/api/middleware/connectivity.rs
- backend/src/api/middleware/mod.rs
- backend/src/api/routes/auth.rs
- backend/src/api/routes/recovery.rs
- backend/src/api/routes/docs.rs
- backend/src/api/routes/mod.rs
- backend/src/api/routes/test_sets.rs
- backend/src/bin/gen-types.rs
- backend/src/core/iteration_engine/recovery.rs
- backend/src/core/iteration_engine/mod.rs
- backend/src/domain/models/recovery.rs
- backend/src/domain/models/mod.rs
- backend/src/infra/db/repositories/mod.rs
- backend/src/infra/db/repositories/optimization_task_repo.rs
- backend/src/infra/db/repositories/recovery_metrics_repo.rs
- backend/src/infra/external/retry.rs
- backend/src/infra/external/connectivity.rs
- backend/src/infra/external/llm_client.rs
- backend/src/infra/external/mod.rs
- backend/src/main.rs
- backend/src/shared/error_codes.rs
- backend/tests/llm_retry_integration_test.rs
- backend/tests/recovery_api_test.rs
- docs/implementation-artifacts/7-2-breakpoint-recovery-and-exception-handling.md
- docs/implementation-artifacts/epic-7-recovery-rollback-test-matrix-2026-01-18.md
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/validation-report-2026-01-18_19-52-13.md
- frontend/src/App.tsx
- frontend/src/features/api-config/DifyCredentialForm.tsx
- frontend/src/features/api-config/GenericLlmCredentialForm.tsx
- frontend/src/features/user-intervention/history/HistoryPanel.tsx
- frontend/src/pages/RunView/RunView.tsx
- frontend/src/features/checkpoint-recovery/components/RecoveryPrompt.tsx
- frontend/src/features/checkpoint-recovery/components/RecoveryPrompt.test.tsx
- frontend/src/features/checkpoint-recovery/components/OfflineBanner.tsx
- frontend/src/features/checkpoint-recovery/components/OfflineBanner.test.tsx
- frontend/src/features/checkpoint-recovery/hooks/useConnectivity.ts
- frontend/src/features/checkpoint-recovery/hooks/useRecovery.ts
- frontend/src/features/checkpoint-recovery/hooks/useUnfinishedTasks.ts
- frontend/src/features/checkpoint-recovery/services/recoveryService.ts
- frontend/src/types/generated/models/ConnectivityResponse.ts
- frontend/src/types/generated/models/ConnectivityStatus.ts
- frontend/src/types/generated/models/RecoveryMetrics.ts
- frontend/src/types/generated/models/RecoveryRequest.ts
- frontend/src/types/generated/models/RecoveryResponse.ts
- frontend/src/types/generated/models/UnfinishedTask.ts
- frontend/src/types/generated/models/UnfinishedTasksResponse.ts
- frontend/src/types/generated/models/UnfinishedTask.ts
- frontend/src/types/generated/models/UnfinishedTasksResponse.ts

## Change Log

- 2026-01-18：完成断点恢复与异常处理（恢复核心/Repo/重试/离线检测/API/前端提示/测试与类型生成）

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [ ] CRITICAL: P3 恢复/回滚测试矩阵脚本未签收，Story 开工应阻塞（已完成签收）
- [ ] CRITICAL: 任务中止状态需对齐 `OptimizationTaskStatus::Terminated`（避免不存在的 Aborted）
- [ ] HIGH: recovery 依赖的 Repo 方法缺失（find_unfinished_with_checkpoints/update_status），需补实现
- [ ] HIGH: OptimizationContext 重建不得使用 `Default::default()`，需明确字段来源
- [ ] HIGH: 需补 pause_state 回退与补偿 Checkpoint（Phase 2 收敛策略）
- [ ] HIGH: 测试用例需覆盖断网恢复与跨版本恢复（NFR5）
- [ ] MEDIUM: 离线检测策略与缓存失效刷新需明确
- [ ] MEDIUM: 权限校验需明确 current_user_id 校验路径
- [x] CRITICAL: 标注已完成的“断网场景下 Checkpoint 保存，网络恢复后可恢复”集成测试未覆盖断网保存与恢复链路
- [x] HIGH: 未完成任务检测未校验 Checkpoint checksum，可能提示不可恢复任务
- [x] HIGH: 恢复率统计仅存在内存计数，未暴露接口且重启丢失，难以支撑 NFR5
- [x] MEDIUM: 恢复完成后缺少“继续迭代/暂停查看”入口与状态刷新
- [x] MEDIUM: 离线中间件仅挂载在连接测试路由，其他需联网接口未被统一拦截
- [x] LOW: 回退仅检查最近 10 个 Checkpoint，极端情况下可能忽略更早的有效 Checkpoint

### Decisions

- [ ] 统一“放弃恢复”状态为 `Terminated`，避免新增 Aborted 造成枚举与迁移扩散
- [ ] 未引入 backoff/reqwest-middleware 等新依赖，保持与现有依赖栈一致
- [ ] 优先沿用 `create_http_client()` 统一超时配置，避免重复实现

### Risks / Tech Debt

- [ ] P3 未签收导致恢复/回滚验证不完整，若强行开工需明确风险与补测计划
- [ ] OptimizationContext 重建字段来源不清晰会导致恢复不完整或数据丢失
- [x] pause_state 补偿仅存于内存，服务重启后无法补偿（已记录该限制）

### Follow-ups

- [x] 补齐 OptimizationTaskRepo 的未完成任务查询与状态更新方法
- [x] 明确恢复流程中的 pause_state 补偿 Checkpoint 生成逻辑
- [x] 补齐 NFR5 的断网恢复与跨版本恢复集成测试
- [x] 明确离线检测策略（被动错误分类 + 主动健康检查）
- [x] [AI-Review][CRITICAL] 补齐“断网场景下 Checkpoint 保存 → 网络恢复后可恢复”的真实集成测试
- [x] [AI-Review][HIGH] 未完成任务检测已校验 Checkpoint checksum，并回退至首个有效 checkpoint
- [x] [AI-Review][HIGH] 恢复率统计已持久化并新增查询接口（NFR5 支撑）
- [x] [AI-Review][MEDIUM] 恢复成功后提供继续/暂停入口并刷新前端状态
- [x] [AI-Review][MEDIUM] 扩展离线中间件到所有需要联网的 API（模型列表 / Dify 参数刷新）
- [x] [AI-Review][LOW] 回退策略已移除硬编码 10 个 Checkpoint，改为按任务实际数量扫描
