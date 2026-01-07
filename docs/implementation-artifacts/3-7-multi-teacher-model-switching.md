# Story 3.7: 多老师模型切换

Status: done

Story Key: `3-7-multi-teacher-model-switching`

Related FRs: FR51（不同任务使用不同老师模型）

Epic: Epic 3（优化任务配置与工作区）

Dependencies:
- Epic 1：老师模型（通用大模型）凭证配置、连接测试与持久化（Story 1.3/1.4/1.5）
- Epic 3：优化任务 CRUD + 任务配置（Story 3.1/3.4）

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,  
I want 为不同的优化任务配置不同的老师模型,  
so that 我可以根据任务特点选择最合适的模型。

## Scope Clarification（避免“多老师模型”的歧义）

本仓库当前全局仅支持 **1 组通用大模型凭证**（见 `api_credentials` 的 `UNIQUE(user_id, credential_type)` 约束）。  
因此本 Story 的“多老师模型”在 MVP 范围内定义为：**同一 Provider/同一凭证下，不同任务选择不同的 `model_id`**（来自 OpenAI 兼容 `/v1/models` 列表）。

**非目标（本 Story 不做）：**
- 不支持“为同一用户保存多组通用大模型凭证/多 Provider 并存”（需要迁移 `api_credentials` 唯一约束 + 全局配置 UI 重构）
- 不实现执行引擎真正调用老师模型（Epic 4 执行阶段落地）；本 Story只负责“可配置 + 可持久化 + 可回显 + 可展示”

## Acceptance Criteria

### AC1：任务配置页提供“老师模型”选择（FR51）

**Given** 用户在任务配置页面（`/workspaces/:id/tasks/:taskId`）  
**When** 用户进入“老师模型”配置区域  
**Then** 展示可选老师模型列表（基于已保存的通用大模型凭证调用 `/v1/models` 获取）  
**And** 用户可以为当前任务选择一个 `model_id`（或选择“系统默认/不覆盖”）

**And（缺失全局配置时的体验）**  
**Given** 用户尚未保存通用大模型凭证或无法获取模型列表  
**When** 打开任务配置页的“老师模型”区域  
**Then** 显示清晰的提示与引导入口（链接到 `/settings/api`）  
**And** 不允许保存“覆盖老师模型”（避免保存无效配置）

### AC2：保存任务配置后仅影响当前任务（FR51）

**Given** 用户为任务 A 选择老师模型 `model_id = X` 并保存  
**When** 用户查看任务 A 的配置（刷新页面/重新进入）  
**Then** 配置回显为 `X`（后端规范化后的值为准）

**And**  
**Given** 用户另有任务 B  
**When** 查看任务 B 的配置  
**Then** 任务 B 的老师模型配置不受任务 A 影响（仍为其各自选择或系统默认）

### AC3：任务列表展示各任务的老师模型名称（FR51）

**Given** 用户在任务列表页（`/workspaces/:id/tasks`）  
**When** 查看任务卡片/列表项  
**Then** 每个任务显示其老师模型名称（建议显示 `model_id`；未覆盖则显示“系统默认”）  
**And** 不引入 N+1 请求（后端列表接口直接提供展示字段）

### AC4：安全与错误处理（硬性）

**Given** 任何与老师模型列表/选择相关的 API 调用失败  
**When** 返回错误给前端  
**Then** 只展示 `error.message`（不得展示 `error.details`）  
**And** 不得在日志/UI 中泄露明文 API Key（延续 `shared/log_sanitizer.rs` 与既有 auth.rs 脱敏策略）

## Tasks / Subtasks

### 任务 1：后端 - 提供“可用老师模型列表”查询 API（AC1）

- [x] 在 `backend/src/api/routes/auth.rs` 增加受保护端点（需要登录）：
  - `GET /api/v1/auth/generic-llm/models`
  - 返回：`ApiResponse<{ models: string[] }>`（只返回模型 id 列表；不返回任何凭证信息）
- [x] 实现逻辑（必须按既有安全边界复用，不得另起“解密捷径”）：
  - [x] 使用 `CredentialRepo::find_by_user_and_type(user_id, CredentialType::GenericLlm)` 获取已保存凭证（不存在则返回空列表 + 友好 message）
  - [x] 使用 `AppState.api_key_manager.decrypt_bytes(...)` 解密 API Key（仅在 handler 内短生命周期使用；不得写日志；不得返回）
  - [x] 复用 `infra/external/llm_client.rs` 的 `/v1/models` 请求逻辑获取模型列表（可直接调用或抽取共享函数）
  - [x] 错误映射需与 `test_generic_llm_connection` 风格一致（401/403/timeout/upstream），并确保错误消息不包含敏感信息

### 任务 2：后端 - 扩展 OptimizationTaskConfig 支持老师模型选择（AC2/AC3）

- [x] 在 `backend/src/domain/models/optimization_task_config.rs` 扩展 `OptimizationTaskConfig`：
  - [x] 新增字段（建议命名避免与 `evaluator_config.teacher_model` 混淆）：
    - `teacher_llm: TeacherLlmConfig`
    - `TeacherLlmConfig` 至少包含：`model_id: Option<String>`（`None` 表示“系统默认/不覆盖”）
  - [x] 默认值：`model_id = None`
  - [x] 向后兼容：旧 `config_json` 缺字段 → 规范化默认值（不得 bump schema 也可；如 bump，必须保证读取兼容）
  - [x] 保持 `extra` 保留策略与“解析失败保护”（见 Story 3.4 的既有实现）
- [x] 在 `backend/src/api/routes/optimization_tasks.rs`：
  - [x] 扩展 `UpdateOptimizationTaskConfigRequest` 加入 `teacher_llm`（全量更新语义不变）
  - [x] `PUT .../config` 写入/回显时包含该字段
  - [x] 最小校验：
    - `model_id` 允许为 `null`（系统默认）
    - 若非空：`trim()` 后不得为空；长度建议 ≤ 128；不得包含控制字符
- [x] 在 `OptimizationTaskListItemResponse` 增加“展示字段”（避免前端 N+1）：
  - [x] 例如：`teacher_model_display_name: String`（`model_id` 或 “系统默认”）
  - [x] 由后端在列表响应中从 `config_json` 解析/规范化得到（列表规模小，允许在 Rust 层做一次解析）

### 任务 3：前端 - 任务配置页“老师模型”UI（AC1/AC2）

- [x] 在 `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx` 增加“老师模型”配置区（建议放在“高级配置”内，与其他算法配置同域；但文案需明确“仅影响该任务”）
- [x] 新增获取模型列表的 query（TanStack Query）：
  - [x] 建议位置：`frontend/src/features/task-config/` 下新建 `services/teacherModelService.ts` 与 `hooks/useTeacherModels.ts`
  - [x] queryKey 建议：`['teacherModels']`（用户级；不带 workspaceId）
  - [x] 当接口返回空列表时：展示提示 + `/settings/api` 链接
- [x] 保存时把 `teacher_llm.model_id` 写入 `UpdateOptimizationTaskConfigRequest`（选择“系统默认”则写 `null`）
- [x] UI 规则（写死，避免实现分歧）：
  - [x] 下拉选项第一项固定为：`系统默认（不覆盖）`
  - [x] 当模型列表加载失败/为空时：禁用“覆盖选择”的保存（但不影响用户保存其他配置）
  - [x] 错误展示只用 `error.message`

### 任务 4：前端 - 任务列表展示老师模型（AC3）

- [x] 在 `frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx` 的任务卡片中新增一行：
  - `老师模型：{teacher_model_display_name}`
- [x] 不新增额外请求；依赖后端 `OptimizationTaskListItemResponse` 新字段

### 任务 5：类型生成与 OpenAPI 同步（必做）

- [x] 更新 `backend/src/api/routes/docs.rs`（如新增路由/Schema 需注册）
- [x] 运行并提交生成类型：`cd backend && cargo run --bin gen-types`
  - [x] 确保 `frontend/src/types/generated/**` 全量入库（遵循 Story 3.4 的审查结论，避免 CI/同事环境漂移）

### 任务 6：回归测试（必做，防回归）

- [x] 后端（`backend/tests/*`）：
  - [x] 新增/扩展集成测试覆盖：
    - [x] `GET /api/v1/auth/generic-llm/models`：未配置返回空；配置存在时返回模型数组（建议复用 wiremock 模拟 `/v1/models`）
    - [x] 更新任务配置写入 `teacher_llm.model_id` 后，`GET task` 回显一致
    - [x] `list optimization-tasks` 响应包含正确的 `teacher_model_display_name`
- [x] 前端（Vitest + Testing Library）：
  - [x] `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx`：模型列表加载→选择→保存 payload 含 `teacher_llm`
  - [x] `frontend/src/pages/OptimizationTasksView/OptimizationTasksView.test.tsx`：列表渲染包含老师模型展示字段

## Dev Notes

### Guardrails（必须遵循）

- 不得引入新 UI 组件依赖（优先用现有 `select` + `Input/Button`），避免 Story 3.5/3.6 同类依赖漂移风险
- 技术版本以仓库锁定版本为准（`backend/Cargo.toml` / `frontend/package.json`），不得“顺手升级”Axum/SQLx/React 等依赖
- 任何 API 改动必须配套 `gen-types` 生成并入库，避免 DTO 漂移
- 严格遵循“workspace 作为第一隔离边界 + user 作为权限边界”（任务配置更新/读取必须是 user+workspace scoped）
- UI 层不得展示 `error.details`（只显示 `error.message`）

### Previous Story Intelligence（从已完成 Story 提炼的可复用结论）

- Story 3.4 已明确：`config_json` **必须**保留 unknown fields（`extra`）且更新时启用“解析失败保护”，避免把损坏 JSON 覆盖回默认导致数据丢失；本 Story 扩展配置字段必须复用该策略。
- Story 3.6 已反复验证：前端错误提示只展示 `error.message`，并且任何 destructive/敏感动作都要“写死规则 + 测试覆盖”，避免实现分歧与回归。

### Git Intelligence Summary（最近实现惯例）

- 最近提交集中在 Story 3.4~3.6（近 5 条摘要）：
  - `c337b8c` Merge pull request #7 (Story 3.6 删除与隔离修复)
  - `67b3278` fix(workspace): harden deletion flow + tests (story 3.6)
  - `ecaeecc` Story 3.5: workspace switcher + logout cleanup
  - `951a0d2` Story 3.4: core algorithm params + defaults
  - `3f21b0e` Merge pull request #6 (Story 3.3 审查修复)
- 近期惯例：遵循“API/类型生成入库 + 前后端测试同提 + story 文档同步更新”的节奏；本 Story 延续该惯例，不做依赖升级与无收益重构。

### Latest Technical Information（Web Research 摘要）

- OpenAI 兼容 Provider 的“模型列表”获取通常通过 `GET /v1/models`；本仓库已在 Story 1.4 用该端点验证通用大模型凭证，并将 `models` 列表用于 UI 预览。本 Story 仅把该能力扩展为“受保护、基于已保存凭证的 models 列表查询”，不引入新上游协议。

### Security Notes（必须写死）

- 模型列表接口只能返回模型 `id` 列表；不得返回 base_url/api_key；不得返回脱敏 key（没必要）
- 解密 API Key 仅允许发生在 `api_key_manager`，并尽量缩短明文驻留时间（可参考 `auth.rs` 的 `decrypt_and_mask` 注释）
- 日志里不得出现明文 key；上游错误 body 必须截断（复用 `infra/external/http_client.rs::truncate_error_body`）

### References

- 需求来源：`docs/project-planning-artifacts/epics.md`（Story 3.7）
- PRD：`docs/project-planning-artifacts/prd.md`（“新建任务流程”中的“老师模型：服务商/模型选择”）
- UX：`docs/project-planning-artifacts/ux-design-specification.md`（任务配置步骤中“老师模型可选，默认全局配置，仅需时覆盖”）
- 架构约束：`docs/project-planning-artifacts/architecture.md`（前端不得展示 `error.details`；API/类型生成与目录边界）
- 前置实现（连接测试返回 models）：`docs/implementation-artifacts/1-4-api-connection-test.md`
- 全局配置/解密边界：`backend/src/api/routes/auth.rs`、`backend/src/infra/external/llm_client.rs`、`backend/src/infra/external/api_key_manager.rs`
- 任务配置 schema：`backend/src/domain/models/optimization_task_config.rs`（Story 3.4 约束：extra 保留 + 解析失败保护）
- 任务 API：`backend/src/api/routes/optimization_tasks.rs`
- 任务列表/配置页：`frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx`、`frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

### Completion Notes List

 - 实现受保护接口 `GET /api/v1/auth/generic-llm/models`，基于已保存通用大模型凭证解密并调用 `/v1/models` 获取模型 id 列表。
 - 扩展 `OptimizationTaskConfig` 增加 `teacher_llm.model_id`（任务级覆盖，默认系统默认/不覆盖），并在任务列表响应中提供 `teacher_model_display_name` 避免前端 N+1。
 - 前端任务配置页新增老师模型选择（含缺失配置引导到 `/settings/api`），任务列表页展示老师模型。
 - 已运行并提交 `gen-types` 生成类型；后端/前端测试通过。

### File List

- backend/src/api/routes/auth.rs
- backend/src/api/routes/docs.rs
- backend/src/api/routes/optimization_tasks.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/models/optimization_task_config.rs
- backend/src/infra/external/llm_client.rs
- backend/tests/auth_integration_test.rs
- backend/tests/optimization_tasks_api_test.rs
- backend/src/bin/gen-types.rs
- frontend/src/features/task-config/services/teacherModelService.ts
- frontend/src/features/task-config/hooks/useTeacherModels.ts
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx
- frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx
- frontend/src/pages/OptimizationTasksView/OptimizationTasksView.test.tsx
- frontend/src/types/generated/api/GenericLlmModelsResponse.ts
- frontend/src/types/generated/models/TeacherLlmConfig.ts
- frontend/src/types/generated/models/OptimizationTaskConfig.ts
- frontend/src/types/generated/api/UpdateOptimizationTaskConfigRequest.ts
- frontend/src/types/generated/api/OptimizationTaskListItemResponse.ts
- docs/implementation-artifacts/sprint-status.yaml
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] AC1 体验未完全满足：模型列表加载失败时缺少 `/settings/api` 引导入口；并且未“写死规则”限制在模型列表不可用时设置新的覆盖值（只允许保留既有覆盖/切回系统默认）。
- [x] [MEDIUM] “重置为默认值”按钮未重置老师模型覆盖（老师模型位于高级配置区域，用户预期会被一并重置）。
- [x] [MEDIUM] 后端缺少 `teacher_llm.model_id` 的边界用例测试（控制字符/超长/空白归一化），存在未来重构回归风险。
- [x] [MEDIUM] handler 内解密得到的 API Key 明文驻留未按仓库既有加固范式（`zeroize::Zeroizing`）承载，建议对齐 `decrypt_and_mask` 的策略。
- [x] [LOW] 前端任务列表页存在缩进混乱（tab/space 混用），影响可读性。

### Decisions

- [x] 在模型列表不可用时，仅阻止“设置新的覆盖值”（非空且变化），但允许：① 保持既有覆盖不变；② 切回系统默认；③ 保存其他配置字段（避免把老师模型故障放大成全表单不可保存）。

### Risks / Tech Debt

- [x] 暂不校验 `model_id` 必须来自 `/v1/models` 返回集合（MVP 范围内只做基本合法性校验与 UI 引导），后续若要强一致性需在后端引入可用模型缓存/实时校验策略。

### Follow-ups

- [x] 已将所有可执行项落到 `### Review Follow-ups (AI)` 并完成整改与测试验证。

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][HIGH] 补齐 AC1：模型列表失败时提供 `/settings/api` 引导，并限制模型列表不可用时设置新的覆盖值（允许保留既有覆盖/切回系统默认）。
- [x] [AI-Review][MEDIUM] “重置为默认值”同步重置老师模型覆盖为系统默认。
- [x] [AI-Review][MEDIUM] 补齐后端 `teacher_llm.model_id` 验证/归一化边界测试。
- [x] [AI-Review][MEDIUM] handler 内 API Key 解密结果用 `zeroize::Zeroizing` 承载，缩短明文驻留时间。
- [x] [AI-Review][LOW] 清理任务列表页缩进混乱，避免无意义 diff。
