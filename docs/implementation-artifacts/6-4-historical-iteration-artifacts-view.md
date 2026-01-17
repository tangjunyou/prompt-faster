# Story 6.4: 历史迭代产物查看

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 6-4-historical-iteration-artifacts-view

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

- **数据获取方式**：使用 HTTP API（`/api/v1/tasks/{task_id}/iterations`）获取历史数据，因为历史数据是静态的，不需要 WebSocket 实时推送。
- **只读实现**：复用 `ArtifactEditor` 组件并新增 `readOnly=true` 模式（与 `disabled` 区分），确保历史产物不可编辑且不展示“请先暂停”文案。
- **视图模式**：在 RunView 的右侧面板增加"历史"tab，与当前迭代产物并列展示。
- **列表展示**：按迭代轮次倒序排列（最新在前），每项显示轮次编号、时间戳、通过率摘要。
- **详情展开**：点击历史项展开显示完整产物（规律假设、候选 Prompt、评估结果、反思总结）。
- **分页策略**：MVP 阶段不分页，但服务端需加防御性上限（例如最多返回最近 100 条）。
- **AR2 遵循**：所有 API 请求必须携带 `correlationId`，确保可追踪。
- **审计日志**：历史查看操作需记录日志（满足 A2 可追踪性要求）。

## Story

As a Prompt 优化用户,
I want 查看历史迭代产物,
so that 我可以回顾优化过程，理解模型的演进路径。

## Acceptance Criteria

1. **Given** 优化任务已执行多轮迭代
   **When** 用户点击"历史"按钮或 tab
   **Then** 显示历史迭代列表（按轮次倒序排列，最新在前）
   **And** 每轮显示：轮次编号、时间戳、通过率

2. **Given** 用户在历史迭代列表中
   **When** 点击某一轮历史
   **Then** 展开显示该轮的完整产物：
     - 规律假设（patterns）
     - 候选 Prompt（candidate_prompts）
     - 评估结果（evaluation_results）
     - 反思总结（reflection_summary）

3. **Given** 用户查看历史产物
   **When** 尝试编辑
   **Then** 历史产物为只读状态，不可编辑
   **And** 显示提示"历史记录仅供查看"

4. **Given** 优化任务尚未执行任何迭代
   **When** 用户打开历史面板
   **Then** 显示空状态提示"暂无历史记录"
   **And** 提示开始优化任务以生成历史
   **And** 提供"开始优化"按钮（或引导 CTA）

5. **Given** 用户查看历史产物详情
   **When** 查看规律假设或候选 Prompt
   **Then** 使用与 Story 6.2 相同的渲染格式（复用 ArtifactEditor 只读模式）
   **And** 支持代码高亮和格式化显示

6. **Given** 历史查看操作
   **When** 用户访问历史详情
   **Then** 操作记录在日志中（含 correlationId、用户 ID、任务 ID、轮次 ID、action、iteration_state、timestamp）
   **And** 支持后续审计与状态回放（A2 要求）

7. **Given** 网络请求失败
   **When** 加载历史数据
   **Then** 显示清晰的错误信息
   **And** 提供重试选项
   **And** 错误信息遵循 NFR24 统一错误文案规范

## Tasks / Subtasks

- [x] 数据库：历史迭代表 Schema 与迁移（AC: 1-7）
  - [x] 创建迁移 `backend/migrations/008_create_iterations.sql`
  - [x] 定义 `iterations` 表字段（见下方 Database Schema）
  - [x] 添加索引：`idx_iterations_task_id`、`idx_iterations_task_id_round_desc`
  - [x] 运行迁移并验证

- [x] 后端：IterationHistory Repo（AC: 1,2,6）
  - [x] 新增 `backend/src/infra/db/repositories/iteration_repo.rs`
  - [x] `list_by_task_id(task_id, limit)` / `get_by_id(task_id, iteration_id)` 查询方法
  - [x] 复用现有 DB pool / 错误处理模式

- [x] 后端：历史迭代查询 API（AC: 1,2,6）
  - [x] 在 `backend/src/api/routes/iterations.rs` 中添加 `GET /api/v1/tasks/{task_id}/iterations` 端点
  - [x] 定义 `IterationHistorySummary` DTO（包含轮次编号、时间戳、通过率等摘要）
  - [x] 定义 `IterationHistoryDetail` DTO（包含完整产物 + evaluation_results）
  - [x] 在 `backend/src/api/routes/iterations.rs` 中实现查询逻辑
  - [x] 实现权限校验（仅任务所有者可查看，复用现有 ownership 校验模式）
  - [x] 记录操作日志（tracing，包含 correlation_id/user_id/task_id/action/iteration_state/timestamp）
  - [x] 对列表 API 加防御性 limit（例如 100）
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增 DTO

- [x] 后端：单个迭代详情 API（AC: 2,5）
  - [x] 在 `backend/src/api/routes/iterations.rs` 中添加 `GET /api/v1/tasks/{task_id}/iterations/{iteration_id}` 端点
  - [x] 返回完整的 `IterationHistoryDetail`（包含 evaluation_results 与 reflection_summary）

- [x] 前端：HistoryPanel 组件（AC: 1,4）
  - [x] 在 `frontend/src/features/user-intervention/history/` 中创建 `HistoryPanel.tsx` 组件
  - [x] 实现历史迭代列表展示（轮次编号、时间戳、通过率）
  - [x] 实现列表项点击展开/折叠
  - [x] 实现空状态显示"暂无历史记录"
  - [x] 增加空状态 CTA（"开始优化"）
  - [x] 使用 TanStack Query 管理数据获取

- [x] 前端：IterationHistoryItem 组件（AC: 2,3,5）
  - [x] 在 `frontend/src/features/user-intervention/history/` 中创建 `IterationHistoryItem.tsx` 组件
  - [x] 显示轮次摘要信息（轮次编号、时间戳、通过率）
  - [x] 展开后复用 `ArtifactEditor` 组件（`readOnly=true`）
  - [x] 显示"历史记录仅供查看"提示

- [x] 前端：HistoryDetailView 组件（AC: 2,5）
  - [x] 在 `frontend/src/features/user-intervention/history/` 中创建 `HistoryDetailView.tsx` 组件
  - [x] 分 tab 展示：规律假设 / 候选 Prompt / 评估结果 / 反思总结
  - [x] 复用 ArtifactEditor 只读模式渲染各产物

- [x] 前端：集成到 RunView 页面（AC: 1,3）
  - [x] 在 `frontend/src/pages/RunView/RunView.tsx` 中集成 HistoryPanel
  - [x] 在右侧面板添加"历史"tab
  - [x] 实现 tab 切换逻辑（当前产物 / 历史）
  - [x] 不影响左侧画布与已有 Pause/Guidance 功能

- [x] 前端：服务层封装（AC: 1,7）
  - [x] 在 `frontend/src/features/user-intervention/history/services/` 中创建 `iterationHistoryService.ts`
  - [x] 实现 `getIterationHistory(taskId)` 函数
  - [x] 实现 `getIterationDetail(taskId, iterationId)` 函数
  - [x] 在 `frontend/src/features/user-intervention/history/hooks/` 中创建 `useIterationHistory.ts`
  - [x] QueryKey 约定：`['iterations', taskId]`

- [x] 前端：ArtifactEditor 只读模式（AC: 3,5）
  - [x] 在 `frontend/src/features/user-intervention/ArtifactEditor.tsx` 新增 `readOnly?: boolean`
  - [x] readOnly 模式下隐藏编辑按钮、不显示“请先暂停任务再编辑”提示
  - [x] Monaco Editor 通过 `readOnly: true` 禁止编辑

- [x] 测试与回归（AC: 1-7）
  - [x] 后端单测：历史列表 API 返回正确数据
  - [x] 后端单测：权限校验（非任务所有者返回 403）
  - [x] 后端单测：空历史返回空数组
  - [x] 前端组件测试：HistoryPanel 渲染、展开折叠
  - [x] 前端组件测试：IterationHistoryItem 只读状态
  - [x] 前端组件测试：空状态显示
  - [x] 集成测试：完整的历史查看流程
  - [x] 回归命令：`cd backend && cargo test`；`cd frontend && npx vitest --run && npm run build`
  - [x] 生成类型：`cd backend && cargo run --bin gen-types` 并提交产物

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review][CRITICAL] 补齐 iterations 表迁移与索引（Schema + Repo + 查询路径）
- [x] [AI-Review][CRITICAL] 明确 `evaluation_results` 结构与来源（独立于 IterationArtifacts）
- [x] [AI-Review][HIGH] ArtifactEditor 增加 `readOnly` 语义，避免 “请先暂停” 文案误导
- [x] [AI-Review][HIGH] 统一 DTO 落点与 `gen-types.rs` 注册清单
- [x] [AI-Review][MEDIUM] RunView 右侧 tab 集成说明与状态管理落点
- [x] [AI-Review][MEDIUM] 前端目录/服务层落点对齐 `features/user-intervention/` 结构
- [x] [AI-Review][LOW] 空状态 CTA 与防御性 limit/QueryKey 约定落地

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
  - Story 6.3 已完成对话引导功能，提供：
    - `UserGuidance` 类型定义
    - `GuidanceInput` 组件
    - `guidance:send`/`guidance:applied` WS 命令框架
  - Epic 4 已完成四层架构；历史数据目标落在 `iterations` 表（本 Story 需补齐迁移）

- **Epic 6 全景（便于对齐业务价值与范围）**：
  - 6.1 暂停与继续迭代（已完成，FR40/FR44）
  - 6.2 编辑中间产物（已完成，FR41）
  - 6.3 对话引导老师模型（已完成，FR42）
  - **6.4 历史迭代产物查看（本 Story，FR43）**
  - 6.5 迭代控制：增加轮数/手动终止（FR45/FR46）

- **业务价值（为什么做）**：让用户可以回顾完整的优化过程，理解模型演进路径，从历史迭代中学习优化经验。这是 Epic 6 用户介入能力的重要补充，帮助用户建立对优化过程的"理解感"（来源：PRD 能力区域 6 / FR43 / UX 情感设计）。

- **依赖关系**：
  - 依赖 Story 6.2 提供的 `IterationArtifacts` 类型和 `ArtifactEditor` 组件
  - 依赖 Epic 4 的迭代数据存储（目标为 `iterations` 表，本 Story 需补齐迁移）
  - 本 Story 与 Epic 7 的 Checkpoint 机制相关但不依赖

- **范围边界（必须遵守）**：
  - 历史产物仅支持只读查看，不支持编辑（编辑是 6.2 的功能）
  - 不支持历史产物的导出（8.1 承接导出功能）
  - 不支持历史回滚（7.3 承接回滚功能）
  - MVP 阶段不支持分页或搜索
  - 历史列表需设置防御性上限（例如最近 100 条）

### Database Schema (Must Create First)

> 说明：当前 migrations 未包含 `iterations` 表，需要新增迁移后才能实现历史查询。

```sql
-- 位置：backend/migrations/008_create_iterations.sql
CREATE TABLE IF NOT EXISTS iterations (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  round INTEGER NOT NULL,
  started_at INTEGER NOT NULL, -- Unix ms
  completed_at INTEGER,
  status TEXT NOT NULL, -- running/completed/failed/terminated
  artifacts TEXT, -- JSON: IterationArtifacts
  evaluation_results TEXT, -- JSON: EvaluationResultSummary[]
  reflection_summary TEXT,
  pass_rate REAL NOT NULL DEFAULT 0.0,
  total_cases INTEGER NOT NULL DEFAULT 0,
  passed_cases INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (task_id) REFERENCES optimization_tasks(id)
);
CREATE INDEX IF NOT EXISTS idx_iterations_task_id ON iterations(task_id);
CREATE INDEX IF NOT EXISTS idx_iterations_task_id_round_desc ON iterations(task_id, round DESC);
```

### Evaluation Results Source

- 历史详情中的 `evaluation_results` 为**独立结构**，不扩展 `IterationArtifacts`，避免影响 Story 6.2/6.3。
- 推荐直接从 `iterations.evaluation_results`（JSON）读取；若暂缺结果，返回空数组并保证前端可渲染。

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/types/iteration_history.rs

/// 历史迭代列表项
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct IterationHistorySummary {
    /// 迭代 ID
    pub id: String,
    /// 轮次编号（从 1 开始）
    pub round: i32,
    /// 迭代开始时间（ISO 8601）
    pub started_at: String,
    /// 迭代结束时间（ISO 8601，进行中为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// 通过率（0.0 - 1.0）
    pub pass_rate: f64,
    /// 测试用例总数
    pub total_cases: i32,
    /// 通过的测试用例数
    pub passed_cases: i32,
    /// 迭代状态
    pub status: IterationStatus,
}

/// 迭代状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum IterationStatus {
    /// 进行中
    Running,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 被用户终止
    Terminated,
}

/// 评估结果摘要（历史查看专用）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct EvaluationResultSummary {
    /// 测试用例 ID
    pub test_case_id: String,
    /// 是否通过
    pub passed: bool,
    /// 分数（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// 失败原因（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

/// 历史迭代详情响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct IterationHistoryDetail {
    /// 基础摘要信息
    #[serde(flatten)]
    pub summary: IterationHistorySummary,
    /// 完整产物
    pub artifacts: IterationArtifacts,
    /// 评估结果
    pub evaluation_results: Vec<EvaluationResultSummary>,
    /// 反思总结（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reflection_summary: Option<String>,
}
```

### Suggested API Endpoints

```
# 获取历史迭代列表
GET /api/v1/tasks/{task_id}/iterations
Query: limit (optional, default 100)
Response: ApiResponse<Vec<IterationHistorySummary>>

# 获取单个迭代详情
GET /api/v1/tasks/{task_id}/iterations/{iteration_id}
Response: ApiResponse<IterationHistoryDetail>
```

### Suggested Component Structure

```tsx
// frontend/src/features/user-intervention/history/HistoryPanel.tsx

interface HistoryPanelProps {
  taskId: string
}

// 内部状态
// - expandedIterationId: string | null
// - isLoading: boolean
// - error: Error | null
```

```tsx
// frontend/src/features/user-intervention/history/IterationHistoryItem.tsx

interface IterationHistoryItemProps {
  summary: IterationHistorySummary
  isExpanded: boolean
  onToggle: () => void
}
```

```tsx
// frontend/src/features/user-intervention/history/HistoryDetailView.tsx

interface HistoryDetailViewProps {
  detail: IterationHistoryDetail
}

// 内部 tab 切换：patterns / prompts / results / reflection
```

### Dev Agent Guardrails（避免常见踩坑）

- **历史产物必须只读**：复用 ArtifactEditor 时务必传入 `readOnly=true`，禁止任何编辑操作。
- **不要使用 disabled 代替只读**：disabled 会展示“请先暂停任务再编辑”，历史查看必须避免该文案。
- **不要忘记 correlationId**：所有 API 请求必须携带 correlationId（AR2）。
- **权限校验必须严格**：只有任务所有者可以查看历史，防止跨用户数据泄露。
- **不要破坏现有编辑逻辑**：历史查看是增量功能，不得影响 Story 6.2 的编辑能力。
- **使用 TanStack Query**：历史数据获取必须通过 TanStack Query，利用缓存减少重复请求。
- **空状态友好处理**：无历史记录时显示引导性空状态，避免用户困惑。

### Technical Requirements（必须满足）

- 列表项和详情按钮必须满足 WCAG 2.1 AA 无障碍标准（点击区域 ≥ 44px × 44px）。
- 历史面板最大高度受限，超出时内部滚动。
- API 响应必须使用 `ApiResponse<T>` 统一结构。
- 所有操作必须记录 tracing 日志，满足 A2 必填字段：`correlation_id`、`user_id`、`task_id`、`action`、`iteration_state`、`timestamp`。只读操作的 `prev_state/new_state` 需明确为同值或 N/A。
- 前端错误提示不得直接展示 `error.details`（遵循架构错误处理规范）。
- 历史数据缓存策略：staleTime 30 秒，允许后台刷新。

### Architecture Compliance（必须遵守）

- **API 端点命名**：遵循 RESTful 风格，`/api/v1/tasks/{task_id}/iterations`
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **状态管理**：TanStack Query 管理服务端状态
- **错误处理**：后端 `thiserror` + `anyhow`，前端统一错误响应结构
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types` 并提交生成产物

### Library / Framework Requirements (Version Snapshot)

- React：项目依赖 `react@^19.2.0`
- React Router：项目依赖 `react-router@^7.12.0`
- TanStack Query：项目依赖 `@tanstack/react-query`
- Zustand：项目依赖 `zustand@^5.x`
- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`

### File Structure Requirements（落点约束）

**后端**：
- 迭代 API 路由：`backend/src/api/routes/iterations.rs`（扩展）
- 迭代 Repo：`backend/src/infra/db/repositories/iteration_repo.rs`（新增）
- 历史 DTO：`backend/src/domain/types/iteration_history.rs`（新增）
- 数据库迁移：`backend/migrations/008_create_iterations.sql`（新增）

**前端**：
- 历史面板组件：`frontend/src/features/user-intervention/history/HistoryPanel.tsx`（新增）
- 历史项组件：`frontend/src/features/user-intervention/history/IterationHistoryItem.tsx`（新增）
- 历史详情组件：`frontend/src/features/user-intervention/history/HistoryDetailView.tsx`（新增）
- 服务层：`frontend/src/features/user-intervention/history/services/iterationHistoryService.ts`（新增）
- Hooks：`frontend/src/features/user-intervention/history/hooks/useIterationHistory.ts`（新增）
- RunView 集成：`frontend/src/pages/RunView/RunView.tsx`（扩展）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 历史 API、权限校验 | 返回正确历史列表；非任务所有者返回 403；空历史返回空数组 |
| 前端组件测试 | HistoryPanel、IterationHistoryItem | 列表渲染、展开折叠、只读状态、空状态 |
| 集成测试 | 端到端历史查看 | 执行迭代 → 查看历史 → 展开详情完整流程 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 复用 `frontend/src/features/user-intervention/` 目录结构（与 ArtifactEditor/GuidanceInput 并列）。
- 复用 `frontend/src/features/user-intervention/ArtifactEditor.tsx` 的只读模式。
- 复用 TanStack Query 的数据获取模式。

### RunView Integration Details

- **面板布局**：右侧面板增加 Tab 切换：`当前产物` | `历史记录`。
- **默认行为**：默认显示“当前产物”（复用 Story 6.2 的 ArtifactEditor）。
- **切换行为**：仅切换右侧内容，不影响左侧画布与实时流式输出。
- **状态管理**：可在本地组件状态或 `useTaskStore` 中维护当前 tab（可选持久化）。

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 6 / Story 6.4）
- PRD 用户介入：`docs/project-planning-artifacts/prd.md#能力区域 6: 用户介入`
- UX 规范：`docs/project-planning-artifacts/ux-design-specification.md`
- 架构（API patterns）：`docs/project-planning-artifacts/architecture.md#API & Communication Patterns`
- Epic 6 状态设计：`docs/implementation-artifacts/epic-6-run-control-state-design.md`
- Epic 6 开工门槛：`docs/implementation-artifacts/epic-5-retro-action-items-2026-01-16.md`
- Epic 6 可追踪性规范（A2）：`docs/implementation-artifacts/epic-6-traceability-verification.md`
- 前序 Story learnings：`docs/implementation-artifacts/6-2-edit-intermediate-artifacts.md`
- 前序 Story learnings：`docs/implementation-artifacts/6-3-dialogue-guidance-for-teacher-model.md`

## Git Intelligence Summary

- Story 6.2/6.3 相关提交：
  - ArtifactEditor 组件（可复用只读模式）
  - IterationArtifacts 类型定义
  - API 路由与 DTO 定义模式（参考 `backend/src/api/routes/optimization_tasks.rs`）
- 历史数据目标存储在 `iterations` 表；需先完成迁移后再查询该表获取历史数据。

## Latest Tech Information (Web/Registry Snapshot)

- 版本以本地依赖快照为准：`frontend/package.json` 与 `backend/Cargo.toml`
- 关键关注点：本 Story 不涉及依赖升级，按现有版本实现即可

## Project Context Reference

- 以 `docs/project-planning-artifacts/*.md`、`docs/developer-guides/*` 与现有代码为准

## Story Completion Status

- Status set to `done`
- Completion note: 历史迭代查看功能完成并完成回归与集成测试；已生成类型产物

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

- N/A

### Completion Notes List

- 右侧面板增加“当前产物/历史记录”Tab，并接入空状态 CTA
- 历史 API 请求统一注入 correlationId
- readOnly 模式启用 Monaco 高亮展示
- 增加历史 API 集成测试与前端组件测试
- 修复日志字段与 limit 边界
- 已补充完整历史查看集成测试，并完成后端/前端回归与 gen-types

### File List

- backend/migrations/008_create_iterations.sql
- backend/src/api/routes/iterations.rs
- backend/src/api/routes/mod.rs
- backend/src/bin/gen-types.rs
- backend/src/domain/types/iteration_history.rs
- backend/src/domain/types/mod.rs
- backend/src/infra/db/repositories/iteration_repo.rs
- backend/src/infra/db/repositories/mod.rs
- backend/src/main.rs
- backend/tests/iterations_api_test.rs
- frontend/src/lib/api.ts
- frontend/src/features/user-intervention/ArtifactEditor.tsx
- frontend/src/features/user-intervention/index.ts
- frontend/src/features/user-intervention/history/HistoryPanel.tsx
- frontend/src/features/user-intervention/history/IterationHistoryItem.tsx
- frontend/src/features/user-intervention/history/HistoryDetailView.tsx
- frontend/src/features/user-intervention/history/index.ts
- frontend/src/features/user-intervention/history/hooks/useIterationHistory.ts
- frontend/src/features/user-intervention/history/services/iterationHistoryService.ts
- frontend/src/pages/RunView/RunView.tsx
- frontend/src/pages/RunView/RunView.test.tsx
- frontend/src/features/user-intervention/history/HistoryPanel.test.tsx
- frontend/src/features/user-intervention/history/IterationHistoryItem.test.tsx
- frontend/src/features/user-intervention/history/HistoryDetailView.test.tsx
- frontend/src/types/generated/models/EvaluationResultSummary.ts
- frontend/src/types/generated/models/IterationHistoryDetail.ts
- frontend/src/types/generated/models/IterationHistorySummary.ts
- frontend/src/types/generated/models/IterationStatus.ts
- docs/implementation-artifacts/6-4-historical-iteration-artifacts-view.md
- docs/implementation-artifacts/sprint-status.yaml

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [CRITICAL] 缺失 iterations 表迁移与索引，导致历史查询无法落地
- [x] [CRITICAL] `evaluation_results` 需求与现有 `IterationArtifacts` 不一致，需独立结构
- [x] [HIGH] `ArtifactEditor` 仅有 disabled 语义，直接复用会出现错误提示
- [x] [HIGH] DTO 落点冲突（handler vs domain/types），需统一并补 `gen-types.rs`
- [x] [MEDIUM] RunView 右侧 tab 集成方式不明确，易与现有布局冲突
- [x] [MEDIUM] 前端目录与服务层落点需对齐现有 feature 组织方式
- [x] [LOW] 空状态 CTA、QueryKey 规范与列表防御性 limit 可提升体验与稳定性

### Decisions

- [x] `evaluation_results` 作为历史详情独立字段，不扩展 `IterationArtifacts`（避免影响 6.2/6.3）
- [x] 前端历史查看落在 `features/user-intervention/history/`（对齐 FR40-46 归属）
- [x] 新增 `readOnly` 语义而非复用 `disabled`（避免错误提示）
- [x] 列表 API 增加防御性 limit（MVP 不分页但防止极端数据量）

### Risks / Tech Debt

- [ ] Epic 7 完整 Checkpoint 未实现前，历史数据来源依赖 iterations 表的直接查询。
- [ ] MVP 阶段不支持分页，大量历史数据可能影响性能（后续可扩展）。
- [ ] 历史查看与回滚功能边界需在 6.4/7.3 中明确划分。
- [ ] Epic 7 完成后需评估历史数据是否迁移到 Checkpoint 查询，避免重复维护两套历史读路径。

### Follow-ups

- [x] 补齐 DB 迁移 + Repo + API 查询实现细节与测试用例
- [x] 明确评估结果 JSON 结构（字段映射、序列化/反序列化）
- [x] 完成 `ArtifactEditor` 只读模式改造与对应组件测试
- [x] 明确 RunView tab 实现方式与状态管理落点
- [x] 补充前端服务层封装与 QueryKey/缓存策略约定
