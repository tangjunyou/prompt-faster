# Story 2.5: 通用 API 自定义变量与固定任务标准答案

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## 用户故事

**As a** Prompt 优化用户，  
**I want** 在“通用 API”执行目标下为测试集声明/维护自定义输入变量，并在固定任务模式下为每条测试用例填写标准答案，  
**So that** 系统后续可以正确构造通用大模型请求，并对固定任务输出做精确匹配评估。

## 验收标准（Acceptance Criteria）

### AC1：通用 API 自定义输入变量（FR11）

**Given** 用户在测试集编辑页选择执行目标为「通用 API」  
**When** 用户打开测试集配置面板  
**Then** 显示“自定义变量编辑器”  
**And** 用户可以新增/编辑/删除变量  
**And** 每个变量至少支持以下字段：
- `name`：变量名（非空、唯一、<= 128 字符）
- `valueType`：变量类型（至少覆盖 `string | number | boolean | json`）
- `defaultValue`：默认值（可空；若存在需与 `valueType` 可兼容）

**Given** 用户保存测试集  
**When** 用户重新进入该测试集  
**Then** 自定义变量配置可正确回显  
**And** 从“测试集模板”创建测试集时，自定义变量配置会被一并复制（不丢字段）

### AC2：固定任务标准答案（FR12）

**Given** 测试集任务模式为「固定任务」  
**When** 用户编辑测试用例  
**Then** 提供“标准答案”输入控件  
**And** 标准答案与测试用例一起持久化保存  
**And** 标准答案落盘结构与现有 `TestCase.reference.Exact.expected` 一致（保持与算法 DTO 对齐）  
**And** 用户切换到 `cases` JSON 编辑/导入视图时，已填写的标准答案应可在 `reference.Exact.expected` 中直接看到（保持单一事实来源）  
**And** 系统后续可使用“精确匹配（trim 后全等）”作为固定任务的默认评估方式（评估实现可在后续 Story 落地）

### AC3：兼容性与约束

- 继续保留 `cases` 作为最终数据源（JSON 编辑 / JSONL 导入仍可用），新增 UI 只是更友好的编辑方式。
- 不能破坏既有 Dify 流程：`dify_config_json` 仍为可选字段，且与通用 API 配置可共存。
- 列表接口继续仅返回 summary（避免把新增大字段塞进 list）。
- 通用变量配置保存需做大小上限校验（建议上限：`<= 32KB`），超限返回校验错误（防止 DB/日志膨胀）。

## 任务拆分（Tasks / Subtasks）

### 任务 1：数据模型与持久化（AC1）

- [x] 设计并落地“通用 API 自定义变量配置”的数据结构（建议测试集维度存储，便于模板复用）
- [x] 新增 DB 字段用于持久化该配置（参考 Story 2.4 的 `dify_config_json` 实现方式，保持一致的迁移与回滚策略）
- [x] 确保模板保存/复用会复制该配置（后端复制 + 前端预填，均需覆盖 `generic_config_json`；对齐 Story 2.3/2.4 的模板链路）

### 任务 2：后端 API（AC1）

- [x] 在 `workspaces/{workspace_id}/test-sets/{test_set_id}` 边界内新增/扩展接口以读写通用变量配置
- [x] 完成输入校验（变量名长度/唯一性、默认值与类型兼容性、避免危险字段膨胀）
- [x] 保存通用变量配置时做 payload 大小校验（建议 `<= 32KB`），避免超大 JSON 进入 DB/日志
- [x] 确保 OpenAPI/类型生成链路更新（对齐现有 `gen-types` 工作流）

### 任务 3：前端 UI（AC1）

- [x] 在测试集编辑页新增“通用 API 自定义变量编辑器”（与 Dify 变量配置并列，但互不干扰）
- [x] 支持新增/编辑/删除变量；创建态可编辑并在“创建测试集”后自动写入；编辑态支持独立保存与回显；支持“一键禁用并清空”
- [x] 与模板创建流程联动：从模板创建后自动预填（含通用变量配置），并在创建请求中一并写入（原子；允许用户创建前调整）

### 任务 4：固定任务标准答案编辑（AC2）

- [x] 在固定任务模式下提供“标准答案”编辑能力，并将其写入 `cases[*].reference.Exact.expected`
- [x] 兼容现有 `cases` JSON：如果用户直接编辑 JSON，UI 应能容错/不破坏数据

### 任务 5：测试与门禁（AC1-AC3）

- [x] 后端：接口测试覆盖（成功/校验失败/越权/模板复制）
- [x] 前端：`TestSetsView` 交互测试覆盖（创建态/编辑态新增变量、保存/自动写入、回显、模板创建链路、32KB 预校验）

### Review Follow-ups（AI）

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 修正前端 services 落点：`frontend/src/features/test-set-manager/services/genericConfigService.ts`（与 `difyService.ts` 并列）
- [x] [AI-Review] 明确不在本 Story 落地 `userPromptInputKey`/Prompt 模板选择（归属 ExecutionTarget/任务配置）
- [x] [AI-Review] 将 `generic_config_json` payload 上限（建议 `<= 32KB`）写入 AC/Tasks，并要求后端校验
- [x] [AI-Review] 模板链路（后端复制 + 前端预填）需覆盖 `generic_config_json`
- [x] [AI-Review] AC2 补充 UI 与 `cases` JSON（`reference.Exact.expected`）一致性描述

## Dev Notes

### Developer Context（给 Dev 的最小上下文）

- 已完成（可复用的既有能力）：
  - Story 2.1：测试集 CRUD + 用户/工作区隔离 + list summary vs detail 约定（见历史实现与测试）
  - Story 2.2：JSONL 导入解析与最小校验（`cases[*].id/input/reference`）
  - Story 2.3：测试集模板保存/复用（新增字段需要被模板链路复制，避免回归）
  - Story 2.4：Dify 变量解析与 `dify_config_json` 持久化（含 refresh/save API、前端绑定 UI、模板复制）
- 本 Story 的目标是补齐两块“通用 API/固定任务”底座能力：
  1) **通用 API 自定义变量（FR11）**：为测试集提供“变量 schema/默认值/类型”的配置入口与持久化（供后续通用执行引擎构造 input 使用）。
  2) **固定任务标准答案（FR12）**：提供更友好的编辑方式，把答案写入既有 `cases[*].reference.Exact.expected`（算法 DTO 已支持）。
- **重要现状**：`cases` JSON 仍是最终权威数据源；本 Story 的 UI/配置只能“帮助用户写对 JSON”，不能引入第二份事实来源导致分叉。

### 技术要求（不可违背）

- 数据结构对齐（强约束）：
  - 固定任务标准答案必须落到 `TestCase.reference.Exact.expected`（见 `backend/src/domain/models/algorithm.rs`）。
  - 不要发明新的 “expectedAnswer” 平铺字段，避免与算法 DTO 脱节。
- 兼容性（强约束）：
  - 仍然支持用户直接编辑 `cases` JSON / 通过 JSONL 导入 `cases`；新 UI 必须容错。
  - `dify_config_json` 仍为可选字段，且与通用变量配置可共存；不能互斥或互相覆盖。
  - 列表接口继续只返回 summary，避免把新增大字段塞进 list（对齐既有 `TestSetListItemResponse`）。
- 安全与错误处理（强约束，架构级）：
  - API 响应继续使用 `ApiResponse<T>`；错误结构不变。
  - 前端不得直接展示 `error.details`（见 `docs/project-planning-artifacts/architecture.md` 的 MUST 约束）。
  - 任何可疑/超大 JSON 需要做大小上限与输入校验（通用变量配置 `generic_config_json` 建议上限 `<= 32KB`，并在后端校验）。

### 架构对齐（必须遵守）

- 路由边界：继续以 `workspaces/{workspace_id}/test-sets/{test_set_id}` 为资源边界扩展（对齐 Story 2.4 的 Dify 子路由风格）。
- 命名约定：Rust snake_case；TypeScript camelCase；跨语言边界 `#[serde(rename_all = "camelCase")]`（见架构文档）。
- 变更最小化：优先复用 Story 2.4 的“配置 JSON 持久化 + 单独 save endpoint + 模板复制”模式，避免重复造轮子。

### 设计建议（落地导向，减少实现分歧）

#### 1) 通用变量配置（FR11）

建议将“通用 API 自定义变量配置”存为 **测试集维度 JSON 字段**（与 `dify_config_json` 并列）：

- 字段名建议：`generic_config_json`（Option<String>）
- 配置建议结构（示意）：
  - `variables: Array<{ name, valueType, defaultValue }>`

执行语义（后续 Story/模块使用）：
- 构造通用执行 input 时：`effectiveInput = defaults(variables) + testCase.input`（同名 key 以 testCase.input 覆盖默认值）。
  - 说明：本 Story **不**定义/落地 “User Prompt 的输入 key / Prompt 模板选择”等执行语义；该能力更偏执行目标/任务配置（参见算法规格中的 `ExecutionTargetConfig::DirectModel.user_prompt_template`），后续 Story 再实现。

#### 2) 固定任务标准答案（FR12）

- 固定任务（推荐默认）：`reference = { Exact: { expected: string } }`
- 创意/约束任务：后续 Story 2.6 扩展 `Constrained/Hybrid` 的 UI；本 Story 只需要：
  - 保证 JSON 编辑与最小校验支持这几种变体（现有已经做了 “Exact/Constrained/Hybrid” 校验）。
  - 在 UI 中提供“标准答案”便捷编辑入口（写入 `Exact.expected`）。

### 文件落点（建议）

- Migration：`backend/migrations/006_add_generic_config_to_test_sets.sql`（新增 `generic_config_json TEXT`）
- Domain：`backend/src/domain/models/test_set.rs`（新增 `generic_config_json: Option<String>`）
- Repo：`backend/src/infra/db/repositories/test_set_repo.rs`（读写该字段；与 `dify_config_json` 对齐）
- Routes：
  - `backend/src/api/routes/test_sets.rs`：扩展 detail 响应返回 `generic_config`（list summary 不返回）
  - 新增/扩展子路由：`/{test_set_id}/generic/config`（保存配置）
- Templates：
  - `backend/src/api/routes/test_set_templates.rs`：模板保存/创建时复制 `generic_config_json`
- Frontend：
  - types：走现有 `backend/src/bin/gen-types.rs` 生成链路
  - UI：`frontend/src/pages/TestSetsView/TestSetsView.tsx` 增加“通用 API 自定义变量”卡片
  - services：仿照 `frontend/src/features/test-set-manager/services/difyService.ts` 的组织方式新增 `frontend/src/features/test-set-manager/services/genericConfigService.ts`（与 `difyService.ts` 并列）

### 测试要求（必须覆盖）

- 后端（集成测试，仿照 `backend/tests/test_set_dify_variables_api_test.rs`）：
  - 保存/读取通用变量配置成功
  - 输入校验失败（空变量名、超长、重复名、默认值类型不匹配、超大 payload）
  - 模板复制链路不丢 `generic_config_json`
  - 权限与隔离：跨用户/跨 workspace 访问一律 404
- 前端（仿照 `frontend/src/pages/TestSetsView/TestSetsView.test.tsx`）：
  - 新增变量 → 保存 → 回显
  - 模板创建后自动预填（含通用变量配置）并写回
  - “标准答案”编辑能正确写入 `cases[*].reference.Exact.expected`，且不破坏用户直接编辑 JSON 的能力

### Web Research（OpenAI 兼容 API 最新信息，供后续实现落地）

> 说明：本项目通用大模型接入是 OpenAI 兼容 API（当前 provider 白名单：`siliconflow` / `modelscope`，见 `backend/src/infra/external/llm_client.rs`）。

- 通用测试连接：通常使用 `GET {base_url}/v1/models` 校验凭证可用（本项目已采用该策略）。
- Chat Completions（后续执行引擎常用）：`POST {base_url}/v1/chat/completions`（OpenAI 兼容语义，具体以 provider 文档为准）。
- DashScope（ModelScope）兼容模式 base_url 已在代码中内置：`https://dashscope.aliyuncs.com/compatible-mode`。

### References

- [Source: docs/project-planning-artifacts/epics.md#Story-2.5-通用-API-自定义变量与固定任务标准答案]
- [Source: docs/project-planning-artifacts/prd.md#能力区域-2-测试集管理]（FR11/FR12）
- [Source: docs/project-planning-artifacts/architecture.md#Project-Structure-&-Boundaries]（结构/命名/错误展示约束）
- [Source: docs/project-planning-artifacts/architecture.md#Error-Handling-Layers]（不得直出 `error.details`）
- [Source: backend/src/domain/models/algorithm.rs]（`TestCase.reference.Exact.expected`）
- [Source: frontend/src/pages/TestSetsView/TestSetsView.tsx]（`cases` JSON 编辑与最小校验、现有 UI 形态）
- [Source: backend/src/infra/external/llm_client.rs]（provider 白名单与 `/v1/models` 测试连接）

### Previous Story Intelligence（来自 Story 2.4，可复用/避免回归）

- **配置 JSON 持久化模式**：对齐 `dify_config_json`（单字段 JSON + 独立 save endpoint + detail 才返回解析后的结构）。
- **模板链路最容易回归**：Story 2.3/2.4 已验证“模板复制必须包含新增字段”；本 Story 新增字段也必须一并复制（建议加集成测试锁死）。
- **安全/UX 边界**：前端不接触明文 API Key；错误展示不直出 `error.details`；上游响应体需截断。
- **接口风格**：list summary 不夹带大字段；详情页按需拉取并渲染配置面板（避免性能问题）。

### Git Intelligence Summary（最近提交的工程约定）

- 最近提交与改动集中在“测试集管理”链路，且已形成一致约定：
  - 迁移文件按序号递增（`00x_*.sql`），新增字段优先走“新增列”方案。
  - routes 按资源聚合（`api/routes/test_sets.rs` + 相关子路由），并同步 OpenAPI/types 生成。
  - 测试优先补齐后端集成测试与前端交互测试，避免 Story 回归。
- 近 5 次提交（用于理解改动惯例）：
  - `63295f8` Story 2.4: Dify variables config
  - `12cf8b0` Story 2.3: test set templates + review fixes
  - `debd9b4` feat(test-sets): harden JSONL batch import (story 2.2)
  - `9ae64b7` chore: rustfmt
  - `cbab13d` Story 2.1: test set CRUD + list summary

### Latest Tech Information（通用 OpenAI 兼容 API：provider 侧注意点）

> 目的：避免实现时用错 endpoint / header / base_url 形态导致联调失败。以当前代码白名单 provider 为准（`siliconflow` / `modelscope`）。

- 连接测试：`GET {base_url}/v1/models`（本项目已用该端点验证凭证）。
- 执行请求（常见）：`POST {base_url}/v1/chat/completions`（OpenAI 兼容 Chat Completions）。
- DashScope（ModelScope）：
  - base_url：`https://dashscope.aliyuncs.com/compatible-mode`
  - 兼容模式下走 OpenAI 路径（如 `/v1/chat/completions`、`/v1/models`），但模型名/限流策略以其文档与控制台为准。
- SiliconFlow：
  - base_url：`https://api.siliconflow.cn`
  - 同样使用 OpenAI 路径（如 `/v1/models`、`/v1/chat/completions`）。
- 统一建议：
  - base_url 入库时做规范化（trim trailing `/`），构造 URL 时防御性 `trim_end_matches('/')`，避免拼出 `...//v1/...`。
  - 日志/错误信息一律脱敏与截断（已有 `truncate_error_body`，复用即可）。

### Project Context Reference

- 未找到 `**/project-context.md`（本项目以 `docs/project-planning-artifacts/*` 与 `docs/analysis/*` 作为主要上下文来源）。

### Story Completion Status

- Story Key：`2-5-generic-api-custom-variables-and-fixed-task-answers`
- 状态：`done`
- 下一步：无

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

### Completion Notes List

- 2026-01-05：补齐本 Story 的审查结论（修正前端 services 落点、明确 `userPromptInputKey` 不在本 Story 落地、补充 `generic_config_json` 大小上限与 AC2 一致性描述）。
- 2026-01-05：补齐创建态“通用 API 自定义变量”编辑与自动写入；修复 `generic_config_json` 空字符串容错；补前端创建态写入与 32KB 预校验测试。
- 2026-01-05：将模板/创建链路改为原子创建（create 时一并写入 dify/generic 配置）；新增通用变量“一键禁用并清空”（前端按钮 + 后端 DELETE 端点）；更新类型生成与测试。
- 2026-01-06：补齐 Story 2.5 的用户故事/AC/任务拆分与 Dev Notes（含复用点、强约束、文件落点、测试门禁）。
- 2026-01-06：实现通用 API 自定义变量配置（`generic_config_json` 持久化/接口/模板复制 + 前端编辑器/模板写回）与固定任务标准答案编辑（写入 `cases[*].reference.Exact.expected`），并补齐后端/前端测试门禁。

### File List

- `docs/implementation-artifacts/2-5-generic-api-custom-variables-and-fixed-task-answers.md`

- `docs/implementation-artifacts/sprint-status.yaml`
- `docs/implementation-artifacts/validation-report-2026-01-06_02-49-30.md`

- `backend/migrations/006_add_generic_config_to_test_sets.sql`
- `backend/src/api/routes/docs.rs`
- `backend/src/api/routes/generic.rs`
- `backend/src/api/routes/mod.rs`
- `backend/src/api/routes/test_set_templates.rs`
- `backend/src/api/routes/test_sets.rs`
- `backend/src/bin/gen-types.rs`
- `backend/src/domain/models/test_set.rs`
- `backend/src/infra/db/repositories/test_set_repo.rs`
- `backend/tests/test_set_generic_config_api_test.rs`

- `frontend/src/features/test-set-manager/services/genericConfigService.ts`
- `frontend/src/features/test-set-manager/services/testSetService.test.ts`
- `frontend/src/features/test-set-manager/services/testSetTemplateService.test.ts`
- `frontend/src/pages/TestSetsView/TestSetsView.tsx`
- `frontend/src/pages/TestSetsView/TestSetsView.test.tsx`
- `frontend/src/types/generated/api/GenericConfig.ts`
- `frontend/src/types/generated/api/GenericInputVariable.ts`
- `frontend/src/types/generated/api/GenericValueType.ts`
- `frontend/src/types/generated/api/SaveGenericConfigRequest.ts`
- `frontend/src/types/generated/api/SaveGenericConfigResponse.ts`
- `frontend/src/types/generated/api/TestSetResponse.ts`
- `frontend/src/types/generated/api/TestSetTemplateResponse.ts`
- `frontend/src/types/generated/models/TestSet.ts`

## Change Log

- 2026-01-05：完善通用变量创建态编辑/撤销（禁用清空）与模板/创建原子写入；新增 `DELETE /generic/config`；更新 types 与测试。
- 2026-01-06：实现通用 API 自定义变量配置（后端字段/接口/模板复制 + 前端编辑器/模板写回）与固定任务标准答案编辑（写入 `cases[*].reference.Exact.expected`），并补齐后端/前端测试门禁。
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [CRITICAL] 修正前端 services 落点：不应创建 `services/tests` 子目录，应与现有 `difyService.ts` 并列放置
- [HIGH] `userPromptInputKey`/Prompt 模板属于执行目标/任务配置语义，不应混入本 Story 的测试集变量配置
- [MEDIUM] 明确 `generic_config_json` 大小上限（建议 `<= 32KB`）并在后端校验，避免 DB/日志膨胀
- [MEDIUM] AC2 补充：UI 与 `cases` JSON 的一致性可验收（标准答案落到 `reference.Exact.expected`）

### Decisions

- 保持 `generic_config_json` 与 `dify_config_json` 并列：避免大范围重构，且与现有实现模式一致
- 不在本 Story 落地 “User Prompt 模板/输入 key 选择”：避免把执行目标语义塞进测试集编辑，减少返工风险

### Risks / Tech Debt

- 后续实现“通用执行引擎/DirectModel”时，需要在任务配置或执行目标配置中定义 user prompt 模板（与算法规格对齐）；届时再决定是否需要从测试集复用某些变量定义

### Follow-ups

- （无）
