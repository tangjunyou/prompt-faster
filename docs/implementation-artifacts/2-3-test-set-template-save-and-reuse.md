# Story 2.3: 测试集模板保存与复用

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 将测试集配置保存为模板复用,
so that 我可以快速创建结构相似的测试集。

## Acceptance Criteria

1. **Given** 用户已配置好一个测试集  
   **When** 用户点击“保存为模板”  
   **Then** 弹出模板命名对话框  
   **And** 保存后模板出现在模板列表中（仅当前 workspace 可见）

2. **Given** 用户在创建新测试集时  
   **When** 用户选择“从模板创建”  
   **Then** 显示可用模板列表  
   **And** 选择后自动填充模板配置（name/description/cases）  
   **And** 用户可以在此基础上修改并保存为新的测试集

### Acceptance Clarifications（为了可测试验收，补齐边界）

- 模板名校验：trim 后不能为空；≤ 128 字符；不要求全局唯一（同名允许，按 created_at 倒序展示）。
- 模板作用域：**按 workspace 隔离**（不做跨 workspace/跨用户共享）。
- 模板语义：模板是“配置快照”（deep copy），后续修改模板/测试集互不影响。
- MVP 不要求：模板编辑/版本管理/导入导出/分享/市场。

## Tasks / Subtasks

- [x] 任务 1：数据模型与迁移（AC: #1, #2）
  - [x] 新增迁移 `backend/migrations/004_add_is_template_to_test_sets.sql`
    - [x] 在 `test_sets` 表新增 `is_template INTEGER NOT NULL DEFAULT 0`（SQLite boolean 用 INTEGER）
    - [x] 增加索引：`(workspace_id, is_template, created_at)`（模板列表按时间倒序）
- [x] 任务 2：后端 Repository（AC: #1, #2）
  - [x] 复用 `backend/src/infra/db/repositories/test_set_repo.rs`，新增 template 专用方法
    - [x] `list_template_summaries_by_workspace_scoped(user_id, workspace_id)`
    - [x] `find_template_by_id_scoped(user_id, workspace_id, template_id)`
    - [x] `create_template_from_test_set_scoped(user_id, workspace_id, source_test_set_id, template_name, template_description?)`
  - [x] 修改既有“普通测试集”查询：确保 list/get/update/delete 都过滤 `is_template = 0`（避免模板混入测试集列表，也避免通过原 CRUD 意外编辑模板）
- [x] 任务 3：后端 API + OpenAPI + ts-rs（AC: #1, #2）
  - [x] 新增 routes：`backend/src/api/routes/test_set_templates.rs`（或在 `test_sets.rs` 内以独立 router 暴露）
    - [x] `GET /api/v1/workspaces/{workspace_id}/test-set-templates`（list summaries）
    - [x] `GET /api/v1/workspaces/{workspace_id}/test-set-templates/{template_id}`（get detail）
    - [x] `POST /api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}/save-as-template`（从已有测试集复制并保存为模板）
  - [x] 统一响应：`ApiResponse<T>`（AR1），错误码延用 `VALIDATION_ERROR`；模板不存在返回 404（不泄露存在性）
  - [x] OpenAPI：在 `backend/src/api/routes/docs.rs` 增加 tag + paths + schemas
  - [x] 类型生成：在 `backend/src/bin/gen-types.rs` 导出新增 DTO，并运行 `cargo run --bin gen-types`
- [x] 任务 4：前端 UI（AC: #1, #2）
  - [x] 复用页面：`frontend/src/pages/TestSetsView/TestSetsView.tsx`
    - [x] 在测试集列表项增加“保存为模板”入口（按钮或下拉菜单均可）
    - [x] 模板命名对话框（name/description 可选）→ 调用后端 save-as-template
    - [x] 增加“从模板创建”入口（建议在“创建测试集”表单附近）
    - [x] 模板列表弹窗：显示模板列表（name/description/cases_count/created_at），点击一项后拉取详情并预填表单
  - [x] 新增 service/hook：放入 `frontend/src/features/test-set-manager/`（对齐既有模式）
    - [x] `listTestSetTemplates` / `getTestSetTemplate` / `saveAsTemplate`
- [x] 任务 5：测试与门禁（AC: #1, #2）
  - [x] 后端：集成测试新增覆盖（模板 list/get/save-as-template；跨用户/跨 workspace 404；校验错误 400）
  - [x] 前端：service 单测（模板接口 happy path + error）与关键交互测试（至少：预填是否生效）
  - [x] 本地预检：`cargo fmt --all -- --check`、`cargo clippy -- -D warnings`、`cargo test --all`；`npm run lint`、`npm run test -- --run`、`npm run build`

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][LOW] 是否需要模板删除功能（MVP 可不做；若做，补 `DELETE /test-set-templates/{template_id}` 并写清权限/404 语义）
- [x] [AI-Review][LOW] 模板数量较多时是否需要分页（触发条件：模板列表明显变慢）
- [x] [AI-Review][LOW] 是否需要模板“预览 cases 摘要”而非直接展示完整 JSON（触发条件：cases 很大导致弹窗卡顿）

## Dev Notes

### Developer Context（给 Dev 的最小上下文）

- 本 Story 实现 FR8：把“测试集配置”保存为模板并复用创建新测试集。
- 前置能力已完成：
  - Story 2.1：TestSet CRUD + 用户隔离 + `ApiResponse<T>` + OpenAPI/类型生成（`docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md`）。
  - Story 2.2：批量导入（JSONL）仅影响 `cases` 的生成/编辑，不引入新持久化形态（`docs/implementation-artifacts/2-2-test-set-batch-import.md`）。
- MVP 的“模板”是快照：包含 `name/description/cases`；从模板创建后用户可以修改再保存为新测试集。

### TestSet Template 合约（避免实现发散）

- Template 数据边界（MVP）：`name`、`description?`、`cases_json`（完整复制 `TestCase[]`）。
- 作用域：按 workspace 隔离（不做全局模板/跨用户共享）。
- 行为：
  - “保存为模板”：从现有测试集复制 `cases_json`，生成新记录，并标记 `is_template=1`。
  - “从模板创建”：读取模板后预填“创建测试集”表单，最终创建普通测试集（`is_template=0`）。

### Technical Requirements（不可违背的硬约束）

- API 响应必须使用 `backend/src/api/response.rs:ApiResponse<T>`（AR1）。
- 时间字段仍使用 Unix 毫秒时间戳（AR3）；本 Story 如新增表/字段，必须遵循同规范。
- 依赖版本策略：以仓库现有 `Cargo.lock` / `package-lock.json` 为准，不在本 Story 擅自升级依赖。

### Architecture Compliance（必须对齐的架构边界）

- 后端分层：`api/routes` → `infra/db/repositories` → `domain/models`。
- 权限：模板与测试集同级隔离，必须通过 `CurrentUser` + scoped query 保证跨用户/跨 workspace 一律 404（不泄露存在性）。

### API Contract（MVP，写死防发散）

**Base Path（Templates）：** `/api/v1/workspaces/{workspace_id}/test-set-templates`

- `GET /` → 列出当前 workspace 的模板（summary）  
  - 200：`ApiSuccess<Vec<TestSetTemplateListItemResponse>>`（建议复用 `cases_count` 风格）
  - 401：`UNAUTHORIZED`
  - 404：`WORKSPACE_NOT_FOUND`（workspace 不存在或不属于当前用户）
- `GET /{template_id}` → 获取模板详情（含 cases）  
  - 200：`ApiSuccess<TestSetTemplateResponse>`
  - 401：`UNAUTHORIZED`
  - 404：`WORKSPACE_NOT_FOUND`（workspace 不存在或不属于当前用户）
  - 404：`TEST_SET_NOT_FOUND`（模板在该 workspace 下不存在；MVP 复用该错误码）

**Save As Template：** `POST /api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}/save-as-template`

- Request：`SaveAsTemplateRequest { name, description? }`
- 200：`ApiSuccess<TestSetTemplateResponse>`（返回新模板）
- 400：`VALIDATION_ERROR`（name 为空/超长）
- 401：`UNAUTHORIZED`
- 404：`WORKSPACE_NOT_FOUND`（workspace 不存在或不属于当前用户）
- 404：`TEST_SET_NOT_FOUND`（测试集在该 workspace 下不存在；含同用户跨 workspace 误用）

### Frontend UX Micro-spec（MVP，避免实现跑偏）

- 页面复用：`frontend/src/pages/TestSetsView/TestSetsView.tsx`
- 保存为模板：
  - 列表项动作入口 → 弹窗输入模板名（必填）/描述（可选）→ 成功 toast：`已保存为模板`
  - 失败：仅展示 `error.message`
- 从模板创建：
  - “创建测试集”区域提供“从模板创建”按钮
  - 弹窗列表：名称/描述/cases_count/时间；选择后自动预填表单（`name/description/casesJson`）
  - 预填后仍允许用户修改并走原创建流程

### File Structure Requirements（建议落点）

- Migration：`backend/migrations/004_add_is_template_to_test_sets.sql`
- Backend repo：`backend/src/infra/db/repositories/test_set_repo.rs`
- Backend routes：新增 `backend/src/api/routes/test_set_templates.rs`（并在 `backend/src/api/routes/mod.rs` / `workspaces.rs` nest）或在 `test_sets.rs` 内拆分 router
- OpenAPI：`backend/src/api/routes/docs.rs`
- Types export：`backend/src/bin/gen-types.rs`
- Frontend services/hooks：`frontend/src/features/test-set-manager/services/*`、`frontend/src/features/test-set-manager/hooks/*`
- Frontend page：`frontend/src/pages/TestSetsView/TestSetsView.tsx`

### Testing Requirements（与 CI 门禁一致）

- Backend：
  - 模板 list/get/save-as-template happy path
  - 跨用户/跨 workspace 访问 404（不泄露存在性）
  - `name` 校验 400（空/超长）
- Frontend：
  - 模板列表加载与错误提示
  - 选择模板后预填是否生效（至少验证 `casesJson` 被替换）

### Project Structure Notes

- 命名：路径 kebab-case（`test-set-templates`），模块 snake_case（`test_set_templates`）。
- 避免新增重复 feature：模板相关能力应放进既有 `test-set-manager` 体系内，保持“测试集管理”内聚。

### References

- [Source: docs/project-planning-artifacts/epics.md#Story-2.3-测试集模板保存与复用] — 原始 Story 与 AC（Line 894-913）
- [Source: docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md] — TestSet CRUD、错误码口径、门禁清单与文件落点范式
- [Source: docs/implementation-artifacts/2-2-test-set-batch-import.md] — cases 编辑/导入与 `TestCase` 合约（避免引入新形态）
- [Source: backend/src/api/routes/test_sets.rs] — 现有路由风格、`ApiResponse<T>`/错误码/用户隔离模式
- [Source: frontend/src/pages/TestSetsView/TestSetsView.tsx] — 现有页面结构与交互风格（复用入口）

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

- `backend`: `cargo run --bin gen-types`、`cargo fmt --all -- --check`、`cargo clippy -- -D warnings`、`cargo test --all`
- `frontend`: `npm run lint`、`npm run test -- --run`、`npm run build`

### Completion Notes List

- 完成“保存为模板 / 从模板创建”的后端与前端 MVP（workspace 隔离、深拷贝、同名允许）。
- 通过 `is_template` 字段复用 `test_sets` 表：普通测试集 CRUD 全部过滤 `is_template = 0`，避免模板混入/被误编辑。
- 新增模板 API：list/get/save-as-template，并补充 OpenAPI + ts-rs 类型导出与生成。
- 补齐后端集成测试、前端 service 单测与 UI 交互测试（预填生效）。
- 决策：本 Story 不做模板删除/分页/大 cases 预览（触发条件见 Review Follow-ups）。

### File List

- backend/Cargo.toml
- backend/migrations/004_add_is_template_to_test_sets.sql
- backend/src/api/routes/docs.rs
- backend/src/api/routes/mod.rs
- backend/src/api/routes/test_set_templates.rs
- backend/src/api/routes/test_sets.rs
- backend/src/api/routes/workspaces.rs
- backend/src/bin/gen-types.rs
- backend/src/infra/db/repositories/test_set_repo.rs
- backend/tests/test_set_templates_api_test.rs
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/2-3-test-set-template-save-and-reuse.md
- frontend/src/features/test-set-manager/hooks/useTestSetTemplates.ts
- frontend/src/features/test-set-manager/services/testSetTemplateService.ts
- frontend/src/features/test-set-manager/services/testSetTemplateService.test.ts
- frontend/src/pages/TestSetsView/TestSetsView.tsx
- frontend/src/pages/TestSetsView/TestSetsView.test.tsx
- frontend/src/types/generated/api/SaveAsTemplateRequest.ts
- frontend/src/types/generated/api/TestSetTemplateListItemResponse.ts
- frontend/src/types/generated/api/TestSetTemplateResponse.ts

## Change Log

- 2026-01-04：实现测试集模板保存与复用（后端 API/Repo/迁移 + 前端 UI/服务 + 测试与门禁）
- 2026-01-05：代码审查后修订（文档口径对齐、前后端校验一致性、模板列表按需加载）

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] “保存为模板”时 `description=null` 会被后端回填为源测试集描述，导致用户无法清空描述（已修复：`description=null` 现在表示“显式清空”，并补了后端集成测试验证）。
- [x] [MEDIUM] 前端保存为模板缺少显式错误捕获，失败时可能出现未处理 Promise rejection；且重开弹窗可能残留上次错误（已修复：增加 try/catch + reset mutation）。
- [x] [MEDIUM] 批量导入提示文案混淆“解析中”和“解析失败”（已修复：拆分提示）。
- [x] [MEDIUM] 后端缺少 save-as-template 的跨 workspace 404 语义测试（同用户不同 workspace）（已补测试）。
- [x] [LOW] 前端模板 service 单测缺少 400（VALIDATION_ERROR）分支覆盖（已补测试）。

### Decisions

- [x] `SaveAsTemplateRequest.description = null` 语义定义为“显式清空模板描述”，不再默认回填源测试集描述；若需要沿用源描述，由前端预填并提交字符串实现，避免后端隐式合并造成困惑。

### Risks / Tech Debt

- [x] ts-rs warning 降噪：已启用 `ts-rs` 的 `no-serde-warnings` feature，减少类型生成噪音。
- [x] 模板列表目前在页面加载时预取（即使用户不打开“从模板创建”弹窗）；已改为按需加载（打开弹窗时才拉取）。

### Follow-ups

- [x] 统一模板名称长度校验口径：前端 `.length` vs 后端 `chars().count()`（多字节字符边界）。

## Senior Developer Review (AI)

**Date:** 2026-01-05  
**Outcome:** Changes Requested → Addressed ✅

### Key Fixes (Post-review)

- 文档与实现口径对齐：补齐 404 错误码说明（`WORKSPACE_NOT_FOUND` vs `TEST_SET_NOT_FOUND`）。
- 文档变更追踪对齐：File List 补充 `backend/Cargo.toml`。
- 前后端一致性：前端模板名长度校验改为按 Unicode code point 计数（与后端 `chars().count()` 口径对齐）。
- 性能/体验：模板列表改为按需加载（仅在打开“从模板创建”弹窗时请求）。
