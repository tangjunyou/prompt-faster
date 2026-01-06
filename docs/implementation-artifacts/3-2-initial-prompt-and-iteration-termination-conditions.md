# Story 3.2: 初始 Prompt 与迭代终止条件

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 配置初始 Prompt 和迭代终止条件,
so that 我可以控制优化的起点和终止条件。

## Acceptance Criteria

### AC1：初始 Prompt（FR20）

**Given** 用户在任务配置页面
**When** 用户配置“初始 Prompt”
**Then** 可以选择留空或填写初始 Prompt
**And** 当用户选择留空时，UI 显示提示文案：『留空时，系统将在首次迭代中基于优化目标和测试集自动生成初始 Prompt』
**And** 保存时：空字符串在后端归一化为 `null`（而不是 `""`）
**And** API 响应中 `initial_prompt` 为 `null` 时，前端必须显示上述提示文案（并视为“用户选择留空”，不是异常）

### AC2：最大迭代轮数（FR23c）

**Given** 用户在任务配置页面
**When** 用户设置“最大迭代轮数”
**Then** 显示数值输入框，带合理范围约束（如 1-100）
**And** 默认值为推荐值（如 10）

### AC3：通过率阈值（FR23c）

**Given** 用户在任务配置页面
**When** 用户设置“通过率阈值”
**Then** 显示百分比输入框
**And** 提供说明文字解释通过率阈值的含义
**And** 默认值为推荐值（如 95%）

### AC4：数据划分策略（FR23c）

**Given** 用户在任务配置页面
**When** 用户配置“数据划分策略”
**Then** 可以以“百分比整数”的形式配置 Train/Validation 划分
**And** 默认值为 80/20
**And** 本 Story 的 UI 仅暴露 Train/Validation；Holdout 在本 Story 固定为 0%（仅在 schema 中预留字段，未来 Story 再开放）

## Tasks / Subtasks

### 任务 1：后端任务配置 Schema（config_json）与校验（AC1-AC4）

- [x] 定义 `OptimizationTaskConfig`（含 schema_version）与序列化/反序列化策略（JSON TEXT）
- [x] 定义默认值（max_iterations=10、pass_threshold=95%、train/val 比例默认值）
- [x] 定义并实现校验规则（范围、空值语义、长度限制、向后兼容）

### 任务 2：后端 API（读取+更新任务配置）（AC1-AC4）

- [x] 为 `GET /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}` 增补返回字段（任务配置或规范化视图）
- [x] 新增更新端点（`PUT /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}/config`）
- [x] 保持 `ApiResponse<T>`、错误码、workspace 权限校验与 tracing/correlationId 规范一致

### 任务 3：前端任务配置页面（初始 Prompt + 停止条件）（AC1-AC4）

- [x] 在 `/workspaces/:id/tasks` 的任务卡片增加“配置”入口（Link）
- [x] 新增任务配置页（建议 `/workspaces/:id/tasks/:taskId`）
- [x] 表单：初始 Prompt（可空 + 提示文案）、最大迭代轮数（1-100）、通过率阈值（百分比）、数据划分比例
- [x] 保存后提示成功，并回显后端规范化后的配置

### 任务 4：测试与回归保护（AC 全覆盖）

- [x] 后端集成测试：更新配置成功、边界值校验、workspace 越权、NotFound
- [x] 前端单测：渲染默认值、留空提示、保存成功/失败 message 展示（不展示 details）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 拍板并落实 API 契约：`config` 永远非空（后端归一化默认值），并统一使用 `PUT .../{task_id}/config`
- [x] [AI-Review] 落实 config_json 防膨胀与脱敏：总大小上限 + 日志不得打印 `initial_prompt` 原文（仅允许长度/摘要）
- [x] [AI-Review] 明确 data split 规则：优先使用测试用例显式 `split`；否则按 `data_split` 生成（如需可复现 seed，后续 schema_version 升级引入）；通过率阈值基于 validation 口径

## Dev Notes

### Developer Context（避免实现偏航）

- 现状：Story 3.1 已实现优化任务的最小闭环（DB 表 `backend/migrations/007_create_optimization_tasks.sql`，后端路由 `backend/src/api/routes/optimization_tasks.rs`，前端列表+创建页 `frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx`）。
- 本 Story 目标：为“任务配置”补齐 **初始 Prompt** 与 **迭代终止条件**（最大轮数/通过率阈值/数据划分比例）的可配置、可持久化、可回显能力。
- 存储边界：延续 Story 3.1 的 `optimization_tasks.config_json`（TEXT）作为配置承载；本 Story 必须给出**明确 schema**（含 `schema_version`）与校验规则，避免后续 Story 3.4（高级算法参数）/Epic 4（执行）出现“字段随意堆叠”的不可维护局面。
- 明确非目标（防 scope creep）：
  - 不实现优化执行引擎/Run View 的真实运行（Epic 4）
  - 不实现老师模型选择/参数细化（已有全局配置，后续 Story 再扩展）
  - 不实现完整的 DataSplitConfig/IterationConfig 高级参数面板（Story 3.4）

### Architecture / UX Guardrails（必须遵守）

- API 响应：必须使用 `ApiResponse<T>`（`backend/src/api/response.rs`），禁止返回裸 JSON。
- 错误展示：前端不得直接展示 `error.details`（仅展示 `error.message`），避免泄露调试信息。
- Workspace 是第一隔离边界：所有读写均必须基于 `workspace_id`，并通过 workspace→user_id 做权限校验（沿用现有 workspaces/test_sets/optimization_tasks 逻辑）。
- 字段命名：对外 JSON 字段一律 `snake_case`（对齐现有 ts-rs 生成类型与 `CreateOptimizationTaskRequest`），避免同一资源同时支持 camelCase + snake_case 的“双口径”。
- TanStack Query 约束：只允许在 `features/*/hooks/` 封装 Query/Mutation；`services/*Service.ts` 只能导出纯 fetch 函数。
- UX 口径冲突处理：UX 文档中“初始 Prompt 必填”的表述与 FR20（可留空）存在冲突；以 FR20 为准实现“可留空 + 明确提示文案”，同时提供推荐默认值，保证新手路径可用。

### 关键设计决策（提前写死，减少歧义）

- 配置存储：使用 `optimization_tasks.config_json`（TEXT）存储 JSON；本 Story 引入 `schema_version`，确保未来扩展（Story 3.4/ Epic 4）可演进而不破坏旧数据。
- 更新方式：新增专用配置更新端点（而不是把配置字段塞进“创建任务”请求体里），以适配“先创建任务→逐步补齐配置→再启动运行”的产品流程。
- 默认值策略：后端必须提供**规范化默认值**（即使数据库里没有 config_json），保证前端无需做“默认值散落在多处”的重复逻辑。
- 数据划分：本 Story 只要求“训练/验证比例可配置并持久化”；Holdout 三分法属于 PRD 目标，但执行语义在 Epic 4 落地。本 Story 的 schema 需预留 holdout（默认 0%）以避免后续迁移成本。


### Technical Requirements（后端契约 + DTO + 兼容策略）

#### 1) 配置 Schema（建议 V1）

- 存储位置：`optimization_tasks.config_json`（TEXT，JSON）
- 顶层必须包含 `schema_version`（number），本 Story 固定为 `1`

实现结构（对外字段保持 `snake_case`）：

```json
{
  "schema_version": 1,
  "initial_prompt": null,
  "max_iterations": 10,
  "pass_threshold_percent": 95,
  "data_split": {
    "train_percent": 80,
    "validation_percent": 20,
    "holdout_percent": 0
  }
}
```

#### 2) 语义与校验（后端强校验，前端做即时校验）

- `initial_prompt`：
  - `null` 表示“留空”（UI 必须展示提示文案）
  - 空字符串必须在后端归一化为 `null`
  - 长度限制：`initial_prompt` 按 UTF-8 字节数计，必须 `<= 20000` bytes
- `max_iterations`：整数，范围 `1..=100`（与 AC 一致）
- `pass_threshold_percent`：整数，范围 `1..=100`（UI 以百分比输入）
- `data_split.*_percent`：
  - 都是整数 `0..=100`
  - **本 Story（schema_version=1）固定 `holdout_percent = 0`**（UI 不暴露），并要求 `train_percent + validation_percent == 100`
  - 与测试用例显式 `split` 的关系（必须写死，防两套口径）：若测试用例已标注 `split`，则执行引擎应优先使用该标注；仅对 `split=unassigned/缺失` 的用例按 `data_split` 生成划分（如需可复现 seed，后续通过 schema_version 升级引入）
  - 通过率阈值口径（必须写死）：`pass_threshold_percent` 以 **validation** 集合的通过率作为比较基准（train 不作为“达标”统计口径）
- 向后兼容：
  - `config_json IS NULL` 视为“无配置”，后端必须在响应里返回 **规范化后的默认配置**
  - 未来 schema_version 升级时，必须支持读取旧版本并在写入时升级/迁移（本 Story 先打基础）
- `config_json` 总大小限制（防膨胀 / 对齐 NFR 资源约束）：序列化后的 UTF-8 字节数必须 `<= 32768` bytes，超出返回 `400 VALIDATION_ERROR`
- 日志脱敏（对齐 NFR 日志脱敏目标）：tracing 日志不得打印 `initial_prompt` 原文或完整 `config_json`；如需调试仅允许记录（1）字节长度（2）schema_version（3）可选 hash/摘要

#### 3) API 契约（建议最小集）

- `GET /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}`
  - 在现有 `OptimizationTaskResponse` 基础上增补 `config`（**规范化后的对象，永远不为 null**；即使 `config_json` 为 NULL，也返回默认值）
- `PUT /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}/config`（幂等更新配置）
  - 请求体：使用“扁平字段 + Optional”的表单语义（前端更易提交，后端统一归一化并持久化为 schema_version=1 的 JSON）
  - 响应：返回更新后的完整任务（复用扩展后的 `OptimizationTaskResponse`，含 `config`）

请求体示例（字段均为 `snake_case`；`null` 表示“用户选择留空/不设置”）：  

```json
{
  "initial_prompt": null,
  "max_iterations": 10,
  "pass_threshold_percent": 95,
  "train_percent": 80,
  "validation_percent": 20
}
```

错误码约束：校验失败统一返回 `400` + `VALIDATION_ERROR`（沿用既有错误码体系），不要为本端点新造错误码。

#### 4) 仓储层落点（建议）

- `backend/src/infra/db/repositories/optimization_task_repo.rs`
  - 新增 `update_config_scoped(...)`：校验 workspace + task 归属后更新 `config_json` 与 `updated_at`
  - 更新需在事务内完成（与权限校验同一事务），避免 TOCTOU



### Library / Framework Requirements（版本与依赖边界）

- 前端：沿用现有依赖与用法（React 19.x、`react-router` 7.x、`@tanstack/react-query` 5.x），本 Story 不做框架升级；新增页面/表单实现保持与现有 `OptimizationTasksView` 一致（受控组件 + mutation）。
- 后端：沿用现有依赖（axum 0.8、sqlx 0.8）；避免引入新的配置/验证库，优先用现有模式完成校验与错误响应。
- 类型：任何后端 DTO 变更后必须运行 `cargo run --bin gen-types` 同步 `frontend/src/types/generated/`，并保证对外字段 `snake_case` 一致。

### File Structure Requirements（建议改动清单）

**后端（Rust）**

- 路由：`backend/src/api/routes/optimization_tasks.rs`
  - 为任务详情响应补齐 `config` 字段（规范化后永远非空）
  - 新增配置更新端点（建议放在同一文件，保持聚合）
- 仓储：`backend/src/infra/db/repositories/optimization_task_repo.rs`
  - 新增 `update_config_scoped`（或同等语义方法）
- 领域模型：
  - 若将配置作为强类型对外/对内：新增 `backend/src/domain/models/optimization_task_config.rs`（或放入 `optimization_task.rs`）
  - 注意：不要去改动 `backend/src/domain/models/algorithm.rs#OptimizationTask`（它是旧占位 DTO）

**前端（React）**

- 新增页面：`frontend/src/pages/OptimizationTaskConfigView/`（或同等命名），并在 `frontend/src/pages/index.ts` 导出
- 路由：`frontend/src/App.tsx` 新增 `/workspaces/:id/tasks/:taskId`
- hooks/services：复用现有 `frontend/src/features/task-config/`，新增 `updateOptimizationTaskConfig(...)` 与对应 hook

### Testing Requirements（与 CI 门禁对齐）

- 后端：在 `backend/tests/optimization_tasks_api_test.rs` 增补配置更新用例：
  - 200：更新成功并回显规范化后的配置
  - 400：越界值（max_iterations、pass_threshold_percent、train_percent + validation_percent != 100）
  - 404：task 不存在 / workspace 不存在
  - 401：未登录
  - 403/404：跨 workspace 访问（沿用当前 repo 的错误语义；保持一致即可）
- 前端：新增 `*.test.tsx`（参考 `frontend/src/pages/OptimizationTasksView/OptimizationTasksView.test.tsx` 的 MSW + QueryClientProvider 模式）：
  - 默认值渲染（后端无 config_json 时的规范化默认值）
  - 初始 Prompt 留空时的提示文案
  - 保存成功后提示/回显
  - 保存失败时只展示 message

### Previous Story Intelligence（从 3.1 继承的经验/坑位）

- `config_json` 会快速膨胀：本 Story 必须明确长度上限与脱敏策略，避免日志/错误信息输出大段 prompt。
- DTO 字段命名继续统一 `snake_case`（避免前后端类型对不上）。
- 以 `workspace_id` 为第一隔离边界，避免重新引入 `user_id` 语义混乱。

### Git Intelligence Summary（最近工程约定）

- 最近提交 `69ed4e3` 已完成 Story 3.1 的优化任务 MVP；本 Story 应尽量“按既有模式扩展”，避免引入新的路由风格/错误结构。

### Latest Tech Information（避免过期实现）

- 依赖现状（本仓库 `frontend/package.json` / `backend/Cargo.toml`）：React 19.x、`react-router` 7.x、TanStack Query 5.x、axum 0.8、sqlx 0.8。
- 本 Story 不做任何依赖升级；实现时只需避免“按旧版本文档写法”导致的 API 用法错误（例如把 TanStack Query v5 写成 v4 的调用形态）。

### Project Context Reference

- 需求来源：`docs/project-planning-artifacts/epics.md`（Story 3.2），`docs/project-planning-artifacts/prd.md`（配置分层、数据划分目标），`docs/project-planning-artifacts/ux-design-specification.md`（首次优化旅程、停止条件交互）。
- 现有实现：`docs/implementation-artifacts/3-1-optimization-task-creation-and-basic-config.md`（关键决策/护栏/路径）。

### Story Completion Status（执行完成后的更新要求）

- 本 Story 完成后（实现+自测+测试通过）：
  - 将本 Story 的 `## Dev Agent Record` 填充完整（实现计划、变更文件清单、完成说明）
  - 将 `## Review Notes` 按统一结构补齐（发现/决策/风险/后续）
  - 确保 `docs/implementation-artifacts/sprint-status.yaml` 中 `3-2-initial-prompt-and-iteration-termination-conditions` 状态按流程推进（ready-for-dev → in-progress → review → done）

### Project Structure Notes

- 与架构文档存在的“理想态 vs 仓库事实”冲突：路由命名仍以现有实现为准（kebab-case + workspace 嵌套），不要在本 Story 引入新的 snake_case 端点风格。

### References

- [Source: docs/project-planning-artifacts/epics.md#Story-3.2-初始-Prompt-与迭代终止条件]（FR20/FR23c）
- [Source: docs/project-planning-artifacts/prd.md#7.3.1-页面结构与配置逻辑]（任务配置分层、页面结构）
- [Source: docs/project-planning-artifacts/prd.md#7.5.4-数据划分与防过拟合]（Train/Validation/Holdout 产品承诺）
- [Source: docs/project-planning-artifacts/ux-design-specification.md#Journey-1-首次优化]（初始 Prompt/停止条件在首次优化流程中的位置）
- [Source: docs/implementation-artifacts/3-1-optimization-task-creation-and-basic-config.md#Review-Notes]（config_json 膨胀风险、snake_case 统一、workspace 边界）
- [Source: backend/migrations/007_create_optimization_tasks.sql]（config_json 字段落点）
- [Source: backend/src/api/routes/optimization_tasks.rs]（现有 task API 风格与错误响应）
- [Source: backend/src/infra/db/repositories/optimization_task_repo.rs]（workspace scoping 与 repo 模式）
- [Source: frontend/src/features/task-config/hooks/useOptimizationTasks.ts]（Query key 与 hook 组织）
- [Source: frontend/src/pages/OptimizationTasksView/OptimizationTasksView.test.tsx]（前端 MSW 测试范式）


## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

Backend：

- `cd backend && cargo fmt --check`
- `cd backend && cargo clippy -- -D warnings`
- `cd backend && cargo test`
- `cd backend && cargo run --bin gen-types`

Frontend：

- `cd frontend && npm test -- --run`
- `cd frontend && npm run lint`

### Completion Notes List

1. 后端：新增 `OptimizationTaskConfig` 强类型 schema（含默认值/校验/空 Prompt 归一化），并限制 `config_json` 总大小（<= 32KB）。
2. 后端：任务详情 `GET` 增补 `config`（永远非空）；新增 `PUT .../{task_id}/config` 更新配置并回显规范化结果。
3. 前端：任务列表增加“配置”入口；新增任务配置页与表单（初始 Prompt + 停止条件 + 数据划分），保存后提示成功并回显。
4. 类型：运行 `cargo run --bin gen-types` 同步 `frontend/src/types/generated/`。
5. 测试：后端/前端新增回归用例覆盖空 Prompt 归一化、边界值校验、NotFound 与错误 message 展示。

### File List

Backend:

- backend/src/api/routes/docs.rs
- backend/src/api/routes/optimization_tasks.rs
- backend/src/bin/gen-types.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/models/optimization_task_config.rs
- backend/src/infra/db/repositories/optimization_task_repo.rs
- backend/tests/optimization_tasks_api_test.rs

Frontend:

- frontend/src/App.tsx
- frontend/src/features/task-config/hooks/useOptimizationTasks.ts
- frontend/src/features/task-config/services/optimizationTaskService.ts
- frontend/src/pages/index.ts
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx
- frontend/src/pages/OptimizationTaskConfigView/index.ts
- frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx
- frontend/src/types/generated/api/OptimizationTaskResponse.ts
- frontend/src/types/generated/api/UpdateOptimizationTaskConfigRequest.ts
- frontend/src/types/generated/models/DataSplitPercentConfig.ts
- frontend/src/types/generated/models/OptimizationTaskConfig.ts

Docs:

- docs/implementation-artifacts/3-2-initial-prompt-and-iteration-termination-conditions.md
- docs/implementation-artifacts/sprint-status.yaml

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] HIGH：API 详情响应统一补齐 `config`（永远非空），并在更新时把空初始 Prompt 归一化为 `null`（而不是空字符串）。
- [x] HIGH：落实 `config_json` 防膨胀/脱敏：总大小上限（<= 32KB）+ 日志不打印 `initial_prompt` 原文（仅记录长度）。
- [x] MEDIUM：更新配置时保留未知字段（`config_json` 顶层 extra），为后续 schema 扩展提供向后兼容空间。

### Decisions

- [x] `PUT .../{task_id}/config` 请求体采用扁平字段（`train_percent`/`validation_percent`），对齐 Dev Notes 示例并降低前端提交复杂度。
- [x] `holdout_percent` 仅在 schema 中预留；本 Story 强校验为 0%，UI 不暴露。
- [x] 数据划分与达标口径约定：执行引擎阶段遵循“显式 `split` 优先；否则按 `data_split` 生成（如需可复现 seed，后续 schema_version 升级引入）；通过率阈值以 validation 口径”为准（本 Story 先完成配置承载与校验）。

### Risks / Tech Debt

- [x] `initial_prompt` 目前会做前后空白 trim：若未来出现“前后空白有意义”的 Prompt 需求，需要调整归一化策略并补测试。
- [x] `seed` 目前未作为显式字段暴露：执行引擎实现 data split 时需明确 seed 来源（固定默认/全局配置/后续配置字段），并补齐端到端验证。

### Follow-ups

- [x] Epic 4（执行引擎）：实现并验证 data split 规则（显式 `split` 优先；否则按 `data_split` 生成；如需可复现 seed，在引入 seed 字段后保持可复现；通过率阈值以 validation 口径）。
- [x] Story 3.4+：若需要更细粒度的配置（如 seed/holdout），在 schema_version 升级时补齐迁移策略与 UI 暴露。

## Change Log

- 2026-01-06：实现 Story 3.2（初始 Prompt + 迭代终止条件配置、后端校验与更新 API、前端配置页、测试与类型生成），状态推进至 `review`。
- 2026-01-06：补齐回归测试（更新配置越权/配置膨胀/初始 Prompt 字节上限）并对齐文档契约，状态推进至 `done`。
