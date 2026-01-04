# Story 2.1: 测试集数据模型与基础 CRUD

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 手动创建、编辑和删除测试集,
so that 我可以管理用于优化任务的测试数据。

## Acceptance Criteria

1. **Given** 用户在测试集管理页面  
   **When** 用户点击“新建测试集”  
   **Then** 显示测试集创建表单（名称、描述）  
   **And** 创建成功后显示在测试集列表中  
   **And** 数据持久化到 SQLite，配置为 WAL 模式 + FULL synchronous（NFR6）

2. **Given** 用户选择一个已有测试集  
   **When** 用户点击“编辑”  
   **Then** 可以修改测试集名称、描述和测试用例  
   **And** 保存后变更立即生效

3. **Given** 用户选择一个测试集  
   **When** 用户点击“删除”并确认  
   **Then** 测试集从列表中移除  
   **And** 关联数据同步清理

## Tasks / Subtasks

- [x] 任务 1：数据模型与迁移（AC: #1, #2, #3）
  - [x] 新增 `test_sets` 表：`backend/migrations/003_create_test_sets.sql`
  - [x] 建立约束/索引：workspace FK（ON DELETE CASCADE）+ workspace_id 索引
  - [x] `cases_json`：存储 `Vec<TestCase>` 的 JSON 字符串
  - [x] 新增/导出领域模型：`backend/src/domain/models/test_set.rs`（并在 `backend/src/domain/models/mod.rs` 导出）
- [x] 任务 2：后端 TestSet CRUD API（AC: #1, #2, #3）
  - [x] Repository：`backend/src/infra/db/repositories/test_set_repo.rs`
  - [x] 路由：`backend/src/api/routes/test_sets.rs`（`/api/v1/workspaces/{workspace_id}/test-sets`）
  - [x] 用户隔离：workspace 归属校验 + scoped 查询/更新/删除（跨用户一律 404）
  - [x] 统一响应：`ApiResponse<T>` + 错误码 `TEST_SET_NOT_FOUND` / `WORKSPACE_NOT_FOUND` / `VALIDATION_ERROR`
  - [x] OpenAPI：`backend/src/api/routes/docs.rs` 新增 `test_sets` tag + paths + schemas
  - [x] 类型生成：`backend/src/bin/gen-types.rs` + `cargo run --bin gen-types`
- [x] 任务 3：前端测试集管理页面（AC: #1, #2, #3）
  - [x] 新增路由：`/workspaces/:id/test-sets`（`frontend/src/App.tsx`，使用 `ProtectedRoute`）
  - [x] 入口：`frontend/src/pages/WorkspaceView/WorkspaceView.tsx` 每行 workspace 增加“管理测试集”
  - [x] Feature：`frontend/src/features/test-set-manager/*`（service + hooks）
  - [x] UI：列表 + 新建/编辑（JSON 编辑 + 本地校验）+ 删除确认（`frontend/src/pages/TestSetsView/TestSetsView.tsx`）
- [x] 任务 4：测试与门禁（AC: #1, #2, #3）
  - [x] 后端：repo 单测 + 集成测试 `backend/tests/test_sets_api_test.rs`
  - [x] 前端：service 单测 `frontend/src/features/test-set-manager/services/testSetService.test.ts`
  - [x] 本地预检：`cargo fmt --all`、`cargo clippy --all -- -D warnings`、`cargo test --all`；`npm run lint`、`npm test -- --run`、`npm run build`

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][DOC] 将本 Story 的 review 结论沉淀到 `## Review Notes`（含风险/遗留）（2026-01-04）
- [x] [AI-Review][LOW] list 改为返回 summary（`cases_count`），编辑时再按需 `GET /{test_set_id}`（2026-01-04）
- [ ] [AI-Review][LOW] 统一 404 的错误码语义（`WORKSPACE_NOT_FOUND` vs `TEST_SET_NOT_FOUND`）

## Dev Notes

### Developer Context（给 Dev 的最小上下文）

- 本 Story 仅实现 **测试集（TestSet）** 的 CRUD（FR6/FR15），为后续 Story 2.2（批量导入）等打地基。
- 数据持久化必须满足 NFR6：SQLite `WAL` + `FULL synchronous`（项目已在连接池层配置）。
- **强约束：测试集必须归属某个 workspace**；后端与前端都以 `workspace_id` 作为第一层边界，避免“全局测试集”导致隔离漏洞与后续返工。
- “关联数据同步清理”的当前解释：MVP 阶段仅删除 `test_sets` 记录（`cases_json` 为内嵌 JSON，无额外关联表）；未来若出现引用（任务/Checkpoint 等），再补充外键或显式清理。
- 术语与命名：本 Story 以 PRD 的 `test_sets`（测试集）为准；`cases` 为 `TestCase[]`（测试用例数组）。避免把“测试集”误实现为单条 test_case 记录。

### TestSet / Cases 合约（避免实现发散）

- `TestSet` 是“测试集”实体；`cases_json` 是一个 JSON 数组，元素结构优先复用 `backend/src/domain/models/algorithm.rs:TestCase`。
- MVP UI 允许“原始 JSON”方式编辑 `cases_json`（避免在本 Story 过早引入复杂表单/字段映射）；但必须做最小校验：
  - JSON 必须可解析为数组
  - 每个元素必须至少包含 `id`、`input`（HashMap）与 `reference`（`TaskReference`）；其他字段按 `TestCase` 结构可选/默认
  - 校验失败：返回 400 + `VALIDATION_ERROR`（错误 message 说明具体字段缺失/格式错误）

示例（仅示意）：

```json
[
  {
    "id": "case-1",
    "input": { "text": "..." },
    "reference": { "Exact": { "expected": "..." } },
    "split": "train"
  }
]
```

### Technical Requirements（不可违背的硬约束）

- API 响应必须使用 `backend/src/api/response.rs:ApiResponse<T>`（data/error 互斥）。
- 命名规范：Rust snake_case，TypeScript camelCase；跨语言字段使用 `serde(rename_all = \"camelCase\")`。
- 时间字段统一使用 INTEGER（Unix ms）。
- 任何“用户可见 UI”不得直接展示后端 `error.details`。
- （可选但建议）SQLite 外键约束：在连接池显式启用 `foreign_keys(true)`，避免环境差异导致 FK/级联行为不一致。

### Architecture Compliance（必须对齐的架构边界）

- 后端分层：`api/routes`（路由） → `infra/db/repositories`（数据访问） → `domain/models`（领域模型）。
- 权限：所有 TestSet CRUD 必须在后端通过 `CurrentUser` 做用户隔离校验（建议通过 workspace_id join workspaces.user_id）。

### API Contract（MVP，写死防发散）

> 统一命名：**路径 kebab-case**（`test-sets`），代码/模块 snake_case（`test_sets`）。

**Base Path：** `/api/v1/workspaces/{workspace_id}/test-sets`

- `GET /` → 列出当前 workspace 的测试集  
  - 200：`ApiSuccess<Vec<TestSetListItemResponse>>`（summary：不返回 `cases`，返回 `cases_count`）
  - 401：`UNAUTHORIZED`
  - 404：`WORKSPACE_NOT_FOUND`（workspace 不存在或不属于当前用户）
- `POST /` → 创建测试集  
  - Request：`CreateTestSetRequest { name, description?, cases }`
  - 200：`ApiSuccess<TestSetResponse>`
  - 400：`VALIDATION_ERROR`（name 空/超长；cases 非法）
  - 401：`UNAUTHORIZED`
  - 404：`WORKSPACE_NOT_FOUND`
- `GET /{test_set_id}` → 获取单个测试集  
  - 200：`ApiSuccess<TestSetResponse>`
  - 401：`UNAUTHORIZED`
  - 404：`TEST_SET_NOT_FOUND`（包含跨用户访问的情况）
- `PUT /{test_set_id}` → 更新测试集（整体覆盖更新）  
  - Request：`UpdateTestSetRequest { name, description?, cases }`
  - 200：`ApiSuccess<TestSetResponse>`
  - 400/401/404：同上
- `DELETE /{test_set_id}` → 删除测试集  
  - 200：`ApiSuccess<DeleteTestSetResponse { message }>`
  - 401：`UNAUTHORIZED`
  - 404：`TEST_SET_NOT_FOUND`

**字段约束（MVP）：**

- `name`：trim 后不能为空；最大 128 字符（对齐 workspaces 的约束风格）
- `cases`：必须能反序列化为 `Vec<TestCase>`（见 `backend/src/domain/models/algorithm.rs:TestCase`）

**错误响应示例（ApiError）：**

```json
{ "error": { "code": "VALIDATION_ERROR", "message": "cases 格式错误：必须是 TestCase 数组", "details": null } }
```

### Frontend UX Micro-spec（MVP，避免实现跑偏）

- 列表空状态：显示“暂无测试集，请先创建一个。”
- JSON 校验失败（本地）：禁用提交按钮 + 显示“JSON 无法解析/不是数组/缺少字段 id|input|reference”
- 删除确认：二次确认文案包含测试集名称

### Library / Framework Requirements（版本与用法）

- 依赖版本策略：**以仓库当前 `Cargo.toml`/`Cargo.lock` 与 `frontend/package.json`/`package-lock.json` 为准，不在本 Story 中擅自升级依赖版本**。

### File Structure Requirements（建议落点）

- Backend routes：`backend/src/api/routes/test_sets.rs` + `backend/src/api/routes/mod.rs` 注册
- Backend repo：`backend/src/infra/db/repositories/test_set_repo.rs` + `backend/src/infra/db/repositories/mod.rs` 注册
- Domain model：`backend/src/domain/models/test_set.rs`（或在 `domain/models/mod.rs` 导出）
- Migration：`backend/migrations/003_create_test_sets.sql`（不要改 `001_initial_schema.sql`；仓库已存在 `002_*.sql`）
- Frontend feature：`frontend/src/features/test-set-manager/`（对齐既有 workspace 模式：service + hooks + 组件 + tests）
- Frontend types：沿用 `ts-rs` 生成的请求/响应 DTO（`frontend/src/types/generated/api/*`）
- OpenAPI：`backend/src/api/routes/docs.rs`（新增 tag + paths + schemas，并在 `/swagger` 可见）
- Types export：`backend/src/bin/gen-types.rs`（新增导出项）

### Testing Requirements（与 CI 门禁一致）

- Backend：`cargo fmt --all -- --check`，`cargo clippy -- -D warnings`，`cargo test --all`
- Frontend：`npm run lint`，`npm run test -- --run`，`npm run build`
- 至少覆盖：未登录 401、跨用户访问 404（不泄露存在性）、CRUD happy path、name 校验（空/超长）、`cases_json` JSON 校验（非 JSON / 非数组 / 缺字段）

### Project Structure Notes

- 现有 SQLite 连接池已配置 WAL/FULL：`backend/src/infra/db/pool.rs`
- 参考现有 CRUD 模式：`backend/src/api/routes/workspaces.rs` + `backend/src/infra/db/repositories/workspace_repo.rs`
- 前端 service/hook 规范：`frontend/src/features/workspace/services/workspaceService.ts` + `frontend/src/features/workspace/hooks/useWorkspaces.ts`

### References

- [Source: docs/project-planning-artifacts/epics.md#Epic-2-测试集管理] — Story 2.1 原始验收标准
- [Source: docs/project-planning-artifacts/prd.md#7.5-数据持久化] — SQLite WAL/FULL & `test_sets` 表定义
- [Source: docs/project-planning-artifacts/architecture.md#Project-Structure-&-Boundaries] — 模块边界与目录结构约束
- [Source: backend/src/infra/db/pool.rs] — WAL/FULL 实际配置
- [Source: backend/src/api/response.rs] — 统一响应结构
- [Source: docs/implementation-artifacts/1-10-ci-pipeline-and-test-gates.md] — CI/测试门禁与本地预检清单
- [Source: docs/implementation-artifacts/epic-1-retro-2026-01-03.md] — Epic 2 启动前边界/质量注意事项

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

N/A

### Completion Notes List

- 结合 4 份审查建议（R1-R4）复核并修订：迁移编号、cases 校验、路径命名、权限语义与前端入口闭环
- 已实现后端 TestSet CRUD（含用户隔离、错误码、OpenAPI）并生成前端类型（`cargo run --bin gen-types`）
- 已实现前端测试集管理页（列表/创建/编辑/删除）与 workspace 列表入口
- 已通过本地门禁：后端 `cargo fmt/clippy/test`，前端 `npm run lint/test/build`

### File List

- backend/migrations/003_create_test_sets.sql
- backend/src/infra/db/pool.rs
- backend/src/domain/models/algorithm.rs
- backend/src/domain/models/test_set.rs
- backend/src/domain/models/mod.rs
- backend/src/infra/db/repositories/test_set_repo.rs
- backend/src/infra/db/repositories/mod.rs
- backend/src/api/routes/test_sets.rs
- backend/src/api/routes/workspaces.rs
- backend/src/api/routes/mod.rs
- backend/src/api/routes/docs.rs
- backend/src/shared/error_codes.rs
- backend/src/bin/gen-types.rs
- backend/tests/test_sets_api_test.rs
- frontend/src/features/test-set-manager/services/testSetService.ts
- frontend/src/features/test-set-manager/services/testSetService.test.ts
- frontend/src/features/test-set-manager/hooks/useTestSets.ts
- frontend/src/pages/TestSetsView/TestSetsView.tsx
- frontend/src/pages/TestSetsView/index.ts
- frontend/src/pages/index.ts
- frontend/src/pages/WorkspaceView/WorkspaceView.tsx
- frontend/src/App.tsx
- frontend/src/types/generated/api/index.ts (generated)
- frontend/src/types/generated/api/CreateTestSetRequest.ts (generated)
- frontend/src/types/generated/api/UpdateTestSetRequest.ts (generated)
- frontend/src/types/generated/api/TestSetListItemResponse.ts (generated)
- frontend/src/types/generated/api/TestSetResponse.ts (generated)
- frontend/src/types/generated/api/DeleteTestSetResponse.ts (generated)
- frontend/src/types/generated/models/index.ts (generated)
- frontend/src/types/generated/models/TestSet.ts (generated)
- docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/validation-report-2026-01-04_20-05-33.md

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- ✅（Fixed / MEDIUM）前端编辑保存失败时未展示错误（仅展示 create error） → 已补齐 update error 展示（`frontend/src/pages/TestSetsView/TestSetsView.tsx`）
- ✅（Fixed / MEDIUM）`name` 长度校验按字节计算，中文场景可能误伤 → 改为按字符数（`backend/src/api/routes/test_sets.rs`、`backend/src/api/routes/workspaces.rs`）
- ✅（Fixed / MEDIUM）集成测试覆盖不足（缺跨用户 update/delete；缺 cases 非法校验）→ 已补齐（`backend/tests/test_sets_api_test.rs`）
- ✅（Fixed / MEDIUM）Dev Agent Record → File List 遗漏了实际改动文件 → 已补齐本段列表
- ✅（Fixed / LOW）`GET /workspaces/{workspace_id}/test-sets` 改为返回 summary（`cases_count`），编辑时再按需 `GET /{test_set_id}` 拉完整 `cases`
- 🟢（Accepted / LOW）404 错误码存在两套语义：目前以“不泄露存在性”为优先，暂不为了统一而额外增加 workspace 探测查询

### Decisions

- 维持 list 返回完整 `cases`：减少前端“点击编辑再额外 GET”的复杂度；后续按性能/数据量再优化
- 统一按字符数做 `name` 长度限制：文案说的是“字符”，以用户直觉为准（尤其中文）

### Risks / Tech Debt

- 若测试集包含大量 cases：编辑会额外触发一次 `GET /{test_set_id}` 拉详情（触发条件：编辑入口明显变慢时再考虑 prefetch/缓存策略）
- Rust 构建时出现较多 `ts-rs failed to parse serde attribute` 的输出噪音（不影响功能，但可能污染 CI 日志；触发条件：影响排查时再处理）

### Follow-ups

- 同步到 `### Review Follow-ups (AI)`（见上方）
