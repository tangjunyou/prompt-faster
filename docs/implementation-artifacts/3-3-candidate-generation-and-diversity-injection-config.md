# Story 3.3: 候选生成与多样性注入配置

Status: done

Story Key: `3-3-candidate-generation-and-diversity-injection-config`

Related FRs: FR22（候选 Prompt 生成数量）, FR23（连续失败触发多样性注入阈值）

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 配置候选 Prompt 生成数量和多样性注入触发条件,
so that 我可以控制优化算法的探索强度。

## Acceptance Criteria

### AC1：候选 Prompt 生成数量（FR22）

**Given** 用户在任务配置页面  
**When** 用户配置“候选 Prompt 生成数量”  
**Then** 显示数值输入框  
**And** 默认值为推荐值（如 3-5 个）  
**And** 提供说明文字解释生成数量对优化效果/成本的影响  
**And** 保存后可回显（刷新/重新进入页面仍保持一致）

### AC2：多样性注入阈值（FR23）

**Given** 用户在任务配置页面  
**When** 用户配置“多样性注入阈值（连续失败次数）”  
**Then** 可以设置连续失败多少次后触发多样性注入  
**And** 默认值为推荐值（如 3 次）  
**And** 提供说明文字解释多样性注入的作用  
**And** 保存后可回显（刷新/重新进入页面仍保持一致）

## Tasks / Subtasks

- [x] 任务 1：后端任务配置 Schema（`optimization_tasks.config_json`）扩展与校验（AC1/AC2）
  - [x] 为 `OptimizationTaskConfig` 增补字段（建议：`candidate_prompt_count`、`diversity_injection_threshold`）
  - [x] 定义默认值（建议：`candidate_prompt_count=5`、`diversity_injection_threshold=3`）与范围校验（与 UI 限制一致）
  - [x] 保持向后兼容：`config_json IS NULL` 或旧 JSON 缺字段 → 返回规范化默认值；未知字段继续落在 `extra` 中（不得丢失）
- [x] 任务 2：后端 API + OpenAPI/ts-rs（AC1/AC2）
  - [x] 扩展 `PUT /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}/config` 请求体（snake_case）
  - [x] 确保 `GET /.../optimization-tasks/{task_id}` 返回的 `config` 包含新增字段（永远非空、已规范化）
  - [x] 运行 `cd backend && cargo run --bin gen-types` 同步 `frontend/src/types/generated/`
- [x] 任务 3：前端任务配置页面（AC1/AC2）
  - [x] 在 `OptimizationTaskConfigView` 增加两个数值输入与说明文案，并按后端返回值回显
  - [x] 保存时携带新增字段；保存成功后以后端回包刷新本地 state
  - [x] 表单校验：数值范围（与后端一致）；错误仅展示 message，不展示 details
- [x] 任务 4：测试与回归保护（AC 全覆盖）
  - [x] 后端集成测试：
    - [x] `GET task`：`config_json IS NULL` 时 `config` 返回规范化默认值（含新增字段默认值）
    - [x] `GET task`：旧 JSON 缺新增字段时，`config` 仍补齐新增字段默认值（读取向后兼容）
    - [x] `PUT .../config`：更新本 Story 字段时不得丢失未知字段（保留 `extra`，防止覆盖未来字段）
    - [x] 更新配置成功（含新增字段）、边界值校验、越权/NotFound
  - [x] 前端单测：默认值渲染（含新增字段）、保存成功/失败 message、回显更新后的值

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 明确 `PUT .../{task_id}/config` 为“全量更新配置”（沿用 Story 3.2：请求体包含既有字段 + 本 Story 新增字段）
- [x] [AI-Review] 增补后端回归测试：`config_json IS NULL` 默认值、旧 JSON 缺字段补默认值、保留 `extra` 防覆盖未来字段
- [x] [AI-Review] 修正文案：Rust 字段 snake_case + `#[serde(rename_all = "snake_case")]`（避免“Rust struct snake_case”的误导）
- [x] [AI-Review] 修正类型生成命令：`cd backend && cargo run --bin gen-types`

## Dev Notes

### Developer Context（避免实现偏航）

- 现状：Story 3.1 已完成“优化任务”创建/列表/详情（workspace 边界 + `optimization_tasks` 表）；Story 3.2 已完成“任务配置”页与 `config_json`（schema_version=1）承载，并提供 `PUT .../{task_id}/config` 更新配置能力。
- 本 Story 目标：在**现有任务配置能力上“加字段 + 加校验 + 加回显”**，让用户可配置：
  - 候选 Prompt 生成数量（控制探索强度/成本）
  - 连续失败触发多样性注入阈值（控制“卡住”时的探索跳出策略）
- 明确非目标（防 scope creep）：
  - 不实现候选生成/多样性注入的算法逻辑（属于 Epic 4/执行引擎阶段）
  - 不引入新的配置系统/验证框架；沿用现有 axum + 手写校验 + `ApiResponse<T>` 模式

### Technical Requirements（DEV Agent Guardrails）

#### 1) 字段设计（建议）

- `candidate_prompt_count: u32`：每轮迭代要生成的候选 Prompt 数量（Layer 2 会用到）
- `diversity_injection_threshold: u32`：连续失败多少次触发多样性注入（FR23 的“连续失败次数阈值”）

> 命名要求：对外 API/JSON 字段必须是 `snake_case`；Rust 字段名用 `snake_case`，并用 `#[serde(rename_all = "snake_case")]` 保证序列化一致（与现有 `OptimizationTaskConfig` 一致）。

#### 2) 默认值与校验（必须对齐前后端）

- 默认值（与 Epic 3 Story 3.3 推荐一致）：
  - `candidate_prompt_count = 5`（“推荐 3-5 个”，默认取上沿，便于探索；后续可根据真实成本调整）
  - `diversity_injection_threshold = 3`
- 建议范围（可在实现时与产品实际体验再微调，但前后端必须一致）：
  - `candidate_prompt_count`：`1..=10`
  - `diversity_injection_threshold`：`1..=10`
- 失败返回：校验失败统一返回 `400` + `VALIDATION_ERROR`（复用既有错误码体系，不新增 code）。

#### 3) 存储与向后兼容（必须）

- 存储：继续使用 `optimization_tasks.config_json`（TEXT）作为配置承载。
- 向后兼容：
  - `config_json IS NULL` 或旧 JSON 缺字段 → 响应中 `config` 必须返回规范化默认值（`config` 永远非空）。
  - 继续保留未知字段到 `extra`（`OptimizationTaskConfigStorage` + `serialize_config_with_existing_extra`），避免后续 Story 3.4/其它版本写入的字段被本端点覆盖丢失。
- 防膨胀：继续遵守 `OPTIMIZATION_TASK_CONFIG_MAX_JSON_BYTES = 32KB`。

#### 4) API 契约（最小集）

- `GET /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}`
  - `OptimizationTaskResponse.config` 必须包含新增字段（规范化后永远非空）。
- `PUT /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}/config`
  - 请求体（snake_case）建议扩展为：
    - `candidate_prompt_count`
    - `diversity_injection_threshold`
  - 语义：该端点为“全量更新配置”（沿用 Story 3.2）；请求体需同时携带 Story 3.2 既有字段 + 本 Story 新增字段。后端将其规范化、校验后写入 `config_json` 并回显完整 `config`。
  - Schema version：保持 `schema_version = 1`（新增字段属于向后兼容扩展，不做版本升级）。
  - 响应：返回更新后的完整任务（含规范化后的 `config`）。

> 注意：不要在 tracing 日志里打印 `initial_prompt` 原文或完整 `config_json`；仅允许记录长度/摘要。新增字段可打印数值，但不得打印敏感信息。

请求体示例（字段均为 `snake_case`，为全量配置更新）：

```json
{
  "initial_prompt": null,
  "max_iterations": 10,
  "pass_threshold_percent": 95,
  "train_percent": 80,
  "validation_percent": 20,
  "candidate_prompt_count": 5,
  "diversity_injection_threshold": 3
}
```

### Architecture Compliance（对齐仓库事实标准）

- Workspace 为第一隔离边界：沿用 `workspaces/{workspace_id}/...` 的嵌套路由模式。
- 统一错误响应：必须使用 `ApiResponse<T>`（禁止返回裸 JSON）。
- 命名与序列化：Rust snake_case、TS camelCase（仅在前端内部），跨边界 DTO 使用 snake_case（ts-rs 生成类型已是 snake_case）。

### Library / Framework Requirements（版本与依赖边界）

- 前端：沿用现有依赖与模式（React 19.x、`react-router` 7.x、`@tanstack/react-query` 5.x、shadcn/ui 风格组件）；本 Story 不引入新库、不做框架升级。
- 后端：沿用现有依赖（axum 0.8、sqlx 0.8、ts-rs 10、utoipa 5）；不引入新验证库。
- 类型：后端 DTO 变更后必须运行 `cd backend && cargo run --bin gen-types` 同步 `frontend/src/types/generated/`，避免前后端漂移。

### File Structure Requirements（建议改动清单）

**后端（Rust）**

- `backend/src/domain/models/optimization_task_config.rs`
  - 扩展 `OptimizationTaskConfig` / `OptimizationTaskConfigStorage` / 默认值 / `validate()`（新增字段 + 范围校验）
- `backend/src/api/routes/optimization_tasks.rs`
  - 扩展 `UpdateOptimizationTaskConfigRequest`
  - 在 `update_optimization_task_config(...)` 里把新增字段写入 `OptimizationTaskConfig`
- `backend/src/api/routes/docs.rs`
  - 更新 OpenAPI schemas / paths（与既有 gen-types 流程一致）
- `backend/tests/optimization_tasks_api_test.rs`
  - 增补/更新配置更新用例（含新增字段）

**前端（React）**

- `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`
  - 增加两个数值输入（含 min/max、helper text），并按后端回包回显
- `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx`
  - 覆盖默认值渲染、保存成功/失败 message、回显更新值
- `frontend/src/features/task-config/services/optimizationTaskService.ts`
  - 一般无需改（仅类型会更新）；如请求字段变化，确保 payload 与生成类型一致
- `frontend/src/types/generated/**`
  - 由 `cd backend && cargo run --bin gen-types` 自动生成（不要手改）

### Testing Requirements（与 CI 门禁对齐）

- 后端（集成测试）：`backend/tests/optimization_tasks_api_test.rs`
  - 200：更新配置成功并回显新增字段（含默认值/覆盖值）
  - 200：`GET task` 对 `config_json IS NULL` / 旧 JSON 缺字段的向后兼容（默认值补齐）
  - 200：更新配置时保留 `extra`（防止覆盖未来字段）
  - 400：候选数/阈值越界（以及其它既有校验不回归）
  - 404：workspace/task 不存在
  - 401：未登录
- 前端（单测）：`frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx`
  - 默认值渲染（后端 config_json 缺字段时的规范化默认值）
  - 保存成功提示 + 回显更新后的数值
  - 保存失败仅展示 message（不展示 details）

### Previous Story Intelligence（从 3.1/3.2 继承的关键约束）

- `config` 永远非空：后端必须对 `config_json` 做规范化（默认值补齐）。
- `config_json` 写入必须保留 `extra`（避免覆盖丢失未来字段）。
- 日志脱敏：`initial_prompt` 不得出现在日志；仅记录长度（已有实现可复用）。

### Git Intelligence Summary（近期变更提示）

- 最近相关提交：
  - `4f771a5`：Story 3.2（task config + review fixes）
  - `26922bd`：修复后端对 `initial_prompt` bytes 的校验
- 结论：本 Story 建议严格沿用 3.2 的“配置写入/回显/生成类型/测试”模式，避免引入新范式造成维护成本。

### Latest Tech Information（本 Story 的“最新情报”结论）

- 本 Story 不涉及外部 API/第三方服务契约变化；实现应以**仓库当前依赖与类型生成链**为准（避免“升级依赖顺便改行为”）。

### Project Context Reference

- 本仓库未发现 `project-context.md`（如后续补齐，请在此处追加引用路径与关键约束）。

### Story Completion Status（Definition of Done）

- ✅ 前后端均可配置并持久化：`candidate_prompt_count` + `diversity_injection_threshold`
- ✅ 刷新/重新进入页面可回显；`GET task` 返回的 `config` 永远非空且包含新增字段
- ✅ 类型生成同步完成；后端/前端测试新增覆盖；无回归

### References

- [Source: `docs/project-planning-artifacts/epics.md`#Epic 3 / Story 3.3]
- [Source: `docs/project-planning-artifacts/prd.md`#8.5 Scope Boundaries / “高级算法特性归属表”]
- [Source: `docs/project-planning-artifacts/architecture.md`#Project Structure & Boundaries]
- [Source: `docs/implementation-artifacts/3-2-initial-prompt-and-iteration-termination-conditions.md`]
- [Source: `backend/src/domain/models/optimization_task_config.rs`]
- [Source: `backend/src/api/routes/optimization_tasks.rs`]
- [Source: `backend/src/infra/db/repositories/optimization_task_repo.rs`]
- [Source: `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`]

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

- `cd backend && cargo test`
- `cd backend && cargo run --bin gen-types`
- `cd frontend && npm run test -- --run`

### Completion Notes List

- 后端 `OptimizationTaskConfig` 新增 `candidate_prompt_count` / `diversity_injection_threshold`（默认值 5/3，范围 1..=10），并在 `config_json` 读取时对缺字段做默认值补齐。
- `PUT .../config` 扩展请求体，写入时保留 `extra` 未知字段，避免覆盖未来字段。
- 前端配置页新增两个数值输入与说明文案，保存后按后端回包回显；表单本地校验与后端范围一致。

### File List

- backend/src/domain/models/optimization_task_config.rs
- backend/src/api/routes/optimization_tasks.rs
- backend/tests/optimization_tasks_api_test.rs
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx
- frontend/src/types/generated/api/UpdateOptimizationTaskConfigRequest.ts
- frontend/src/types/generated/models/OptimizationTaskConfig.ts
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/3-3-candidate-generation-and-diversity-injection-config.md
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] (HIGH) 测试与回归：补充向后兼容与 `extra` 保留的后端回归测试（防止未来字段被覆盖丢失）
- [x] (MEDIUM) API 契约：明确 `PUT .../config` 为“全量更新配置”（避免实现 partial update 造成语义漂移）
- [x] (LOW) 文档可用性：明确 `gen-types` 命令在 `backend/` 目录执行；命名表述避免误导

### Decisions

- [x] `PUT .../{task_id}/config` 采用“全量更新配置”语义（与 Story 3.2 一致，降低心智负担与维护成本）
- [x] `schema_version` 维持为 `1`（新增字段属于向后兼容扩展，不做版本升级）
- [x] 新增字段类型与范围：`u32`，`1..=10`；默认值：`candidate_prompt_count=5`、`diversity_injection_threshold=3`

### Risks / Tech Debt

- [ ] 若未覆盖“保留 `extra`”的测试，后续 Story（如 3.4）引入的新字段可能被本端点写入时覆盖丢失（必须用回归测试锁死）

### Follow-ups

- [ ] Epic 4（执行引擎）：使用 `candidate_prompt_count` 与 `diversity_injection_threshold` 控制探索强度与多样性注入触发
- [ ] Story 3.4：如引入更多核心算法参数字段，继续沿用 `extra` 保留策略与回归测试模式
