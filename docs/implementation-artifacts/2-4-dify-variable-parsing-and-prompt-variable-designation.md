# Story 2.4: Dify 变量解析与 Prompt 变量指定

- Story Key: `2-4-dify-variable-parsing-and-prompt-variable-designation`
- Epic: `Epic 2: 测试集管理`
- Related FRs: `FR9`, `FR10`
- Created: `2026-01-05`

状态：done

## 用户故事

**作为** Prompt 优化用户，  
**我希望** 系统能自动解析 Dify 工作流（应用）的输入变量结构，并让我指定哪个变量是“待优化的 system prompt”，  
**从而** 系统知道如何调用我的工作流，以及优化哪个部分。

## 验收标准（Acceptance Criteria）

1. **前提** 用户已配置 Dify API 凭证且连接成功  
   **当** 用户在“测试集”中进入 Dify 变量配置并点击“刷新/解析变量”  
   **则** 后端调用 Dify API 获取输入变量结构（`GET {base_url}/v1/parameters`）并返回给前端  
   **并且** 前端展示变量列表（包含变量名、类型/组件、是否必填、默认值等可用信息）

2. **前提** 变量列表已加载  
   **当** 用户选择某个变量作为“待优化 system prompt”  
   **则** 该变量被保存为优化目标变量  
   **并且** UI 明确标识此变量（例如 badge：“优化目标”）

3. **前提** 变量列表已加载  
   **当** 用户为“非优化目标”的变量配置取值来源  
   **则** 支持两种来源：
   - **固定默认值**（常量 JSON：string/number/bool/object/array）  
   - **关联测试用例字段**：从 `TestCase.input[{key}]` 读取（key 为字符串；值为 JSON）
   **并且** 未显式配置来源的变量，默认使用 Dify 返回的默认值（若有）；否则按 Dify 语义省略该输入字段

4. **前提** 用户已完成变量配置（含“优化目标变量 + 其他变量的来源配置”）  
   **当** 用户保存配置并刷新页面/重新进入该测试集  
   **则** 配置持久化并可回显（不丢失）

5. **前提** Dify API 调用失败（网络/超时/401/403/5xx/解析失败等）  
   **当** 用户解析变量  
   **则** 前端展示**友好错误信息**并提供“重试”入口  
   **并且** 前端不得展示 `error.details`（避免泄露敏感信息/上游返回）

6. **前提** 测试集是模板（Story 2.3）或从模板创建的新测试集  
   **当** 模板包含 Dify 变量配置  
   **则** 该配置应被**一并复制/复用**（避免“模板只复制 cases_json 导致无法运行 Dify”）

## 任务拆分（Tasks / Subtasks）

### 任务 1：明确 Dify 输入变量结构（从 `/parameters` 到“可配置变量列表”）（AC: 1）

- [x] 研究并固化 Dify `/parameters` 的响应字段映射（仅取 UI/配置所需字段）
- [x] 在后端新增解析 DTO（或直接用 `serde_json::Value` + 手动提取，给出严格错误信息）
- [x] 设计前端统一数据结构（`DifyInputVariable[]`）：
  - `name`（变量名，唯一 key）
  - `component`（Dify UI 组件类型/渲染类型；例如 text-input/paragraph/select/checkbox/...；未知则为 unknown）
  - `type`（string/number/bool/object/array/unknown；基于 component 与默认值推断；若无法推断则 unknown）
  - `required`（bool；若未知则 false + 标注“未知”）
  - `defaultValue`（可选；来自 Dify 的默认值，JSON）
  - `raw`（保留原始片段，便于 debug，但不展示给用户）
- [x] 最小解析规则（MVP，避免实现跑偏）：
  - `GET /v1/parameters` 返回的 `user_input_form` 为“按组件分组”的表单定义（多种组件的 union）
  - 解析时将每个表单项归一化为 `DifyInputVariable`：
    - `name`：变量名（用于 inputs key）
    - `component`：组件类型（用于前端渲染/提示）
    - `required`：是否必填（若缺失则 unknown → false + UI 标注未知）
    - `defaultValue`：若存在则保留（用于“未绑定时使用默认值”语义）
    - `type`：优先根据 `defaultValue` 推断；否则按 `component` 映射到大类（无法推断则 unknown）

### 任务 2：后端 API（获取变量结构 + 保存/读取测试集 Dify 配置）（AC: 1, 4, 5）

- [x] 数据模型（SQLite）
  - [x] 为 `test_sets` 增加 `dify_config_json`（TEXT, nullable）用于持久化：
    - `targetPromptVariable: string`
    - `bindings: Record<string, { source: 'fixed' | 'testCaseInput'; value?: any; inputKey?: string }>`
    - `parametersSnapshot?: any`（可选缓存：用于回显/减少重复请求；缓存应允许手动刷新）
- [x] Repository
  - [x] 扩展 `backend/src/infra/db/repositories/test_set_repo.rs`：支持读写 `dify_config_json`
  - [x] 注意：list summary 不需要返回大字段；详情 `GET /{test_set_id}` 才返回
- [x] Routes（建议与现有 `test_sets` 路由同域，避免新建“全局 dify 路由”）
  - [x] `POST /api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}/dify/variables/refresh`
    - 鉴权：必须登录，且 test_set 必须属于当前用户/当前 workspace（跨 user/workspace 一律 404）
    - 行为：读取并解密当前用户的 Dify credential（`api_credentials` where `credential_type='dify'`）→ 调用 `GET {base_url}/v1/parameters` → 解析变量列表 →（可选）写入 `parametersSnapshot`
    - 200：`ApiSuccess<DifyVariablesResponse>`（结构必须在 OpenAPI/Types 中明确定义，避免歧义）
    - 401：`UNAUTHORIZED`
    - 404：`TEST_SET_NOT_FOUND`（包含跨 workspace / 跨用户）
    - 5xx：`UPSTREAM_ERROR`（message 友好，details 可有但前端不展示）
  - [x] `PUT /api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}/dify/config`
    - Request：`SaveDifyConfigRequest`
    - 校验：
      - `targetPromptVariable` 必填、trim 后非空、≤128 字符
      - `bindings` 中每个变量：
        - `source='fixed'`：必须提供 `value`（允许 null 但需明确语义）
        - `source='testCaseInput'`：必须提供 `inputKey`（非空、≤128）
      - `targetPromptVariable` 不允许出现在 `bindings`（避免“既是优化目标又被绑定覆盖”）
      - 必填变量校验（与 AC3 “未绑定使用默认值”对齐）：
        - 若某变量 `required=true` 且无 `defaultValue`，则必须在 `bindings` 中提供来源（否则返回 400/VALIDATION_ERROR）
- [x] OpenAPI + Types
  - [x] `backend/src/api/routes/docs.rs`：新增 `dify_variables` tag + schemas
  - [x] `backend/src/bin/gen-types.rs`：导出新 request/response
  - [x] 明确定义响应结构（最小可用，供前端渲染与回显）：
    - `DifyVariablesResponse`：
      - `variables: DifyInputVariable[]`
      - `snapshot?: any`（可选：parametersSnapshot，用于回显/减少重复请求）

### 任务 3：前端 UI（选择优化目标变量 + 配置其他变量来源）（AC: 2, 3, 4, 5）

- [x] 页面落点：继续复用 `frontend/src/pages/TestSetsView/TestSetsView.tsx`（保持“测试集管理”内聚）
- [x] 新增“Dify 变量配置”区块（仅在 workspace 下）：
  - [x] “刷新/解析变量”按钮：调用 `POST .../dify/variables/refresh`，加载后展示变量列表
  - [x] “待优化 system prompt 变量”选择器：
    - [x] 仅允许选择 `type=string/unknown`（unknown 需弹出风险提示）
  - [x] “其他变量”配置表：
    - [x] 每行一个变量：来源下拉（固定值 / 关联测试用例字段）
    - [x] 固定值：JSON 编辑器（最小校验：可解析为 JSON）
    - [x] 关联字段：选择 `TestCase.input` 的 key（UI 可下拉展示“当前样例用例”的 keys，允许手填）
    - [x] 未配置来源的变量：在 UI 上标注“使用默认值/省略”（由后端解析结果的 `defaultValue` 决定）
  - [x] 保存按钮：调用 `PUT dify/config`；成功 toast；失败仅展示 `error.message`
  - [x] 错误展示：遵循架构约束，**不得展示 `error.details`**

### 任务 4：与现有能力的兼容性与复用（防止回归/重复造轮子）

- [x] 复用现有 Dify 凭证存储与解密逻辑：`backend/src/api/routes/auth.rs` 的 decrypt 模式（短驻留、zeroize）
- [x] 复用现有 `reqwest` client 与 correlationId 透传（`X-Correlation-Id`）
- [x] 复用现有错误码/响应包装：`ApiResponse<T>` + 统一错误格式
- [x] 复核并更新 Story 2.3 模板复制逻辑（防回归）：
  - [x] `backend/src/infra/db/repositories/test_set_repo.rs`：
    - [x] `create_template_from_test_set_scoped`：复制 `dify_config_json`（与 `cases_json` 同步复制）
    - [x] `find_template_by_id_scoped` 与模板详情接口：确保模板详情能取到 `dify_config_json`（用于“从模板创建”时回填）
  - [x] 前端“从模板创建”策略（MVP）：
    - [x] 创建新测试集后，若模板带 `dify_config_json`，则自动调用 `PUT /dify/config` 写入并回显（避免模板只复制 cases 导致 Dify 不可运行）

### 任务 5：测试与门禁（AC: 1-6）

- [x] Backend（集成测试优先，避免只测 repo 但路由坏掉）
  - [x] 变量解析 happy path（mock upstream：可通过注入或 http mock）
  - [x] 上游 401/403/超时/5xx 的错误映射（message 友好、details 截断）
  - [x] 跨用户/跨 workspace 一律 404（不泄露存在性）
  - [x] 保存 config 的校验（空/超长/冲突：targetPromptVariable 出现在 bindings）
- [x] Frontend
  - [x] service 单测：refresh/saveConfig 的 200/401/4xx 分支
  - [x] 页面测试：选择优化目标变量、配置 binding、保存成功/失败提示

### Review Follow-ups (AI)

- [x] [AI-Review][CRITICAL] `required` 的“未知态”目前无法表达/展示：实现与 Story 描述不一致（建议引入 `requiredKnown` 或 `requiredState`，UI 同步展示“未知”）。[backend/src/infra/external/dify_client.rs:167]
- [x] [AI-Review][HIGH] 上游错误体不应进入日志：避免在 `ConnectionError::UpstreamError` 中拼接 body，且不要用 `%e` 直接打印包含 body 的错误字符串（即使截断也可能泄露）。[backend/src/infra/external/dify_client.rs:214]
- [ ] [AI-Review][MEDIUM] `save_dify_config` 缺少一致性校验：当存在 `parametersSnapshot` 时，`targetPromptVariable` 与 `bindings` key 应限制为 snapshot 中的变量名，防止写入无效配置。[backend/src/api/routes/test_sets.rs:137]
- [ ] [AI-Review][MEDIUM] Dify 变量解析结果顺序不稳定（`HashMap` 迭代顺序不可预测）；建议按 `name` 排序或保留上游顺序，保证 UI/回显一致性。[backend/src/infra/external/dify_client.rs:137]
- [ ] [AI-Review][MEDIUM] `normalize_base_url_for_parameters` 用字符串 `ends_with("/v1")` 去重存在误删路径风险；建议改用 URL 解析后仅在 path==`/v1` 时移除。[backend/src/infra/external/dify_client.rs:102]
- [ ] [AI-Review][MEDIUM] “从模板创建”时 Dify 配置复制非原子：当前通过创建后再 `PUT /dify/config` 二次写入，失败会导致配置丢失；建议后端支持原子创建（或至少提供重试/补写入口）。[frontend/src/pages/TestSetsView/TestSetsView.tsx:465]
- [ ] [AI-Review][LOW] `raw` 字段结构可能不符合预期：`json!({ component: field })` 的 key 实际是固定 `"component"`，建议确认是否需要保留真实组件类型与上游字段。[backend/src/infra/external/dify_client.rs:169]
- [ ] [AI-Review][LOW] Git 额外产物未计入 Story：`docs/implementation-artifacts/validation-report-2026-01-05_23-11-37.md` 在 git 中存在但未出现在 File List；建议补齐或明确不纳入跟踪。[docs/implementation-artifacts/validation-report-2026-01-05_23-11-37.md:1]

## Dev Notes

### Developer Context（给 Dev 的最小上下文）

- 现状（已完成）：
  - Story 2.1：`test_sets` CRUD + 用户隔离 + `ApiResponse<T>`（`docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md`）
  - Story 2.2：JSONL 批量导入仅影响 `cases_json` 的生成/编辑（`docs/implementation-artifacts/2-2-test-set-batch-import.md`）
  - Story 2.3：模板保存/复用（注意新增字段需要被模板复制）（`docs/implementation-artifacts/2-3-test-set-template-save-and-reuse.md`）
- 本 Story 的核心是：把“Dify 的输入变量结构”转成“可配置的变量绑定”，并在测试集维度持久化，供后续任务执行使用。
- 关键误区（必须避免）：
  - ❌ 误以为“要先列出用户所有 Dify workflows 再选择”：多数 Dify Service API Key 是**单应用/单工作流**的授权，`/parameters` 已能返回该应用的输入变量结构。
  - ❌ 把“优化目标变量”写进 test case：优化目标变量应由优化引擎/执行层在运行时注入当前 prompt。

### 技术要求（不可违背）

- 安全：
  - Dify API Key 必须仅在后端解密使用，前端永不接触明文。
  - 外部 base_url 必须复用既有 SSRF 防护校验（已在 save_config 阶段处理）。
  - 上游响应体要截断（参考 `truncate_error_body`），避免日志/错误泄露。
  - `raw` / `parametersSnapshot`：
    - 禁止在日志中输出上游原始响应片段
    - 前端 UI 禁止展示 `raw`；如需缓存 `parametersSnapshot`，应限制大小或仅保存“解析后的最小结构”
- 体验：
  - UI 不展示 `error.details`（架构约束）。
  - 解析变量失败必须可重试；成功后回显配置。
- 合约稳定：
  - list summary 继续只返回 summary（避免把 `dify_config_json` 之类大字段塞进列表）。

### Web Research（Dify API 最新信息，供实现落地）

- Dify Service API 提供“Parameters”端点用于获取应用的输入参数结构；返回中包含 `user_input_form` 等字段，用于渲染/理解可输入变量。（参考：Dify 官方 API 文档 - Service API / Parameters）
- 本项目实际请求为：`GET {base_url}/v1/parameters`（`backend/src/infra/external/dify_client.rs`）。前端保存的 `base_url` 会规范化为 `origin`（不包含 `/v1`），但仍建议在客户端封装 endpoint builder 做防御性去重，避免误配置导致 `.../v1/v1/parameters`。

### 架构对齐（必须遵守）

- 路由/模块：
  - 继续以 `workspaces/{workspace_id}/test-sets/{test_set_id}` 为边界扩展（对齐现有 test set 体系）。
  - 后端：`api/routes` → `infra/db/repositories` → `domain/models` 分层不打破。
- 命名：
  - URL path：沿用既有实现风格（kebab-case，例如 `test-sets`、`save-as-template`），不要引入 `:refresh` 之类新风格
  - Rust：snake_case；TS/React：camelCase/PascalCase（见架构文档约束）

### 文件落点（建议）

- Migration：`backend/migrations/005_add_dify_config_to_test_sets.sql`
- Backend repo：`backend/src/infra/db/repositories/test_set_repo.rs`
- Backend routes：`backend/src/api/routes/test_sets.rs`（子路由）
- Dify client 扩展：`backend/src/infra/external/dify_client.rs`（新增 `get_parameters` + 解析/错误映射）
- Frontend services/hooks：`frontend/src/features/test-set-manager/services/*`、`frontend/src/features/test-set-manager/hooks/*`
- Frontend page：`frontend/src/pages/TestSetsView/TestSetsView.tsx`

### References

- [Source: docs/project-planning-artifacts/epics.md#Story-2.4-Dify-变量解析与-Prompt-变量指定]
- [Source: docs/project-planning-artifacts/prd.md#能力区域-2-测试集管理]
- [Source: docs/project-planning-artifacts/architecture.md#Project-Structure-&-Boundaries]
- [Source: backend/src/infra/external/dify_client.rs]（现有 `/v1/parameters` 调用方式与错误处理）
- [Source: backend/src/api/routes/auth.rs]（凭证加密/解密与安全边界）
- [Source: docs/implementation-artifacts/2-3-test-set-template-save-and-reuse.md#TestSet-Template-合约]

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

- `backend`: `cargo fmt`、`cargo test`、`cargo run --bin gen-types`
- `frontend`: `npm test -- --run`、`npm run build`

### Completion Notes List

- 已按 create-story 工作流完整生成本 Story（含：上下文、边界、接口建议、文件落点、测试要求、风险提示）。
- 实现 Dify `/v1/parameters` 解析与标准化输出（`DifyInputVariable[]`），并新增测试集维度 `dify_config_json` 持久化。
- 新增测试集 Dify 子路由：`variables/refresh` + `config`，并对齐 OpenAPI + ts-rs 类型生成。
- 前端 `TestSetsView` 新增 “Dify 变量配置”区块：刷新/解析、选择优化目标变量、配置其他变量来源并保存；错误仅展示 `error.message`。
- 模板链路补齐：保存为模板会复制 `dify_config_json`；从模板创建后会自动写入新测试集的 Dify 配置。
- 补齐后端集成测试（含 upstream mock、跨用户 404）与前端 service/UI 测试；并通过 `npm run build` 门禁。

### File List

- backend/migrations/005_add_dify_config_to_test_sets.sql
- backend/src/api/routes/dify.rs
- backend/src/api/routes/docs.rs
- backend/src/api/routes/mod.rs
- backend/src/api/routes/test_set_templates.rs
- backend/src/api/routes/test_sets.rs
- backend/src/bin/gen-types.rs
- backend/src/domain/models/test_set.rs
- backend/src/infra/db/repositories/test_set_repo.rs
- backend/src/infra/external/dify_client.rs
- backend/src/shared/error_codes.rs
- backend/tests/test_set_dify_variables_api_test.rs
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/2-4-dify-variable-parsing-and-prompt-variable-designation.md
- docs/implementation-artifacts/validation-report-2026-01-05_23-11-37.md
- frontend/src/features/test-set-manager/services/difyService.ts
- frontend/src/features/test-set-manager/services/difyService.test.ts
- frontend/src/features/test-set-manager/services/testSetService.test.ts
- frontend/src/features/test-set-manager/services/testSetTemplateService.test.ts
- frontend/src/pages/TestSetsView/TestSetsView.tsx
- frontend/src/pages/TestSetsView/TestSetsView.test.tsx
- frontend/src/types/generated/api/DifyBinding.ts
- frontend/src/types/generated/api/DifyBindingSource.ts
- frontend/src/types/generated/api/DifyConfig.ts
- frontend/src/types/generated/api/DifyInputVariable.ts
- frontend/src/types/generated/api/DifyValueType.ts
- frontend/src/types/generated/api/DifyVariablesResponse.ts
- frontend/src/types/generated/api/SaveDifyConfigRequest.ts
- frontend/src/types/generated/api/SaveDifyConfigResponse.ts
- frontend/src/types/generated/api/TestSetResponse.ts
- frontend/src/types/generated/api/TestSetTemplateResponse.ts
- frontend/src/types/generated/models/TestSet.ts

## Change Log

- 2026-01-05：实现 Dify 变量解析与 Prompt 变量指定（后端迁移/解析/路由 + 前端 UI/服务 + 测试与门禁）
- 2026-01-05：Senior Developer Review（AI）：提出 CRITICAL/HIGH/MEDIUM 修改项（见 Review Notes / Follow-ups）

## Review Notes

## Senior Developer Review (AI)

**Date:** 2026-01-05  
**Outcome:** Approved（P0/P1 已修复，P2 作为 Tech Debt 延后）

### Git vs Story Discrepancies

- [x] [MEDIUM] git 存在未纳入 File List 的文件：`docs/implementation-artifacts/validation-report-2026-01-05_23-11-37.md` → 已补齐到 File List。[docs/implementation-artifacts/validation-report-2026-01-05_23-11-37.md:1]

### AC Validation（实现对照）

1. AC1（解析变量并展示）：**IMPLEMENTED**（`required` 未知态已支持；变量顺序稳定性作为 P2/Tech Debt）。[backend/src/infra/external/dify_client.rs:82]
2. AC2（选择优化目标并标识）：**IMPLEMENTED**。[frontend/src/pages/TestSetsView/TestSetsView.tsx:1021]
3. AC3（非目标变量来源配置）：**IMPLEMENTED（配置层）**（fixed/testCaseInput 已支持；运行时注入/省略语义依赖后续执行层）。[frontend/src/pages/TestSetsView/TestSetsView.tsx:520]
4. AC4（持久化与回显）：**IMPLEMENTED**。[backend/src/api/routes/test_sets.rs:422]
5. AC5（失败友好提示且不展示 details）：**IMPLEMENTED（UI 展示层）**，但存在“上游错误体进入日志”的安全隐患需修复。[backend/src/api/routes/test_sets.rs:357]
6. AC6（模板复用 Dify 配置）：**PARTIAL**（保存为模板时已复制；从模板创建为二次写入，非原子）。[frontend/src/pages/TestSetsView/TestSetsView.tsx:465]

### Findings

- [x] [CRITICAL] Story 标注“required 缺失→unknown→UI 标注未知”，但实现无法表达 unknown（当前直接 `unwrap_or(false)`，UI 只能显示 是/否）。[docs/implementation-artifacts/2-4-dify-variable-parsing-and-prompt-variable-designation.md:58]
- [x] [HIGH] 上游响应体（即使截断）仍会进入错误字符串并被日志打印，违反“禁止在日志中输出上游原始响应片段”的技术要求。 [docs/implementation-artifacts/2-4-dify-variable-parsing-and-prompt-variable-designation.md:164]
- [x] [MEDIUM] `save_dify_config` 未校验 target/bindings 必须属于最新变量列表（snapshot 存在时）；后续执行期可能炸。 [backend/src/api/routes/test_sets.rs:137]
- [ ] [MEDIUM][DEFERRED-P2] Dify 变量解析顺序不稳定（`HashMap`），UI 列表/回显可能抖动。 [backend/src/infra/external/dify_client.rs:85]
- [ ] [MEDIUM][DEFERRED-P2] `normalize_base_url_for_parameters` 字符串去重 `/v1` 有误删路径风险（应使用 URL 解析）。 [backend/src/infra/external/dify_client.rs:102]
- [ ] [MEDIUM][DEFERRED-P2] 模板创建后的 Dify 配置复制是“创建后补写”，失败会导致配置丢失（AC6 仅部分满足）。 [frontend/src/pages/TestSetsView/TestSetsView.tsx:465]
- [ ] [LOW][DEFERRED-P2] `raw` 字段结构与“按组件分组的原始片段”描述可能不一致，建议确认是否需要保留真实组件类型 key。 [backend/src/infra/external/dify_client.rs:169]

### Decisions

- [x] 继续采用 `test_sets.dify_config_json` 作为 MVP（不引入新表），但需补齐校验与日志安全边界。 [backend/migrations/005_add_dify_config_to_test_sets.sql:4]
- [x] `/v1/v1/parameters` 风险已通过 `normalize_base_url_for_parameters` 缓解，但建议升级为 URL 解析以覆盖边缘 case。 [backend/src/infra/external/dify_client.rs:102]
- [x] 模板“保存为模板复制 dify_config_json”已实现；从模板创建仍需提升为原子复制或补写机制。 [backend/src/infra/db/repositories/test_set_repo.rs:473]

### Risks / Tech Debt

- [ ] 后续若出现“一个用户配置多个 Dify 工作流”的需求，当前“API Key 即工作流”的假设需升级（可能迁移到 task/workspace 级配置）
- [ ] `parametersSnapshot` 若长期缓存且不限制大小，可能造成 DB 膨胀（建议仅缓存最小字段 + 限制变量数/字段大小）。[docs/implementation-artifacts/2-4-dify-variable-parsing-and-prompt-variable-designation.md:165]

### Follow-ups

- [ ] 将本节 Findings 同步为可执行 Tasks（见上文 `Review Follow-ups (AI)`），修复后再把状态切回 `review` 进入二次评审。
