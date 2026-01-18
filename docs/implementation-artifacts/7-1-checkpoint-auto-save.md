# Story 7.1: Checkpoint 自动保存

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 7-1-checkpoint-auto-save

## Epic 7 开工门槛（必须先满足）

> ⚠️ **重要**：本 Story 开工前需确认以下门槛已完成或已有明确计划。
> 跟踪文件：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`

- [ ] **P1 Checkpoint ↔ pause_state 收敛设计完成并评审通过**
  - Owner：Winston（Architect）+ Charlie（Senior Dev）
  - 成功标准：Checkpoint 必须覆盖用户介入状态（RunControlState + IterationArtifacts + UserGuidance）
  - 交付物：收敛设计文档与迁移路径

- [ ] **P2 iterations ↔ checkpoints 历史口径统一策略明确**
  - Owner：Winston（Architect）
  - 成功标准：历史查询口径统一为单一入口，避免双轨漂移
  - 交付物：历史口径统一策略文档

- [ ] **P3 恢复/回滚测试矩阵与验证脚本就绪**
  - Owner：Dana（QA Engineer）
  - 成功标准：必测场景清单（断电/断网/崩溃/跨版本）与验证脚本
  - 交付物：测试矩阵与验证脚本

- [ ] **P4 Epic 7 规划复核会议完成并更新实现路径**
  - Owner：Bob（Scrum Master）
  - 成功标准：Epic 7 路径更新说明写入 epics.md
  - 证据：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`

## Key Decisions (MVP)

- **Checkpoint 时机**：在每个迭代步骤（Layer 1/2/3/4 完成后）自动保存 Checkpoint。
- **Checkpoint 内容**：必须包含 `task_id`、`iteration`、`state`（IterationState）、`run_control_state`（RunControlState）、`current_prompt`、`rule_system`、`artifacts`（IterationArtifacts）、`user_guidance`、`timestamp`、`checksum`。
- **分支与谱系**：对齐现有 Checkpoint 模型，包含 `branch_id` / `parent_id` / `lineage_type`（用于回滚与分支治理）。
- **持久化策略**：使用 SQLite WAL 模式 + FULL synchronous 确保数据安全（NFR6）。
- **完整性校验**：每个 Checkpoint 需生成 checksum，恢复时校验完整性（NFR7）。
- **内存管理**：内存中仅保留最近 N 个 Checkpoint，旧 Checkpoint 持久化到磁盘后释放内存（NFR25）。
- **与 pause_state 关系**：本 Story 实现完整 Checkpoint 机制，pause_state 作为过渡方案在本 Story 中保持兼容，后续 Story 收敛替换。
- **空闲自动保存**：无运行任务时 5 分钟自动保存当前状态。
- **AR2 遵循**：所有 Checkpoint 操作需记录 correlationId，支持全链路追踪。
- **日志规范**：遵循 A2 日志字段齐全要求（correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp）。

## Story

As a 系统,
I want 在每个迭代步骤自动保存 Checkpoint,
so that 可以在任何时刻恢复到已保存的状态。

## Acceptance Criteria

1. **Given** 优化任务正在执行
   **When** 完成一个迭代步骤（如 Layer 1/2/3/4 执行完毕）
   **Then** 自动保存当前状态为 Checkpoint
   **And** 使用 WAL + FULL synchronous 模式确保数据持久化（NFR6）

2. **Given** 保存 Checkpoint
   **When** 写入完成
   **Then** 生成并存储完整性校验 checksum（SHA-256）
   **And** Checkpoint 包含：任务 ID、当前迭代轮次、IterationState、RunControlState、current_prompt、rule_system、artifacts（含 patterns/candidate_prompts/user_guidance）、branch_id/parent_id/lineage_type、时间戳
   **And** 提供完整性校验能力，支撑 NFR7（Checkpoint 完整性 100%）

3. **Given** 系统运行中
   **When** 内存中 Checkpoint 数量超过阈值（默认 10 个）
   **Then** 旧 Checkpoint 持久化到磁盘后释放内存
   **And** 遵循内存管理策略（NFR25）
   **And** 提供内存使用监控与告警阈值配置

4. **Given** 无运行任务（RunControlState ≠ Running）且存在最近一次可恢复的任务上下文
   **And** 距离上一次 Checkpoint 保存 ≥ 5 分钟
   **When** 空闲计时器触发
   **Then** 自动保存当前任务的状态快照（如有未保存的上下文）
   **And** 计时器在用户介入/状态变更（暂停/继续/编辑）后重置

5. **Given** 迭代引擎检测到 Checkpoint 保存点
   **When** 调用 checkpoint_save
   **Then** 同步将状态写入 `checkpoints` 表
   **And** 操作记录 tracing 日志（包含 correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp）

6. **Given** Checkpoint 保存失败
   **When** 发生 IO 错误或数据库错误
   **Then** 记录错误日志并触发告警
   **And** 不阻塞当前迭代（降级策略：记录错误后继续执行）
   **And** 错误信息遵循 NFR24 统一错误文案规范

7. **Given** checkpoints 表中存在数据
   **When** 查询特定任务的 Checkpoint 列表
   **Then** 按 created_at 降序返回
   **And** 每个 Checkpoint 包含 checksum 用于完整性验证

8. **Given** 用户介入操作（暂停/编辑产物/引导）已保存在 pause_state
   **When** 下一个 Checkpoint 保存点触发
   **Then** Checkpoint 包含用户介入状态（RunControlState + IterationArtifacts + UserGuidance）
   **And** 确保恢复后可正确还原用户介入上下文

## Tasks / Subtasks

- [x] 后端：依赖与模块注册（AC: 2,5）
  - [x] 在 `backend/Cargo.toml` 添加依赖 `sha2`
  - [x] 在 `backend/src/domain/models/mod.rs` 注册 `checkpoint` 模块并导出类型
  - [x] 在 `backend/src/core/iteration_engine/mod.rs` 注册 `checkpoint` 模块
  - [x] 在 `backend/src/infra/db/repositories/mod.rs` 注册 `checkpoint_repo` 模块并导出 Repo

- [x] 后端：数据库 Schema 设计（AC: 1,2,7）
  - [x] 创建迁移 `backend/migrations/010_checkpoints_table.sql`
  - [x] 创建 `checkpoints` 表：id, task_id, iteration, state (TEXT/JSON), run_control_state, prompt, rule_system (JSON), artifacts (JSON), user_guidance (JSON), branch_id, parent_id, lineage_type, branch_description, checksum, created_at
  - [x] 添加索引：`idx_checkpoints_task_id`、`idx_checkpoints_created_at`
  - [x] 运行迁移并验证

- [x] 后端：Checkpoint 数据结构定义（AC: 2,8）
  - [x] 在 `backend/src/domain/models/checkpoint.rs` 定义 `CheckpointEntity`（数据库实体）
  - [x] 扩展现有 `Checkpoint` 结构体（algorithm.rs）或创建 `CheckpointFull` 包含完整字段
  - [x] 定义 `CheckpointCreateRequest` DTO
  - [x] 定义 `CheckpointResponse` DTO（API 返回）
  - [x] 包含 `artifacts: IterationArtifacts`、`user_guidance: Option<UserGuidance>`、`run_control_state: RunControlState`
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增 DTO

- [x] 后端：Checkpoint Repository 实现（AC: 1,5,7）
  - [x] 在 `backend/src/infra/db/repositories/` 创建 `checkpoint_repo.rs`
  - [x] 实现 `create_checkpoint(checkpoint: CheckpointEntity) -> Result<CheckpointEntity>`
  - [x] 实现 `get_checkpoint_by_id(id: &str) -> Result<Option<CheckpointEntity>>`
  - [x] 实现 `list_checkpoints_by_task(task_id: &str, limit: u32) -> Result<Vec<CheckpointEntity>>`
  - [x] 实现 `delete_old_checkpoints(task_id: &str, keep_count: u32) -> Result<u32>`
  - [x] 确保使用 WAL 模式写入（NFR6）

- [x] 后端：Checkpoint 完整性校验（AC: 2,7）
  - [x] 在 `backend/src/core/iteration_engine/checkpoint.rs` 实现 checksum 计算（SHA-256）
  - [x] 实现 `compute_checksum(req: &CheckpointCreateRequest) -> String`
  - [x] 实现 `verify_checksum(checkpoint: &CheckpointEntity) -> bool`
  - [x] checksum 覆盖所有关键字段（task_id/iteration/state/run_control_state/prompt/rule_system/artifacts/user_guidance/branch_id/parent_id/lineage_type）

- [x] 后端：Checkpoint 核心逻辑实现（AC: 1,3,4,5,6,8）
  - [x] 在 `backend/src/core/iteration_engine/checkpoint.rs` 实现主模块
  - [x] 实现 `save_checkpoint(ctx: &OptimizationContext, user_id: &str, correlation_id: &str) -> Result<CheckpointEntity>`
  - [x] 从 `OptimizationContext` 提取完整状态（含 run_control_state、artifacts、user_guidance）
  - [x] 集成 pause_state 中的用户介入数据（兼容期）
  - [x] 实现内存 Checkpoint 管理（LRU 缓存，阈值 10 个）
  - [x] 实现磁盘持久化与内存释放策略（NFR25）
  - [x] 实现降级策略（保存失败不阻塞迭代）

- [x] 后端：迭代引擎集成 Checkpoint（AC: 1,5）
  - [x] 修改 `backend/src/core/optimization_engine/common.rs`
  - [x] 在 `checkpoint_pause_if_requested` 中增加 Checkpoint 保存调用
  - [x] 在每个 Layer 完成后调用 `save_checkpoint`
  - [x] 确保 correlationId 透传（AR2）
  - [x] 修改 `backend/src/core/optimization_engine/default_impl.rs` 集成 Checkpoint

- [x] 后端：空闲自动保存（AC: 4）
  - [x] 实现 5 分钟空闲保存计时器
  - [x] 使用 tokio 定时任务
  - [x] 检测无运行任务时触发保存

- [x] 后端：Checkpoint API（AC: 7）
  - [x] 在 `backend/src/api/routes/checkpoints.rs` 创建路由
  - [x] 实现 `GET /api/v1/tasks/{task_id}/checkpoints` 获取 Checkpoint 列表
  - [x] 实现 `GET /api/v1/checkpoints/{checkpoint_id}` 获取单个 Checkpoint 详情
  - [x] 添加权限校验（仅任务所有者可访问）
  - [x] 在 `backend/src/api/routes/mod.rs` 导出 `checkpoints` 模块
  - [x] 在 `backend/src/main.rs` 挂载路由并加鉴权中间件（/api/v1/tasks/{task_id}/checkpoints 与 /api/v1/checkpoints/{checkpoint_id}）

- [x] 后端：日志与监控（AC: 5,6）
  - [x] 每次 Checkpoint 保存记录 tracing 日志
  - [x] 日志包含 A2 必填字段：correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp
  - [x] Checkpoint 失败记录 ERROR 级别日志
  - [x] 预留内存使用监控指标接口

- [x] 前端：Checkpoint 可视化（可选，MVP 最小实现）
  - [x] 在 `frontend/src/features/checkpoint-recovery/` 预留目录结构
  - [x] 创建 `checkpointService.ts` 服务层封装
  - [x] 创建 `useCheckpoints.ts` TanStack Query hooks
  - [x] 可选：在历史面板显示 Checkpoint 列表

- [x] 测试与回归（AC: 1-8）
  - [x] 后端单测：Checkpoint 保存与加载
  - [x] 后端单测：checksum 计算与校验
  - [x] 后端单测：内存管理与磁盘持久化
  - [x] 后端单测：API 权限校验
  - [x] 集成测试：完整迭代流程中 Checkpoint 自动保存
  - [x] 集成测试：Checkpoint 包含用户介入状态
  - [x] 集成测试：保存失败降级策略
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

- [x] [AI-Review] Review 结论已沉淀到 `## Review Notes` 并完成修复项落地

## Dev Notes

### Developer Context (Read This First)

- **现状基线（已完成）**：
  - Epic 6 已完成用户介入功能，提供关键基础设施：
    - `RunControlState` 状态机（Idle/Running/Paused/Stopped）
    - `PauseController` 暂停控制器（`backend/src/core/iteration_engine/pause_state.rs`）
    - `PauseStateSnapshot` 暂停状态快照
    - `IterationArtifacts` 类型定义（patterns/candidate_prompts/user_guidance）
    - correlationId 全链路追踪
  - `Checkpoint` 结构体已定义在 `backend/src/domain/models/algorithm.rs`（基础字段）
  - 架构中预留 `checkpoint.rs` 和 `checkpoint_repo.rs`（尚未实现）

- **Epic 7 全景（便于对齐业务价值与范围）**：
  - **7.1 Checkpoint 自动保存（本 Story，FR52）**
  - 7.2 断点恢复与异常处理（FR53）
  - 7.3 历史 Checkpoint 回滚（FR54）
  - 7.4 完整迭代历史记录（FR55）

- **业务价值（为什么做）**：为系统提供可靠的状态持久化能力，确保在任何异常情况下（崩溃、断电、网络中断）都能恢复到已保存的状态，实现用户的"安全感"（来源：PRD 能力区域 8 / FR52 / NFR5-7）。

- **依赖关系**：
  - 依赖 Epic 6 提供的 `RunControlState` 状态机
  - 依赖 Epic 6 提供的 `IterationArtifacts` 类型
  - 依赖 `OptimizationContext` 上下文结构
  - 需与 `pause_state` 保持兼容（过渡期）

- **范围边界（必须遵守）**：
  - 本 Story 仅实现 Checkpoint 保存功能
  - 恢复功能由 Story 7.2 承接
  - 回滚功能由 Story 7.3 承接
  - 不修改 pause_state 核心逻辑，仅做兼容集成
  - 不实现前端完整 Checkpoint 管理 UI（仅预留接口）
  - 历史展示仍以 `iterations` 为权威来源，统一入口由 Story 7.4 处理

### 与 pause_state 的关系

| 功能 | pause_state（现状） | Checkpoint（本 Story） |
| --- | --- | --- |
| 触发时机 | 用户请求暂停 | 每个 Layer 完成后自动 |
| 存储位置 | 文件系统（JSON 文件） | SQLite `checkpoints` 表 |
| 包含内容 | 最小上下文快照 | 完整状态（含 artifacts/guidance） |
| 完整性校验 | 无 | SHA-256 checksum |
| 内存管理 | 无 | LRU 缓存 + 磁盘持久化 |
| WAL 模式 | 否 | 是（NFR6） |

**兼容期策略**：
- Checkpoint 保存时从 `PauseController` 获取用户介入数据（artifacts/user_guidance）
- pause_state 继续处理暂停/继续/编辑逻辑
- 后续 Story 将 pause_state 数据迁移到 Checkpoint 机制
- P1 收敛设计文档完成后，以该文档为实现与迁移路径的唯一准则

### Database Schema

```sql
-- 位置：backend/migrations/010_checkpoints_table.sql

-- Checkpoint 表
CREATE TABLE IF NOT EXISTS checkpoints (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    iteration INTEGER NOT NULL,
    state TEXT NOT NULL,                    -- IterationState JSON（用于映射 PRD 的 step 粒度）
    run_control_state TEXT NOT NULL,        -- RunControlState 枚举值
    prompt TEXT NOT NULL,                   -- current_prompt
    rule_system TEXT NOT NULL,              -- RuleSystem JSON
    artifacts TEXT,                         -- IterationArtifacts JSON (nullable)
    user_guidance TEXT,                     -- UserGuidance JSON (nullable)
    branch_id TEXT NOT NULL,                -- 分支 ID（与现有 Checkpoint 模型一致）
    parent_id TEXT,                         -- 父 Checkpoint（用于回滚/分支）
    lineage_type TEXT NOT NULL,             -- LineageType 枚举值
    branch_description TEXT,                -- 分支描述（可选）
    checksum TEXT NOT NULL,                 -- SHA-256 完整性校验
    created_at INTEGER NOT NULL,            -- Unix ms 时间戳
    
    FOREIGN KEY (task_id) REFERENCES optimization_tasks(id) ON DELETE CASCADE
    -- workspace_id 可通过 optimization_tasks 关联获取
);

-- 索引：按任务查询
CREATE INDEX IF NOT EXISTS idx_checkpoints_task_id ON checkpoints(task_id);

-- 索引：按时间排序
CREATE INDEX IF NOT EXISTS idx_checkpoints_created_at ON checkpoints(created_at DESC);
```

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/models/checkpoint.rs

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::domain::models::LineageType;
use crate::domain::types::{IterationArtifacts, IterationState, RunControlState, UserGuidance, RuleSystem};

/// Checkpoint 数据库实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointEntity {
    pub id: String,
    pub task_id: String,
    pub iteration: u32,
    pub state: IterationState,
    pub run_control_state: RunControlState,
    pub prompt: String,
    pub rule_system: RuleSystem,
    pub artifacts: Option<IterationArtifacts>,
    pub user_guidance: Option<UserGuidance>,
    pub branch_id: String,
    pub parent_id: Option<String>,
    pub lineage_type: LineageType,
    pub branch_description: Option<String>,
    pub checksum: String,
    pub created_at: i64,  // Unix ms
}

/// Checkpoint 创建请求（内部使用）
#[derive(Debug, Clone)]
pub struct CheckpointCreateRequest {
    pub task_id: String,
    pub iteration: u32,
    pub state: IterationState,
    pub run_control_state: RunControlState,
    pub prompt: String,
    pub rule_system: RuleSystem,
    pub artifacts: Option<IterationArtifacts>,
    pub user_guidance: Option<UserGuidance>,
    pub branch_id: String,
    pub parent_id: Option<String>,
    pub lineage_type: LineageType,
    pub branch_description: Option<String>,
}

/// 模型关系说明
/// - CheckpointEntity：DB 持久化实体
/// - Checkpoint（domain/models/algorithm.rs）：领域模型（含 lineage 字段）
/// - 建议实现 From<CheckpointEntity> for Checkpoint / From<CheckpointCreateRequest> to CheckpointEntity

/// Checkpoint API 响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CheckpointResponse {
    pub id: String,
    pub task_id: String,
    pub iteration: u32,
    pub state: String,
    pub run_control_state: String,
    pub prompt_preview: String,           // 前 200 字符
    pub has_artifacts: bool,
    pub has_user_guidance: bool,
    pub checksum: String,
    pub created_at: String,               // ISO 8601
}

/// Checkpoint 列表响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CheckpointListResponse {
    pub checkpoints: Vec<CheckpointResponse>,
    pub total: u32,
}
```

### Checkpoint 核心模块

```rust
/// 位置：backend/src/core/iteration_engine/checkpoint.rs

use sha2::{Sha256, Digest};
use crate::domain::models::checkpoint::{CheckpointEntity, CheckpointCreateRequest};
use crate::domain::models::LineageType;
use crate::domain::types::OptimizationContext;
use crate::core::iteration_engine::pause_state::PauseController;
use crate::infra::db::repositories::checkpoint_repo::CheckpointRepo;

/// 计算 Checkpoint checksum（SHA-256）
pub fn compute_checksum(req: &CheckpointCreateRequest) -> String {
    let mut hasher = Sha256::new();
    hasher.update(req.task_id.as_bytes());
    hasher.update(req.iteration.to_le_bytes());
    hasher.update(format!("{:?}", req.state).as_bytes());
    hasher.update(format!("{:?}", req.run_control_state).as_bytes());
    hasher.update(req.prompt.as_bytes());
    hasher.update(serde_json::to_string(&req.rule_system).unwrap_or_default().as_bytes());
    if let Some(ref artifacts) = req.artifacts {
        hasher.update(serde_json::to_string(artifacts).unwrap_or_default().as_bytes());
    }
    if let Some(ref guidance) = req.user_guidance {
        hasher.update(serde_json::to_string(guidance).unwrap_or_default().as_bytes());
    }
    hasher.update(req.branch_id.as_bytes());
    if let Some(ref parent_id) = req.parent_id {
        hasher.update(parent_id.as_bytes());
    }
    hasher.update(format!("{:?}", req.lineage_type).as_bytes());
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
        state: checkpoint.state.clone(),
        run_control_state: checkpoint.run_control_state.clone(),
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

/// 从 OptimizationContext 保存 Checkpoint
pub async fn save_checkpoint(
    ctx: &OptimizationContext,
    pause_controller: Option<&PauseController>,
    user_id: &str,
    correlation_id: &str,
    repo: &CheckpointRepo,
) -> Result<CheckpointEntity, CheckpointError> {
    // 从 pause_controller 获取用户介入数据（兼容期）
    let (artifacts, user_guidance) = if let Some(pc) = pause_controller {
        (pc.get_artifacts().await, pc.get_guidance().await)
    } else {
        (None, None)
    };

    let req = CheckpointCreateRequest {
        task_id: ctx.task_id.clone(),
        iteration: ctx.iteration,
        state: ctx.state.clone(),
        run_control_state: ctx.run_control_state.clone(),
        prompt: ctx.current_prompt.clone(),
        rule_system: ctx.rule_system.clone(),
        artifacts,
        user_guidance,
        branch_id: ctx.task_id.clone(), // 示例：按任务或会话生成 branch_id（以实际分支策略为准）
        parent_id: None,                // 由调用方决定是否指定 parent_id
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
        created_at: chrono::Utc::now().timestamp_millis(),
    };

    // 记录日志
    tracing::info!(
        user_id = %user_id,
        task_id = %entity.task_id,
        correlation_id = %correlation_id,
        prev_state = ?entity.state,
        new_state = ?entity.state,
        iteration_state = ?entity.state,
        iteration = entity.iteration,
        state = ?entity.state,
        action = "checkpoint_saved",
        timestamp = %entity.created_at,
        "Checkpoint 已保存"
    );

    repo.create_checkpoint(entity.clone()).await?;
    Ok(entity)
}
```

### Suggested API Endpoints

```
# 获取任务的 Checkpoint 列表
GET /api/v1/tasks/{task_id}/checkpoints
Query: limit (可选, 默认 20, 最大 100)
Response: ApiResponse<CheckpointListResponse>
权限校验：仅任务所有者可访问

# 获取单个 Checkpoint 详情
GET /api/v1/checkpoints/{checkpoint_id}
Response: ApiResponse<CheckpointResponse>
权限校验：仅任务所有者可访问
```

### Dev Agent Guardrails（避免常见踩坑）

- **WAL 模式必须启用**：确保 SQLite 连接使用 WAL 模式，写入使用 FULL synchronous（NFR6）。
- **checksum 覆盖关键字段**：task_id/iteration/state/run_control_state/prompt/rule_system/artifacts/user_guidance/branch_id/parent_id/lineage_type 必须参与 checksum 计算。
- **内存管理不能遗漏**：实现 LRU 缓存，超过阈值时持久化到磁盘（NFR25）。
- **降级策略必须实现**：Checkpoint 保存失败不能阻塞迭代，记录错误后继续执行。
- **兼容 pause_state**：从 PauseController 获取 artifacts/user_guidance，不破坏现有暂停功能。
- **不要忘记 correlationId**：所有操作必须携带 correlationId（AR2）。
- **日志字段齐全**：必须包含 correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp。
- **权限校验严格**：只有任务所有者可以访问 Checkpoint。
- **类型生成**：新增类型后运行 `cargo run --bin gen-types` 并提交产物。

### Technical Requirements（必须满足）

- SQLite 使用 WAL 模式 + FULL synchronous（NFR6）
- Checkpoint 完整性校验使用 SHA-256（NFR7）
- checksum 必须覆盖所有持久化字段（含 run_control_state/user_guidance/lineage 字段）
- 内存 Checkpoint 数量阈值默认 10 个，支持配置（NFR25）
- API 响应使用 `ApiResponse<T>` 统一结构
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/core/iteration_engine/checkpoint.rs`：核心逻辑
  - `backend/src/infra/db/repositories/checkpoint_repo.rs`：数据访问
  - `backend/src/api/routes/checkpoints.rs`：API 路由
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **错误处理**：后端 `thiserror` + `anyhow`
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types`

### Library / Framework Requirements (Version Snapshot)

- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`（WAL 模式）
- sha2：用于 checksum 计算
- tokio：异步运行时 + 定时任务
- chrono：时间戳处理

### File Structure Requirements（落点约束）

**后端**：
- 数据库迁移：`backend/migrations/010_checkpoints_table.sql`（新增）
- Checkpoint 模型：`backend/src/domain/models/checkpoint.rs`（新增）
- 模型导出：`backend/src/domain/models/mod.rs`（更新）
- Checkpoint 核心逻辑：`backend/src/core/iteration_engine/checkpoint.rs`（新增/扩展）
- 模块导出：`backend/src/core/iteration_engine/mod.rs`（更新）
- Checkpoint Repository：`backend/src/infra/db/repositories/checkpoint_repo.rs`（新增）
- Repository 导出：`backend/src/infra/db/repositories/mod.rs`（更新）
- Checkpoint API：`backend/src/api/routes/checkpoints.rs`（新增）
- 路由导出：`backend/src/api/routes/mod.rs`（更新）
- 迭代引擎集成：`backend/src/core/optimization_engine/common.rs`（扩展）
- 迭代引擎集成：`backend/src/core/optimization_engine/default_impl.rs`（扩展）
- 主路由挂载：`backend/src/main.rs`（更新）

**前端**（预留）：
- 服务层：`frontend/src/features/checkpoint-recovery/services/checkpointService.ts`（新增）
- Hooks：`frontend/src/features/checkpoint-recovery/hooks/useCheckpoints.ts`（新增）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | Checkpoint 保存 | 正确保存所有字段；checksum 计算正确 |
| 后端单测 | checksum 校验 | 校验通过/失败场景 |
| 后端单测 | 内存管理 | 超过阈值时持久化旧 Checkpoint |
| 后端单测 | 降级策略 | 保存失败不阻塞迭代 |
| 后端单测 | API 权限校验 | 非任务所有者返回 403 |
| 集成测试 | 迭代流程 | 每个 Layer 完成后自动保存 Checkpoint |
| 集成测试 | 用户介入状态 | Checkpoint 包含 artifacts/user_guidance |
| 集成测试 | WAL 模式 | 验证 SQLite WAL 模式启用 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 复用 `backend/src/core/iteration_engine/` 目录结构
- 复用 `PauseController` 获取用户介入数据
- 遵循 Repository 模式访问数据库
- 与 `iterations` 表保持独立（历史口径统一由后续 Story 处理）

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 7 / Story 7.1）
- PRD 可靠性：`docs/project-planning-artifacts/prd.md#能力区域 8: 可靠性与恢复`
- 架构（Checkpoint）：`docs/project-planning-artifacts/architecture.md#8. 可靠性与恢复`
- Epic 7 规划复核：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`
- Epic 7 规划复核准备：`docs/implementation-artifacts/epic-7-planning-review-prep-2026-01-18.md`
- Epic 6 复盘：`docs/implementation-artifacts/epic-6-retro-2026-01-18.md`
- pause_state 实现：`backend/src/core/iteration_engine/pause_state.rs`
- 前序 Story learnings：`docs/implementation-artifacts/6-5-iteration-control-add-rounds-manual-terminate.md`

## Git Intelligence Summary

- Epic 6 关键提交：
  - `RunControlState` 状态机实现
  - `PauseController` 暂停控制器
  - `IterationArtifacts` 类型定义
  - iterations 表迁移（008）

## Latest Tech Information (Web/Registry Snapshot)

- 版本以本地依赖快照为准：`frontend/package.json` 与 `backend/Cargo.toml`
- 关键关注点：
  - sha2 crate 用于 checksum 计算
  - SQLx WAL 模式配置

## Project Context Reference

- 以 `docs/project-planning-artifacts/*.md`、`docs/developer-guides/*` 与现有代码为准

## Story Completion Status

- Status set to `done`
- Completion note: Code review 修复已完成（校验稳定性、空闲保存重置/测试、权限校验、WAL 校验、缓存指标/阈值等）

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

- ✅ checksum 计算改为稳定 JSON（避免 Debug/HashMap 非确定性）
- ✅ 记录 prev_iteration_state 修复 A2 日志语义
- ✅ 空闲自动保存计时器在用户介入/状态变更时重置，并补齐核心测试
- ✅ Checkpoint 详情按用户范围查询，统一 404 防枚举
- ✅ API 返回 integrityOk，列表总数使用 COUNT
- ✅ WAL synchronous=FULL 校验补齐测试
- ✅ 复合索引优化列表查询路径
- ✅ Cache 配置注入 + checkpoint-metrics 监控端点
- ✅ 前端历史面板展示 Checkpoint 列表并更新测试
- ✅ 已运行：`cargo test`、`cargo run --bin gen-types`、`npx vitest --run`、`npm run build`

### File List

- backend/Cargo.toml
- backend/Cargo.lock
- backend/src/api/routes/checkpoints.rs
- backend/src/api/routes/iteration_control.rs
- backend/src/api/routes/meta.rs
- backend/src/api/routes/mod.rs
- backend/src/bin/gen-types.rs
- backend/src/core/iteration_engine/checkpoint.rs
- backend/src/core/iteration_engine/mod.rs
- backend/src/core/iteration_engine/pause_state.rs
- backend/src/core/optimization_engine/alternate_impl.rs
- backend/src/core/optimization_engine/common.rs
- backend/src/core/optimization_engine/default_impl.rs
- backend/src/core/optimization_engine/mod.rs
- backend/src/domain/models/checkpoint.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/types/extensions.rs
- backend/src/domain/types/mod.rs
- backend/src/infra/db/pool.rs
- backend/src/infra/db/repositories/checkpoint_repo.rs
- backend/src/infra/db/repositories/mod.rs
- backend/src/main.rs
- backend/src/shared/config.rs
- backend/migrations/010_checkpoints_table.sql
- backend/tests/auth_integration_test.rs
- backend/tests/checkpoint_cache_test.rs
- backend/tests/checkpoint_checksum_test.rs
- backend/tests/checkpoint_degrade_test.rs
- backend/tests/checkpoint_exports_test.rs
- backend/tests/checkpoint_guidance_integration_test.rs
- backend/tests/checkpoint_layer_integration_test.rs
- backend/tests/checkpoint_models_test.rs
- backend/tests/checkpoint_repo_test.rs
- backend/tests/checkpoint_save_test.rs
- backend/tests/checkpoint_schema_test.rs
- backend/tests/checkpoint_wal_test.rs
- backend/tests/checkpoints_api_test.rs
- backend/tests/error_handling_test.rs
- backend/tests/iterations_api_test.rs
- backend/tests/meta_iteration_stages_api_test.rs
- backend/tests/optimization_tasks_api_test.rs
- backend/tests/test_set_dify_variables_api_test.rs
- backend/tests/test_set_generic_config_api_test.rs
- backend/tests/test_set_templates_api_test.rs
- backend/tests/test_sets_api_test.rs
- backend/tests/workspaces_api_test.rs
- backend/tests/ws_guidance_integration_test.rs
- backend/tests/ws_pause_resume_integration_test.rs
- docs/implementation-artifacts/7-1-checkpoint-auto-save.md
- docs/implementation-artifacts/epic-7-checkpoint-pause-state-convergence-design-2026-01-18.md
- docs/implementation-artifacts/epic-7-history-source-unification-strategy-2026-01-18.md
- docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md
- docs/implementation-artifacts/epic-7-recovery-rollback-test-matrix-2026-01-18.md
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/validation-report-20260118-141222.md
- frontend/src/features/checkpoint-recovery/hooks/useCheckpoints.ts
- frontend/src/features/checkpoint-recovery/services/checkpointService.ts
- frontend/src/features/user-intervention/history/HistoryPanel.test.tsx
- frontend/src/features/user-intervention/history/HistoryPanel.tsx
- frontend/src/types/generated/models/CheckpointListResponse.ts
- frontend/src/types/generated/models/CheckpointResponse.ts
- scripts/epic-7/

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] 修复 checksum 非确定性（HashMap/Debug 输出导致校验不稳定）
- [x] 暂停保存时 RunControlState 写入为 Paused，确保用户介入状态可恢复
- [x] 空闲自动保存计时器在暂停/继续/编辑后重置，并补齐核心测试
- [x] Checkpoint 详情权限校验改为 user-scope 查询，避免资源枚举
- [x] API 返回 integrityOk，提供完整性校验能力
- [x] 列表 total 使用 COUNT，总数准确
- [x] WAL 测试补齐 synchronous=FULL 校验
- [x] 复合索引优化列表查询路径
- [x] 监控与阈值配置：启动注入默认值 + 暴露 checkpoint-metrics

### Decisions

- [x] Checkpoint 详情统一 404（未命中/无权限）以避免泄露资源存在性
- [x] 通过 extensions 记录 prev_iteration_state，避免日志语义失真
- [x] 监控指标走 meta 路由，降低侵入性（不引入额外依赖）

### Risks / Tech Debt

- [ ] 无明显遗留风险（如需更严格防篡改，可在读路径强制校验失败即阻断）

### Follow-ups

- [ ] 无
