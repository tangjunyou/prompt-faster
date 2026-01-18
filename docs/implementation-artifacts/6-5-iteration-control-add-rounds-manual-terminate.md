# Story 6.5: 迭代控制（增加轮数/手动终止）

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 6-5-iteration-control-add-rounds-manual-terminate

## Epic 6 开工门槛（必须先满足）

> ⚠️ **重要**：本 Story 开工前需确认以下门槛已完成或已有明确计划。
> 跟踪文件：`docs/implementation-artifacts/epic-5-retro-action-items-2026-01-16.md`

- [x] **A1 明确"暂停/编辑/继续"的状态一致性与权限边界**
  - Owner：Winston（Architect）+ Charlie（Senior Dev）
  - 成功标准：形成一份状态与权限边界说明（包含状态机、允许的操作与触发条件），并在实现前完成评审确认。
  - 证据：`docs/implementation-artifacts/epic-6-run-control-state-design.md`

- [x] **A2 用户介入的可追踪证据链**
  - Owner：Dana（QA Engineer）+ Amelia（Dev Agent）
  - 成功标准：定义并落地"日志/状态回放入口"的最小证据链要求，确保用户介入操作可追踪与可回放。
  - 证据：`docs/implementation-artifacts/epic-6-traceability-verification.md`

- [ ] **A3 构建体积告警拆分评估**（可与本 Story 并行推进）
  - Owner：Winston（Architect）
  - 成功标准：形成可执行拆分方案清单，并在后续 Story 中落实为具体任务。
  - 证据：`docs/implementation-artifacts/epic-5-build-size-warning-2026-01-16.md`

## Key Decisions (MVP)

- **API 选型**：使用 HTTP API（而非 WS 命令）处理增加轮数和终止操作，因为这些是用户主动触发的非实时操作。
- **候选 Prompt 来源**：从 `iterations.artifacts` 解析 `IterationArtifacts.candidate_prompts`（`CandidatePrompt.content`），优先 `is_best`，否则取第一条；后端生成 `prompt_preview`（前 200 字符）。
- **终止状态**：运行控制层使用 `RunControlState::Stopped`（A1 既定），持久化层使用 `optimization_tasks.status = terminated` + `terminated_at` 区分 `completed`。
- **最终 Prompt 存储**：用户选定的 Prompt 保存到 `optimization_tasks.final_prompt` 字段。
- **增加轮数逻辑**：更新 `max_iterations` 后需同步运行中引擎的内存配置；当前轮次继续执行。
- **状态校验**：增加轮数和终止操作仅在 `Running` 或 `Paused` 状态下可用；Paused 下增加轮数不自动恢复。
- **二次确认**：终止操作不可逆；Dialog 内仅一个 Primary，文案需说明作用范围与不可撤销。
- **AR2 遵循**：所有 API 请求必须携带 `correlationId`，确保可追踪。
- **审计日志**：增加轮数和终止操作需记录日志（满足 A2 可追踪性要求）。
- **多端同步（可选增强）**：终止成功可推送 `task:terminated` WS 事件。

## Story

As a Prompt 优化用户,
I want 随时增加迭代轮数或手动终止并选择满意的 Prompt,
so that 我可以灵活控制优化进程。

## Acceptance Criteria

1. **Given** 优化任务正在运行或已暂停
   **When** 用户想增加迭代轮数
   **Then** 显示"增加轮数"输入框
   **And** 用户可以输入要增加的轮数（正整数，1-100）

2. **Given** 用户输入增加轮数
   **When** 点击"确认"
   **Then** 系统将最大迭代轮数增加指定数量
   **And** 继续执行直到达到新的上限或成功
   **And** 显示更新后的最大轮数
   **And** 若当前状态为 Paused，保持 Paused，需用户手动点击"继续"后才恢复执行

3. **Given** 优化任务正在运行或已暂停
   **When** 用户点击"终止"按钮
   **Then** 显示候选 Prompt 列表（按通过率降序排列）
   **And** 每个候选项显示：轮次编号、通过率、Prompt 预览
   **And** 用户可以选择满意的 Prompt

4. **Given** 用户选择 Prompt
   **When** 点击"确认终止"
   **Then** 终止迭代并保存用户选择的 Prompt 作为最终结果
   **And** 标记任务为 `terminated`（用户终止）状态
   **And** 界面更新为终止状态视图

5. **Given** 优化任务处于 Idle 或 Stopped 状态
   **When** 用户尝试增加轮数或终止
   **Then** 相关控件禁用或隐藏
   **And** 显示适当的状态提示

6. **Given** 无可用候选 Prompt（未执行任何成功迭代）
   **When** 用户尝试终止
   **Then** 显示提示"暂无可用的候选 Prompt"
   **And** 提供"直接终止（不保存结果）"选项

7. **Given** 增加轮数或终止操作
   **When** 操作执行
   **Then** 操作记录在日志中（含 correlationId、用户 ID、任务 ID、action、prev_state/new_state、iteration_state、timestamp）
   **And** 对于数值类变更（如增加轮数）额外记录 prev_value/new_value
   **And** 支持后续审计与状态回放（A2 要求）

8. **Given** 网络请求失败
   **When** 执行增加轮数或终止操作
   **Then** 显示清晰的错误信息
   **And** 提供重试选项
   **And** 错误信息遵循 NFR24 统一错误文案规范

## Tasks / Subtasks

- [x] 后端：数据结构定义（AC: 1-4）
  - [x] 在 `backend/src/domain/types/iteration_control.rs` 定义 `AddRoundsRequest` / `AddRoundsResponse` DTO
  - [x] 定义 `TerminateTaskRequest` / `TerminateTaskResponse` DTO
  - [x] 定义 `CandidatePromptSummary` DTO（候选 Prompt 列表项）
  - [x] 扩展 `OptimizationTaskStatus`（新增 `terminated` / `completed`）并同步序列化/反序列化映射
  - [x] 扩展 `OptimizationTaskEntity` / 相关 Response，包含 `final_prompt` / `terminated_at` / `selected_iteration_id`
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增 DTO

- [x] 后端：数据库扩展（AC: 4）
  - [x] 创建迁移 `backend/migrations/009_add_final_prompt.sql`
  - [x] 为 `optimization_tasks` 表添加 `final_prompt` 字段（TEXT, nullable）
  - [x] 为 `optimization_tasks` 表添加 `terminated_at` 字段（INTEGER, nullable）
  - [x] 为 `optimization_tasks` 表添加 `selected_iteration_id` 字段（TEXT, nullable）
  - [x] 运行迁移并验证

- [x] 后端：增加轮数 API（AC: 1,2,7）
  - [x] 在 `backend/src/api/routes/optimization_tasks.rs` 添加 `PATCH /api/v1/tasks/{task_id}/config` 端点
  - [x] 明确仅允许更新 `max_iterations`（与 workspace config 全量更新端点区分）
  - [x] 实现 max_iterations 更新逻辑（DB + config_json）
  - [x] 同步运行中引擎的内存配置（共享状态/registry/上下文更新）
  - [x] 实现状态校验（仅 Running/Paused 状态可操作）
  - [x] 实现权限校验（仅任务所有者可操作）
  - [x] 记录操作日志（tracing，包含 correlation_id/user_id/task_id/action/prev_value/new_value/timestamp）
  - [x] 更新 CORS 允许 `PATCH`（浏览器端调用必需）

- [x] 后端：获取候选 Prompt 列表 API（AC: 3,6）
  - [x] 在 `backend/src/api/routes/optimization_tasks.rs` 添加 `GET /api/v1/tasks/{task_id}/candidates` 端点
  - [x] 从 `iterations` 表查询各轮候选 Prompt（复用 iteration_repo）
  - [x] 解析 `iterations.artifacts` 为 `IterationArtifacts`，取 `candidate_prompts`（优先 `is_best`，否则取第一条）
  - [x] 后端生成 `prompt_preview`（前 200 字符）
  - [x] 按通过率降序排列
  - [x] 添加防御性上限（例如最多返回最近 100 条）
  - [x] 返回 `CandidatePromptSummary` 列表

- [x] 后端：终止任务 API（AC: 3,4,6,7）
  - [x] 在 `backend/src/api/routes/optimization_tasks.rs` 添加 `POST /api/v1/tasks/{task_id}/terminate` 端点
  - [x] 实现终止迭代引擎逻辑（复用 RunControlState 状态机 + 引擎 stop 信号/控制通道）
  - [x] 保存选定 Prompt 到 `final_prompt` 字段
  - [x] 保存 `selected_iteration_id`（无候选时为 null）
  - [x] 更新任务状态为 `terminated`
  - [x] 记录 `terminated_at` 时间戳
  - [x] 支持无候选时的"直接终止"选项
  - [x] 实现状态校验和权限校验
  - [x] 推送 `task:terminated` WS 事件（多端同步）
  - [x] 记录操作日志
  - [x] 在路由注册中挂载 `/api/v1/tasks/{task_id}/*` 的新端点并加鉴权中间件

- [x] 前端：IterationControlPanel 组件（AC: 1,3,5）
  - [x] 在 `frontend/src/features/user-intervention/control/` 创建 `IterationControlPanel.tsx`
  - [x] 显示当前轮次 / 最大轮次
  - [x] 显示"增加轮数"按钮
  - [x] 显示"终止"按钮
  - [x] 根据 RunControlState 控制按钮启用/禁用状态
  - [x] 按钮点击区域 ≥ 44px × 44px（UX 无障碍规范）

- [x] 前端：AddRoundsDialog 组件（AC: 1,2）
  - [x] 在 `frontend/src/features/user-intervention/control/` 创建 `AddRoundsDialog.tsx`
  - [x] 数字输入框（1-100 范围校验）
  - [x] 显示当前最大轮数
  - [x] 显示更新后的最大轮数预览
  - [x] 确认/取消按钮

- [x] 前端：TerminateDialog 组件（AC: 3,4,6）
  - [x] 在 `frontend/src/features/user-intervention/control/` 创建 `TerminateDialog.tsx`
  - [x] 显示候选 Prompt 列表
  - [x] 支持单选某个候选 Prompt
  - [x] 显示二次确认警告（终止操作不可逆）
  - [x] Dialog 内仅一个 Primary，文案明确“仅终止当前任务、不可撤销”
  - [x] 空候选时显示"直接终止"选项
  - [x] 确认终止/取消按钮

- [x] 前端：CandidatePromptList 组件（AC: 3）
  - [x] 在 `frontend/src/features/user-intervention/control/` 创建 `CandidatePromptList.tsx`
  - [x] 显示轮次编号、通过率、Prompt 预览（使用后端 `prompt_preview`）
  - [x] 支持点击展开完整 Prompt（使用后端返回的完整 `prompt`）
  - [x] 支持单选

- [x] 前端：服务层封装（AC: 1-4,8）
  - [x] 在 `frontend/src/features/user-intervention/control/services/` 创建 `iterationControlService.ts`
  - [x] 实现 `addRounds(taskId, additionalRounds)` 函数
  - [x] 实现 `getCandidates(taskId)` 函数
  - [x] 实现 `terminateTask(taskId, selectedPromptId)` 函数
  - [x] 在 `frontend/src/features/user-intervention/control/hooks/` 创建 `useIterationControl.ts`
  - [x] 使用 TanStack Query 的 useMutation 处理状态变更
  - [x] QueryKey 约定：`['task-config', taskId]`、`['candidates', taskId]`
  - [x] 成功后 `invalidateQueries(['task-config', taskId])`，必要时刷新任务详情/状态视图
  - [x] 处理 `task:terminated` WS 事件（可选：多端同步）

- [x] 前端：RunView 集成（AC: 1,3,5）
  - [x] 在 `frontend/src/pages/RunView/RunView.tsx` 集成 `IterationControlPanel`
  - [x] 放置在暂停/继续控件附近（与 PauseResumeControl 并列或嵌入）
  - [x] 不影响现有功能（暂停/继续、编辑产物、引导、历史查看）

- [x] 测试与回归（AC: 1-8）
  - [x] 后端单测：增加轮数 API 返回正确数据
  - [x] 后端单测：终止 API 正确保存选定 Prompt
  - [x] 后端单测：权限校验（非任务所有者返回 403）
  - [x] 后端单测：状态校验（非 Running/Paused 状态返回 400）
  - [x] 前端组件测试：IterationControlPanel 渲染与按钮状态
  - [x] 前端组件测试：AddRoundsDialog 输入校验
  - [x] 前端组件测试：TerminateDialog 选择与确认流程
  - [x] 集成测试：完整增加轮数流程
  - [x] 集成测试：完整终止并选择 Prompt 流程
  - [x] 回归命令：`cd backend && cargo test`；`cd frontend && npx vitest --run && npm run build`
  - [x] 生成类型：`cd backend && cargo run --bin gen-types` 并提交产物

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review][HIGH] 增加轮数仅更新 DB 配置，缺少运行中引擎内存配置同步（导致新上限不生效）。[backend/src/api/routes/iteration_control.rs#L140]
- [x] [AI-Review][HIGH] 终止操作仅更新 DB，缺少引擎停止信号与 RunControlState 更新（可能继续执行并写入）。[backend/src/api/routes/iteration_control.rs#L462]
- [x] [AI-Review][MEDIUM] 优化任务模型/响应未包含 final_prompt、terminated_at、selected_iteration_id（终止结果无法回显）。[backend/src/domain/models/optimization_task.rs#L40] [backend/src/api/routes/optimization_tasks.rs#L39]
- [x] [AI-Review][MEDIUM] RunView 传入的 max_iterations 为硬编码 10，UI 无法展示真实更新后的最大轮数。[frontend/src/pages/RunView/RunView.tsx#L366]
- [x] [AI-Review][MEDIUM] 增加轮数/终止对话框仅展示错误文本，缺少重试入口与 NFR24 统一错误文案处理。[frontend/src/features/user-intervention/control/AddRoundsDialog.tsx#L138] [frontend/src/features/user-intervention/control/TerminateDialog.tsx#L109]
- [x] [AI-Review][MEDIUM] 终止对话框文案未明确“仅终止当前任务、不可撤销”，二次确认提示不足（UX 规范未满足）。[frontend/src/features/user-intervention/control/TerminateDialog.tsx#L76]
- [x] [AI-Review][MEDIUM] Dev Agent Record 的 File List 为空，与实际 git 变更不一致（需补充变更清单）。[docs/implementation-artifacts/6-5-iteration-control-add-rounds-manual-terminate.md#L623]
- [x] [AI-Review][MEDIUM] 缺少后端/前端 API & 交互测试，仅有 DTO 校验测试，无法覆盖关键流程。[backend/src/domain/types/iteration_control.rs#L131]

## Dev Notes

### Developer Context (Read This First)

- **现状基线（已完成）**：
  - Story 6.1 已完成暂停/继续功能，提供关键基础设施：
    - `RunControlState` 状态机（Idle/Running/Paused/Stopped）- 复用 `Stopped` 表示用户终止
    - `PauseResumeControl` 组件（`frontend/src/features/user-intervention/`）
    - 最小暂停持久化（`backend/src/core/iteration_engine/pause_state.rs`）
    - WS 命令处理框架（`task:pause`/`task:resume`）
    - correlationId 全链路追踪
  - Story 6.2 已完成中间产物编辑功能，提供：
    - `IterationArtifacts` 类型定义（包含候选 Prompt）
    - `ArtifactEditor` 组件
  - Story 6.3 已完成对话引导功能
  - Story 6.4 已完成历史迭代查看功能，提供：
    - `iterations` 表（迁移 008）- 包含各轮候选 Prompt 数据
    - `iteration_repo.rs` - 可复用查询方法
    - 历史迭代列表 API

- **Epic 6 全景（便于对齐业务价值与范围）**：
  - 6.1 暂停与继续迭代（已完成，FR40/FR44）
  - 6.2 编辑中间产物（已完成，FR41）
  - 6.3 对话引导老师模型（已完成，FR42）
  - 6.4 历史迭代产物查看（已完成，FR43）
  - **6.5 迭代控制：增加轮数/手动终止（本 Story，FR45/FR46）**

- **业务价值（为什么做）**：赋予用户对优化进程的完全控制权——当用户发现当前迭代轮数不够时可以动态增加，当用户对某轮结果满意时可以提前终止并锁定结果。这是 Epic 6 用户介入能力的最后拼图，实现用户的"掌控感"（来源：PRD 能力区域 6 / FR45/FR46 / UX 情感设计）。

- **依赖关系**：
  - 依赖 Story 6.1 提供的 `RunControlState` 状态机
  - 依赖 Story 6.4 提供的 `iterations` 表和 `iteration_repo`
  - 依赖 `optimization_tasks` 表的 `max_iterations` 配置字段

- **范围边界（必须遵守）**：
  - 增加轮数是动态修改配置，不会影响当前正在执行的迭代步骤
  - 终止是不可逆操作，必须有二次确认
  - 终止后任务进入终态，不可再继续或回滚（回滚由 Epic 7 承接）
  - 不支持"减少轮数"（避免复杂边界条件）
  - 不支持多选 Prompt 合并（只能选一个）

### 状态层级说明

| 状态层 | 字段/枚举 | 值 | 说明 |
| --- | --- | --- | --- |
| 运行控制 | `RunControlState` | `Stopped` | 用户请求终止（A1 既定状态机） |
| 任务持久化 | `optimization_tasks.status` | `terminated` / `completed` | 用户终止 / 自动完成 |
| 终止时间 | `optimization_tasks.terminated_at` | Unix ms | 终止时间戳 |

### 增加轮数与引擎同步

- 更新 DB 中 `max_iterations` 的同时，**必须同步运行中引擎的内存配置**（共享状态/registry/上下文更新）。
- 若任务处于 `Paused`，增加轮数后**保持 Paused**，不会自动恢复。

### 终止操作与引擎协调机制

**终止流程顺序（建议）**：
1. 触发引擎停止信号（新增 stop flag 或扩展 PauseController，避免仅更新 DB）。
2. 等待当前 Layer 结束后停止执行。
3. 将 `RunControlState` 置为 `Stopped`。
4. 写入 `final_prompt` / `selected_iteration_id` / `terminated_at` / `status=terminated`。
5. 可选：推送 `task:terminated` WS 事件。

### WS 事件契约（新增）

```
type: "task:terminated"
payload: {
  taskId: string,
  terminatedAt: string,
  finalPrompt?: string,
  selectedIterationId?: string
}
correlationId: string
```

### 候选 Prompt 数据来源

- `iterations.artifacts` 存储 `IterationArtifacts` JSON。
- `IterationArtifacts.candidate_prompts` 为 `Vec<CandidatePrompt>`，候选内容为 `CandidatePrompt.content`。
- 优先选择 `is_best = true` 的候选；否则取第一条。
- `prompt_preview` 由后端截断生成（前 200 字符）。

### API 路由关系

- `PATCH /api/v1/tasks/{task_id}/config`：仅允许更新 `max_iterations`（局部更新）。
- `PUT /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}/config`：全量更新配置（现有路由）。

### 权限校验模式

- 在 handler 中使用 `CurrentUser` 获取 `user_id`。
- 查询任务后校验 `task.user_id == user_id`，失败返回 403。

### 前端状态同步

- `addRounds` / `terminate` 成功后 `invalidateQueries(['task-config', taskId])`，必要时刷新任务详情。
- 可选：订阅 `task:terminated` WS 事件以支持多端同步。

### Database Schema Changes

```sql
-- 位置：backend/migrations/009_add_final_prompt.sql

-- 添加最终选定 Prompt 字段
ALTER TABLE optimization_tasks ADD COLUMN final_prompt TEXT;

-- 添加终止时间戳
ALTER TABLE optimization_tasks ADD COLUMN terminated_at INTEGER;

-- 添加终止原因/选定迭代 ID
ALTER TABLE optimization_tasks ADD COLUMN selected_iteration_id TEXT;
```

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/types/iteration_control.rs

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 增加轮数请求
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct AddRoundsRequest {
    /// 要增加的轮数（1-100）
    pub additional_rounds: i32,
}

/// 增加轮数响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct AddRoundsResponse {
    /// 更新前的最大轮数
    pub previous_max_iterations: i32,
    /// 更新后的最大轮数
    pub new_max_iterations: i32,
    /// 当前已执行轮数
    pub current_round: i32,
}

/// 候选 Prompt 摘要
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CandidatePromptSummary {
    /// 迭代 ID
    pub iteration_id: String,
    /// 轮次编号
    pub round: i32,
    /// 通过率（0.0 - 1.0）
    pub pass_rate: f64,
    /// 通过的测试用例数
    pub passed_cases: i32,
    /// 测试用例总数
    pub total_cases: i32,
    /// 候选 Prompt 内容
    pub prompt: String,
    /// Prompt 预览（前 200 字符，后端截断生成）
    pub prompt_preview: String,
    /// 迭代完成时间（ISO 8601）
    pub completed_at: String,
}

/// 终止任务请求
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TerminateTaskRequest {
    /// 选定的迭代 ID（可选，无候选时可为空表示直接终止）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_iteration_id: Option<String>,
}

/// 终止任务响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TerminateTaskResponse {
    /// 任务 ID
    pub task_id: String,
    /// 终止时间（ISO 8601）
    pub terminated_at: String,
    /// 选定的最终 Prompt（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_prompt: Option<String>,
    /// 选定的迭代轮次（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_round: Option<i32>,
}
```

### Suggested API Endpoints

```
# 增加迭代轮数
PATCH /api/v1/tasks/{task_id}/config
Body: AddRoundsRequest
Response: ApiResponse<AddRoundsResponse>
说明：仅更新 max_iterations（与 workspace config 全量更新区分）
状态校验：仅 Running/Paused 状态可操作

# 获取候选 Prompt 列表
GET /api/v1/tasks/{task_id}/candidates
Response: ApiResponse<Vec<CandidatePromptSummary>>
说明：按通过率降序排列，仅返回已完成的迭代（可加 limit 上限）

# 终止任务
POST /api/v1/tasks/{task_id}/terminate
Body: TerminateTaskRequest
Response: ApiResponse<TerminateTaskResponse>
状态校验：仅 Running/Paused 状态可操作
```

### Suggested Component Structure

```tsx
// frontend/src/features/user-intervention/control/IterationControlPanel.tsx

interface IterationControlPanelProps {
  taskId: string
  currentRound: number
  maxIterations: number
  runState: RunControlState // Idle/Running/Paused/Stopped
}

// 显示：当前轮次 / 最大轮数
// 按钮：增加轮数、终止
// 状态控制：仅 Running/Paused 时启用按钮
```

```tsx
// frontend/src/features/user-intervention/control/AddRoundsDialog.tsx

interface AddRoundsDialogProps {
  isOpen: boolean
  onClose: () => void
  taskId: string
  currentMaxIterations: number
  onSuccess: (newMax: number) => void
}

// 输入框：增加轮数（1-100）
// 预览：更新后的最大轮数
// 按钮：确认、取消
```

```tsx
// frontend/src/features/user-intervention/control/TerminateDialog.tsx

interface TerminateDialogProps {
  isOpen: boolean
  onClose: () => void
  taskId: string
  onSuccess: () => void
}

// 列表：候选 Prompt（CandidatePromptList）
// 空状态：无候选时显示"直接终止"选项
// 警告：终止操作不可逆
// 按钮：确认终止、取消
```

```tsx
// frontend/src/features/user-intervention/control/CandidatePromptList.tsx

interface CandidatePromptListProps {
  candidates: CandidatePromptSummary[]
  selectedId: string | null
  onSelect: (id: string) => void
}

// 列表项：轮次、通过率、Prompt 预览
// 单选：支持选中某个候选
// 展开：点击展开完整 Prompt
```

### Dev Agent Guardrails（避免常见踩坑）

- **状态校验必须严格**：增加轮数和终止仅在 Running/Paused 状态下可用，其他状态返回 400 错误。
- **终止操作不可逆**：必须有二次确认对话框，明确告知用户操作不可撤销。
- **终止必须停止引擎**：HTTP 终止不可只改 DB，需触发引擎 stop 信号并停止执行。
- **增加轮数需同步引擎**：更新 DB 后必须同步运行中引擎内存配置。
- **候选列表按通过率排序**：帮助用户快速找到最佳结果。
- **候选数据来源明确**：从 `iterations.artifacts` 解析 `candidate_prompts`，优先 `is_best`。
- **不要忘记 correlationId**：所有 API 请求必须携带 correlationId（AR2）。
- **权限校验必须严格**：只有任务所有者可以操作，防止跨用户误操作。
- **使用 TanStack Query**：数据获取和变更必须通过 TanStack Query，利用缓存和状态管理。
- **复用现有状态机**：不要重新实现状态管理，复用 `RunControlState`。
- **输入校验**：增加轮数的输入必须是 1-100 的正整数。
- **空候选处理**：无可用候选时提供"直接终止"选项，不阻塞用户。

### Technical Requirements（必须满足）

- 所有按钮必须满足 WCAG 2.1 AA 无障碍标准（点击区域 ≥ 44px × 44px）。
- API 响应必须使用 `ApiResponse<T>` 统一结构。
- 所有操作必须记录 tracing 日志，满足 A2 必填字段：`correlation_id`、`user_id`、`task_id`、`action`、`prev_state`、`new_state`、`iteration_state`、`timestamp`。
- 对于数值类变更（如增加轮数），额外记录 `prev_value` / `new_value` 作为补充字段。
- 前端错误提示不得直接展示 `error.details`（遵循架构错误处理规范）。
- 终止确认对话框必须有明确的警告文案。
- 终止对话框仅允许一个 Primary 按钮，并明确“仅终止当前任务、不可撤销”。

### Architecture Compliance（必须遵守）

- **API 端点命名**：遵循 RESTful 风格
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **状态管理**：TanStack Query 管理服务端状态，Zustand 管理本地状态
- **错误处理**：后端 `thiserror` + `anyhow`，前端统一错误响应结构
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types` 并提交生成产物
- **路由挂载**：新增 `/api/v1/tasks/{task_id}/*` 端点需在主路由中挂载并加鉴权中间件

### Library / Framework Requirements (Version Snapshot)

- React：项目依赖 `react@^19.2.0`
- React Router：项目依赖 `react-router@^7.12.0`
- TanStack Query：项目依赖 `@tanstack/react-query`
- Zustand：项目依赖 `zustand@^5.x`
- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`
- shadcn/ui：Dialog、Button、Input 组件

### File Structure Requirements（落点约束）

**后端**：
- 迭代控制 DTO：`backend/src/domain/types/iteration_control.rs`（新增）
- 优化任务 API 扩展：`backend/src/api/routes/optimization_tasks.rs`（扩展）
- 数据库迁移：`backend/migrations/009_add_final_prompt.sql`（新增）

**前端**：
- 控制面板组件：`frontend/src/features/user-intervention/control/IterationControlPanel.tsx`（新增）
- 增加轮数对话框：`frontend/src/features/user-intervention/control/AddRoundsDialog.tsx`（新增）
- 终止对话框：`frontend/src/features/user-intervention/control/TerminateDialog.tsx`（新增）
- 候选列表组件：`frontend/src/features/user-intervention/control/CandidatePromptList.tsx`（新增）
- 服务层：`frontend/src/features/user-intervention/control/services/iterationControlService.ts`（新增）
- Hooks：`frontend/src/features/user-intervention/control/hooks/useIterationControl.ts`（新增）
- 导出索引：`frontend/src/features/user-intervention/control/index.ts`（新增）
- RunView 集成：`frontend/src/pages/RunView/RunView.tsx`（扩展）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 增加轮数 API | 正确更新 max_iterations；输入校验（1-100）；状态校验（非 Running/Paused 返回 400） |
| 后端单测 | 终止 API | 正确保存选定 Prompt；更新任务状态为 terminated；无候选时直接终止 |
| 后端单测 | 权限校验 | 非任务所有者返回 403 |
| 后端单测 | 候选列表 API | 按通过率排序；仅返回已完成迭代 |
| 前端组件测试 | IterationControlPanel | 按钮渲染、状态控制 |
| 前端组件测试 | AddRoundsDialog | 输入校验、预览更新 |
| 前端组件测试 | TerminateDialog | 候选选择、二次确认 |
| 前端组件测试 | CandidatePromptList | 列表渲染、单选、展开 |
| 集成测试 | 端到端增加轮数 | 运行中任务 → 增加轮数 → 验证 max_iterations 更新 |
| 集成测试 | Paused 增加轮数后继续 | Paused → 增加轮数 → Resume → 使用新上限 |
| 集成测试 | 端到端终止 | 运行中任务 → 终止 → 选择 Prompt → 验证任务状态与 final_prompt |
| 集成测试 | 终止停止引擎 | Running → 终止 → 引擎停止 + 状态正确 |
| 集成测试 | 空候选直接终止 | 无候选 → 直接终止 → 不保存 final_prompt |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 复用 `frontend/src/features/user-intervention/` 目录结构，新增 `control/` 子目录。
- 复用 `RunControlState` 状态机进行状态校验。
- 复用 `iteration_repo` 查询候选 Prompt 数据。
- 与 `PauseResumeControl` 组件并列放置在 RunView 中。

### RunView Integration Details

- **面板布局**：在暂停/继续控件区域添加"增加轮数"和"终止"按钮。
- **状态同步**：复用 `useTaskStore` 中的 `runState` 控制按钮启用状态。
- **对话框管理**：使用 shadcn/ui Dialog 组件，本地状态管理对话框开关。
- **操作反馈**：成功后 invalidate `['task-config', taskId]` 并更新任务状态；失败时显示错误 Toast。
- **多端同步（可选）**：监听 `task:terminated` WS 事件刷新视图。

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 6 / Story 6.5）
- PRD 用户介入：`docs/project-planning-artifacts/prd.md#能力区域 6: 用户介入`
- UX 规范：`docs/project-planning-artifacts/ux-design-specification.md`
- 架构（API patterns）：`docs/project-planning-artifacts/architecture.md#API & Communication Patterns`
- Epic 6 状态设计：`docs/implementation-artifacts/epic-6-run-control-state-design.md`
- Epic 6 开工门槛：`docs/implementation-artifacts/epic-5-retro-action-items-2026-01-16.md`
- Epic 6 可追踪性规范（A2）：`docs/implementation-artifacts/epic-6-traceability-verification.md`
- 前序 Story learnings：`docs/implementation-artifacts/6-1-pause-and-resume-iteration.md`
- 前序 Story learnings：`docs/implementation-artifacts/6-4-historical-iteration-artifacts-view.md`

## Git Intelligence Summary

- Story 6.1 相关提交：
  - `RunControlState` 状态机实现
  - `PauseResumeControl` 组件
  - WS 命令处理框架
- Story 6.4 相关提交：
  - `iterations` 表迁移（008）
  - `iteration_repo.rs` 查询方法
  - 历史迭代 API

## Latest Tech Information (Web/Registry Snapshot)

- 版本以本地依赖快照为准：`frontend/package.json` 与 `backend/Cargo.toml`
- 关键关注点：本 Story 不涉及依赖升级，按现有版本实现即可

## Project Context Reference

- 以 `docs/project-planning-artifacts/*.md`、`docs/developer-guides/*` 与现有代码为准

## Story Completion Status

- Status set to `done`
- Completion note: 后端/前端实现与测试已完成，回归与迁移验证通过（见 Completion Notes）。

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

- 回归执行：`cargo test` ✅、`npx vitest --run` ✅、`npm run build` ✅
- 迁移验证：安装 `sqlx-cli` 后以 `DATABASE_URL=sqlite:data/prompt_faster.db?mode=rwc` 执行 `sqlx migrate run` ✅
### File List

- backend/migrations/009_add_final_prompt.sql
- backend/src/domain/types/iteration_control.rs
- backend/src/domain/types/mod.rs
- backend/src/domain/models/optimization_task.rs
- backend/src/infra/db/repositories/optimization_task_repo.rs
- backend/src/infra/db/repositories/iteration_repo.rs
- backend/src/api/routes/iteration_control.rs
- backend/src/api/routes/mod.rs
- backend/src/api/routes/optimization_tasks.rs
- backend/src/core/iteration_engine/pause_state.rs
- backend/src/core/optimization_engine/common.rs
- backend/src/core/optimization_engine/default_impl.rs
- backend/src/core/optimization_engine/alternate_impl.rs
- backend/src/shared/ws.rs
- backend/src/api/ws/events.rs
- backend/src/bin/gen-types.rs
- backend/src/main.rs
- frontend/src/features/user-intervention/control/IterationControlPanel.tsx
- frontend/src/features/user-intervention/control/AddRoundsDialog.tsx
- frontend/src/features/user-intervention/control/TerminateDialog.tsx
- frontend/src/features/user-intervention/control/CandidatePromptList.tsx
- frontend/src/features/user-intervention/control/services/iterationControlService.ts
- frontend/src/features/user-intervention/control/hooks/useIterationControl.ts
- frontend/src/features/user-intervention/control/index.ts
- frontend/src/features/user-intervention/control/IterationControlFlow.test.tsx
- frontend/src/features/user-intervention/control/IterationControlPanel.test.tsx
- frontend/src/features/user-intervention/control/AddRoundsDialog.test.tsx
- frontend/src/features/user-intervention/control/TerminateDialog.test.tsx
- frontend/src/features/user-intervention/control/CandidatePromptList.test.tsx
- frontend/src/hooks/useWebSocket.ts
- frontend/src/pages/RunView/RunView.tsx
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx
- frontend/src/stores/useTaskStore.ts
- frontend/src/components/ui/dialog.tsx
- frontend/package.json
- frontend/package-lock.json
- frontend/src/types/generated/api/OptimizationTaskResponse.ts
- frontend/src/types/generated/models/OptimizationTaskEntity.ts
- frontend/src/types/generated/models/OptimizationTaskStatus.ts
- frontend/src/types/generated/models/AddRoundsRequest.ts
- frontend/src/types/generated/models/AddRoundsResponse.ts
- frontend/src/types/generated/models/CandidatePromptListResponse.ts
- frontend/src/types/generated/models/CandidatePromptSummary.ts
- frontend/src/types/generated/models/TerminateTaskRequest.ts
- frontend/src/types/generated/models/TerminateTaskResponse.ts
- frontend/src/types/generated/ws/TaskTerminatedPayload.ts
- frontend/src/types/generated/ws/index.ts
- docs/implementation-artifacts/6-5-iteration-control-add-rounds-manual-terminate.md
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/validation-report-20260117-225144.md

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] 增加轮数仅更新 DB 配置，缺少运行中引擎内存配置同步，AC2 “继续执行直到达到新上限”不可保证。[backend/src/api/routes/iteration_control.rs#L140]
- [x] [HIGH] 终止操作仅更新 DB，缺少引擎停止信号与 RunControlState 更新，任务可能继续执行并写入新迭代。[backend/src/api/routes/iteration_control.rs#L462]
- [x] [MEDIUM] 优化任务模型/响应未包含 final_prompt、terminated_at、selected_iteration_id，终止结果无法回显到 UI。[backend/src/domain/models/optimization_task.rs#L40] [backend/src/api/routes/optimization_tasks.rs#L39]
- [x] [MEDIUM] RunView 传入 max_iterations 为硬编码 10，UI 无法展示更新后的最大轮数。[frontend/src/pages/RunView/RunView.tsx#L366]
- [x] [MEDIUM] 错误处理仅显示 error.message，缺少“重试”入口与 NFR24 统一文案处理。[frontend/src/features/user-intervention/control/AddRoundsDialog.tsx#L138] [frontend/src/features/user-intervention/control/TerminateDialog.tsx#L109]
- [x] [MEDIUM] 终止对话框文案未明确“仅终止当前任务、不可撤销”，二次确认提示不足。[frontend/src/features/user-intervention/control/TerminateDialog.tsx#L76]
- [x] [MEDIUM] Dev Agent Record File List 为空，与实际变更不一致（可追踪性不足）。[docs/implementation-artifacts/6-5-iteration-control-add-rounds-manual-terminate.md#L623]
- [x] [MEDIUM] 缺少 API/交互测试，仅有 DTO 校验测试，关键流程未覆盖。[backend/src/domain/types/iteration_control.rs#L131]

### Decisions

- [x] Review 结论已落地并完成修复，相关问题已闭环。
- [x] 2026-01-18：补齐迭代控制状态提示、候选列表 API 403/404 区分与 N+1 优化（仅 completed）、控制面板刷新一致性、候选列表无障碍提示。

### Risks / Tech Debt

- [ ] Epic 7 Checkpoint 机制未实现前，终止后的状态无法回滚
- [ ] 增加轮数功能仅支持"增加"，不支持"减少"（避免复杂边界条件）
- [ ] 候选 Prompt 列表可能较长，MVP 不支持分页（后续可扩展）
- [ ] 终止操作不可逆，需在 UX 层面充分提示用户

### Follow-ups

- [x] 已同步到 `### Review Follow-ups (AI)`（见上方清单）
