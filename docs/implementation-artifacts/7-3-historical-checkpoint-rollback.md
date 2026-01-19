# Story 7.3: 历史 Checkpoint 回滚

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 7-3-historical-checkpoint-rollback

## Epic 7 开工门槛（必须先满足）

> ⚠️ **重要**：本 Story 继承 Epic 7 开工门槛，7.1/7.2 已完成门槛验证。
> 跟踪文件：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`

- [x] **P1 Checkpoint ↔ pause_state 收敛设计完成并评审通过**（7.1 已完成）
- [x] **P2 iterations ↔ checkpoints 历史口径统一策略明确**（7.1 已完成）
- [x] **P3 恢复/回滚测试矩阵与验证脚本就绪**（脚本已具备；WAL/FULL 已有自动化测试覆盖；已执行全量回归 `cargo test` + `vitest` + `vite build`）
- [x] **P4 Epic 7 规划复核会议完成并更新实现路径**（7.1 已完成）

## Key Decisions (MVP)

- **回滚与恢复的区别**：7.2 自动恢复最近状态，7.3 用户主动选择历史 Checkpoint 回滚。
- **归档策略**：回滚后的 Checkpoint 标记为归档（不删除），保留完整追溯链。
- **归档可见不可回滚**：归档 Checkpoint 仅供查看，不可作为回滚目标。
- **分支机制**：回滚后创建新分支（branch_id），从回滚点开始新的迭代路径。
- **通过率摘要**：Checkpoint 列表展示通过率摘要，帮助用户选择回滚目标。
- **历史入口单一**：UI 使用统一 History API 展示回滚候选，避免 iterations/checkpoints 双入口。
- **AR2 遵循**：所有回滚操作需记录 correlationId，支持全链路追踪。
- **日志规范**：遵循 A2 日志字段齐全要求。
- **权限校验**：仅任务所有者可执行回滚操作。

## Story

As a Prompt 优化用户,
I want 从任意历史 Checkpoint 回滚,
so that 我可以撤销错误的操作或尝试不同的优化路径。

## Acceptance Criteria

1. **Given** 优化任务已执行多轮迭代
   **When** 用户点击"回滚"按钮
   **Then** 显示所有可用的 Checkpoint 列表
   **And** 每个 Checkpoint 显示：时间戳、迭代轮次、通过率摘要

2. **Given** 用户选择某个 Checkpoint
   **When** 点击"确认回滚"
   **Then** 系统恢复到该 Checkpoint 的状态
   **And** 该 Checkpoint 之后的状态被归档（不删除，可追溯）
   **And** 记录回滚日志（包含 A2 必填字段）
   **And** 归档的 Checkpoint 仅可查看，不可作为回滚目标

3. **Given** 回滚完成
   **When** 用户继续迭代
   **Then** 从回滚点开始新的迭代分支
   **And** 历史记录保留完整的分支信息

4. **Given** 回滚目标 Checkpoint 的 checksum 校验失败
   **When** 执行回滚操作
   **Then** 提示用户"该 Checkpoint 数据已损坏，无法回滚"
   **And** 建议选择其他有效 Checkpoint

5. **Given** 用户查看已归档的 Checkpoint
   **When** 打开历史面板
   **Then** 归档的 Checkpoint 以特殊样式标识（如灰色/斜体）
   **And** 显示归档原因（如"被回滚操作归档"）

## Tasks / Subtasks

- [x] 后端：数据库迁移（AC: 2,3,5）
  - [x] 创建 `backend/migrations/012_checkpoint_rollback_support.sql`
  - [x] 添加 `archived_at` 字段（INTEGER，可选，归档时间戳，Unix ms）
  - [x] 添加 `archive_reason` 字段（VARCHAR，可选，归档原因）
  - [x] 添加 `pass_rate_summary` 字段（TEXT，可选，JSON 格式通过率摘要）
  - [x] 说明：`branch_id` / `parent_id` 已在 7.1 schema 中存在，无需新增

- [x] 后端：CheckpointRepo 扩展（AC: 1,2,3,5）
  - [x] 在 `backend/src/infra/db/repositories/checkpoint_repo.rs` 新增：
    - `list_checkpoint_summaries(task_id: &str, include_archived: bool, limit: usize, offset: usize) -> Result<Vec<CheckpointSummary>>`
    - `archive_checkpoints_after(task_id: &str, checkpoint_id: &str, reason: &str) -> Result<u32>`
    - `get_checkpoint_with_summary(checkpoint_id: &str) -> Result<Option<CheckpointWithSummary>>`
  - [x] 更新 `CheckpointEntity` 结构体添加新字段
    - `archived_at: Option<i64>`
    - `archive_reason: Option<String>`
    - `pass_rate_summary: Option<PassRateSummary>`
  - [x] 归档“之后”判定：同一 task_id + 同一分支路径 + created_at > 目标 Checkpoint.created_at（避免误归档其他分支）

- [x] 后端：回滚核心逻辑（AC: 2,3,4）
  - [x] 在 `backend/src/core/iteration_engine/recovery.rs` 新增：
    - `rollback_to_checkpoint(task_id: &str, checkpoint_id: &str, user_id: &str, correlation_id: &str) -> Result<RollbackResponse>`
  - [x] 实现回滚流程：
    1. 校验 Checkpoint 有效性（checksum）
    2. 生成新 branch_id（UUID）
    3. 归档当前 Checkpoint 之后的所有 Checkpoint（同分支路径 + created_at > 目标）
    4. 复用 `recover_from_checkpoint` 恢复状态
    5. 更新 OptimizationContext 的 branch_id，确保后续保存走新分支（如任务有持久化字段则同步更新）
  - [x] 记录回滚日志（A2 必填字段）

- [x] 后端：回滚 API 端点（AC: 1,2）
  - [x] 在 `backend/src/api/routes/checkpoints.rs` 扩展：
    - `GET /api/v1/tasks/{task_id}/checkpoints` 支持 include_archived + 归档原因/摘要字段
  - [x] 在 `backend/src/api/routes/recovery.rs` 新增：
    - `POST /api/v1/tasks/{task_id}/rollback` 执行回滚操作
  - [x] 添加权限校验（仅任务所有者可操作）
  - [x] 添加 OpenAPI 文档描述

- [x] 后端：历史入口统一（P2 约束）
  - [x] 若已有 `GET /api/v1/tasks/{task_id}/iterations`，新增 `GET /api/v1/tasks/{task_id}/history` 聚合层（迭代历史 + 回滚候选）
  - [x] UI 仅调用 History API（禁止前端同时直接调用 iterations + checkpoints）
  - [x] 若暂不在本 Story 落地，需在 `## Dev Notes` 标注并由 7.4 承接

- [x] 后端：数据结构定义（AC: 1,2,3）
  - [x] 在 `backend/src/domain/models/recovery.rs` 新增：
    - `CheckpointSummary` 结构体（id, task_id, iteration, state, pass_rate_summary, created_at, archived_at）
    - `CheckpointWithSummary` 结构体（checkpoint + pass_rate_summary + archived 字段）
    - `RollbackRequest` DTO
    - `RollbackResponse` DTO
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增 DTO

- [x] 前端：Checkpoint 列表 UI（AC: 1,5）
  - [x] 在 `frontend/src/features/checkpoint-recovery/components/` 创建 `CheckpointList.tsx`
  - [x] 显示 Checkpoint 列表（时间戳、迭代轮次、通过率摘要）
  - [x] 归档 Checkpoint 使用特殊样式标识
  - [x] 支持选择 Checkpoint 进行回滚
  - [x] 列表数据来源走 History API 单入口（避免直接拼接 iterations + checkpoints）

- [x] 前端：回滚确认对话框（AC: 2,4）
  - [x] 在 `frontend/src/features/checkpoint-recovery/components/` 创建 `RollbackConfirmDialog.tsx`
  - [x] 显示回滚目标 Checkpoint 信息
  - [x] 警告提示"回滚后，该 Checkpoint 之后的状态将被归档"
  - [x] 确认和取消按钮

- [x] 前端：服务层封装（AC: 1-5）
  - [x] 在 `frontend/src/features/checkpoint-recovery/services/` 扩展 `recoveryService.ts`
    - `getCheckpointList(taskId: string): Promise<CheckpointSummary[]>`
    - `rollbackToCheckpoint(taskId: string, checkpointId: string): Promise<RollbackResponse>`
  - [x] 创建 `useCheckpointList.ts` TanStack Query hook
  - [x] 创建 `useRollback.ts` mutation hook
  - [x] 若 History API 已提供，优先从 History API 获取可回滚节点（避免 UI 直连 checkpoints 列表）

- [x] 前端：集成与入口（AC: 1-5）
  - [x] 在任务详情页或历史面板中添加"回滚"按钮入口
  - [x] 回滚成功后刷新状态并提示用户

- [x] 测试与回归（AC: 1-5）
  - [x] 后端单测：Checkpoint 列表查询（含/不含归档）
  - [x] 后端单测：归档逻辑（正确归档目标之后的 Checkpoint）
  - [x] 后端单测：回滚逻辑（checksum 校验、分支创建、状态恢复）
  - [x] 后端单测：权限校验（非任务所有者返回 403）
  - [x] 集成测试：完整回滚流程（选择 Checkpoint → 确认 → 恢复 → 继续迭代）
  - [x] 集成测试：回滚后继续迭代创建新分支
  - [x] 集成测试：归档 Checkpoint 可查看但不可再次回滚到其之后的状态
  - [x] 前端测试：Checkpoint 列表 UI 交互
  - [x] 前端测试：回滚确认对话框交互
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

- [x] [AI-Review] 将本 Story 的 review 结论沉淀到 `## Review Notes`（含风险/遗留）
- [x] [AI-Review][HIGH] 回滚事务提交后再恢复上下文，若恢复失败会造成“已归档/改分支但回滚失败”的不一致状态，应保证回滚与恢复的原子性或提供补偿机制。`backend/src/core/iteration_engine/recovery.rs:352`
- [x] [AI-Review][HIGH] Checkpoint 列表仅拉取 10 条，未满足“显示所有可用 Checkpoint”AC；需要分页/加载更多或取消硬编码限制。`frontend/src/features/user-intervention/history/HistoryPanel.tsx:45`
- [x] [AI-Review][MEDIUM] current_branch_id 取“首个未归档”存在多分支/截断时误判风险，应基于最新活动分支或任务当前分支来源。`backend/src/api/routes/checkpoints.rs:132`
- [x] [AI-Review][MEDIUM] Git 变更文件未同步到 Story File List（本 Story 文件与 validation 报告），需补齐变更清单。`docs/implementation-artifacts/7-3-historical-checkpoint-rollback.md:1`
- [x] [AI-Review][MEDIUM] history 端 iterations_limit 未做上限约束，可能被恶意/误用导致大查询。`backend/src/api/routes/history.rs:95`
- [x] [AI-Review][LOW] Checkpoint 摘要查询仍拉取大字段（prompt/artifacts/rule_system），影响列表性能；应改为仅选必要列。`backend/src/infra/db/repositories/checkpoint_repo.rs:173`

## Dev Notes

### Developer Context (Read This First)

- **现状基线（Story 7.1/7.2 已完成）**：
  - Checkpoint 自动保存机制已实现（`backend/src/core/iteration_engine/checkpoint.rs`）
  - 断点恢复机制已实现（`backend/src/core/iteration_engine/recovery.rs`）
  - `CheckpointEntity` 包含：iteration、state、run_control_state、prompt、rule_system、artifacts、user_guidance、checksum
  - `CheckpointRepo` 提供 CRUD 操作和 `list_checkpoints_by_task`
  - `recover_from_checkpoint` 函数可复用
  - WAL 模式 + FULL synchronous 已配置
  - 现有 schema 已包含 `branch_id` / `parent_id` / `lineage_type`（不要新增同义字段）

- **Epic 7 全景（便于对齐业务价值与范围）**：
  - 7.1 Checkpoint 自动保存（已完成，FR52）
  - 7.2 断点恢复与异常处理（已完成，FR53）
  - **7.3 历史 Checkpoint 回滚（本 Story，FR54）**
  - 7.4 完整迭代历史记录（FR55）

- **业务价值（为什么做）**：允许用户撤销错误操作或尝试不同的优化路径，通过分支机制保留完整的决策历史，提升用户对优化过程的掌控感。

- **依赖关系**：
  - 依赖 Story 7.1 提供的 Checkpoint 机制
  - 依赖 Story 7.2 提供的 `recover_from_checkpoint` 函数
  - 依赖 `CheckpointEntity` 和 `verify_checksum`

- **数据来源与口径（必须明确）**：
  - `pass_rate_summary` 需从现有评估结果口径提取（优先使用 `OptimizationContext.extensions` 中的评估/统计，如 `EXT_EVALUATIONS_BY_TEST_CASE_ID` 或 `CandidateStats.pass_rate`）
  - 若历史 Checkpoint 缺少评估结果，返回 `null`（不强制回溯计算）

- **范围边界（必须遵守）**：
  - 本 Story 实现：Checkpoint 列表展示、单次回滚、归档机制、基础分支支持
  - 不包含：复杂分支可视化、分支合并、分支对比、分支切换
  - 7.4 承接：完整历史记录展示和时间线视图
  - 历史入口需遵循“单一入口”策略（P2），UI 不应同时直连 iterations + checkpoints

### 与 Story 7.1/7.2 的关系

| 功能 | Story 7.1 | Story 7.2 | Story 7.3（本 Story） |
| --- | --- | --- | --- |
| Checkpoint 保存 | ✅ 已实现 | 复用 | 复用 |
| checksum 校验 | ✅ 已实现 | 复用 | 复用 |
| Checkpoint 加载 | `get_checkpoint_by_id` | 复用 | 复用 |
| 恢复逻辑 | - | `recover_from_checkpoint` | 复用 + 扩展 |
| Checkpoint 列表 | `list_checkpoints_by_task` | - | 扩展（含摘要） |
| 归档机制 | - | - | 新增 |
| 分支支持 | - | - | 新增 |
| 回滚 API | - | - | 新增 |

### Database Schema Changes

```sql
-- backend/migrations/012_checkpoint_rollback_support.sql

-- 添加回滚支持字段
-- 注意：branch_id / parent_id 已在 7.1 schema 中存在
ALTER TABLE checkpoints ADD COLUMN archived_at INTEGER;
ALTER TABLE checkpoints ADD COLUMN archive_reason VARCHAR(255);
ALTER TABLE checkpoints ADD COLUMN pass_rate_summary TEXT;

-- 索引优化
CREATE INDEX idx_checkpoints_branch_id ON checkpoints(branch_id);
CREATE INDEX idx_checkpoints_archived_at ON checkpoints(archived_at);
CREATE INDEX idx_checkpoints_parent ON checkpoints(parent_id);
```

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/models/recovery.rs（扩展）

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Checkpoint 摘要（用于列表展示）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CheckpointSummary {
    pub id: String,
    pub task_id: String,
    pub iteration: u32,
    pub state: String,               // IterationState 字符串
    pub pass_rate_summary: Option<PassRateSummary>,
    pub created_at: String,          // ISO 8601（由 Unix ms 转换）
    pub archived_at: Option<String>, // ISO 8601，如果已归档
    pub archive_reason: Option<String>,
    pub branch_id: String,
    pub parent_id: Option<String>,
}

/// Checkpoint 详情（含摘要字段）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CheckpointWithSummary {
    pub checkpoint: CheckpointEntity,           // 复用 7.1 定义
    pub pass_rate_summary: Option<PassRateSummary>,
    pub archived_at: Option<String>,
    pub archive_reason: Option<String>,
}

/// 通过率摘要
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PassRateSummary {
    pub total_cases: u32,
    pub passed_cases: u32,
    pub pass_rate: f64,  // 0.0 - 1.0
}

/// Checkpoint 列表响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CheckpointListResponse {
    pub checkpoints: Vec<CheckpointSummary>,
    pub total: u32,
    pub current_branch_id: Option<String>, // 当前分支（可从最新 Checkpoint.branch_id 推断；无独立持久化字段）
}

/// 回滚请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackRequest {
    pub checkpoint_id: String,
    pub confirm: bool,  // 用户确认回滚
}

/// 回滚响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct RollbackResponse {
    pub success: bool,
    pub task_id: String,
    pub checkpoint_id: String,
    pub new_branch_id: String,
    pub archived_count: u32,  // 被归档的 Checkpoint 数量
    pub iteration: u32,
    pub state: String,
    pub message: String,
}
```

### Repository Signatures (Required)

```rust
/// 位置：backend/src/infra/db/repositories/checkpoint_repo.rs（扩展）

impl CheckpointRepo {
    /// 获取任务的 Checkpoint 列表（支持是否包含归档）
    pub async fn list_checkpoint_summaries(
        pool: &SqlitePool,
        task_id: &str,
        include_archived: bool,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<CheckpointSummary>, CheckpointRepoError>;

    /// 归档指定 Checkpoint 之后的所有 Checkpoint
    /// “之后”= 同 task_id + 同分支路径 + created_at > 目标 created_at
    pub async fn archive_checkpoints_after(
        pool: &SqlitePool,
        task_id: &str,
        checkpoint_id: &str,
        reason: &str,
    ) -> Result<u32, CheckpointRepoError>;

    /// 获取 Checkpoint 详情（含通过率摘要）
    pub async fn get_checkpoint_with_summary(
        pool: &SqlitePool,
        checkpoint_id: &str,
    ) -> Result<Option<CheckpointWithSummary>, CheckpointRepoError>;

    /// 更新 Checkpoint 的分支信息
    pub async fn update_branch_info(
        pool: &SqlitePool,
        checkpoint_id: &str,
        branch_id: &str,
        parent_id: Option<&str>,
    ) -> Result<(), CheckpointRepoError>;
}
```

### Rollback Core Module

```rust
/// 位置：backend/src/core/iteration_engine/recovery.rs（扩展）

use uuid::Uuid;

/// 回滚到指定 Checkpoint
pub async fn rollback_to_checkpoint(
    task_id: &str,
    checkpoint_id: &str,
    user_id: &str,
    correlation_id: &str,
    pool: &SqlitePool,
    checkpoint_repo: &CheckpointRepo,
) -> Result<RollbackResponse, RecoveryError> {
    // 1. 加载目标 Checkpoint
    let checkpoint = checkpoint_repo
        .get_checkpoint_by_id(checkpoint_id)
        .await?
        .ok_or(RecoveryError::CheckpointNotFound)?;

    // 2. 校验 checksum
    if !verify_checksum(&checkpoint) {
        tracing::warn!(
            checkpoint_id = %checkpoint_id,
            correlation_id = %correlation_id,
            "Checkpoint checksum 校验失败，无法回滚"
        );
        return Err(RecoveryError::ChecksumMismatch);
    }

    // 3. 验证任务所有权
    // （通过 task_repo 确认 task.user_id == user_id）

    // 4. 生成新分支 ID
    let new_branch_id = Uuid::new_v4().to_string();

    // 5. 归档目标 Checkpoint 之后的所有 Checkpoint（同分支路径 + created_at > 目标）
    // 事务包裹步骤 5-7，任一步失败则回滚
    let archive_reason = format!("rollback_to_checkpoint_{}", checkpoint_id);
    let archived_count = {
        // let mut tx = pool.begin().await?;
        let count = checkpoint_repo
            .archive_checkpoints_after(pool, task_id, checkpoint_id, &archive_reason)
            .await?;

        // 6. 更新目标 Checkpoint 的分支信息（作为新分支的起点）
        checkpoint_repo
            .update_branch_info(pool, checkpoint_id, &new_branch_id, None)
            .await?;
        // tx.commit().await?;
        count
    };

    // 7. 复用 recover_from_checkpoint 恢复状态
    let context = recover_from_checkpoint(
        checkpoint_id,
        user_id,
        correlation_id,
        pool,
        checkpoint_repo,
    ).await?;

    // 8. 记录回滚日志
    tracing::info!(
        user_id = %user_id,
        task_id = %task_id,
        checkpoint_id = %checkpoint_id,
        correlation_id = %correlation_id,
        new_branch_id = %new_branch_id,
        archived_count = archived_count,
        prev_state = ?checkpoint.state,
        new_state = ?checkpoint.state,
        iteration_state = ?checkpoint.state,
        iteration = checkpoint.iteration,
        action = "checkpoint_rollback",
        timestamp = %chrono::Utc::now().timestamp_millis(),
        "Checkpoint 回滚成功"
    );

    Ok(RollbackResponse {
        success: true,
        task_id: task_id.to_string(),
        checkpoint_id: checkpoint_id.to_string(),
        new_branch_id,
        archived_count,
        iteration: checkpoint.iteration,
        state: checkpoint.state.clone(),
        message: format!("已回滚到迭代 {} 的 Checkpoint，归档了 {} 个后续 Checkpoint", 
                         checkpoint.iteration, archived_count),
    })
}
```

### Suggested API Endpoints

```
# 获取任务的 Checkpoint 列表
GET /api/v1/tasks/{task_id}/checkpoints?include_archived=false
Response: ApiResponse<CheckpointListResponse>
权限校验：仅任务所有者可访问
说明：该接口位于 `backend/src/api/routes/checkpoints.rs`（不要在 recovery 路由重复定义）

# 执行回滚操作
POST /api/v1/tasks/{task_id}/rollback
Body: RollbackRequest { checkpoint_id, confirm: true }
Response: ApiResponse<RollbackResponse>
权限校验：仅任务所有者可操作

# 历史入口统一（P2 约束，若已有 History API 则复用）
# UI 仅调用统一 History API 获取可回滚节点；不直接拼接 iterations + checkpoints
# 参考：/api/v1/tasks/{task_id}/history（聚合迭代历史 + 回滚候选）
```

### Dev Agent Guardrails（避免常见踩坑）

- **checksum 校验必须执行**：回滚前必须调用 `verify_checksum`，校验失败时拒绝回滚。
- **归档而非删除**：回滚后的 Checkpoint 必须保留，仅标记 `archived_at` 和 `archive_reason`。
- **归档不可回滚**：归档 Checkpoint 仅展示，不可作为回滚目标。
- **分支 ID 唯一性**：使用 UUID v4 生成分支 ID，确保全局唯一。
- **不要忘记 correlationId**：所有操作必须携带 correlationId（AR2）。
- **日志字段齐全**：必须包含 correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp。
- **权限校验严格**：只有任务所有者可以查看和回滚 Checkpoint。
- **复用 7.2 代码**：回滚核心逻辑复用 `recover_from_checkpoint`。
- **类型生成**：新增类型后运行 `cargo run --bin gen-types` 并提交产物。
- **归档查询优化**：列表查询默认不包含归档 Checkpoint，通过参数控制。
- **字段复用**：使用既有 `branch_id` / `parent_id` / `lineage_type`，不新增同义列。

### Technical Requirements（必须满足）

- Checkpoint checksum 校验使用 SHA-256（复用 7.1 实现）
- 归档操作使用事务保证原子性
- `pass_rate_summary` 从既有评估口径提取；无来源则返回 `null`
- 归档范围限定：同 task_id + 同分支路径 + created_at > 目标
- API 响应使用 `ApiResponse<T>` 统一结构
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/core/iteration_engine/recovery.rs`：回滚核心逻辑（扩展）
  - `backend/src/infra/db/repositories/checkpoint_repo.rs`：Checkpoint 仓库（扩展）
  - `backend/src/api/routes/recovery.rs`：API 路由（扩展）
  - `backend/src/api/routes/checkpoints.rs`：Checkpoint 列表路由（已有，扩展而非新增）
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **错误处理**：后端 `thiserror` + `anyhow`
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types`
- **历史入口**：遵循 P2 单一入口策略（UI 只调用 History API）

### Library / Framework Requirements (Version Snapshot)

- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`
- tokio：异步运行时
- uuid：分支 ID 生成
- chrono：时间戳处理

### File Structure Requirements（落点约束）

**后端**：
- 数据库迁移：`backend/migrations/012_checkpoint_rollback_support.sql`（新增）
- Checkpoint Repo：`backend/src/infra/db/repositories/checkpoint_repo.rs`（扩展）
- 恢复模块：`backend/src/core/iteration_engine/recovery.rs`（扩展）
- 恢复模型：`backend/src/domain/models/recovery.rs`（扩展）
- 恢复 API：`backend/src/api/routes/recovery.rs`（扩展）
- 历史入口：`backend/src/api/routes/history.rs`（如新增统一 History API；或扩展 `iterations.rs`）
- 类型生成：`backend/src/bin/gen-types.rs`（更新）

**前端**：
- Checkpoint 列表：`frontend/src/features/checkpoint-recovery/components/CheckpointList.tsx`（新增）
- 回滚对话框：`frontend/src/features/checkpoint-recovery/components/RollbackConfirmDialog.tsx`（新增）
- 恢复服务：`frontend/src/features/checkpoint-recovery/services/recoveryService.ts`（扩展）
- 列表 Hook：`frontend/src/features/checkpoint-recovery/hooks/useCheckpointList.ts`（新增）
- 回滚 Hook：`frontend/src/features/checkpoint-recovery/hooks/useRollback.ts`（新增）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | Checkpoint 列表查询 | 正确返回列表；include_archived 参数生效 |
| 后端单测 | 归档逻辑 | 正确归档目标之后的 Checkpoint；不影响目标及之前的 |
| 后端单测 | 归档范围 | 仅归档同分支路径的 Checkpoint（不影响其他分支） |
| 后端单测 | 回滚逻辑 | checksum 校验通过时正确回滚；校验失败时拒绝 |
| 后端单测 | 分支创建 | 正确生成 branch_id；更新 Checkpoint 分支信息 |
| 后端单测 | 权限校验 | 非任务所有者返回 403 |
| 集成测试 | 完整回滚流程 | 选择 Checkpoint → 确认 → 恢复 → 状态正确 |
| 集成测试 | 回滚后继续迭代 | 新迭代使用新 branch_id |
| 集成测试 | 归档可追溯 | 归档 Checkpoint 可查询但有特殊标识 |
| 前端测试 | Checkpoint 列表 UI | 列表显示正确；归档项样式区分 |
| 前端测试 | 回滚确认对话框 | 对话框显示/确认/取消交互 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 复用 `backend/src/core/iteration_engine/checkpoint.rs` 的 `verify_checksum`
- 复用 `backend/src/core/iteration_engine/recovery.rs` 的 `recover_from_checkpoint`
- 复用 `backend/src/infra/db/repositories/checkpoint_repo.rs` 的 Checkpoint 查询
- `backend/src/api/routes/checkpoints.rs` 已有列表接口，务必在原路由扩展
- 遵循 Repository 模式访问数据库

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 7 / Story 7.3）
- PRD 可靠性：`docs/project-planning-artifacts/prd.md#能力区域 8: 可靠性与恢复`
- 架构（Checkpoint）：`docs/project-planning-artifacts/architecture.md#8. 可靠性与恢复`
- Epic 7 规划复核：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`
- Story 7.1（前序）：`docs/implementation-artifacts/7-1-checkpoint-auto-save.md`
- Story 7.2（前序）：`docs/implementation-artifacts/7-2-breakpoint-recovery-and-exception-handling.md`
- Checkpoint 实现：`backend/src/core/iteration_engine/checkpoint.rs`
- Recovery 实现：`backend/src/core/iteration_engine/recovery.rs`
- Checkpoint Repo：`backend/src/infra/db/repositories/checkpoint_repo.rs`

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

- `cd backend && cargo test`
- `cd backend && cargo test --test checkpoint_repo_test`
- `cd backend && cargo test`
- `cd backend && cargo test --test checkpoints_api_test --test recovery_api_test`
- `cd backend && cargo test --test history_api_test`
- `cd backend && cargo run --bin gen-types`
- `cd backend && cargo test rollback_updates_context_branch_extension`
- `cd backend && cargo test --test recovery_api_test`
- `cd backend && cargo test --test ws_pause_resume_integration_test`
- `cd backend && cargo test`
- `cd frontend && npx vitest --run src/features/checkpoint-recovery/components/CheckpointList.test.tsx src/features/checkpoint-recovery/components/RollbackConfirmDialog.test.tsx src/features/user-intervention/history/HistoryPanel.test.tsx`
- `cd frontend && npx vitest --run`
- `cd frontend && npm run build`
- `cd backend && cargo test`
- `cd backend && cargo fmt -- --check`
- `cd backend && cargo clippy --all-targets --all-features`
- `cd frontend && npm test -- --run`
- `cd frontend && npm run lint`
- `cd frontend && npm run build`

### Completion Notes List

- ✅ 新增回滚支持迁移（archived_at/archive_reason/pass_rate_summary）并补齐 schema 测试覆盖。
- ✅ 扩展 CheckpointRepo（摘要列表/归档/详情）并补齐归档范围与摘要查询测试；Checkpoint 保存补齐通过率摘要提取。
- ✅ 实现回滚核心流程（校验/归档/新分支/事务/日志/校验和更新），新增 rollback 单元测试并确保归档节点不参与恢复。
- ✅ 扩展 Checkpoint 列表与回滚 API（含权限校验与 OpenAPI），补齐 API 测试覆盖。
- ✅ 新增任务历史聚合 API 并调整历史面板改走单一入口（History），补齐前后端测试覆盖。
- ✅ 新增 CheckpointList / RollbackConfirmDialog 组件与回滚服务 Hook，历史面板集成回滚入口并提示结果。
- ✅ 补齐归档不可回滚与分支上下文更新测试，修复 WS 暂停/恢复测试事件过滤；完成类型生成与全量回归。

### File List

- backend/migrations/012_checkpoint_rollback_support.sql
- docs/implementation-artifacts/sprint-status.yaml
- backend/tests/checkpoint_schema_test.rs
- backend/src/infra/db/repositories/checkpoint_repo.rs
- backend/src/domain/models/checkpoint.rs
- backend/src/domain/models/recovery.rs
- backend/src/domain/models/history.rs
- backend/src/domain/models/mod.rs
- backend/src/bin/gen-types.rs
- backend/src/core/iteration_engine/checkpoint.rs
- backend/src/core/iteration_engine/recovery.rs
- backend/src/infra/db/repositories/optimization_task_repo.rs
- backend/tests/checkpoint_repo_test.rs
- backend/tests/checkpoint_checksum_test.rs
- backend/tests/checkpoints_api_test.rs
- backend/tests/recovery_api_test.rs
- backend/tests/history_api_test.rs
- backend/tests/ws_pause_resume_integration_test.rs
- backend/src/api/routes/checkpoints.rs
- backend/src/api/routes/recovery.rs
- backend/src/api/routes/history.rs
- backend/src/api/routes/docs.rs
- backend/src/api/routes/mod.rs
- backend/src/main.rs
- backend/src/domain/types/extensions.rs
- backend/src/domain/types/mod.rs
- frontend/src/App.routes.test.tsx
- frontend/src/features/user-intervention/history/HistoryPanel.tsx
- frontend/src/features/user-intervention/history/HistoryPanel.test.tsx
- frontend/src/features/user-intervention/history/hooks/useTaskHistory.ts
- frontend/src/features/user-intervention/history/services/taskHistoryService.ts
- frontend/src/features/user-intervention/history/index.ts
- frontend/src/features/checkpoint-recovery/components/CheckpointList.test.tsx
- frontend/src/features/checkpoint-recovery/components/CheckpointList.tsx
- frontend/src/features/checkpoint-recovery/components/RollbackConfirmDialog.test.tsx
- frontend/src/features/checkpoint-recovery/components/RollbackConfirmDialog.tsx
- frontend/src/features/checkpoint-recovery/hooks/useCheckpointList.ts
- frontend/src/features/checkpoint-recovery/hooks/useRollback.ts
- frontend/src/features/checkpoint-recovery/services/recoveryService.ts
- frontend/src/lib/formatters.ts
- frontend/src/types/generated/models/CheckpointListResponse.ts
- frontend/src/types/generated/models/CheckpointSummary.ts
- frontend/src/types/generated/models/PassRateSummary.ts
- frontend/src/types/generated/models/RollbackRequest.ts
- frontend/src/types/generated/models/RollbackResponse.ts
- frontend/src/types/generated/models/TaskHistoryResponse.ts
- docs/implementation-artifacts/validation-report-2026-01-19-093359.md
- docs/implementation-artifacts/7-3-historical-checkpoint-rollback.md

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] 回滚事务安全：改为在事务提交前验证恢复上下文，失败则回滚。`backend/src/core/iteration_engine/recovery.rs:352`
- [x] [HIGH] Checkpoint 列表分页加载，支持加载全部可用项（含归档）。`frontend/src/features/user-intervention/history/HistoryPanel.tsx:45`
- [x] [MEDIUM] current_branch_id 改为取最新未归档分支，避免误判。`backend/src/api/routes/checkpoints.rs:132` `backend/src/api/routes/history.rs:164`
- [x] [MEDIUM] history 端 iterations_limit 增加上限保护。`backend/src/api/routes/history.rs:95`
- [x] [MEDIUM] 回滚幂等性保护：重复回滚返回幂等结果。`backend/src/core/iteration_engine/recovery.rs:340`
- [x] [MEDIUM] 回滚日志 prev_state/new_state 语义修正。`backend/src/core/iteration_engine/recovery.rs:386`
- [x] [MEDIUM] 回滚上下文重建提前到事务前，并使用过滤后的 checkpoint 列表，避免单连接池超时。`backend/src/core/iteration_engine/recovery.rs:360`
- [x] [LOW] Checkpoint 摘要查询仅选必要字段，避免列表性能浪费。`backend/src/infra/db/repositories/checkpoint_repo.rs:173`
- [x] [LOW] 前端时间/通过率格式化函数抽取复用。`frontend/src/lib/formatters.ts`
- [x] [LOW] File List 同步补齐本 Story 与 validation 报告。`docs/implementation-artifacts/7-3-historical-checkpoint-rollback.md:609`

### Decisions

- [x] 采用分页 + “加载更多”方式展示 Checkpoint，满足“可查看全部”且控制单次加载量。

### Risks / Tech Debt

- [ ] 归档边界仍以时间戳 + 迭代号为判定，若未来出现乱序写入需补充更强的分支链路校验。

### Follow-ups

- [x] Review Follow-ups 已全部关闭。
