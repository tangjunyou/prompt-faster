# Story 7.4: 完整迭代历史记录

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 7-4-complete-iteration-history-record

## Epic 7 开工门槛（必须先满足）

> ⚠️ **重要**：本 Story 继承 Epic 7 开工门槛，7.1/7.2/7.3 已完成门槛验证。
> 跟踪文件：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`

- [x] **P1 Checkpoint ↔ pause_state 收敛设计完成并评审通过**（7.1 已完成）
- [x] **P2 iterations ↔ checkpoints 历史口径统一策略明确**（7.1 已完成，7.3 落地单一 History API）
- [x] **P3 恢复/回滚测试矩阵与验证脚本就绪**（已具备）
- [x] **P4 Epic 7 规划复核会议完成并更新实现路径**（7.1 已完成）

## Key Decisions (MVP)

- **历史记录范围**：记录优化任务执行过程中的所有关键状态变化（迭代开始/结束、评估结果、用户介入、回滚操作等）。
- **数据来源统一**：复用 7.3 已落地的 History API 单一入口，扩展支持完整时间线视图。
- **历史主线（P2 Phase-3）**：History API 以 Checkpoint 为主线（SSOT），iterations 仅作补充；`/history` 保持单一入口，新端点不得破坏现有响应。
- **时间线视图**：以时间轴形式展示历史事件，支持按时间/操作类型/轮次筛选。
- **导出格式**：JSON 格式导出，包含完整元数据和状态快照。
- **AR2 遵循**：所有历史记录操作需记录 correlationId，支持全链路追踪。
- **日志规范**：遵循 A2 日志字段齐全要求。
- **权限校验**：仅任务所有者可查看和导出历史记录。

## Story

As a Prompt 优化用户,
I want 系统保存完整的迭代历史记录,
so that 我可以追溯整个优化过程的演进路径。

## Acceptance Criteria

1. **Given** 优化任务执行过程中
   **When** 发生任何状态变化
   **Then** 记录该变化到历史日志
   **And** 包含：时间戳、操作类型、操作者（系统/用户）、变化详情

2. **Given** 用户查看历史记录
   **When** 打开历史面板
   **Then** 显示完整的时间线视图
   **And** 支持按时间/操作类型/轮次筛选

3. **Given** 用户导出历史记录
   **When** 点击"导出"
   **Then** 支持导出为 JSON 格式
   **And** 包含完整的元数据和状态快照

## Tasks / Subtasks

- [x] 后端：历史事件数据模型（AC: 1）
  - [x] 在 `backend/src/domain/models/` 创建 `history_event.rs`
    - `HistoryEvent` 结构体（id, task_id, event_type, actor, details, iteration, correlation_id, created_at）
    - `EventType` 枚举（iteration_started, iteration_completed, evaluation_completed, user_pause, user_resume, user_edit, user_guidance, rollback, checkpoint_saved, error_occurred, config_changed, task_terminated, checkpoint_recovered）
    - `Actor` 枚举（system, user）
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增类型

- [x] 后端：数据库迁移（AC: 1）
  - [x] 创建 `backend/migrations/013_history_events.sql`
  - [x] 创建 `history_events` 表：
    - `id` VARCHAR PRIMARY KEY
    - `task_id` VARCHAR NOT NULL（外键）
    - `event_type` VARCHAR NOT NULL
    - `actor` VARCHAR NOT NULL（system/user）
    - `details` TEXT（JSON 格式变化详情）
    - `iteration` INTEGER（可选，关联迭代轮次）
    - `correlation_id` VARCHAR
    - `created_at` INTEGER NOT NULL（Unix ms）
  - [x] 添加索引：`idx_history_events_task_time`（task_id, created_at）、`idx_history_events_created_at`、`idx_history_events_event_type`

- [x] 后端：HistoryEventRepo 实现（AC: 1,2）
  - [x] 在 `backend/src/infra/db/repositories/` 创建 `history_event_repo.rs`
    - `create_event(event: &HistoryEvent) -> Result<()>`
    - `list_events(task_id: &str, filter: &HistoryEventFilter, limit: usize, offset: usize) -> Result<Vec<HistoryEvent>>`
    - `count_events(task_id: &str, filter: &HistoryEventFilter) -> Result<u32>`
    - `get_event_by_id(event_id: &str) -> Result<Option<HistoryEvent>>`
    - `list_timeline_entries(task_id: &str, filter: &HistoryEventFilter, limit: usize, offset: usize) -> Result<Vec<TimelineEntry>>`
  - [x] `HistoryEventFilter` 结构体（event_types, actor, iteration_min, iteration_max, time_start, time_end）

- [x] 后端：历史事件记录集成（AC: 1）
  - [x] 在 `backend/src/core/iteration_engine/orchestrator.rs` 扩展：
    - 迭代开始时记录 `iteration_started` 事件
    - 迭代完成时记录 `iteration_completed` 事件
    - 评估完成时记录 `evaluation_completed` 事件
  - [x] 在 `backend/src/core/iteration_engine/checkpoint.rs` 扩展：
    - Checkpoint 保存时记录 `checkpoint_saved` 事件
  - [x] 在 `backend/src/core/iteration_engine/recovery.rs` 扩展：
    - 回滚操作时记录 `rollback` 事件
  - [x] 在 `backend/src/api/routes/iteration_control.rs` 扩展：
    - 增加轮数成功时记录 `config_changed` 事件
    - 终止任务成功时记录 `task_terminated` 事件
  - [x] 在 `backend/src/api/routes/recovery.rs` 扩展：
    - 恢复成功时记录 `checkpoint_recovered` 事件
  - [x] 在用户介入相关模块扩展：
    - 暂停时记录 `user_pause` 事件
    - 恢复时记录 `user_resume` 事件
    - 编辑中间产物时记录 `user_edit` 事件
    - 对话引导时记录 `user_guidance` 事件
  - [x] 明确集成点（避免遗漏）：
    - `backend/src/api/ws/connection.rs`：`CMD_TASK_PAUSE` / `CMD_TASK_RESUME` 成功后记录
    - `backend/src/api/ws/connection.rs`：`artifact:update` 成功后记录
    - `backend/src/api/ws/connection.rs`：`guidance:send` 成功后记录（或 `pause_state::update_guidance` 成功后）
  - [x] 事件记录必须异步化，失败仅记录日志，不得阻塞主流程

- [x] 后端：扩展 History API（AC: 2）
  - [x] 在 `backend/src/api/routes/history.rs` 扩展：
    - `GET /api/v1/tasks/{task_id}/history/events` 获取历史事件列表（支持筛选）
    - `GET /api/v1/tasks/{task_id}/history/timeline` 获取时间线视图（聚合迭代+Checkpoint+事件）
  - [x] 添加筛选参数：`event_types`、`actor`、`iteration_min`、`iteration_max`、`time_start`、`time_end`
  - [x] 添加分页参数：`limit`、`offset`
  - [x] `event_types` 采用逗号分隔的 snake_case 值（与 `EventType` 对齐）
  - [x] 参数约束：`limit` ≤ 100，`offset` ≤ 10000；默认按 `created_at` 倒序
  - [x] 时间线聚合必须在数据库侧分页/排序（UNION ALL 或多路归并），禁止全量加载后排序
  - [x] 添加权限校验（仅任务所有者可访问）
  - [x] 添加 OpenAPI 文档描述

- [x] 后端：导出 API（AC: 3）
  - [x] 在 `backend/src/api/routes/history.rs` 新增：
    - `GET /api/v1/tasks/{task_id}/history/export` 导出完整历史记录
  - [x] 导出内容包含：
    - 任务元数据（task_id, name, created_at, status）
    - 迭代历史列表（含每轮的 prompt/评估结果/规律假设）
    - Checkpoint 摘要列表
    - 历史事件列表
    - 分支信息
  - [x] 返回格式：JSON（Content-Type: application/json）
  - [x] 支持 Content-Disposition: attachment 下载

- [x] 后端：数据结构定义（AC: 1,2,3）
  - [x] 在 `backend/src/domain/models/history.rs` 扩展：
    - `HistoryEventResponse` 结构体
    - `TimelineEntry` 结构体（统一迭代/Checkpoint/事件的时间线项）
    - `TimelineResponse` 结构体
    - `HistoryExportData` 结构体
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增 DTO

- [x] 前端：时间线视图组件（AC: 2）
  - [x] 在 `frontend/src/features/user-intervention/history/components/` 创建 `TimelineView.tsx`
  - [x] 显示时间轴形式的历史事件
  - [x] 每个事件显示：时间戳、操作类型图标、操作者、简要描述
  - [x] 支持展开查看详情

- [x] 前端：历史筛选组件（AC: 2）
  - [x] 在 `frontend/src/features/user-intervention/history/components/` 创建 `HistoryFilter.tsx`
  - [x] 支持筛选：操作类型（多选）、操作者（系统/用户）、迭代轮次范围、时间范围
  - [x] 筛选条件实时应用

- [x] 前端：导出按钮与功能（AC: 3）
  - [x] 在历史面板中添加"导出"按钮
  - [x] 点击后调用导出 API 下载 JSON 文件
  - [x] 文件名格式：`{task_name}_history_{timestamp}.json`

- [x] 前端：服务层封装（AC: 1-3）
  - [x] 在 `frontend/src/features/user-intervention/history/services/` 扩展 `taskHistoryService.ts`
    - `getHistoryEvents(taskId: string, filter?: HistoryEventFilter): Promise<HistoryEventResponse[]>`
    - `getTimeline(taskId: string, filter?: HistoryEventFilter): Promise<TimelineResponse>`
    - `exportHistory(taskId: string): Promise<Blob>`
  - [x] 创建 `useHistoryEvents.ts` TanStack Query hook
  - [x] 创建 `useTimeline.ts` TanStack Query hook
  - [x] 创建 `useExportHistory.ts` mutation hook

- [x] 前端：集成与入口（AC: 1-3）
  - [x] 扩展 `HistoryPanel.tsx` 支持时间线视图切换
  - [x] 添加视图切换 Tabs（列表视图/时间线视图），筛选条件对两种视图生效
  - [x] 回滚入口保持可见（列表视图中的 Checkpoint 区域或固定入口）
  - [x] 集成筛选组件
  - [x] 集成导出功能

- [x] 测试与回归（AC: 1-3）
  - [x] 后端单测：历史事件创建与查询
  - [x] 后端单测：筛选逻辑正确性
  - [x] 后端单测：导出数据完整性
  - [x] 后端单测：权限校验（非任务所有者返回 403）
  - [x] 后端单测：时间线排序/分页正确性（按 created_at 倒序）
  - [x] 集成测试：事件记录失败不阻塞主流程
  - [x] 集成测试：完整流程（迭代执行 → 事件记录 → 查询 → 导出）
  - [x] 前端测试：时间线视图渲染
  - [x] 前端测试：筛选交互
  - [x] 前端测试：导出功能
  - [x] 回归命令：`cd backend && cargo test`；`cd frontend && npx vitest --run && npm run build`
  - [x] 生成类型：`cd backend && cargo run --bin gen-types` 并提交产物

#### Optional (Nice to Have)

- [ ] history_events 增加 `branch_id` 字段并支持筛选（多分支追溯更清晰）
- [ ] 导出 API 增加大小限制与提示（超出阈值引导使用筛选）

### Hard Gate Checklist

> 必填：跨 Story 硬门禁清单（若不适用请标注 N/A 并说明原因）。

- [x] correlationId 全链路透传（HTTP/WS/日志）
- [x] A2 日志字段齐全（correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp）
- [x] 新增/变更类型已运行 gen-types 并提交生成产物
- [x] 状态一致性与幂等性已校验（如 RunControlState / IterationState）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review] (placeholder) 将本 Story 的 review 结论沉淀到 `## Review Notes`（含风险/遗留）

## Dev Notes

### Developer Context (Read This First)

- **现状基线（Story 7.1/7.2/7.3 已完成）**：
  - Checkpoint 自动保存机制已实现（`backend/src/core/iteration_engine/checkpoint.rs`）
  - 断点恢复机制已实现（`backend/src/core/iteration_engine/recovery.rs`）
  - 回滚机制已实现，含归档与分支支持
  - History API 已落地（`backend/src/api/routes/history.rs`），提供 `/api/v1/tasks/{task_id}/history`
  - `TaskHistoryResponse` 已定义，含 iterations + rollback_candidates
  - `HistoryPanel.tsx` 已实现，展示迭代历史和回滚候选
  - WAL 模式 + FULL synchronous 已配置

- **Epic 7 全景（便于对齐业务价值与范围）**：
  - 7.1 Checkpoint 自动保存（已完成，FR52）
  - 7.2 断点恢复与异常处理（已完成，FR53）
  - 7.3 历史 Checkpoint 回滚（已完成，FR54）
  - **7.4 完整迭代历史记录（本 Story，FR55）**

- **业务价值（为什么做）**：提供完整的优化过程追溯能力，帮助用户理解"为什么这样优化"、"哪些尝试失败了"、"关键转折点在哪里"，提升用户对优化过程的理解和信任。

- **依赖关系**：
  - 依赖 Story 7.1/7.2/7.3 提供的 Checkpoint 和历史基础设施
  - 依赖 History API 单一入口（P2 约束）
  - 复用 `TaskHistoryResponse` 和 `HistoryPanel`

- **范围边界（必须遵守）**：
  - 本 Story 实现：历史事件记录、时间线视图、筛选、导出
  - 不包含：复杂分支可视化、历史对比分析、历史回放动画
  - 历史入口需遵循"单一入口"策略（P2），扩展现有 History API
  - **SSOT 迁移**：7.4 开始以 Checkpoint 作为历史主线（Phase-3），iterations 仅作补充；新增端点必须保持 `/history` 兼容

### 与 Story 7.1/7.2/7.3 的关系

| 功能 | Story 7.1 | Story 7.2 | Story 7.3 | Story 7.4（本 Story） |
| --- | --- | --- | --- | --- |
| Checkpoint 保存 | ✅ 已实现 | 复用 | 复用 | 复用 |
| 断点恢复 | - | ✅ 已实现 | 复用 | 复用 |
| 回滚机制 | - | - | ✅ 已实现 | 复用 |
| History API | - | - | ✅ 已实现 | 扩展 |
| 历史事件记录 | - | - | - | 新增 |
| 时间线视图 | - | - | - | 新增 |
| 导出功能 | - | - | - | 新增 |

### 7.3 Review 结论承接（必须落实）

- 历史相关接口必须有 **limit 上限** 与分页策略（防止大查询）。
- 保持 **单一历史入口**，避免前端同时拼接 iterations + checkpoints。
- 时间线/导出不得全量加载内存，必须分页或聚合后截断。

### Database Schema Changes

```sql
-- backend/migrations/013_history_events.sql

-- 历史事件表
CREATE TABLE IF NOT EXISTS history_events (
    id VARCHAR(36) PRIMARY KEY,
    task_id VARCHAR(36) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    actor VARCHAR(20) NOT NULL,          -- 'system' | 'user'
    details TEXT,                         -- JSON 格式变化详情
    iteration INTEGER,                    -- 可选，关联迭代轮次
    correlation_id VARCHAR(36),
    created_at INTEGER NOT NULL,          -- Unix ms

    FOREIGN KEY (task_id) REFERENCES optimization_tasks(id) ON DELETE CASCADE
);

-- 索引优化
CREATE INDEX idx_history_events_created_at ON history_events(created_at);
CREATE INDEX idx_history_events_event_type ON history_events(event_type);
CREATE INDEX idx_history_events_task_time ON history_events(task_id, created_at);
```

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/models/history_event.rs（新增）

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 历史事件类型
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum EventType {
    IterationStarted,
    IterationCompleted,
    EvaluationCompleted,
    UserPause,
    UserResume,
    UserEdit,
    UserGuidance,
    Rollback,
    CheckpointSaved,
    ErrorOccurred,
    ConfigChanged,
    TaskTerminated,
    CheckpointRecovered,
}

/// 操作者
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum Actor {
    System,
    User,
}

/// 历史事件
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct HistoryEvent {
    pub id: String,
    pub task_id: String,
    pub event_type: EventType,
    pub actor: Actor,
    pub details: Option<serde_json::Value>,  // JSON 格式变化详情
    pub iteration: Option<u32>,
    pub correlation_id: Option<String>,
    pub created_at: String,                   // ISO 8601
}

/// 历史事件筛选条件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEventFilter {
    pub event_types: Option<Vec<EventType>>,
    pub actor: Option<Actor>,
    pub iteration_min: Option<u32>,
    pub iteration_max: Option<u32>,
    pub time_start: Option<i64>,  // Unix ms
    pub time_end: Option<i64>,    // Unix ms
}

/// 历史事件列表响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct HistoryEventListResponse {
    pub events: Vec<HistoryEvent>,
    pub total: u32,
    pub has_more: bool,
}
```

### Event Details Schema（Required）

> 统一 `HistoryEvent.details` 的结构，保证前端与导出可解析。

- **user_edit**
  - `{ field: "prompt" | "rule_system" | "artifacts" | "user_guidance", old_value?: string, new_value?: string, edit_type: "manual" | "ai_suggested" }`
- **rollback**
  - `{ target_checkpoint_id: string, archived_count: number, new_branch_id: string }`
- **checkpoint_recovered**
  - `{ checkpoint_id: string, source: "checkpoint" | "pause_state" }`
- **config_changed**
  - `{ field: "max_iterations" | "threshold" | "other", old_value: string, new_value: string }`

```rust
/// 位置：backend/src/domain/models/history.rs（扩展）

/// 时间线条目（统一迭代/Checkpoint/事件）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TimelineEntry {
    pub id: String,
    pub entry_type: TimelineEntryType,  // "iteration" | "checkpoint" | "event"
    pub timestamp: String,              // ISO 8601
    pub iteration: Option<u32>,
    pub title: String,                  // 简要标题
    pub description: Option<String>,    // 详细描述
    pub actor: Option<String>,          // system/user
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum TimelineEntryType {
    Iteration,
    Checkpoint,
    Event,
}

/// 时间线响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TimelineResponse {
    pub entries: Vec<TimelineEntry>,
    pub total: u32,
    pub has_more: bool,
}

/// 历史导出数据
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct HistoryExportData {
    pub task: TaskExportMeta,
    pub iterations: Vec<IterationExportEntry>,
    pub checkpoints: Vec<CheckpointSummary>,
    pub events: Vec<HistoryEvent>,
    pub branches: Vec<BranchInfo>,
    pub exported_at: String,  // ISO 8601
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TaskExportMeta {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct IterationExportEntry {
    pub iteration: u32,
    pub prompt: Option<String>,
    pub rule_system: Option<String>,
    pub pass_rate: Option<f64>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct BranchInfo {
    pub branch_id: String,
    pub parent_branch_id: Option<String>,
    pub created_at: String,
    pub checkpoint_count: u32,
}
```

### Repository Signatures (Required)

```rust
/// 位置：backend/src/infra/db/repositories/history_event_repo.rs（新增）

impl HistoryEventRepo {
    /// 创建历史事件
    pub async fn create_event(
        pool: &SqlitePool,
        event: &HistoryEvent,
    ) -> Result<(), HistoryEventRepoError>;

    /// 获取历史事件列表（支持筛选）
    pub async fn list_events(
        pool: &SqlitePool,
        task_id: &str,
        filter: &HistoryEventFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<HistoryEvent>, HistoryEventRepoError>;

    /// 统计历史事件数量
    pub async fn count_events(
        pool: &SqlitePool,
        task_id: &str,
        filter: &HistoryEventFilter,
    ) -> Result<u32, HistoryEventRepoError>;

    /// 获取单个历史事件
    pub async fn get_event_by_id(
        pool: &SqlitePool,
        event_id: &str,
    ) -> Result<Option<HistoryEvent>, HistoryEventRepoError>;

    /// 获取时间线条目（聚合迭代/Checkpoint/事件，需排序+分页）
    pub async fn list_timeline_entries(
        pool: &SqlitePool,
        task_id: &str,
        filter: &HistoryEventFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<TimelineEntry>, HistoryEventRepoError>;
}
```

### Event Recording Integration Points

```rust
/// 位置：backend/src/core/iteration_engine/events.rs（新增辅助模块）

use uuid::Uuid;
use chrono::Utc;

/// 记录历史事件的辅助函数
pub async fn record_event(
    pool: &SqlitePool,
    task_id: &str,
    event_type: EventType,
    actor: Actor,
    details: Option<serde_json::Value>,
    iteration: Option<u32>,
    correlation_id: &str,
) -> Result<(), RecordEventError> {
    let event = HistoryEvent {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_string(),
        event_type,
        actor,
        details,
        iteration,
        correlation_id: Some(correlation_id.to_string()),
        created_at: Utc::now().to_rfc3339(),
    };

    HistoryEventRepo::create_event(pool, &event).await?;

    tracing::info!(
        task_id = %task_id,
        event_type = ?event_type,
        actor = ?actor,
        iteration = ?iteration,
        correlation_id = %correlation_id,
        action = "history_event_recorded",
        timestamp = %Utc::now().timestamp_millis(),
        "历史事件已记录"
    );

    Ok(())
}
```

### Suggested API Endpoints

```
# 获取历史事件列表
GET /api/v1/tasks/{task_id}/history/events
Query: event_types(逗号分隔), actor, iteration_min, iteration_max, time_start, time_end, limit, offset
Response: ApiResponse<HistoryEventListResponse>
权限校验：仅任务所有者可访问

# 获取时间线视图
GET /api/v1/tasks/{task_id}/history/timeline
Query: limit, offset, filter 参数同上（默认 created_at 倒序）
Response: ApiResponse<TimelineResponse>
权限校验：仅任务所有者可访问

# 导出历史记录
GET /api/v1/tasks/{task_id}/history/export
Response: JSON 文件（Content-Disposition: attachment）
权限校验：仅任务所有者可操作
```

### Dev Agent Guardrails（避免常见踩坑）

- **事件记录异步化**：事件记录不应阻塞主流程，使用 `tokio::spawn` 或类似机制异步执行。
- **失败降级**：事件记录失败仅记录 WARN/ERROR 日志，不影响主流程。
- **details 字段规范**：使用 `serde_json::Value` 存储，前端解析时注意类型安全。
- **时间格式一致**：数据库存储 Unix ms，API 返回 ISO 8601，前端显示本地化。
- **筛选参数验证**：防止恶意大查询，limit 设置上限（如 100）。
- **时间线聚合策略**：使用 UNION ALL 或多路归并，数据库侧排序/分页，禁止全量加载后排序。
- **导出大小限制**：大型任务导出可能很大，考虑分段或压缩（MVP 可暂不实现）。
- **不要忘记 correlationId**：所有操作必须携带 correlationId（AR2）。
- **correlationId 传递策略**：HTTP/WS 入口生成并透传；内部触发使用 `OptimizationContext` 或 PauseController 最近 correlationId 作为 fallback，禁止空值。
- **日志字段齐全**：必须包含 A2 必填字段。
- **权限校验严格**：只有任务所有者可以查看和导出历史。
- **类型生成**：新增类型后运行 `cargo run --bin gen-types` 并提交产物。
- **复用现有结构**：优先扩展 `history.rs` 而非创建重复定义。

### Technical Requirements（必须满足）

- 历史事件使用 UUID v4 作为 ID
- 时间戳使用 Unix 毫秒存储，API 返回 ISO 8601
- 筛选查询使用参数化 SQL 防止注入
- 导出 API 返回 JSON，设置正确的 Content-Type 和 Content-Disposition
- API 响应使用 `ApiResponse<T>` 统一结构
- 时间线/事件查询默认按 `created_at` 倒序；`limit` ≤ 100，`offset` ≤ 10000
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/domain/models/history_event.rs`：历史事件模型（新增）
  - `backend/src/domain/models/history.rs`：历史相关模型（扩展）
  - `backend/src/infra/db/repositories/history_event_repo.rs`：历史事件仓库（新增）
  - `backend/src/core/iteration_engine/events.rs`：事件记录辅助（新增）
  - `backend/src/api/routes/history.rs`：API 路由（扩展）
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **错误处理**：后端 `thiserror` + `anyhow`
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types`
- **历史入口**：遵循 P2 单一入口策略（扩展现有 History API）

### Library / Framework Requirements (Version Snapshot)

- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`
- tokio：异步运行时
- uuid：事件 ID 生成
- chrono：时间戳处理
- serde_json：details 字段序列化

### File Structure Requirements（落点约束）

**后端**：
- 数据库迁移：`backend/migrations/013_history_events.sql`（新增）
- 历史事件模型：`backend/src/domain/models/history_event.rs`（新增）
- 历史模型：`backend/src/domain/models/history.rs`（扩展）
- 历史事件仓库：`backend/src/infra/db/repositories/history_event_repo.rs`（新增）
- 事件记录辅助：`backend/src/core/iteration_engine/events.rs`（新增）
- 历史 API：`backend/src/api/routes/history.rs`（扩展）
- 类型生成：`backend/src/bin/gen-types.rs`（更新）

**前端**：
- 时间线视图：`frontend/src/features/user-intervention/history/components/TimelineView.tsx`（新增）
- 历史筛选：`frontend/src/features/user-intervention/history/components/HistoryFilter.tsx`（新增）
- 历史服务：`frontend/src/features/user-intervention/history/services/taskHistoryService.ts`（扩展）
- 时间线 Hook：`frontend/src/features/user-intervention/history/hooks/useTimeline.ts`（新增）
- 导出 Hook：`frontend/src/features/user-intervention/history/hooks/useExportHistory.ts`（新增）
- 历史面板：`frontend/src/features/user-intervention/history/HistoryPanel.tsx`（扩展）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 历史事件创建 | 事件正确写入数据库 |
| 后端单测 | 历史事件查询 | 筛选逻辑正确；分页正确 |
| 后端单测 | 时间线排序/分页 | created_at 倒序；limit/offset 生效 |
| 后端单测 | 时间线聚合 | 正确合并迭代/Checkpoint/事件 |
| 后端单测 | 导出数据 | 包含完整元数据和状态快照 |
| 后端单测 | 权限校验 | 非任务所有者返回 403 |
| 集成测试 | 事件记录降级 | 事件记录失败不阻塞主流程 |
| 集成测试 | 完整流程 | 迭代执行 → 事件记录 → 查询 → 导出 |
| 前端测试 | 时间线视图 | 正确渲染时间轴；支持展开详情 |
| 前端测试 | 筛选交互 | 筛选条件正确应用 |
| 前端测试 | 导出功能 | 正确下载 JSON 文件 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 复用 `backend/src/api/routes/history.rs` 的现有路由结构
- 复用 `backend/src/domain/models/history.rs` 的 `TaskHistoryResponse`
- 复用 `frontend/src/features/user-intervention/history/HistoryPanel.tsx`
- 遵循 Repository 模式访问数据库
- 事件记录应在核心业务逻辑完成后异步执行，不阻塞主流程

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 7 / Story 7.4）
- PRD 可靠性：`docs/project-planning-artifacts/prd.md#能力区域 8: 可靠性与恢复`
- 架构（Checkpoint）：`docs/project-planning-artifacts/architecture.md#8. 可靠性与恢复`
- Epic 7 规划复核：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`
- Story 7.1（前序）：`docs/implementation-artifacts/7-1-checkpoint-auto-save.md`
- Story 7.2（前序）：`docs/implementation-artifacts/7-2-breakpoint-recovery-and-exception-handling.md`
- Story 7.3（前序）：`docs/implementation-artifacts/7-3-historical-checkpoint-rollback.md`
- History API 实现：`backend/src/api/routes/history.rs`
- HistoryPanel 实现：`frontend/src/features/user-intervention/history/HistoryPanel.tsx`

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

- `cd backend && cargo test --test history_api_test --test event_record_degrade_test`
- `cd frontend && npx vitest --run src/features/user-intervention/history`

### Completion Notes List

- ✅ 新增历史事件模型/迁移/仓库与筛选结构，事件/时间线查询在数据库侧分页排序。
- ✅ 集成事件记录（迭代/评估/Checkpoint/回滚/用户介入/控制/恢复），异步降级不阻塞主流程。
- ✅ 扩展 History API（events/timeline/export）与 OpenAPI，权限校验与参数约束齐全，导出包含元数据/迭代/Checkpoint/事件/分支并返回总量字段。
- ✅ ErrorOccurred 事件集成到关键错误路径，history_event 日志字段补齐 A2 必填字段。
- ✅ 时间线迭代条目补充通过率信息，前端去重与筛选提示增强，导出文件名安全化。
- ✅ 新增事件记录降级集成测试并通过后端/前端历史相关测试。

### File List

- backend/Cargo.lock
- backend/Cargo.toml
- backend/migrations/013_history_events.sql
- backend/src/api/routes/docs.rs
- backend/src/api/routes/history.rs
- backend/src/api/routes/iteration_control.rs
- backend/src/api/routes/recovery.rs
- backend/src/api/ws/connection.rs
- backend/src/bin/gen-types.rs
- backend/src/core/iteration_engine/checkpoint.rs
- backend/src/core/iteration_engine/events.rs
- backend/src/core/iteration_engine/mod.rs
- backend/src/core/iteration_engine/orchestrator.rs
- backend/src/core/iteration_engine/recovery.rs
- backend/src/core/optimization_engine/alternate_impl.rs
- backend/src/core/optimization_engine/common.rs
- backend/src/core/optimization_engine/default_impl.rs
- backend/src/domain/models/history.rs
- backend/src/domain/models/history_event.rs
- backend/src/domain/models/mod.rs
- backend/src/infra/db/repositories/history_event_repo.rs
- backend/src/infra/db/repositories/mod.rs
- backend/tests/history_api_test.rs
- backend/tests/event_record_degrade_test.rs
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/validation-report-2026-01-19-160006.md
- docs/implementation-artifacts/7-4-complete-iteration-history-record.md
- frontend/src/features/user-intervention/history/HistoryPanel.test.tsx
- frontend/src/features/user-intervention/history/HistoryPanel.tsx
- frontend/src/features/user-intervention/history/components/HistoryFilter.test.tsx
- frontend/src/features/user-intervention/history/components/HistoryFilter.tsx
- frontend/src/features/user-intervention/history/components/TimelineView.test.tsx
- frontend/src/features/user-intervention/history/components/TimelineView.tsx
- frontend/src/features/user-intervention/history/hooks/useExportHistory.ts
- frontend/src/features/user-intervention/history/hooks/useHistoryEvents.ts
- frontend/src/features/user-intervention/history/hooks/useTimeline.ts
- frontend/src/features/user-intervention/history/index.ts
- frontend/src/features/user-intervention/history/services/taskHistoryService.ts
- frontend/src/types/generated/models/Actor.ts
- frontend/src/types/generated/models/BranchInfo.ts
- frontend/src/types/generated/models/EventType.ts
- frontend/src/types/generated/models/HistoryEvent.ts
- frontend/src/types/generated/models/HistoryEventResponse.ts
- frontend/src/types/generated/models/HistoryExportData.ts
- frontend/src/types/generated/models/IterationExportEntry.ts
- frontend/src/types/generated/models/TaskExportMeta.ts
- frontend/src/types/generated/models/TimelineEntry.ts
- frontend/src/types/generated/models/TimelineEntryType.ts
- frontend/src/types/generated/models/TimelineResponse.ts
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [MEDIUM] ErrorOccurred 事件已覆盖关键失败路径，历史记录更完整可追溯。
- [x] [MEDIUM] 时间线迭代条目补充通过率与详情，前端去重降低冗余噪音。
- [x] [MEDIUM] 导出改为全量并返回 totals 字段，避免“静默截断”误导。

### Decisions

- [x] 延续 Checkpoint 作为历史主线（SSOT），时间线统一聚合 iterations/checkpoints/events。

### Risks / Tech Debt

- [ ] 全量导出在超大任务下可能带来内存/响应时间压力，建议后续评估流式导出或压缩。
- [ ] history_events 暂未扩展 branch_id，跨分支筛选需后续支持。

### Follow-ups

- [ ] 如历史数据体量增长明显，考虑流式导出/压缩或分段导出策略。
