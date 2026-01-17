# Story 6.2: 编辑中间产物

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 6-2-edit-intermediate-artifacts

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

- **中间产物类型权威**：新增 `backend/src/domain/types/artifacts.rs`，定义 `IterationArtifacts`（含 patterns + candidate_prompts）；ts-rs 导出供前端消费。
- **编辑权限边界**：复用 Story 6.1 的权限校验逻辑——仅 `RunControlState::Paused` 状态下、任务所有者可编辑。
- **编辑粒度**：支持逐条编辑（规律假设列表、候选 Prompt 列表），每条可单独修改/删除；不支持新增（新增由系统生成）。
- **编辑模式 UI**：使用 Monaco Editor（与现有技术栈一致，复用 Epic 5 的 StreamingText 经验）。
- **持久化策略**：扩展 `pause_state.rs` 的 `context_snapshot` 存储编辑后产物；Epic 7 完成后替换为完整 Checkpoint 机制。
- **WS 事件驱动**：编辑通过 WebSocket 命令发送（`artifact:get`/`artifact:update`），状态变更通过 WS 事件推送（`artifact:updated`）。
- **AR2 遵循**：所有操作必须携带 `correlationId`，确保多任务场景下消息不串台。
- **审计日志**：编辑操作必须记录完整日志（含 correlationId/user_id/task_id/artifact_type/edit_action）。

## Story

As a Prompt 优化用户,  
I want 直接编辑当前迭代的中间产物（规律假设、Prompt）,  
so that 我可以基于自己的理解修正或引导优化方向。

## Acceptance Criteria

1. **Given** 迭代已暂停  
   **When** 用户查看中间产物  
   **Then** 显示当前的规律假设和候选 Prompt  
   **And** 提供"编辑"按钮  
   **And** 编辑按钮点击区域 ≥ 44px × 44px（UX 无障碍规范）

2. **Given** 用户点击"编辑"按钮  
   **When** 进入编辑模式  
   **Then** 规律假设和 Prompt 变为可编辑状态  
   **And** 提供"保存"和"取消"按钮  
   **And** 显示编辑提示"修改后的内容将用于后续迭代"

3. **Given** 用户完成编辑  
   **When** 点击"保存"  
   **Then** 保存用户修改  
   **And** 后续迭代使用用户修改后的产物  
   **And** 显示"保存成功"提示

4. **Given** 用户编辑中  
   **When** 点击"取消"  
   **Then** 放弃所有修改  
   **And** 恢复为编辑前的内容  
   **And** 退出编辑模式

5. **Given** 任务未暂停（Running/Idle/Stopped）  
   **When** 用户尝试编辑  
   **Then** 编辑按钮禁用  
   **And** 显示提示"请先暂停任务再编辑"

6. **Given** 用户编辑并保存产物  
   **When** 操作完成  
   **Then** 操作记录在日志中（含 correlationId、用户 ID、时间戳、产物类型）  
   **And** 支持后续审计与状态回放

7. **Given** 系统意外重启或断线  
   **When** 用户重新连接  
   **Then** 可以恢复到编辑后的状态（扩展最小暂停持久化实现）

## Tasks / Subtasks

- [x] 定义中间产物类型（AC: 1,3）
  - [x] 在 `backend/src/domain/types/artifacts.rs` 中定义 `IterationArtifacts` 结构
  - [x] 定义 `PatternHypothesis`（规律假设）和 `CandidatePrompt`（候选 Prompt）类型
  - [x] 实现 ts-rs 导出（`#[ts(export_to = "models/")]`）
  - [x] 在 `backend/src/domain/types/mod.rs` 中导出新模块
  - [x] 明确 `PatternHypothesis` 与 `RuleSystem` 的映射与回写策略（避免重复语义）

- [x] 扩展 pause_state 持久化（AC: 3,7）
  - [x] 在 `backend/src/core/iteration_engine/pause_state.rs` 中扩展 `PauseStateSnapshot`（使用 `context_snapshot` 存储 `artifacts`，无需新增字段）
  - [x] 实现 `update_artifacts` 方法
  - [x] 恢复时加载用户编辑版本，并映射回 `OptimizationContext`（`rule_system`/`current_prompt`/`extensions`）

- [x] 实现后端 WS 命令处理（AC: 1,3,6）
  - [x] 在 `backend/src/api/ws/connection.rs` 中添加 `artifact:get` 命令处理
  - [x] 添加 `artifact:update` 命令处理与 ACK
  - [x] 添加 `artifact:updated` 事件推送
  - [x] 实现权限校验（仅 Paused 状态 + 任务所有者）
  - [x] 仅允许修改/删除已有 id，禁止新增产物（后端强制校验）
  - [x] 确保 correlationId 全链路透传（AR2）
  - [x] 记录操作日志（tracing，包含 correlationId/user_id/task_id/artifact_type/edit_action/prev_state/new_state/iteration_state/timestamp）

- [x] 实现前端 ArtifactEditor 组件（AC: 1,2,3,4,5）
  - [x] 安装依赖 `@monaco-editor/react`（如未安装）
  - [x] 在 `frontend/src/features/user-intervention/` 中创建 `ArtifactEditor.tsx` 组件
  - [x] 实现规律假设列表展示与编辑
  - [x] 实现候选 Prompt 列表展示与编辑
  - [x] 使用 Monaco Editor 实现编辑区域
  - [x] 实现编辑/保存/取消按钮（点击区域 ≥ 44px × 44px）
  - [x] 实现编辑模式状态切换

- [x] 扩展 useTaskStore（AC: 1,2,3）
  - [x] 在 `frontend/src/stores/useTaskStore.ts` 中添加 `artifacts` 状态
  - [x] 添加 `editingArtifact` 编辑状态
  - [x] 实现 `setArtifacts`/`updateArtifact` actions

- [x] 集成到 RunView 页面（AC: 1,5）
  - [x] 在 `frontend/src/pages/RunView/RunView.tsx` 中集成 ArtifactEditor
  - [x] 根据 RunControlState 控制组件显示/禁用状态
  - [x] 监听 WS 事件更新产物状态

- [x] 测试与回归（AC: 1-7）
  - [x] 后端单测：权限校验、状态校验、WS 命令处理
  - [x] 前端组件测试：ArtifactEditor 渲染、编辑模式切换、Monaco 集成
  - [x] 集成测试：暂停 → 编辑 → 保存 → 继续完整流程
  - [x] 回归命令：`cd backend && cargo test`；`cd frontend && npx vitest --run && npm run build`

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review] Review Notes 已更新（含风险/遗留）
- [x] [AI-Review][MEDIUM] 补齐“暂停 → 编辑 → 保存 → 继续”的端到端集成测试（含编辑后恢复）并覆盖失败场景
- [x] [AI-Review][MEDIUM] 运行全量回归：`cd backend && cargo test`；`cd frontend && npx vitest --run && npm run build`
- [x] [AI-Review][LOW] 基于 A3 评估 Monaco Editor 构建体积影响，并记录结论（main chunk 642.94 kB）

## Dev Notes

### Developer Context (Read This First)

- **现状基线（已完成）**：
  - Story 6.1 已完成暂停/继续功能，提供关键基础设施：
    - `RunControlState` 状态机（Idle/Running/Paused/Stopped）
    - `PauseResumeControl` 组件（`frontend/src/features/user-intervention/`）
    - 最小暂停持久化（`backend/src/core/iteration_engine/pause_state.rs`）
    - WS 命令处理框架（`task:pause`/`task:resume`）
    - correlationId 全链路追踪
  - Epic 4 已完成四层架构，产物类型在 `backend/src/domain/models/algorithm.rs`

- **Epic 6 全景（便于对齐业务价值与范围）**：
  - 6.1 暂停与继续迭代（已完成，FR40/FR44）
  - **6.2 编辑中间产物（本 Story，FR41）**
  - 6.3 对话引导老师模型（FR42）
  - 6.4 历史迭代产物查看（FR43）
  - 6.5 迭代控制：增加轮数/手动终止（FR45/FR46）

- **业务价值（为什么做）**：让用户在优化过程中可以直接修正系统产出的规律假设和候选 Prompt，将自己的领域知识注入优化过程，实现"人机协作优化"。这是 Epic 6 的核心能力之一（来源：PRD 能力区域 6 / FR41）。

- **依赖关系**：
  - 依赖 Story 6.1 提供的暂停状态和权限校验
  - 依赖 Epic 4 的四层架构产出中间产物
  - 本 Story 仅扩展最小暂停持久化，Epic 7 完成后替换为完整 Checkpoint 机制

- **范围边界（必须遵守）**：
  - 不实现"新增"产物（新增由系统生成）
  - 不实现产物历史版本对比（6.4 承接）
  - 不实现对话引导（6.3 承接）
  - 编辑仅限 Paused 状态

### Artifact Source & Mapping（必须先对齐）

- **规律假设映射**：`PatternHypothesis` 是面向编辑视图的轻量结构；必须明确其与 `RuleSystem.rules[].description` 的映射/回写方式，避免与 `Rule` 语义冲突。
- **候选 Prompt 来源**：明确候选列表来源（当前迭代生成范围/最佳候选/当前 Prompt），并说明写回到 `OptimizationContext.current_prompt`/`extensions` 的路径。
- **应用路径**：用户编辑后的 `IterationArtifacts` 必须在恢复/继续前映射回 `OptimizationContext`（保证 AC3 “后续迭代使用修改产物”）。
- **日志安全**：编辑日志不得回显 prompt 原文（仅记录 id/type/action）。

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/types/artifacts.rs

/// 规律假设
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct PatternHypothesis {
    /// 唯一标识
    pub id: String,
    /// 规律描述
    pub pattern: String,
    /// 来源（system/user_edited）
    pub source: ArtifactSource,
    /// 置信度（0-1）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

/// 候选 Prompt
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct CandidatePrompt {
    /// 唯一标识
    pub id: String,
    /// Prompt 内容
    pub content: String,
    /// 来源（system/user_edited）
    pub source: ArtifactSource,
    /// 评估分数（如有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

/// 产物来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum ArtifactSource {
    /// 系统生成
    System,
    /// 用户编辑
    UserEdited,
}

/// 迭代中间产物集合
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct IterationArtifacts {
    /// 规律假设列表
    pub patterns: Vec<PatternHypothesis>,
    /// 候选 Prompt 列表
    pub candidate_prompts: Vec<CandidatePrompt>,
    /// 最后更新时间戳
    /// ISO 8601 格式，如 "2026-01-17T12:00:00Z"
    pub updated_at: String,
}
```

### Suggested WS Commands

```typescript
// 客户端发送
{ type: 'artifact:get', payload: { taskId: string }, correlationId: string }
{ type: 'artifact:update', payload: { taskId: string, artifacts: IterationArtifacts }, correlationId: string }

// 服务端推送
{ type: 'artifact:get:ack', payload: { taskId: string, ok: boolean, artifacts: IterationArtifacts, reason?: string }, correlationId: string }
{ type: 'artifact:update:ack', payload: { taskId: string, ok: boolean, applied: boolean, artifacts?: IterationArtifacts, reason?: string }, correlationId: string }
{ type: 'artifact:updated', payload: { taskId: string, artifacts: IterationArtifacts, editedBy: string }, correlationId: string }
```

### Suggested Component Structure

```tsx
// frontend/src/features/user-intervention/ArtifactEditor.tsx

interface ArtifactEditorProps {
  taskId: string
  artifacts: IterationArtifacts
  onSave: (artifacts: IterationArtifacts, correlationId: string) => void
  disabled?: boolean  // 非 Paused 状态时禁用
}

// 内部状态
// - editMode: boolean
// - editingPatterns: PatternHypothesis[]
// - editingPrompts: CandidatePrompt[]
// - selectedIndex: number (当前编辑项)
```

### Dev Agent Guardrails（避免常见踩坑）

- **不要在非 Paused 状态允许编辑**：复用 Story 6.1 的 `canPause`/`canResume` 逻辑模式。
- **不要忘记 correlationId**：所有 WS 命令和事件必须携带 correlationId（AR2）。
- **不要直接修改 store 状态**：编辑操作需通过 WS 命令同步到后端，后端确认后再更新前端状态。
- **不要忽略错误处理**：编辑操作失败时需返回用户友好的错误信息（NFR24）。
- **不要破坏现有暂停逻辑**：编辑是增量功能，不得影响 Story 6.1 已有的暂停/继续能力。
- **不要忽略可追踪性**：所有编辑操作必须记录日志，支持审计（A2 门槛要求）。
- **标记产物来源**：用户编辑后需将 `source` 设为 `UserEdited`，便于后续分析。
- **禁止新增产物**：后端必须校验仅修改/删除已有 id，新增 id 直接拒绝。
- **状态二次校验**：`artifact:update` 执行时必须确认任务仍为 `Paused` 且 owner 匹配。
- **日志安全**：禁止在日志/错误/WS payload 中回显 prompt 原文。

### Technical Requirements（必须满足）

- 编辑/保存/取消按钮必须满足 WCAG 2.1 AA 无障碍标准（点击区域 ≥ 44px × 44px）。
- 编辑操作必须在后端确认成功后才更新前端状态（以 `artifact:update:ack.ok=true` 为准；失败需回滚或保持编辑态）。
- WS 事件必须携带完整的产物信息，前端无需额外请求。
- 编辑后产物必须持久化到最小暂停持久化记录，支持重启恢复；Epic 7 完成后替换为完整 Checkpoint。
- 所有操作必须记录 tracing 日志（含 correlationId、user_id、task_id、artifact_type、edit_action、prev_state、new_state、iteration_state、timestamp）。
- 后端必须校验仅修改/删除已有 id，禁止新增产物。
- `artifact:update` 处理时必须确认任务仍为 `Paused` 且 owner 匹配（服务端强制）。
- 编辑内容需做长度校验（Prompt 参考 `OPTIMIZATION_TASK_CONFIG_MAX_INITIAL_PROMPT_BYTES`），失败给出可读错误提示。
- 日志/错误/WS payload 不得回显 prompt 原文（遵循 contracts 禁止回显敏感内容）。
- Monaco Editor 需配置为 Markdown/PlainText 模式，支持语法高亮和基本编辑功能。

### Architecture Compliance（必须遵守）

- **WS 事件命名**：遵循 `{domain}:{action}` 格式（`artifact:get`, `artifact:update`）
- **WS envelope 形状**：遵循 `WsMessage<T>` 结构，`correlationId` 全链路透传
  - `docs/project-planning-artifacts/architecture.md#Communication Patterns`
  - `docs/developer-guides/contracts.md`
- **状态管理**：Zustand（全局）+ TanStack Query（服务端状态）
- **错误处理**：后端 `thiserror` + `anyhow`，前端统一错误响应结构
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types` 并提交生成产物

### Library / Framework Requirements (Version Snapshot)

- React：项目依赖 `react@^19.2.0`
- Monaco Editor：`@monaco-editor/react@^4.x`（如未安装需添加依赖）
- React Flow：项目锁定 `@xyflow/react@12.10.0`
- Zustand：项目依赖 `zustand@^5.x`
- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`

### File Structure Requirements（落点约束）

**后端**：
- 中间产物类型定义：`backend/src/domain/types/artifacts.rs`（新增）
- 类型模块导出：`backend/src/domain/types/mod.rs`（扩展）
- WS 命令处理：`backend/src/api/ws/connection.rs`（扩展）
- 暂停持久化扩展：`backend/src/core/iteration_engine/pause_state.rs`（扩展）
- WS 事件定义：`backend/src/api/ws/events.rs`（扩展）

**前端**：
- 产物编辑组件：`frontend/src/features/user-intervention/ArtifactEditor.tsx`（新增）
- 组件导出：`frontend/src/features/user-intervention/index.ts`（扩展）
- 状态 store：`frontend/src/stores/useTaskStore.ts`（扩展）
- RunView 集成：`frontend/src/pages/RunView/RunView.tsx`（扩展）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 权限校验、状态校验、WS 命令处理 | 非 Paused 状态编辑被拒绝；非任务所有者编辑被拒绝；新增 id 被拒绝；过长内容被拒绝 |
| 前端组件测试 | ArtifactEditor 渲染与状态 | 编辑模式切换、保存/取消按钮、禁用态 |
| 集成测试 | 端到端编辑流程 | 暂停 → 编辑 → 保存 → 继续流程；编辑后恢复 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 复用 `frontend/src/features/user-intervention/` 目录结构（与 PauseResumeControl 并列）。
- 复用 `frontend/src/stores/useTaskStore.ts` 的状态管理模式。
- Monaco Editor 配置参考 Epic 5 的 StreamingText 组件经验。

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 6 / Story 6.2）
- PRD 用户介入：`docs/project-planning-artifacts/prd.md#能力区域 6: 用户介入`
- UX 规范：`docs/project-planning-artifacts/ux-design-specification.md`
- 架构（WS events/correlationId）：`docs/project-planning-artifacts/architecture.md#Communication Patterns`
- WS 契约单一权威：`docs/developer-guides/contracts.md`
- Epic 6 状态设计：`docs/implementation-artifacts/epic-6-run-control-state-design.md`
- Epic 6 开工门槛：`docs/implementation-artifacts/epic-5-retro-action-items-2026-01-16.md`
- 前序 Story learnings：`docs/implementation-artifacts/6-1-pause-and-resume-iteration.md`

## Git Intelligence Summary

- Story 6.1 相关提交：
  - 暂停/继续 WS 命令处理框架
  - PauseResumeControl 组件
  - pause_state.rs 最小持久化
- 现有 WS 事件处理已包含 `task:pause`/`task:resume`/`iteration:paused`/`iteration:resumed`，需扩展以支持 `artifact:*` 系列命令。
- 现有 `useTaskStore` 可直接扩展以存储 `artifacts` 状态。

## Latest Tech Information (Web/Registry Snapshot)

- Monaco Editor `@monaco-editor/react@4.x`：稳定版本，支持 React 19
- React `19.2.x`：稳定版本，无需升级
- Axum `0.8.x`：稳定版本，无需升级
- 关键关注点：如未安装 Monaco Editor，需添加依赖 `npm install @monaco-editor/react`

## Project Context Reference

- 以 `docs/project-planning-artifacts/*.md`、`docs/developer-guides/*` 与现有代码为准

## Story Completion Status

- Status set to `in-progress`
- Completion note: 代码已实现并修复关键缺陷；仍需补齐端到端集成测试与全量回归

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

### Completion Notes List

### File List

- `backend/src/api/ws/connection.rs`
- `backend/src/api/ws/events.rs`
- `backend/src/bin/gen-types.rs`
- `backend/src/core/iteration_engine/pause_state.rs`
- `backend/src/core/optimization_engine/common.rs`
- `backend/src/core/optimization_engine/mod.rs`
- `backend/src/domain/types/artifacts.rs`
- `backend/src/domain/types/mod.rs`
- `backend/src/shared/ws.rs`
- `backend/tests/ws_pause_resume_integration_test.rs`
- `docs/implementation-artifacts/6-2-edit-intermediate-artifacts.md`
- `docs/implementation-artifacts/code-review-report.md`
- `docs/implementation-artifacts/sprint-status.yaml`
- `frontend/package.json`
- `frontend/package-lock.json`
- `frontend/src/components/ui/tabs.tsx`
- `frontend/src/features/user-intervention/ArtifactEditor.tsx`
- `frontend/src/features/user-intervention/ArtifactEditor.test.tsx`
- `frontend/src/features/user-intervention/index.ts`
- `frontend/src/hooks/useWebSocket.ts`
- `frontend/src/pages/RunView/RunView.tsx`
- `frontend/src/stores/useTaskStore.ts`
- `frontend/src/types/generated/models/ArtifactSource.ts`
- `frontend/src/types/generated/models/CandidatePrompt.ts`
- `frontend/src/types/generated/models/IterationArtifacts.ts`
- `frontend/src/types/generated/models/PatternHypothesis.ts`
- `frontend/src/types/generated/ws/ArtifactGetAckPayload.ts`
- `frontend/src/types/generated/ws/ArtifactGetPayload.ts`
- `frontend/src/types/generated/ws/ArtifactUpdateAckPayload.ts`
- `frontend/src/types/generated/ws/ArtifactUpdatePayload.ts`
- `frontend/src/types/generated/ws/ArtifactUpdatedPayload.ts`
- `frontend/src/types/generated/ws/index.ts`

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] 暂停快照未包含 `artifacts`，导致 `artifact:get` 返回空且编辑功能不可用（已修复）
- [x] [HIGH] 缺失“编辑产物 → OptimizationContext”映射，AC3 无法保证后续迭代使用编辑结果（已修复）
- [x] [HIGH] 非 Paused 状态未展示禁用编辑入口与提示，AC5 不满足（已修复）
- [x] [MEDIUM] 编辑内容长度校验缺失（已修复）
- [x] [MEDIUM] 测试缺口：新增 WS artifact 集成测试 + ArtifactEditor 组件测试（已补）
- [x] [MEDIUM] 端到端“暂停 → 编辑 → 保存 → 继续/恢复”集成测试已补齐

### Decisions

- [x] 暂停时生成 `IterationArtifacts` 写入快照，恢复时映射回 `OptimizationContext`，确保 AC3 成立
- [x] resume 时先清理落盘快照，避免新连接误判为暂停；待应用编辑后再清理内存快照
- [x] 保存成功提示在前端展示，失败保留编辑态并提示错误信息

### Risks / Tech Debt

- [ ] Epic 7 完整 Checkpoint 未实现前，恢复能力仍是"最小暂停快照"级别。
- [x] Monaco Editor 已改为动态导入，并通过 manualChunks 拆分，构建不再触发 chunk warning（最大包 277.52 kB / gzip 86.54 kB）。

### Follow-ups

- [x] 补齐端到端集成测试覆盖“暂停 → 编辑 → 保存 → 继续/恢复”
- [x] 执行全量回归（backend + frontend + build）
