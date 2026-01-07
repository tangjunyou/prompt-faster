# Story 3.5: 工作区创建与切换

Status: review

Story Key: `3-5-workspace-creation-and-switching`

Related FRs: FR47（创建多个工作区）, FR48（切换工作区）

Epic: Epic 3（优化任务配置与工作区）

Dependencies: Epic 1（登录/认证与用户隔离）, Epic 2（测试集管理）, Story 3.1-3.4（工作区路由与任务/测试集能力已落地）

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 创建多个工作区并在它们之间切换,
so that 我可以隔离不同项目或实验的数据。

## Acceptance Criteria

### AC1：工作区选择器展示工作区列表与“新建工作区”（FR47/FR48）

**Given** 用户在系统中  
**When** 用户点击工作区选择器  
**Then** 显示当前工作区列表  
**And** 显示“新建工作区”选项

### AC2：创建工作区后自动切换（FR47）

**Given** 用户点击“新建工作区”  
**When** 用户输入工作区名称  
**Then** 创建新工作区  
**And** 自动切换到新工作区

### AC3：切换工作区后展示该工作区数据（FR48）

**Given** 用户在工作区列表中  
**When** 用户点击另一个工作区  
**Then** 切换到该工作区  
**And** 界面显示该工作区的数据（任务、测试集等）

### AC4：切换过程平滑（FR48）

**Given** 用户切换工作区  
**When** 系统加载该工作区数据  
**Then** 切换过程平滑，无明显延迟

## Tasks / Subtasks

### 任务 1：全局工作区选择器（Header）+ 当前工作区解析（AC1/AC3）

- [x] 在 Header 中加入“工作区选择器”（下拉/弹层均可），支持：
  - [x] 仅在已登录（`authStatus === 'authenticated'`）时渲染；未登录不显示
  - [x] 展示当前工作区名称（未选择时显示占位）
  - [x] 点击后拉取并展示工作区列表（`GET /api/v1/workspaces`）
  - [x] 列表项点击后切换工作区（路由导航到新的 `workspaceId`）
- [x] 当前工作区的来源与优先级（必须写死，避免实现分歧）：
  - [x] 若当前 URL 命中 `/workspaces/:id/...` → `id` 即当前工作区
  - [x] 否则若存在本地记录的 `lastWorkspaceId`（仅存 workspaceId；不存 token）：
    - [x] `lastWorkspaceId` 必须与 `currentUser.id` 绑定（或登出时清空），避免跨用户串扰
    - [x] 仅当该 id 仍存在于 workspaces 列表中时才使用；否则忽略并走后续兜底
  - [x] 否则若 workspaces 列表非空 → 选择第一个（按后端返回顺序，当前为 created_at DESC）
  - [x] 否则（列表为空）→ 引导创建工作区
- [x] 切换工作区的导航策略（必须写死）：
  - [x] 若当前在 `/workspaces/:id/<section>`（`test-sets` / `tasks` / `tasks/:taskId`）→ 保持 `<section>`，仅替换 `:id`
  - [x] 若当前 section 不在上述列表 → 切换后进入 `/workspaces/:id/tasks`（统一兜底）
  - [x] 若当前不在 workspace-scoped 路由 → 切换后进入 `/workspaces/:id/tasks`

### 任务 2：新建工作区（AC1/AC2）

- [x] 在选择器中提供“新建工作区”入口（推荐 Dialog）：
  - [x] 字段：`name`（必填）、`description`（可选）
  - [x] 前端本地校验：`name.trim()` 非空、长度 ≤ 128（与后端一致）
  - [x] 调用 `POST /api/v1/workspaces` 创建成功后：
    - [x] 关闭 Dialog
    - [x] 将新 workspace 作为当前工作区
    - [x] 自动导航到新 workspace（按任务 1 的导航策略）
  - [x] 创建失败：展示 `error.message`（不得展示 `error.details`）

### 任务 3：切换体验平滑化（AC4）

- [x] 切换前预取关键数据，减少“空白/抖动”：
  - [x] 预取 `GET /api/v1/workspaces/:workspaceId/optimization-tasks`
  - [x] 预取 `GET /api/v1/workspaces/:workspaceId/test-sets`
  - [x] 预取必须复用既有 TanStack Query queryKey / hooks（不要新造一套缓存维度）：
    - [x] `['optimizationTasks', workspaceId]`（见 `frontend/src/features/task-config/hooks/useOptimizationTasks.ts`）
    - [x] `['testSets', workspaceId]`（见 `frontend/src/features/test-set-manager/hooks/useTestSets.ts`）
  - [x] 实现方式：使用 `useQueryClient()` + `queryClient.prefetchQuery(...)`；预取不 await（fire-and-forget）
  - [x] 预取失败不阻塞切换（静默失败即可）
- [x] 统一加载态策略（必须明确）：
  - [x] 切换瞬间不清空旧列表（避免闪烁），直到新 workspace 数据返回后再刷新 UI
  - [x] 若新 workspace 数据加载中，显示轻量 loading 状态（如 skeleton/“加载中...”）

### 任务 4：回归测试（AC1-AC4）

- [x] 前端测试（Vitest + Testing Library）覆盖：
  - [x] 认证态下 Header 出现工作区选择器
  - [x] 选择工作区后路由跳转（保持 section 的规则）
  - [x] 创建工作区成功后自动切换（以服务端返回 id 为准）
  - [x] 切换时不会显示其他工作区的任务/测试集数据（queryKey 必须包含 workspaceId）
  - [x] workspaces 列表为空：展示创建引导；创建成功后自动导航到 `/workspaces/:id/tasks`
  - [x] `lastWorkspaceId` 无效/过期（不在列表或后端 404）：忽略并回退到“第一个 workspace / 引导创建”

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 已将 review 结论沉淀到 `## Review Notes`（含风险/遗留）
- [ ] [AI-Review][LOW] WorkspaceSelector：将自绘“新建工作区” Dialog 替换为 `shadcn/ui` Dialog（focus trap / ESC / aria）

## Dev Notes

### Developer Context（避免实现偏航）

- Epic 3 目标：用户可创建优化任务、配置算法参数，并管理多个工作区（工作区用于隔离项目/实验数据）。  
- Epic 3 stories 速览（用于 cross-story 依赖与避免重复实现）：
  - 3.1 任务创建（基础配置）
  - 3.2 初始 Prompt + 终止条件
  - 3.3 候选生成 + 多样性注入阈值
  - 3.4 核心算法参数 + 默认值/重置（高级配置 UI）
  - 3.5（本 Story）工作区创建与切换（全局选择器 + 路由级切换）
  - 3.6 工作区删除 + 数据隔离验证（后续）
  - 3.7 多老师模型切换（后续）
- 本 Story 的定位：在不改动后端数据模型/路由结构的前提下，把“工作区创建与切换”的交互补齐到 **全局可用**（Header 级别），让用户在任何工作区相关页面都能快速切换。  
- 明确非目标（防 scope creep）：
  - 不做工作区删除（FR49/Story 3.6）
  - 不改“数据隔离”的实现方式（FR50：后端已按 `user_id` + `workspace_id` 做隔离；本 Story 只做 UI 切换与正确加载）
  - 不做 UI 大重构（例如把三视图合并成“同一画布不同布局”的架构重写，属于后续 UX/架构演进）

### Hard Prerequisites（硬前置：必须已存在/不得改语义）

- 后端已存在并可用的接口（需登录）：
  - `GET /api/v1/workspaces`：列出当前用户工作区
  - `POST /api/v1/workspaces`：创建工作区（name 非空、≤128）
  - `GET /api/v1/workspaces/{id}`：读取工作区详情
  - 以及 workspace-scoped 资源：`/api/v1/workspaces/{workspace_id}/test-sets`、`/optimization-tasks` 等
- 数据库与隔离基础（本 Story 不改 Schema，但开发者必须知晓以避免“越权/串数据”）：
  - `workspaces` 表字段包含 `user_id`（见 `backend/migrations/001_initial_schema.sql`）
  - 后端查询以 `user_id` 过滤，且对跨用户访问返回 404（见 `backend/tests/workspaces_api_test.rs`）
- 前端认证约束：`sessionToken` **仅存内存**，不得落地到 localStorage/sessionStorage（见 `frontend/src/stores/useAuthStore.ts`）。

### Technical Requirements（DEV Agent Guardrails）

#### 1) “当前工作区”的单一事实来源

- 对数据加载的 workspaceId：以 **URL** 为准（`/workspaces/:id/...`）。不要引入“URL 与 Store 不一致”的双源状态。
- 允许存储 `lastWorkspaceId`（仅 workspaceId，非敏感）用于“未在 workspace-scoped 路由时的默认选择”，但：
  - `lastWorkspaceId` 必须与 `currentUser.id` 绑定（或登出时清空）
  - 若 `lastWorkspaceId` 不存在/不在列表/后端返回 404 → 视为无效，回退到“第一个 workspace / 引导创建”
  - 一旦进入 `/workspaces/:id/...`，必须以 URL 覆盖。

#### 2) 切换行为必须可预测（不要让用户迷路）

- 切换时尽量保持用户所在 section（`test-sets` / `tasks` / `tasks/:taskId`），只替换 workspaceId。
- 若当前不在 workspace-scoped 路由，切换后进入 `/workspaces/:id/tasks`（把“切换后的落点”固定下来，避免实现分歧）。

#### 3) 安全与错误展示

- UI 不得展示后端 `error.details`（架构强约束）；仅展示 `error.message`。
- 发生 `UNAUTHORIZED`：沿用现有全局 unauthorized handler（`registerUnauthorizedHandler`）处理登出/跳转。

#### 4) 输入校验必须对齐后端

- 工作区名称：`trim()` 后非空；字符数 ≤ 128（后端同约束）。
- 失败提示：必须可见、可理解、且不泄露内部信息。

#### 5) “切换平滑”的最低实现门槛

- 切换前预取（prefetch）该 workspace 的关键列表数据（任务/测试集）。
- 切换时不要瞬间把旧列表清空成空白；要么保留旧内容并显示 loading（推荐），要么展示 skeleton（避免闪烁）。

### Architecture Compliance（必须对齐项目架构）

- 前端：沿用 “React Router 页面 + TanStack Query hooks + services” 的既有结构：
  - workspaces CRUD：`frontend/src/features/workspace/*`
  - 业务页面：`frontend/src/pages/*`
  - 公共 header/切换器：`frontend/src/components/common/*`
- 后端：Axum 路由 + Repo 模式；API 响应必须为 `ApiResponse<T>`（不新增返回格式/不破坏 OpenAPI）。

### Library & Framework Requirements（不要用错版本/不要顺手升级）

- 前端依赖（以仓库为准）：React `19.2.0`、React Router `7.0.0`、TanStack Query `5.x`、Zustand `5.x`（见 `frontend/package.json`）。
- 后端依赖（以仓库为准）：Axum `0.8`、SQLx `0.8`、Rust `1.85`（见 `backend/Cargo.toml`）。
- 本 Story 不做依赖升级；若需要确认 API 用法，优先查官方文档（尤其是 React Router v7 / TanStack Query v5）。

### File Structure Requirements（落点清单）

- 前端（预期会改/加的文件）：
  - `frontend/src/App.tsx`：Header 中挂载工作区选择器（仅在已登录时显示）
  - `frontend/src/components/common/WorkspaceSelector.tsx`（新增）：工作区选择器 UI（列表 + 新建）
  - `frontend/src/features/workspace/hooks/useWorkspaces.ts`：复用既有 hooks（必要时暴露 queryKey/预取辅助函数）
  - `frontend/src/stores/useWorkspaceStore.ts`（可选新增）：存储 `lastWorkspaceId`（仅 workspaceId）
  - `frontend/src/App.routes.test.tsx`（扩展）：覆盖 selector 出现/切换/创建后的导航

### Testing Requirements（必须可回归验证）

- 前端单测（Vitest + Testing Library）：
  - workspaces list/create 的 API mock：复用既有 MSW 模式（参考 `frontend/src/features/workspace/services/workspaceService.test.ts`）
  - 路由跳转断言：使用 `MemoryRouter` 并断言目标页面 testid（或 location）
  - 关键断言：切换后 queryKey 维度必须包含 workspaceId，避免“串 workspace 数据”

### Previous Story Intelligence（继承既有约束，避免踩坑）

- Story 3.4/3.3 的共识可复用点：
  - UI 状态以服务端回显为准，不做“看起来成功但未持久化”的假象（创建工作区同理：以返回的 `id` 为准再切换）
  - TanStack Query 的 queryKey 必须显式包含 workspaceId（已有模式：`['testSets', workspaceId]` / `['optimizationTasks', workspaceId]`）
  - 不做依赖升级（尤其避免引入“顺手升级导致的破坏性变更”）

### Git Intelligence Summary（最近变更的实现惯例）

- 最近一次提交完成 Story 3.4，沿用：
  - “Story 文件驱动开发 + 产物入库（含生成 types）”
  - “后端集成测试 + 前端单测同提”

### Latest Technical Information（Web Research 摘要）

- React Router v7：本项目使用 `react-router` 包直接提供 `Link/NavLink/useNavigate`（与当前代码一致）。  
- TanStack Query v5：`invalidateQueries`/`prefetchQuery` 是实现“切换平滑”的推荐手段（本 Story 预取策略使用它们即可）。  
- Zustand：若做 `lastWorkspaceId` 记忆，可用官方 `persist` 中间件，但不得持久化任何 token。

### Project Structure Notes

- 现状与架构文档的轻微偏差：架构文档提到 `features/workspace-manager/` 为预留模块，但当前代码已使用 `frontend/src/features/workspace/`。本 Story 延续既有路径，不做目录迁移（避免无收益重构）。
- UX 设计文档建议 “layoutMode 控制同一画布的不同布局”，但当前实现为多路由多页面；本 Story 仅实现工作区选择与切换，不推动视图架构重写。

### References

- 需求来源：`docs/project-planning-artifacts/epics.md`（Story 3.5）
- PRD 工作区能力区域：`docs/project-planning-artifacts/prd.md`（能力区域 7: FR47/FR48）
- 架构组件映射与约束：`docs/project-planning-artifacts/architecture.md`（工作区管理映射、前端不得展示 error.details）
- UX 视图切换与 Workspace View：`docs/project-planning-artifacts/ux-design-specification.md`（Workspace View、快捷键）
- 项目索引（当前未发现 `**/project-context.md`）：`docs/implementation-artifacts/index.md`
- 现有前端入口与路由：`frontend/src/App.tsx`
- 现有 Workspace 页面与 hooks：`frontend/src/pages/WorkspaceView/WorkspaceView.tsx`、`frontend/src/features/workspace/hooks/useWorkspaces.ts`
- 现有后端 workspaces 路由：`backend/src/api/routes/workspaces.rs`、`backend/tests/workspaces_api_test.rs`

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

### Completion Notes List

- 实现 Header 工作区选择器（列表/切换/新建），仅在已登录时显示
- 加入 `lastWorkspaceId` 记忆（按 `currentUser.id` 分桶持久化，仅存 workspaceId）
- 切换前预取 tasks/test-sets，并通过 `keepPreviousData` + 轻量 loading 平滑切换
- 回归：`npm --prefix frontend test -- --run`、`npm --prefix frontend run lint`、`cargo test`

### File List

- `docs/implementation-artifacts/3-5-workspace-creation-and-switching.md`
- `docs/implementation-artifacts/sprint-status.yaml`
- `frontend/src/App.tsx`
- `frontend/src/App.routes.test.tsx`
- `frontend/src/components/common/WorkspaceSelector.tsx`
- `frontend/src/features/task-config/hooks/useOptimizationTasks.ts`
- `frontend/src/features/test-set-manager/hooks/useTestSets.ts`
- `frontend/src/features/workspace/hooks/useWorkspaces.ts`
- `frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx`
- `frontend/src/pages/TestSetsView/TestSetsView.tsx`
- `frontend/src/stores/useWorkspaceStore.ts`

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] 登出/401 时未清理 TanStack Query 缓存，存在跨用户短时间看到旧缓存数据的风险（已修复：登出与 401 handler 统一 `queryClient.clear()`）。
- [x] [MEDIUM] 登出时未清理本地持久化的 `lastWorkspaceIdByUser`（隐私残留/设备共享场景不理想）（已修复：`useAuthStore.logout()` 重置 `useWorkspaceStore`）。
- [x] [LOW] `WorkspaceSelector` 里预取（prefetch）手写 queryKey，未来 hooks 变更时可能漂移（已修复：改为复用 hooks 导出的 queryOptions）。
- [ ] [LOW] 自绘 Dialog 的无障碍与键盘体验仍可优化（focus trap / ESC 关闭等），但不属于本 Story 的硬性验收范围。

### Decisions

- [x] 不扩展“通用 section 保持”路由匹配：Story 已明确写死只保持 `test-sets/tasks(/:taskId)`，其他一律兜底到 `/tasks`，避免实现分歧与 scope creep。
- [x] 不在前端做“workspaceId 属于当前用户”的额外校验：以 URL 为单一事实来源，越权由后端拦截；前端额外校验会引入不必要的请求与复杂度。

### Risks / Tech Debt

- [x] 若后续引入更多 workspace-scoped 路由，需要在 Story 级别明确“切换后保持哪些 section”，再扩展 `getWorkspaceSwitchTargetPath`，避免隐式行为变化。
- [x] 若要提升无障碍体验，建议用 `shadcn/ui` 的 Dialog/Popover 替换自绘结构（独立小改动即可）。

### Follow-ups

- [ ] [AI-Review][LOW] WorkspaceSelector：将自绘“新建工作区” Dialog 替换为 `shadcn/ui` Dialog（focus trap / ESC / aria 体验更好）
