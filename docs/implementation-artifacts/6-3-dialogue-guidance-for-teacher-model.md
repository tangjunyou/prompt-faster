# Story 6.3: 对话引导老师模型

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 6-3-dialogue-guidance-for-teacher-model

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

- **对话引导模式**：用户引导信息作为"附加上下文"注入下一轮迭代的老师模型调用，不修改核心 Prompt 模板。
- **引导存储**：引导消息存储在 `IterationArtifacts.user_guidance` 字段（扩展 Story 6.2 的产物结构）。
- **引导时机**：仅在 `RunControlState::Paused` 状态下允许发送引导，继续迭代时生效。
- **引导生效范围**：引导信息仅影响下一轮迭代（单轮生效），不持久影响后续轮次。
- **多次发送规则**：Paused 状态下重复发送引导采用“Last One Wins”，覆盖当前 Pending 引导，不保留历史。
- **WS 事件驱动**：引导通过 WebSocket 命令发送（`guidance:send`），状态变更通过 WS 事件推送（`guidance:sent`/`guidance:applied`）。
- **AR2 遵循**：所有操作必须携带 `correlationId`，确保多任务场景下消息不串台。
- **审计日志**：引导操作必须记录完整日志（满足 A2 必填字段 + guidance_preview）。
- **日志安全**：日志仅记录引导内容的前 50 字符预览，不记录完整原文（遵循 contracts 禁止回显敏感内容）。
- **持久化复用**：`user_guidance` 随 `IterationArtifacts` 一起存入 pause_state 的 `context_snapshot`（复用 Story 6.2 产物存储路径），不新增并行字段。

## Story

As a Prompt 优化用户,
I want 通过对话引导老师模型,
so that 我可以告诉模型我的想法，让它朝特定方向优化。

## Acceptance Criteria

1. **Given** 迭代已暂停
   **When** 用户想要引导模型
   **Then** 显示对话输入框
   **And** 输入框提示"告诉老师模型你的想法..."
   **And** 输入框点击区域 ≥ 44px 高度（UX 无障碍规范）

2. **Given** 用户在对话输入框中输入引导信息
   **When** 点击"发送"按钮
   **Then** 系统将用户引导信息保存到当前任务状态
   **And** 显示"引导已保存，将在下一轮迭代生效"提示
   **And** 发送按钮点击区域 ≥ 44px × 44px（UX 无障碍规范）

3. **Given** 用户已发送引导信息
   **When** 用户点击"继续"恢复迭代
   **Then** 下一轮迭代时老师模型参考用户引导
   **And** 引导信息作为附加上下文传递给 Layer 1-4
   **And** 引导生效后标记为"已应用"
   **And** 任务进入 Running 时 GuidanceInput 保持可见但禁用，并显示 Pending/Applied 状态直至本轮结束或下一次暂停

4. **Given** 用户输入无效（空内容或超长）
   **When** 点击"发送"
   **Then** 显示清晰的错误信息
   **And** 错误信息说明问题原因与解决方法
   **And** 引导内容限制在 2000 字符以内
   **And** 错误信息遵循 NFR24 统一错误文案规范

5. **Given** 任务未暂停（Running/Idle/Stopped）
   **When** 用户尝试发送引导
   **Then** 发送按钮禁用
   **And** 显示提示"请先暂停任务再发送引导"

6. **Given** 当前已有 Pending 引导
   **When** 用户再次发送引导
   **Then** 新引导覆盖旧引导（Last One Wins）
   **And** 旧引导不再生效，前端展示最新引导内容与状态

7. **Given** 用户发送引导信息
   **When** 操作完成
   **Then** 操作记录在日志中（含 correlationId、用户 ID、时间戳、prev_state/new_state/iteration_state、引导预览）
   **And** 支持后续审计与状态回放

8. **Given** 系统意外重启或断线
   **When** 用户重新连接
   **Then** 可以恢复到已保存的引导状态（扩展最小暂停持久化实现）

## Tasks / Subtasks

- [x] 扩展 IterationArtifacts 支持引导消息（AC: 1,2,3）
  - [x] 在 `backend/src/domain/types/artifacts.rs` 中添加 `UserGuidance` 结构
  - [x] 扩展现有 `IterationArtifacts` 添加 `user_guidance: Option<UserGuidance>` 字段（不重复定义原有字段）
  - [x] 实现 ts-rs 导出（`#[ts(export_to = "models/")]`）
  - [x] 定义引导状态枚举：`Pending`/`Applied`
  - [x] 在 `backend/src/domain/types/extensions.rs` 中新增 `EXT_USER_GUIDANCE` 常量并补齐回归测试（契约要求）

- [x] 扩展 pause_state 持久化（AC: 3,7）
  - [x] 复用 `context_snapshot` 中的 `artifacts` 存储路径持久化 `user_guidance`（不新增并行字段）
  - [x] 实现 `update_guidance` 方法（复用/参考 `update_artifacts` 的校验与落盘逻辑）
  - [x] 恢复时加载已保存的引导信息

- [x] 实现后端 WS 命令处理（AC: 1,2,3,6）
  - [x] 在 `backend/src/api/ws/connection.rs` 中添加 `guidance:send` 命令处理
  - [x] 添加 `guidance:send:ack` 响应
  - [x] 添加 `guidance:sent` 事件推送
  - [x] 添加 `guidance:applied` 事件推送（引导应用时触发，Layer 1 开始前）
  - [x] 在 `backend/src/api/ws/events.rs` 中定义 guidance 相关事件与 payload 结构
  - [x] 实现权限校验（仅 Paused 状态 + 任务所有者）
  - [x] 实现输入校验（非空、长度 ≤ 2000）
  - [x] 确保 correlationId 全链路透传（AR2）
  - [x] 记录操作日志（tracing，包含 correlationId/user_id/task_id/guidance_preview）

- [x] 扩展迭代引擎以消费引导信息（AC: 3）
  - [x] 在 `backend/src/core/optimization_engine/common.rs` 中读取 `user_guidance`
  - [x] 在恢复迭代时将引导信息注入 `OptimizationContext.extensions`
  - [x] 在 Layer 1-4 的老师模型调用中读取并使用引导信息
  - [x] 在 Layer 1 开始前应用引导并推送 `guidance:applied` 事件，标记状态为 `Applied`
  - [x] 本轮迭代结束后清理 `extensions` 中的引导信息，确保单轮生效

- [x] 实现前端 GuidanceInput 组件（AC: 1,2,4,5）
  - [x] 在 `frontend/src/features/user-intervention/` 中创建 `GuidanceInput.tsx` 组件
  - [x] 实现对话输入框（placeholder: "告诉老师模型你的想法..."）
  - [x] 实现发送按钮（点击区域 ≥ 44px × 44px）
  - [x] 实现输入校验与错误提示
  - [x] 实现禁用状态（非 Paused 时）
  - [x] 显示已发送的引导信息与状态（Pending/Applied）
  - [x] Running 状态保持组件可见但禁用，并持续展示当前引导状态

- [x] 扩展 useTaskStore（AC: 1,2,3）
  - [x] 在 `frontend/src/stores/useTaskStore.ts` 中添加 `userGuidance` 状态
  - [x] 实现 `setGuidance`/`clearGuidance` actions
  - [x] 监听 `guidance:sent`/`guidance:applied` 事件更新状态

- [x] 集成到 RunView 页面（AC: 1,5）
  - [x] 在 `frontend/src/pages/RunView/RunView.tsx` 中集成 GuidanceInput
  - [x] 根据 RunControlState 控制组件显示/禁用状态
  - [x] 监听 WS 事件更新引导状态

- [x] 测试与回归（AC: 1-8）
  - [x] 后端单测：权限校验、状态校验、WS 命令处理、输入校验
  - [x] 后端单测：日志字段符合 A2（包含 prev_state/new_state/iteration_state/timestamp）
  - [x] 后端单测：引导在 Layer 调用前被消费（可通过 mock TeacherModel 验证 prompt 包含引导）
  - [x] 后端单测：引导在本轮结束后清理，后续轮次不再生效
  - [x] 前端组件测试：GuidanceInput 渲染、编辑状态、禁用态
  - [x] 前端组件测试：Running 状态下保持可见但禁用，并显示 Pending/Applied
  - [x] 集成测试：暂停 → 发送引导 → 继续 → 引导生效完整流程
  - [x] 集成测试：Paused 状态多次发送仅保留最后一次（Last One Wins）
  - [x] 集成测试：引导应用后清理不影响下一轮
  - [x] 回归命令：`cd backend && cargo test`；`cd frontend && npx vitest --run && npm run build`
  - [x] 生成类型：`cd backend && cargo run --bin gen-types` 并提交产物

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review][CRITICAL] 在 `backend/src/domain/types/extensions.rs` 添加 `EXT_USER_GUIDANCE` 并补齐回归测试（契约要求）
- [x] [AI-Review][CRITICAL] 补齐 A2 日志字段（prev_state/new_state/iteration_state/timestamp）与相关测试
- [x] [AI-Review][HIGH] 明确引导生命周期：Last One Wins、Layer 1 前应用并发 `guidance:applied`、轮次结束清理
- [x] [AI-Review][HIGH] 明确 guidance 事件在 `backend/src/api/ws/events.rs` 的类型定义落点
- [x] [AI-Review][MEDIUM] 增加“引导被消费/清理”的可验证测试（mock TeacherModel 或断言 prompt 包含引导）

## Dev Notes

### Developer Context (Read This First)

- **现状基线（已完成）**：
  - Story 6.1 已完成暂停/继续功能，提供关键基础设施：
    - `RunControlState` 状态机（Idle/Running/Paused/Stopped）
    - `PauseResumeControl` 组件（`frontend/src/features/user-intervention/`）
    - 最小暂停持久化（`backend/src/core/iteration_engine/pause_state.rs`）
    - WS 命令处理框架（`task:pause`/`task:resume`）
    - correlationId 全链路追踪
  - Story 6.2 已完成中间产物编辑功能，提供：
    - `IterationArtifacts` 类型定义（`backend/src/domain/types/artifacts.rs`）
    - `ArtifactEditor` 组件（`frontend/src/features/user-intervention/`）
    - `artifact:get`/`artifact:update` WS 命令框架
    - Monaco Editor 集成
  - Epic 4 已完成四层架构，老师模型调用在 `backend/src/core/teacher_model/`

- **Epic 6 全景（便于对齐业务价值与范围）**：
  - 6.1 暂停与继续迭代（已完成，FR40/FR44）
  - 6.2 编辑中间产物（已完成，FR41）
  - **6.3 对话引导老师模型（本 Story，FR42）**
  - 6.4 历史迭代产物查看（FR43）
  - 6.5 迭代控制：增加轮数/手动终止（FR45/FR46）

- **业务价值（为什么做）**：让用户在优化过程中可以主动"告诉"老师模型自己的想法和方向偏好，实现"人机对话式协作优化"。这是 Epic 6 的核心差异化能力之一（来源：PRD 能力区域 6 / FR42）。区别于 6.2（直接编辑产物），本功能强调"引导"而非"替换"，用户提供方向性建议，模型自行决定如何调整。

- **依赖关系**：
  - 依赖 Story 6.1 提供的暂停状态和权限校验
  - 依赖 Story 6.2 提供的 `IterationArtifacts` 类型框架
  - 依赖 Epic 4 的四层架构提供老师模型调用能力
  - 本 Story 仅扩展最小暂停持久化，Epic 7 完成后替换为完整 Checkpoint 机制

- **范围边界（必须遵守）**：
  - 不实现多轮对话（本 MVP 仅支持单条引导，下一轮生效）
  - 不实现引导历史记录查看（6.4 承接历史产物查看）
  - 不实现引导模板/预设（超出 MVP 范围）
  - 引导仅在 Paused 状态下允许发送
  - 引导持久化复用 `IterationArtifacts`（与 6.2 一致），不引入并行结构

### Guidance Injection Strategy（引导注入策略）

- **注入点**：引导信息注入 `OptimizationContext.extensions[EXT_USER_GUIDANCE]`
- **消费方**：Layer 1-4 在调用老师模型时读取引导信息，拼接到老师模型 Prompt 的"用户补充说明"部分
- **生效时机**：继续迭代（resume）后的第一个 Layer 调用开始生效
- **清理时机**：引导应用后（标记为 Applied），在该轮迭代结束后从 extensions 中移除
- **示例老师模型 Prompt 片段**：
  ```
  ## 用户补充说明
  用户提供了以下引导信息，请在本轮优化中参考：
  "{user_guidance_content}"
  ```

**实现落点（默认实现文件，供快速定位）：**
- Layer 1：`backend/src/core/rule_engine/default_impl.rs`
- Layer 2：`backend/src/core/prompt_generator/default_impl.rs`
- Layer 3：`backend/src/core/evaluator/default_impl.rs`（teacher model 评估入口）
- Layer 4：`backend/src/core/optimizer/default_impl.rs`（或反思聚合相关实现）

**契约提醒**：
- `user_guidance` 的 extensions key 必须在 `backend/src/domain/types/extensions.rs` 中登记并测试（避免魔法字符串扩散）。

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/types/artifacts.rs

/// 用户引导消息
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct UserGuidance {
    /// 唯一标识
    pub id: String,
    /// 引导内容
    pub content: String,
    /// 引导状态
    pub status: GuidanceStatus,
    /// 创建时间（ISO 8601）
    pub created_at: String,
    /// 应用时间（ISO 8601，仅 Applied 状态有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_at: Option<String>,
}

/// 引导状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum GuidanceStatus {
    /// 等待应用
    Pending,
    /// 已应用
    Applied,
}

/// 扩展 IterationArtifacts
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct IterationArtifacts {
    /// 规律假设列表
    pub patterns: Vec<PatternHypothesis>,
    /// 候选 Prompt 列表
    pub candidate_prompts: Vec<CandidatePrompt>,
    /// 用户引导（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_guidance: Option<UserGuidance>,
    /// 最后更新时间戳
    pub updated_at: String,
}
```

### Suggested WS Commands

```typescript
// 客户端发送
{ type: 'guidance:send', payload: { taskId: string, content: string }, correlationId: string }

// 服务端推送
{ type: 'guidance:send:ack', payload: { taskId: string, ok: boolean, guidanceId?: string, status?: string, reason?: string }, correlationId: string }
{ type: 'guidance:sent', payload: { taskId: string, guidanceId: string, contentPreview: string, status: string, createdAt: string, sentBy: string }, correlationId: string }
{ type: 'guidance:applied', payload: { taskId: string, guidanceId: string, appliedAt: string }, correlationId: string }
```

### Suggested Component Structure

```tsx
// frontend/src/features/user-intervention/GuidanceInput.tsx

interface GuidanceInputProps {
  taskId: string
  currentGuidance?: UserGuidance
  onSend: (content: string, correlationId: string) => void
  disabled?: boolean  // 非 Paused 状态时禁用
}

// 内部状态
// - inputValue: string
// - isSending: boolean
// - error: string | null
```

### Dev Agent Guardrails（避免常见踩坑）

- **不要在非 Paused 状态允许发送引导**：复用 Story 6.1/6.2 的权限校验逻辑模式。
- **不要忘记 correlationId**：所有 WS 命令和事件必须携带 correlationId（AR2）。
- **不要直接修改 store 状态**：发送操作需通过 WS 命令同步到后端，后端确认后再更新前端状态。
- **不要忽略错误处理**：发送失败时需返回用户友好的错误信息（NFR24）。
- **不要破坏现有暂停/编辑逻辑**：引导是增量功能，不得影响 Story 6.1/6.2 已有能力。
- **不要忽略可追踪性**：所有引导操作必须记录日志，支持审计（A2 门槛要求）。
- **不要记录引导原文**：日志仅记录前 50 字符预览（遵循 contracts 敏感内容禁止回显）。
- **引导单轮生效**：引导应用后需清理，不持久影响后续轮次。
- **状态二次校验**：`guidance:send` 执行时必须确认任务仍为 `Paused` 且 owner 匹配。

### Technical Requirements（必须满足）

- 发送按钮必须满足 WCAG 2.1 AA 无障碍标准（点击区域 ≥ 44px × 44px）。
- 输入框高度 ≥ 44px，支持多行输入。
- 引导操作必须在后端确认成功后才更新前端状态（以 `guidance:send:ack.ok=true` 为准）。
- WS 事件必须携带完整的引导信息，前端无需额外请求。
- 引导信息必须持久化到最小暂停持久化记录，支持重启恢复；Epic 7 完成后替换为完整 Checkpoint。
- 所有操作必须记录 tracing 日志，满足 A2 必填字段：`correlation_id`、`user_id`、`task_id`、`action`、`prev_state`、`new_state`、`iteration_state`、`timestamp`，并附 `guidance_preview`。
- `guidance:send` 处理时必须确认任务仍为 `Paused` 且 owner 匹配（服务端强制）。
- 引导内容长度校验：非空、≤ 2000 字符，失败给出可读错误提示。
- 日志/错误/WS payload 不得回显引导完整原文（遵循 contracts 禁止回显敏感内容）。
- 前端错误提示不得直接展示 `error.details`（遵循架构错误处理规范）。

### Architecture Compliance（必须遵守）

- **WS 事件命名**：遵循 `{domain}:{action}` 格式（`guidance:send`, `guidance:sent`）
- **WS envelope 形状**：遵循 `WsMessage<T>` 结构，`correlationId` 全链路透传
  - `docs/project-planning-artifacts/architecture.md#Communication Patterns`
  - `docs/developer-guides/contracts.md`
- **状态管理**：Zustand（全局）+ TanStack Query（服务端状态）
- **错误处理**：后端 `thiserror` + `anyhow`，前端统一错误响应结构
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types` 并提交生成产物

### Library / Framework Requirements (Version Snapshot)

- React：项目依赖 `react@^19.2.0`
- React Router：项目依赖 `react-router@^7.12.0`
- React Flow：项目锁定 `@xyflow/react@12.10.0`
- Zustand：项目依赖 `zustand@^5.x`
- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`

### File Structure Requirements（落点约束）

**后端**：
- 引导类型定义：`backend/src/domain/types/artifacts.rs`（扩展）
- extensions key：`backend/src/domain/types/extensions.rs`（扩展）
- WS 命令处理：`backend/src/api/ws/connection.rs`（扩展）
- WS 事件定义：`backend/src/api/ws/events.rs`（扩展）
- 暂停持久化扩展：`backend/src/core/iteration_engine/pause_state.rs`（扩展）
- 迭代编排层：`backend/src/core/iteration_engine/orchestrator.rs`（扩展）
- 老师模型调用：`backend/src/core/teacher_model/`（扩展以消费引导）

**前端**：
- 引导输入组件：`frontend/src/features/user-intervention/GuidanceInput.tsx`（新增）
- 组件导出：`frontend/src/features/user-intervention/index.ts`（扩展）
- 状态 store：`frontend/src/stores/useTaskStore.ts`（扩展）
- RunView 集成：`frontend/src/pages/RunView/RunView.tsx`（扩展）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 权限校验、状态校验、WS 命令处理、输入校验 | 非 Paused 状态发送被拒绝；非任务所有者发送被拒绝；空内容被拒绝；超长内容被拒绝；日志字段符合 A2（含 prev_state/new_state/iteration_state/timestamp） |
| 前端组件测试 | GuidanceInput 渲染与状态 | 输入框交互、发送按钮、禁用态、错误提示、Running 时保持可见但禁用 |
| 集成测试 | 端到端引导流程 | 暂停 → 发送引导 → 继续 → 引导生效完整流程；多次发送仅保留最后一次；引导应用后清理 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 复用 `frontend/src/features/user-intervention/` 目录结构（与 PauseResumeControl、ArtifactEditor 并列）。
- 复用 `frontend/src/stores/useTaskStore.ts` 的状态管理模式。
- 复用 Story 6.1/6.2 的 WS 命令处理框架。

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 6 / Story 6.3）
- PRD 用户介入：`docs/project-planning-artifacts/prd.md#能力区域 6: 用户介入`
- UX 规范：`docs/project-planning-artifacts/ux-design-specification.md`
- 架构（WS events/correlationId）：`docs/project-planning-artifacts/architecture.md#Communication Patterns`
- WS 契约单一权威：`docs/developer-guides/contracts.md`
- Epic 6 状态设计：`docs/implementation-artifacts/epic-6-run-control-state-design.md`
- Epic 6 开工门槛：`docs/implementation-artifacts/epic-5-retro-action-items-2026-01-16.md`
- 前序 Story learnings：`docs/implementation-artifacts/6-1-pause-and-resume-iteration.md`
- 前序 Story learnings：`docs/implementation-artifacts/6-2-edit-intermediate-artifacts.md`

## Git Intelligence Summary

- Story 6.1/6.2 相关提交：
  - 暂停/继续 WS 命令处理框架
  - PauseResumeControl 组件
  - ArtifactEditor 组件
  - pause_state.rs 最小持久化
  - artifacts.rs 类型定义
- 现有 WS 事件处理已包含 `task:pause`/`task:resume`/`artifact:get`/`artifact:update`，需扩展以支持 `guidance:*` 系列命令。
- 现有 `useTaskStore` 可直接扩展以存储 `userGuidance` 状态。

## Latest Tech Information (Web/Registry Snapshot)

- 版本以本地依赖快照为准：`frontend/package.json` 与 `backend/Cargo.toml`
- 关键关注点：本 Story 不涉及依赖升级，按现有版本实现即可

## Project Context Reference

- 以 `docs/project-planning-artifacts/*.md`、`docs/developer-guides/*` 与现有代码为准

## Story Completion Status

- Status set to `done`
- Completion note: Guidance consumption completed for Layer 1/4 metadata + L2/L3 prompts; backend/integration tests added; regressions passed (`cargo test`, `npx vitest --run`, `npm run build`).

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List
- Added Layer 1/4 guidance metadata propagation and coverage tests across rule engine/optimizer/evaluator/prompt generator.
- Added deterministic A2 log field test helper and updated pause_state logging coverage.
- Added guidance WS integration tests (pause → send → resume → apply/clear, last-one-wins, validations).
- Regression runs: `cd backend && cargo test`; `cd frontend && npx vitest --run`; `cd frontend && npm run build`.
- Resolved Vitest MSW WebSocket warnings and act() warnings in tests.

### File List

- `backend/src/api/ws/connection.rs`
- `backend/src/api/ws/events.rs`
- `backend/src/bin/gen-types.rs`
- `backend/src/core/evaluator/default_impl.rs`
- `backend/src/core/iteration_engine/pause_state.rs`
- `backend/src/core/optimization_engine/alternate_impl.rs`
- `backend/src/core/optimization_engine/common.rs`
- `backend/src/core/optimization_engine/default_impl.rs`
- `backend/src/core/optimization_engine/mod.rs`
- `backend/src/core/optimizer/default_impl.rs`
- `backend/src/core/prompt_generator/default_impl.rs`
- `backend/src/core/rule_engine/default_impl.rs`
- `backend/src/domain/types/artifacts.rs`
- `backend/src/domain/types/extensions.rs`
- `backend/src/domain/types/mod.rs`
- `backend/src/shared/ws.rs`
- `backend/tests/ws_guidance_integration_test.rs`
- `docs/implementation-artifacts/6-3-dialogue-guidance-for-teacher-model.md`
- `docs/implementation-artifacts/sprint-status.yaml`
- `frontend/src/App.routes.test.tsx`
- `frontend/src/components/streaming/StreamingText.test.tsx`
- `frontend/src/components/ui/textarea.tsx`
- `frontend/src/features/user-intervention/ArtifactEditor.test.tsx`
- `frontend/src/features/user-intervention/ArtifactEditor.tsx`
- `frontend/src/features/user-intervention/GuidanceInput.test.tsx`
- `frontend/src/features/user-intervention/GuidanceInput.tsx`
- `frontend/src/features/user-intervention/index.ts`
- `frontend/src/pages/RunView/RunView.tsx`
- `frontend/src/stores/useTaskStore.ts`
- `frontend/src/types/generated/models/GuidanceStatus.ts`
- `frontend/src/types/generated/models/IterationArtifacts.ts`
- `frontend/src/types/generated/models/UserGuidance.ts`
- `frontend/src/types/generated/ws/GuidanceAppliedPayload.ts`
- `frontend/src/types/generated/ws/GuidanceSendAckPayload.ts`
- `frontend/src/types/generated/ws/GuidanceSendPayload.ts`
- `frontend/src/types/generated/ws/GuidanceSentPayload.ts`
- `frontend/src/types/generated/ws/index.ts`

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [CRITICAL] `OptimizationContext.extensions` 新 key 未登记到 `backend/src/domain/types/extensions.rs`，违反 contracts 要求
- [x] [CRITICAL] A2 可追踪性日志字段不全（缺 prev_state/new_state/iteration_state/timestamp）
- [x] [HIGH] 引导生命周期未明确（多次发送覆盖、Applied 时机、清理位置）
- [x] [HIGH] guidance 相关 WS 事件类型落点不明确（events.rs 权威）
- [x] [MEDIUM] 增加“引导被消费/清理”的可验证测试方法
- [x] [LOW] “Latest Tech Information”需要以本地依赖快照为准

### Decisions

- [x] 明确“Last One Wins”覆盖策略，避免队列/历史管理复杂度
- [x] 复用 pause_state 的 `context_snapshot`（沿用 6.2 路径），不引入并行字段
- [x] 设定引导在 Layer 1 开始前应用并推送 `guidance:applied`，轮次结束清理
- [x] UI 在 Running 状态保持可见但禁用，以便用户确认引导状态

### Risks / Tech Debt

- [ ] Epic 7 完整 Checkpoint 未实现前，恢复能力仍是"最小暂停快照"级别。
- [ ] 本 MVP 仅支持单条引导/单轮生效，未来可扩展为多轮对话或引导历史。
- [ ] 引导注入涉及 Layer 1-4 多处改动，若未统一规范易出现不一致行为。

### Follow-ups

- [x] 将上述 [AI-Review] 项同步到实施任务与测试计划
