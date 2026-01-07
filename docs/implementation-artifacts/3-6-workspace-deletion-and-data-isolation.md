# Story 3.6: 工作区删除与数据隔离

Status: done

Story Key: `3-6-workspace-deletion-and-data-isolation`

Related FRs: FR49（删除工作区）, FR50（工作区数据隔离）

Epic: Epic 3（优化任务配置与工作区）

Dependencies: Story 3.5（工作区创建与切换）, Epic 1（登录/认证与用户隔离）, Epic 2（测试集管理）, Story 3.1-3.4（工作区路由与任务/测试集能力已落地）

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 删除不需要的工作区，并确保工作区之间数据完全隔离,
so that 我可以保持系统整洁，且不同项目/实验互不干扰。

## Acceptance Criteria

### AC1：工作区管理页提供删除入口并二次确认（FR49）

**Given** 用户在工作区管理页面（Workspace View）  
**When** 用户对某个工作区点击“删除”  
**Then** 必须弹出确认对话框（Destructive Dialog）  
**And** 明确告知删除范围：该工作区下的任务/测试集等数据将被删除，且不可撤销  
**And** 用户取消则不发生任何删除请求  

### AC2：确认删除后调用后端删除接口并正确反馈（FR49）

**Given** 用户在确认对话框中点击“确认删除”  
**When** 前端调用 `DELETE /api/v1/workspaces/{id}`  
**Then** 删除成功后刷新工作区列表（`GET /api/v1/workspaces`）  
**And** UI 展示成功反馈（toast 或页面提示均可）  
**And** 删除失败时只展示 `error.message`（不得展示 `error.details`）  

### AC3：删除当前工作区后的自动切换（FR49）

**Given** 用户删除的是“当前工作区”（满足其一即可：URL 为 `/workspaces/:id/...` 的 `:id`；或该用户的 `lastWorkspaceId` 等于被删 id）  
**When** 删除成功  
**Then** 若仍存在其他工作区 → 自动切换到一个可用工作区（后端列表顺序的第一个即可）  
**And** 若工作区列表为空 → 自动进入 `/workspace` 并引导创建工作区  
**And** 不得出现“停留在已删除工作区路由导致 404/空白页”的体验  

### AC4：工作区之间数据完全隔离（FR50）

**Given** 系统存在工作区 A 与工作区 B  
**When** 用户在 A 中创建任务/测试集等数据  
**Then** 这些数据不会出现在 B 中  
**And** 数据库层面通过 `workspace_id` 做隔离（所有 workspace-scoped 查询必须包含 `workspace_id` 过滤）  

### AC5：删除工作区后数据被彻底清理且不串扰（FR49/FR50）

**Given** 被删除的工作区下存在测试集/优化任务/任务-测试集关联等数据  
**When** 删除工作区成功  
**Then** 相关数据必须被一并删除（通过外键 `ON DELETE CASCADE`）  
**And** 前端不得继续展示已删除工作区缓存数据（需清理/失效对应 workspace 的 TanStack Query 缓存）  

## Tasks / Subtasks

### 任务 1：工作区管理页的删除入口 + 二次确认（AC1）

- [x] 在 `frontend/src/pages/WorkspaceView/WorkspaceView.tsx` 的工作区列表中，为每个工作区提供“删除”按钮（Destructive 样式）
- [x] 确认对话框实现（写死，避免“临场发挥”导致 a11y/UX 偏离）：
  - [x] 本仓库当前未内置 `AlertDialog`/`Dialog` 组件；**不得新增依赖**（不引入 `@radix-ui/react-alert-dialog` 等）
  - [x] 采用与 `frontend/src/components/common/WorkspaceSelector.tsx` 一致的 overlay dialog 模式（`fixed inset-0` 遮罩 + `role="dialog"` + `aria-modal="true"` + `Card` 容器）
  - [x] 文案必须明确：删除不可撤销，且会删除该工作区下的任务/测试集/关联数据
  - [x] 支持取消；确认按钮在请求中禁用并显示 loading（防重复点击/双击）
- [x] 仅在 `authStatus === 'authenticated'` 时允许触发删除（页面本身应已受保护，但组件仍需稳健）

### 任务 2：删除请求、错误展示与列表刷新（AC2）

- [x] 复用已有 `useDeleteWorkspace()`（`frontend/src/features/workspace/hooks/useWorkspaces.ts`）发起删除
- [x] 删除成功后：
  - [x] `invalidateQueries({ queryKey: ['workspaces'] })` 刷新列表
  - [x] 显示成功提示（toast/inline 均可）
- [x] 删除失败：
  - [x] 只展示 `error.message`（不得展示 `error.details`；架构强约束）
  - [x] 若是 401/UNAUTHORIZED：沿用全局 unauthorized handler 处理登出/跳转

### 任务 3：删除当前工作区后的自动切换 + 缓存清理（AC3/AC5）

- [x] 定义“当前工作区”判定（必须写死，避免实现分歧）：
  - [x] 若当前 URL 命中 `/workspaces/:id/...` → `id` 为当前工作区
  - [x] 否则使用 `useWorkspaceStore.lastWorkspaceIdByUser[currentUser.id]`（若存在）
- [x] 删除成功后导航策略（必须明确且可测）：
  - [x] 若仍有其他工作区 → 导航到 `/workspaces/:nextId/tasks`（或保持当前 section 的策略，**必须与** `frontend/src/components/common/WorkspaceSelector.tsx` 内 `getWorkspaceSwitchTargetPath` **一致**）
  - [x] 若无其他工作区 → 导航到 `/workspace`（展示创建引导）
- [x] “导航逻辑”落点（写死，避免实现分歧）：
  - [x] 在 `WorkspaceView.tsx`（调用处）处理导航（需要依赖当前路由 `location.pathname`）
  - [x] `useDeleteWorkspace()` 保持为“发请求 + 刷新 workspaces 列表”的通用 hook；**不要**把路由导航塞进 hook
- [x] 清理 `lastWorkspaceIdByUser`：
  - [x] 若被删 id 等于该用户的 lastWorkspaceId → 清除该键，避免下次默认落入 404
- [x] TanStack Query 缓存清理（最小可行）：
  - [x] **必须在导航前**清理（避免短时间窗口渲染旧缓存）
  - [x] `removeQueries({ queryKey: ['testSets', deletedId] })`（见 `frontend/src/features/test-set-manager/hooks/useTestSets.ts`）
  - [x] `removeQueries({ queryKey: ['optimizationTasks', deletedId] })`（见 `frontend/src/features/task-config/hooks/useOptimizationTasks.ts`）
  - [x] `removeQueries({ queryKey: ['testSetTemplates', deletedId] })`（见 `frontend/src/features/test-set-manager/hooks/useTestSetTemplates.ts`）
  - [x] `removeQueries({ queryKey: ['workspaces', deletedId] })`（workspace 详情缓存，见 `frontend/src/features/workspace/hooks/useWorkspaces.ts`）

### 任务 4：后端与数据库层验证（AC4/AC5）

- [x] 确认后端删除接口与鉴权行为符合预期：
  - [x] `DELETE /api/v1/workspaces/{id}` 必须按 `user_id` 作用域删除（禁止跨用户删除；返回 404）
  - [x] SQLite 外键启用（`foreign_keys = true`）以保证 `ON DELETE CASCADE` 生效
- [x] 回归/补充测试（推荐新增/补足缺口）：
  - [x] 后端集成测试：删除工作区后，访问其 workspace-scoped 资源返回 404（并验证列表中不存在）
  - [x] 后端集成测试：同一用户多工作区下的数据隔离（A 的测试集/任务不应出现在 B）
  - [x] 后端集成测试：删除工作区应级联删除其 test_sets/optimization_tasks/关联表（至少验证“不可再访问 + 列表为空”）
  - [x] （可选但推荐）在 `backend/tests/workspaces_api_test.rs` 中按意图命名补齐用例：
    - [x] `test_delete_workspace_requires_ownership_returns_404`
    - [x] `test_delete_workspace_cascades_to_test_sets_and_optimization_tasks`
    - [x] `test_delete_workspace_cascades_to_optimization_task_test_sets`

### 任务 5：前端回归测试（AC1-AC3/AC5）

- [x] Vitest + Testing Library 覆盖：
  - [x] Workspace View 中点击删除 → 弹出确认对话框
  - [x] 取消 → 不触发删除请求
  - [x] 确认删除成功 → 列表刷新且移除该工作区
  - [x] 删除当前工作区时 → 自动导航到其它 workspace 或 `/workspace`
  - [x] 删除失败 → 展示 `error.message`（不展示 `error.details`）
  - [x] 边界场景（避免回归/交互毛刺）：
    - [x] 确认按钮 loading 禁用：快速连续点击不会触发重复请求
    - [x] 删除最后一个工作区：仍停留在 `/workspace` 并看到“创建工作区”入口（不出现空白页）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 确认对话框实现路径写死：复用 `WorkspaceSelector.tsx` overlay 模式，不新增 Dialog/AlertDialog 依赖
- [x] [AI-Review] 删除后的导航规则与 `WorkspaceSelector.tsx` 保持一致（必要时抽取为共享 util，避免两处漂移）
- [x] [AI-Review] 删除成功后缓存清理清单补齐：`testSets` / `optimizationTasks` / `testSetTemplates` / `workspaces:detail`
- [x] [AI-Review] 补齐依赖版本钉死（React/Router/Query/Zustand + Axum/SQLx/Rust）

## Dev Notes

### Developer Context（避免实现偏航）

- Epic 3 目标：多工作区隔离管理，让用户在同一系统中区分不同项目/实验数据。
- 本 Story（3.6）定位：补齐“删除工作区 + 删除后的体验/安全/缓存清理”，并验证“工作区级隔离”不会被破坏。
- 明确非目标（防 scope creep）：
  - 不做“软删除/回收站/恢复”（本 Story 视为硬删除）
  - 不做“跨用户数据迁移”
  - 不重构路由结构（沿用 `/workspaces/:id/...` 与 `/workspace`）

### Hard Prerequisites（硬前置：必须已存在/不得改语义）

- 后端已存在并可用的接口（需登录）：
  - `GET /api/v1/workspaces`
  - `DELETE /api/v1/workspaces/{id}`
  - workspace-scoped 资源（用于隔离与删除后验证）：`/api/v1/workspaces/{workspace_id}/test-sets`、`/api/v1/workspaces/{workspace_id}/optimization-tasks`、`/api/v1/workspaces/{workspace_id}/test-set-templates`
- 数据库约束（删除与隔离的根基）：
  - `test_sets.workspace_id` 外键 `REFERENCES workspaces(id) ON DELETE CASCADE`（见 `backend/migrations/003_create_test_sets.sql`）
  - `optimization_tasks.workspace_id` 外键 `REFERENCES workspaces(id) ON DELETE CASCADE`（见 `backend/migrations/007_create_optimization_tasks.sql`）
  - SQLite 连接必须启用外键：`foreign_keys(true)`（见 `backend/src/infra/db/pool.rs`）
- 前端能力已存在（可直接复用）：
  - workspace service 已有 `deleteWorkspace`（见 `frontend/src/features/workspace/services/workspaceService.ts`）
  - hook 已有 `useDeleteWorkspace` + `WORKSPACES_QUERY_KEY`（见 `frontend/src/features/workspace/hooks/useWorkspaces.ts`）
  - `WorkspaceSelector` + `lastWorkspaceIdByUser` 已存在（Story 3.5 产物）

### Technical Requirements（DEV Agent Guardrails）

#### 1) 删除必须“可感知 + 可撤销的最后机会”

- 删除是 Destructive 操作：必须二次确认（Dialog）。
- 确认文案必须明确“作用范围”和“不可撤销”（推荐文案，便于测试断言）：
  - 标题：删除工作区
  - 正文：此操作将删除该工作区及其所有数据（包括测试集、优化任务等），且无法撤销。确定要删除“{workspaceName}”吗？
  - 按钮：取消 / 确认删除（Destructive）

#### 2) 删除当前工作区后的导航必须可预测

- 删除成功后不能停留在已删除的 `/workspaces/:id/...`。
- 建议落点策略（简单且可测试）：
  - 若仍有其他 workspace：进入 `/workspaces/:nextId/tasks`
  - 若没有：进入 `/workspace` 并提示“请先创建工作区”
- 特例：若用户当前就在 `/workspace`（工作区管理页），删除成功后仍停留在 `/workspace` 即可；但必须同步更新/清理 `lastWorkspaceIdByUser`（避免后续默认落入 404）。

#### 3) 缓存与本地持久化必须同步清理（防串扰）

- `useWorkspaceStore.lastWorkspaceIdByUser[currentUser.id]` 若指向被删 workspace，必须清除。
- 删除成功后，必须清理该 workspace 维度的 TanStack Query 缓存（至少 `testSets` 与 `optimizationTasks`）。
- 不得展示已删除 workspace 的旧缓存数据（尤其是在切换/删除后的短时间窗口）。

#### 4) 错误展示与安全约束

- UI 不得展示后端 `error.details`；仅展示 `error.message`（架构强约束）。
- 越权/不存在：后端对跨用户访问返回 404；前端按普通错误展示 message，并走兜底导航策略。
- 401/UNAUTHORIZED：沿用全局 unauthorized handler（会清空缓存并登出）。

### Architecture Compliance（必须对齐项目架构）

- 前端：沿用 “React Router 页面 + TanStack Query hooks + services + shadcn/ui”：
  - 页面：`frontend/src/pages/WorkspaceView/WorkspaceView.tsx`
  - workspaces hooks/services：`frontend/src/features/workspace/*`
  - 共享 store：`frontend/src/stores/useWorkspaceStore.ts`
- 后端：Axum 路由 + Repo 模式；删除必须按 `user_id` 作用域（见 `backend/src/api/routes/workspaces.rs` + `backend/src/infra/db/repositories/workspace_repo.rs`）。

### Library & Framework Requirements（不要顺手升级）

- 前端依赖（以仓库为准）：React `19.2.0`、React Router `7.0.0`、TanStack Query `5.x`、Zustand `5.x`（见 `frontend/package.json`）。
- 后端依赖（以仓库为准）：Axum `0.8`、SQLx `0.8`、Rust `1.85`（见 `backend/Cargo.toml`）。
- 本 Story 不做依赖升级；确认对话框按任务 1 约束复用既有 overlay 模式实现（不引入新的 Dialog/AlertDialog 依赖）。

### File Structure Requirements（落点清单）

- 前端（预期会改/加的文件）：
  - `frontend/src/pages/WorkspaceView/WorkspaceView.tsx`：增加删除入口 + 确认对话框 + 删除后导航
  - `frontend/src/features/workspace/hooks/useWorkspaces.ts`：复用 `useDeleteWorkspace`；必要时补充缓存清理辅助
  - `frontend/src/features/workspace/utils/workspaceRouting.ts`（可选新增）：抽取 workspaceId 解析 + 导航规则（与 `WorkspaceSelector.tsx` 保持一致，避免规则漂移）
  - `frontend/src/stores/useWorkspaceStore.ts`：复用 `clearLastWorkspaceId(userId)`
  - `frontend/src/App.routes.test.tsx` 或 `frontend/src/pages/WorkspaceView/WorkspaceView.test.tsx`：补充删除流程测试
- 后端（通常无需改代码，但建议补测试）：
  - `backend/tests/workspaces_api_test.rs`：补充删除 + 级联清理/404 的集成测试

### Testing Requirements（必须可回归验证）

- 前端：
  - 覆盖“确认/取消/错误展示/删除当前 workspace 后导航”的关键路径
  - 断言缓存清理效果：删除后不应继续渲染旧 workspace 的列表数据（可通过 query mocks 或 UI 文案断言）
- 后端：
  - 删除接口：只能删除当前用户的 workspace；跨用户应 404
  - 删除级联：删除 workspace 后，其 test_sets/optimization_tasks 不可再被读取（404 或列表为空）
  - 隔离：同一用户多 workspace 的资源必须按 `workspace_id` 隔离（已有 repo 级测试可复用/补齐）

### Previous Story Intelligence（继承既有约束，避免踩坑）

- Story 3.5 约束可直接复用：
  - `lastWorkspaceIdByUser` 需要随登出/401 清理；删除 workspace 时也要清理对应键，避免“默认落到 404”
  - workspace 切换/导航策略要固定，避免实现分歧（删除后落点同理）

### Git Intelligence Summary（最近变更的实现惯例）

- 最近一次与工作区相关的提交已建立惯例：前端缓存清理与登出/401 handler 协同（参考最近提交信息与 `frontend/src/App.tsx` 的处理）。

### Latest Technical Information（Web Research 摘要）

- TanStack Query：对“删除后不展示旧数据”，优先用 `removeQueries` 清除 workspace 维度缓存，并配合 `invalidateQueries(['workspaces'])` 刷新列表。
- 本仓库 UI primitives 现状：已存在 `Button/Card/Input/Label` 等，confirm dialog 采用既有 overlay 模式实现（见任务 1 的“写死”要求）。

### Project Structure Notes

- 架构文档中 `features/workspace-manager/` 被标记为 Phase 2 预留，但实际代码已落在 `frontend/src/features/workspace/`；本 Story 延续现有路径，不做目录迁移。

### References

- 需求来源：`docs/project-planning-artifacts/epics.md`（Story 3.6）
- PRD 工作区能力区域：`docs/project-planning-artifacts/prd.md`（能力区域 7: FR49/FR50）
- 架构约束（错误展示）：`docs/project-planning-artifacts/architecture.md`（前端不得展示 `error.details`）
- UX 规范：`docs/project-planning-artifacts/ux-design-specification.md`（破坏性操作需二次确认）
- 后端 workspaces 路由：`backend/src/api/routes/workspaces.rs`
- 后端 workspace repo：`backend/src/infra/db/repositories/workspace_repo.rs`
- SQLite 外键启用：`backend/src/infra/db/pool.rs`
- 数据库外键级联：`backend/migrations/003_create_test_sets.sql`、`backend/migrations/007_create_optimization_tasks.sql`
- 前端 workspaces hooks/services：`frontend/src/features/workspace/hooks/useWorkspaces.ts`、`frontend/src/features/workspace/services/workspaceService.ts`
- 工作区选择与 lastWorkspace/overlay modal 模式：`frontend/src/components/common/WorkspaceSelector.tsx`、`frontend/src/stores/useWorkspaceStore.ts`
- workspace-scoped query keys：`frontend/src/features/test-set-manager/hooks/useTestSets.ts`、`frontend/src/features/task-config/hooks/useOptimizationTasks.ts`、`frontend/src/features/test-set-manager/hooks/useTestSetTemplates.ts`

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

- 后端：`cargo fmt --all`、`cargo fmt --all -- --check`、`cargo clippy -- -D warnings`、`cargo test --all`
- 前端：`npm -C frontend run lint`、`npm -C frontend test -- --run`

### Completion Notes List

- 实现 Workspace View 的“删除工作区”入口 + destructive 确认对话框（overlay 模式，禁用重复提交）。
- 删除成功后：刷新工作区列表、清理 TanStack Query 的 workspace 维度缓存、必要时自动导航到可用 workspace（避免停留在已删除路由）。
- 抽取共享路由工具，保证 `WorkspaceSelector` 与删除后的导航规则一致，避免规则漂移。
- 新增/补齐前端 Vitest 用例与后端集成测试，覆盖删除/级联/隔离关键路径。

### File List

- `docs/implementation-artifacts/3-5-workspace-creation-and-switching.md`
- `docs/implementation-artifacts/3-6-workspace-deletion-and-data-isolation.md`
- `docs/implementation-artifacts/sprint-status.yaml`

- `frontend/src/pages/WorkspaceView/WorkspaceView.tsx`
- `frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx`
- `frontend/src/components/common/WorkspaceSelector.tsx`
- `frontend/src/features/workspace/utils/workspaceRouting.ts`
- `frontend/src/features/workspace/utils/workspaceRouting.test.ts`
- `frontend/src/App.routes.test.tsx`
- `backend/tests/workspaces_api_test.rs`
- `backend/src/infra/db/pool.rs`

## Change Log

- 2026-01-07：实现工作区删除确认 + 删除后导航与缓存清理；补齐前后端回归测试与级联/隔离验证；修复删除反馈/错误展示与测试可靠性（sqlite::memory）。

## Senior Developer Review (AI)

- 2026-01-07：完成对抗性审查并修复：sqlite::memory 测试连接池隐患、删除错误文案重复前缀、删除弹窗 error reset/ESC/a11y、删除当前工作区后的可见成功反馈、workspaceRouting 单测补齐。

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [ ] [CRITICAL] Story 原要求 `AlertDialog`，但仓库未内置 Dialog primitives，若不写死实现路径将导致引依赖分歧或 a11y/UX 偏离
- [ ] [HIGH] Story 指向可复用的 `getWorkspaceSwitchTargetPath`，但其当前为私有函数（未导出）；需要明确“复用方式”，避免导航规则漂移
- [ ] [HIGH] `useDeleteWorkspace` 现状仅刷新 workspaces 列表；Story 需要额外清理 workspace-scoped 缓存，必须明确由调用处负责并可测
- [ ] [MEDIUM] 缓存清理除 `testSets/optimizationTasks` 外，还存在 `testSetTemplates` 与 workspace detail 缓存，需要纳入最小清单

### Decisions

- [ ] 不新增 Dialog/AlertDialog 依赖：采用既有 overlay 模式实现确认对话框（降低依赖变更/回归风险）
- [ ] 导航放在调用处（WorkspaceView）实现：需要 `location.pathname` 且便于按“删除当前工作区”做条件分支

### Risks / Tech Debt

- [ ] 若导航规则未抽取共享 util：未来 `WorkspaceSelector` 与 `WorkspaceView` 可能出现规则漂移（触发条件：一处改动未同步另一处）

### Follow-ups

- [ ] 同步完成 `### Review Follow-ups (AI)` 中的 4 项（作为进入 `dev-story` 的前置自检）
