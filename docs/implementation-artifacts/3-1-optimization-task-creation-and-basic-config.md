# Story 3.1: 优化任务创建与基本配置

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 创建一个优化任务并完成最小可用的基础配置（执行目标/任务模式/优化目标/关联测试集）,
so that 我可以为后续“启动优化（Run View）”准备好可持久化、可恢复、可复用的任务上下文。

## Acceptance Criteria

### AC1：任务创建向导入口与基础信息（FR16）

**Given** 用户在某个工作区内（Workspace boundary）  
**When** 用户点击“新建优化任务”  
**Then** 打开任务创建向导  
**And** 用户可以填写：
- 任务名称（必填）
- 任务描述（可选）
- 优化目标（必填，FR19）

**And** 提交成功后，任务被持久化（刷新页面/重启应用后仍可见）。

### AC2：执行目标类型选择（FR17）

**Given** 用户在任务创建向导中  
**When** 用户选择“执行目标”  
**Then** 支持在以下两类之间选择：
- Dify 工作流
- 通用 API（直连模型）

**And** UI 根据选择展示对应的配置占位/说明（本 Story 只要求“可选择 + 可持久化”，细节字段可在后续 Story 3.2/3.3/3.4 补齐）。

### AC3：任务模式选择（FR18）

**Given** 用户在任务创建向导中  
**When** 用户选择“任务模式”  
**Then** 支持：
- 固定任务（Fixed）：以 `reference.Exact` / `reference.Hybrid` 为主（有“标准答案”语义）
- 创意任务（Creative）：以 `reference.Constrained` / `reference.Hybrid` 为主（无标准答案，靠约束/维度评估）

**And（后端强校验）** 当用户提交创建请求时：
- Fixed 模式：所关联测试集内 **不得** 存在 `TaskReference::Constrained`（否则 400）
- Creative 模式：所关联测试集内 **不得** 存在 `TaskReference::Exact`（否则 400）
- `Hybrid` 允许同时用于两类模式（校验按“变体类型”判断；评估语义在 Story 3.4 明确）

> 目的：把“模式语义”变成可验收的硬约束，避免后续执行阶段出现“模式与测试集不匹配”的隐性灾难。

### AC4：关联测试集（FR21）

**Given** 用户在任务创建向导中  
**When** 用户选择“关联测试集”  
**Then** 展示当前工作区可用测试集列表（来自 Epic 2）  
**And** 支持选择 1..N 个测试集  
**And** 提交后任务与测试集的关联被持久化并可回显。

### AC5：任务列表可见（FR16）

**Given** 用户已创建至少一个任务  
**When** 用户进入“任务管理/任务列表”页面  
**Then** 能看到任务卡片/列表项，至少包含：名称、优化目标摘要、执行目标类型、任务模式、创建时间、最近更新时间。

## Tasks / Subtasks

### 任务 1：后端数据模型与持久化（AC1/AC2/AC3/AC4）

- [x] 新增数据库迁移（SQLite / SQLx migrations）：
  - `optimization_tasks`（snake_case, 复数）
  - `optimization_task_test_sets`（任务-测试集关联表；支持 1..N）
- [x] 约束与索引（防重复与性能）：
  - `optimization_task_test_sets`：`UNIQUE(optimization_task_id, test_set_id)`
  - 索引建议：`idx_optimization_tasks_workspace_id`、`idx_optimization_task_test_sets_optimization_task_id`
- [x] 设计最小可用字段（MVP 可落地，向后兼容）：
  - `id` TEXT PK
  - `workspace_id` TEXT FK → `workspaces(id)`（**必须**以 workspace 作为第一隔离边界）
  - `name` TEXT NOT NULL（<= 128 字符）
  - `description` TEXT NULL
  - `goal` TEXT NOT NULL（FR19）
  - `execution_target_type` TEXT NOT NULL（`dify` | `generic`）
  - `task_mode` TEXT NOT NULL（`fixed` | `creative`）
  - `status` TEXT NOT NULL（建议：`draft` 作为创建态默认值）
  - `config_json` TEXT NULL（预留给后续 Story 3.2/3.4 的高级配置；本 Story 允许为空）
  - `created_at/updated_at` INTEGER (Unix ms)
- [x] Repository 层：实现“按 workspace + user 作用域”的 CRUD（至少 create/list/get）
  - 参照：`backend/src/infra/db/repositories/workspace_repo.rs`、`backend/src/infra/db/repositories/test_set_repo.rs`

### 任务 2：后端 API（AC1/AC2/AC3/AC4/AC5）

- [x] 新增 routes：`/api/v1/workspaces/{workspace_id}/optimization-tasks`
  - `POST` 创建任务（包含 test_set_ids 数组）
  - `GET` 列出任务（分页可延后；按 `updated_at DESC`）
  - （可选）`GET /{task_id}` 获取详情（便于后续继续配置）
- [x] API 契约：
  - 全部返回 `ApiResponse<T>`（data/error 互斥）
  - 401/403/404/400 错误码沿用现有风格（例如 `WORKSPACE_NOT_FOUND`，新增 `OPTIMIZATION_TASK_NOT_FOUND` 可选）
  - 不得把 `error.details` 直接展示给用户（架构约束）
- [x] 复用鉴权与隔离：
  - 通过 workspace→user_id 作用域校验，确保跨用户无法访问他人任务
- [x] 后端强校验实现 AC3 模式-测试集一致性：
  - create 时读取所有关联测试集 `cases_json` 并检查 `TaskReference` 变体

### 任务 3：前端任务创建向导 + 列表（AC1/AC2/AC4/AC5）

- [x] 路由与入口（建议）：
  - 在 `frontend/src/pages/WorkspaceView/WorkspaceView.tsx` 的工作区卡片中新增“管理任务”入口：`/workspaces/:id/tasks`
  - 新增页面：任务列表 + “新建任务”按钮
- [x] 创建向导（MVP）：
  - 表单字段：name / description / goal / execution_target_type / task_mode / test_set_ids[]
  - 测试集列表复用：`frontend/src/features/test-set-manager/hooks/useTestSets.ts`
  - 错误展示：只显示 `error.message`（不得展示 `error.details`）
- [x] API 调用层（参照 workspace/test-set-manager 的 pattern）：
  - `frontend/src/features/task-config/services/optimizationTaskService.ts`
  - `frontend/src/features/task-config/hooks/useOptimizationTasks.ts`

### 任务 4：测试与回归保护（AC 全覆盖）

- [x] Backend integration tests：`backend/tests/optimization_tasks_api_test.rs`
  - 未登录 → 401
  - workspace 越权 → 404/403（与现有 workspace/test_sets 行为保持一致）
  - create 成功 + list 可见
  - test_set_id 不存在 → 404（错误码沿用现有资源风格）
  - 模式校验：Fixed 关联 Constrained → 400；Creative 关联 Exact → 400
  - 1..N 关联回显：创建时指定多个 `test_set_ids`，查询时正确回显
- [x] Frontend tests（Vitest）：
  - 创建向导：必填校验 + 提交成功后列表刷新
  - 模式校验错误的用户提示（来自后端 message）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 将本 Story 的 review 结论沉淀到 `## Review Notes`（含风险/遗留）

## Dev Notes

### Developer Context（避免实现偏航）

- 现状：项目已完成 Epic 1（认证/凭证）、Epic 2（测试集管理），但“优化任务”仅在 `backend/src/domain/models/algorithm.rs` 里有占位 DTO，前后端尚无任务 API 与 UI。
- 重要：**不要直接复用** `backend/src/domain/models/algorithm.rs#OptimizationTask`（当前为 `user_id` 语义占位）。本 Story 以 `workspace_id` 作为第一隔离边界落库与对外契约，应新建/迁移到以 workspace 为边界的实体/DTO。
- 本 Story 目标：把“优化任务”作为一等实体落库，并提供创建向导 + 列表（配置阶段），为后续 Story 3.2+（初始 Prompt/终止条件/高级配置）与 Epic 4（自动迭代执行）提供稳定落点。
- 明确非目标（防 scope creep）：
  - 不实现执行引擎/RunView 真实运行（Epic 4）
  - 不实现 teacher model 选择/参数（Story 3.7/已有全局配置）
  - 不实现高级算法参数（Story 3.4）

### Architecture / UX Guardrails（必须遵守）

- API 响应：必须使用 `ApiResponse<T>`，禁止返回裸 JSON（见 `backend/src/api/response.rs`）。
- 错误展示：前端不得直接展示 `error.details`（见 `docs/project-planning-artifacts/architecture.md` 的 Error Handling 约束）。
- Workspace 是第一隔离边界：所有任务必须带 `workspace_id`，并通过 workspace→user_id 进行权限校验（沿用现有 workspaces/test_sets 逻辑）。
- 命名与结构：
  - Rust：snake_case；TS：camelCase（变量/函数/组件命名）。
  - API DTO 字段命名：**snake_case**（对齐现有 `workspaces`/`test-sets` 与 ts-rs 生成类型），禁止新增“同一资源同时支持 camelCase + snake_case”的双口径。
  - 测试位置：后端 `backend/tests/`；前端 `*.test.ts(x)`（见 `docs/project-planning-artifacts/architecture.md`）。

### 关键设计决策（提前写死，减少歧义）

- 任务-测试集关系：必须支持 1..N（PRD/UX 明确“可选择一个或多个测试集”），推荐用关联表而非 JSON 字段，避免后续查询/约束困难。
- 任务模式（Fixed/Creative）是“可验收语义”，必须在创建时做后端强校验（见 AC3），避免后续执行阶段才爆炸。
- 端点命名一致性：
  - 架构文档建议 snake_case（`/api/v1/optimization_tasks`），但现有实现已采用 kebab-case + workspace 嵌套（`/api/v1/workspaces/:id/test-sets`）。
  - 本 Story 以“与现有代码一致”为最高优先：采用 `/api/v1/workspaces/{workspace_id}/optimization-tasks`（kebab-case）。

### Technical Requirements（后端契约 + DTO）

**1) API 端点（建议最小集）**

- `POST /api/v1/workspaces/{workspace_id}/optimization-tasks`
- `GET /api/v1/workspaces/{workspace_id}/optimization-tasks`
- （可选）`GET /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}`

**2) Create 请求体（MVP）**

```json
{
  "name": "我的第一个优化任务",
  "description": "可选",
  "goal": "让 system prompt 更简洁、稳定，并提高测试通过率",
  "execution_target_type": "dify",
  "task_mode": "fixed",
  "test_set_ids": ["<test_set_id_1>", "<test_set_id_2>"]
}
```

**3) 返回字段（MVP）**

- `id`, `workspace_id`, `name`, `description`, `goal`
- `execution_target_type`, `task_mode`, `status`
- `test_set_ids`（便于前端回显；后端内部可来自关联表）
- `created_at`, `updated_at`（Unix ms）

**4) 校验规则（必须实现）**

- `name.trim().isEmpty()` → 400（与 `workspaces.rs`/`test_sets.rs` 风格一致）
- `name` 长度上限：建议 128
- `goal.trim().isEmpty()` → 400
- `test_set_ids` 至少 1 个；ID 需属于同一 workspace
- `execution_target_type` 仅允许：`dify` | `generic`
- `task_mode` 仅允许：`fixed` | `creative`
- AC3 模式一致性校验：基于 test_sets 的 `cases_json` 扫描 `TaskReference` 变体

**5) 错误码（建议）**

- 复用：`UNAUTHORIZED` / `VALIDATION_ERROR` / `DATABASE_ERROR` / `WORKSPACE_NOT_FOUND` / `TEST_SET_NOT_FOUND`
- 可新增：`OPTIMIZATION_TASK_NOT_FOUND`（与现有资源错误码命名一致）

**6) OpenAPI / TS Types**

- 更新 `backend/src/api/routes/docs.rs` 的 `paths(...)` 与 `components(schemas(...))`
- 变更后运行类型生成：`cd backend && cargo run --bin gen-types`

### Library / Framework Requirements（版本与依赖边界）

- 后端：Rust `edition = 2024`（见 `backend/Cargo.toml`）；Axum / SQLx / Utoipa 已存在，禁止为本 Story 引入新 web/db 框架。
- 前端：React + React Router + TanStack Query 已存在（见 `frontend/package.json`）；UI 继续使用 shadcn/ui 组件与 Tailwind，不新增重量级表单库完成 MVP。
- 本 Story 不做依赖升级；以仓库锁定版本为准（避免“升级导致全项目破坏”）。

### File Structure Requirements（建议改动清单）

**Backend**

- `backend/migrations/007_create_optimization_tasks.sql`（新建 tasks 表与关联表）
- `backend/src/infra/db/repositories/optimization_task_repo.rs`（或 `task_repo.rs`，保持命名一致即可）
- `backend/src/api/routes/optimization_tasks.rs`（或 `tasks.rs`；建议与路由 path 同名，减少迷路）
- `backend/src/api/routes/mod.rs`（导出新模块）
- `backend/src/api/routes/workspaces.rs`（在 workspace router 下 `nest("/{workspace_id}/optimization-tasks", ...)`）
- `backend/src/api/routes/docs.rs`（OpenAPI 注册）
- `backend/src/shared/error_codes.rs`（如新增错误码）
- `backend/src/bin/gen-types.rs`（如新增请求/响应 DTO，需要 export）
- `backend/tests/optimization_tasks_api_test.rs`

**Frontend**

- `frontend/src/features/task-config/services/optimizationTaskService.ts`
- `frontend/src/features/task-config/hooks/useOptimizationTasks.ts`
- `frontend/src/pages/TasksView/TasksView.tsx`（或复用现有 pages 组织方式）
- `frontend/src/pages/index.ts`（导出新页面）
- `frontend/src/App.tsx`（新增路由：`/workspaces/:id/tasks`；并用 `ProtectedRoute`）
- `frontend/src/pages/WorkspaceView/WorkspaceView.tsx`（在 workspace 卡片新增入口按钮）
- `frontend/src/pages/TasksView/TasksView.test.tsx`（最小交互测试）

### Testing Requirements（与 CI 门禁对齐）

- Backend：`cd backend && cargo test`
- Frontend：
  - `cd frontend && npm run lint`
  - `cd frontend && npm test -- --run`
  - `cd frontend && npm run build`

### Git Intelligence Summary（最近工程约定）

- 最近与“工作区/测试集”相关提交可作为实现范式参考：
  - `c581863`（测试集相关边界修复 + retrospective）
  - `a0cadfa`（Story 2.6 review fixes：校验与 UX 边界）
  - `181610c`（Story 2.5：generic config + 标准答案）
  - `63295f8`（Story 2.4：Dify config）

### Project Context Reference

- 未发现 `project-context.md`（本仓库以 `docs/project-planning-artifacts/*.md` 为主要真相源）。

### Story Completion Status（执行完成后的更新要求）

- 本 Story 完成并 code-review 通过后：
  - 将 `docs/implementation-artifacts/sprint-status.yaml` 中 `3-1-optimization-task-creation-and-basic-config` 更新为 `review`（提交 code review 时）→ `done`（合入/验收完成后）
  - 确保 OpenAPI 与前端生成类型已更新（避免前后端 DTO 漂移）

### Project Structure Notes

- 建议新增模块遵循现有落点（避免“写到架构文档路径但项目实际没这么组织”）：
  - 后端 routes：`backend/src/api/routes/`（参照 `workspaces.rs` / `test_sets.rs`）
  - 后端 repo：`backend/src/infra/db/repositories/`（参照 `workspace_repo.rs` / `test_set_repo.rs`）
  - 前端 feature：`frontend/src/features/` 下新增 `task-config/`（与 `workspace/`、`test-set-manager/` 并列）
  - 前端 page：`frontend/src/pages/` 下新增任务页（或在 `WorkspaceView` 内嵌也可，但要保证可测试）
- 与文档的偏差记录（必须在实现时处理/同步）：
  - `docs/project-planning-artifacts/architecture.md` 中对 routes 文件名（`tasks.rs`）与端点样式（snake_case）是“理想态”；当前仓库已经形成了 kebab-case + workspace 嵌套路由的事实标准，应以仓库事实为准，并在必要时补文档。

### References

- [Source: docs/project-planning-artifacts/epics.md#Epic-3-优化任务配置与工作区（Story-3.1）]（FR16/FR17/FR18/FR19/FR21）
- [Source: docs/project-planning-artifacts/prd.md#7.3.1-页面结构与配置逻辑]（新建任务流程、任务列表/任务配置分层）
- [Source: docs/project-planning-artifacts/ux-design-specification.md#Journey-1-首次优化]（配置→Run View 的用户旅程）
- [Source: docs/project-planning-artifacts/architecture.md#Implementation-Patterns--Consistency-Rules]（命名/ApiResponse/测试位置/错误展示约束）
- [Source: backend/src/api/routes/workspaces.rs]（受保护路由 + repo 调用 + 统一错误响应的实现范式）
- [Source: backend/src/api/routes/test_sets.rs]（workspace 边界、作用域校验、DTO/ts-rs 导出范式）
- [Source: backend/src/domain/models/algorithm.rs#OptimizationTask]（当前占位 DTO：后续实现需要迁移/重构）
- [Source: frontend/src/pages/WorkspaceView/WorkspaceView.tsx]（页面 + TanStack Query pattern）
- [Source: frontend/src/features/test-set-manager/hooks/useTestSets.ts]（以 workspace 为边界的 query key 组织方式）

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Implementation Plan

1. 后端：新增 migrations（optimization_tasks + optimization_task_test_sets），实现 repo（create/list/get，user+workspace 作用域）。
2. 后端：新增 optimization-tasks routes（POST/GET/GET by id），实现校验（name/goal/test_set_ids + AC3 模式一致性）。
3. OpenAPI + ts-rs：补齐 docs.rs paths/schemas，运行 `cargo run --bin gen-types` 生成前端类型。
4. 前端：新增 `/workspaces/:id/tasks` 页面 + “新建任务”向导（MVP 字段），实现列表展示与错误提示（仅 message）。
5. 测试：后端 integration tests 覆盖 AC3/越权/1..N 回显；前端 Vitest 覆盖必填与创建后刷新。

### Debug Log References

- Backend：
  - `cd backend && cargo test`
  - `cd backend && cargo fmt --check`
  - `cd backend && cargo clippy -- -D warnings`
  - `cd backend && cargo run --bin gen-types`
- Frontend：
  - `cd frontend && npm run lint`
  - `cd frontend && npm test -- --run`
  - `cd frontend && npm run build`

### Completion Notes List

1. 数据库：新增 `optimization_tasks` + `optimization_task_test_sets`（workspace 隔离、1..N 关联、唯一约束与索引）
2. 后端：新增 `/api/v1/workspaces/{workspace_id}/optimization-tasks`（POST/GET/GET by id），并实现 AC3 模式-测试集强校验
3. 前端：新增 `/workspaces/:id/tasks` 页面（创建向导 + 列表），并在 Workspace 卡片加入“管理任务”入口
4. 类型：更新 OpenAPI + `gen-types` 导出，生成前端 ts-rs 类型（避免 DTO 漂移）
5. 测试：新增后端 integration tests + 前端 Vitest 覆盖必填/创建刷新/模式校验提示

### File List

Backend:
- backend/migrations/007_create_optimization_tasks.sql
- backend/src/api/routes/docs.rs
- backend/src/api/routes/mod.rs
- backend/src/api/routes/optimization_tasks.rs
- backend/src/api/routes/workspaces.rs
- backend/src/bin/gen-types.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/models/optimization_task.rs
- backend/src/infra/db/repositories/mod.rs
- backend/src/infra/db/repositories/optimization_task_repo.rs
- backend/src/shared/error_codes.rs
- backend/tests/optimization_tasks_api_test.rs

Frontend:
- frontend/src/App.tsx
- frontend/src/features/task-config/hooks/useOptimizationTasks.ts
- frontend/src/features/task-config/services/optimizationTaskService.ts
- frontend/src/pages/OptimizationTasksView/OptimizationTasksView.test.tsx
- frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx
- frontend/src/pages/OptimizationTasksView/index.ts
- frontend/src/pages/WorkspaceView/WorkspaceView.tsx
- frontend/src/pages/index.ts
- frontend/src/types/generated/api/CreateOptimizationTaskRequest.ts
- frontend/src/types/generated/api/OptimizationTaskListItemResponse.ts
- frontend/src/types/generated/api/OptimizationTaskResponse.ts
- frontend/src/types/generated/models/ExecutionTargetType.ts
- frontend/src/types/generated/models/OptimizationTaskEntity.ts
- frontend/src/types/generated/models/OptimizationTaskMode.ts
- frontend/src/types/generated/models/OptimizationTaskStatus.ts

Docs:
- docs/implementation-artifacts/3-1-optimization-task-creation-and-basic-config.md
- docs/implementation-artifacts/sprint-status.yaml

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] HIGH：API DTO 字段命名统一为 snake_case（对齐现有 `workspaces`/`test-sets` 与 ts-rs 生成类型），避免前后端对不上。
- [x] HIGH：明确不要复用 `backend/src/domain/models/algorithm.rs#OptimizationTask` 的 `user_id` 占位语义；本 Story 以 `workspace_id` 为边界实现。
- [x] MEDIUM：补齐关联表唯一约束与索引建议（防重复与性能）。
- [x] MEDIUM：补齐测试用例：test_set_id 不存在、1..N 关联回显。
- [x] LOW：AC3 对 `Hybrid` 的“按变体校验”表述更清晰（语义不变，降低误读）。

### Decisions

- [x] API 对外字段：沿用仓库现有 snake_case（最小改动、最小漂移风险）。
- [x] 模式-测试集校验：以 `TaskReference` 变体类型做硬校验（创建即失败），评估语义留到 Story 3.4。
- [x] 路由风格：沿用现有 kebab-case + workspace 嵌套路由（与 `test-sets` 一致）。

### Risks / Tech Debt

- [x] `config_json` 预留字段后续会快速膨胀：实现时建议限制大小（例如 32KB）并避免日志直出原文；具体边界在 Story 3.2/3.4 再细化。
- [x] `Hybrid` 的评估语义未定义：Story 3.4 必须补齐，否则可能出现“允许创建但评估不可解释”的产品体验问题。

### Follow-ups

- [x] Story 3.2/3.4：明确 `config_json` 的 schema/DTO 与校验边界。

## Change Log

- 2026-01-06：实现 Story 3.1（优化任务落库 + API + 前端创建/列表 + 测试与类型生成），状态推进至 `review`。
- 2026-01-06：修复 Code Review 发现的问题：`test_set_ids` 归一化、优化任务列表去 N+1、API `status` 类型对齐、移除占位 `OptimizationTask(user_id)` 生成与误用风险，并补充回归测试。
