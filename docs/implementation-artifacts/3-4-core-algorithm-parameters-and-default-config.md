# Story 3.4: 核心算法参数与默认配置

Status: done

Story Key: `3-4-core-algorithm-parameters-and-default-config`

Related FRs: FR23a（核心算法参数配置）, FR23b（默认配置与一键重置）

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a 高级用户,
I want 配置核心算法参数（评估器、迭代策略等），并可一键重置为默认值,
so that 我可以精细调整优化算法的行为。

## Acceptance Criteria

### AC1：高级配置区展示核心算法参数（FR23a）

**Given** 用户在任务配置页面  
**When** 用户展开“高级配置”区域  
**Then** 展示核心算法参数配置项：  
- OutputConfig（输出配置）  
- EvaluatorConfig（评估器选择与配置）  
- AdvancedDataSplitConfig（高级数据划分：交叉验证折数、采样策略）

> 说明：IterationConfig（最大迭代轮数/通过率阈值/候选数/多样性注入阈值）属于“基础配置”（Story 3.2/3.3 已实现），本 Story 不改变其语义，仅做分组展示与默认值/重置策略的对齐。

### AC2：评估器类型选择与配置（FR23a）

**Given** 用户配置评估器  
**When** 用户选择评估器类型  
**Then** 可以选择：自动选择（auto）/ 精确匹配 / 语义相似度 / 约束检查 / 老师模型评估  
**And** 每种评估器有对应的配置项  
**And** 保存后可回显（刷新/重新进入页面仍保持一致）

> 说明：`evaluator_type=auto` 仅表示“使用系统默认评估策略”（配置承载占位符）；本 Story 不实现自动选择逻辑，具体执行行为在 Epic 4（执行引擎）阶段落地。

### AC3：一键重置为默认值（FR23b）

**Given** 用户修改了高级配置（OutputConfig / EvaluatorConfig / AdvancedDataSplitConfig）  
**When** 用户点击“重置为默认值”  
**Then** 所有高级配置恢复为系统推荐的默认值（不影响基础配置与初始 Prompt）  
**And** 显示确认提示  
**And** 重置结果会持久化（刷新/重新进入页面仍为默认值）

### AC4：默认配置足以应对常见场景（FR23b）

**Given** 用户不修改高级配置  
**When** 使用默认值  
**Then** 系统使用经过验证的默认参数  
**And** 默认配置足以应对常见场景

## Tasks / Subtasks

### 任务 1：后端任务配置 Schema 扩展（config_json）（AC1-AC4）

- [x] 在 `backend/src/domain/models/optimization_task_config.rs` 扩展 `OptimizationTaskConfig`，新增核心算法配置字段（保持 `schema_version=1`，读取向后兼容：旧 JSON 缺字段 → 规范化默认值）
- [x] 保持 `config_json` “未知字段保留”策略：继续使用 `#[serde(flatten)] extra`，写入必须保留 existing extra（不得覆盖丢失未来字段）
- [x] 保持“解析失败保护”：更新配置写入前必须严格解析 existing config_json，解析失败则返回 `VALIDATION_ERROR`（避免“回退默认后覆盖写入”造成数据丢失）
- [x] 默认值与校验规则落地（与前端一致，且不破坏既有字段默认值/范围）：
  - OutputConfig 默认值：`strategy=single`、`conflict_alert_threshold=3`、`auto_recommend=true`
  - EvaluatorConfig 默认值：新增 `evaluator_type=auto`（推荐），并为各类型提供默认子配置（即使当前未选中，也必须可序列化/可回显）
  - AdvancedDataSplitConfig（高级）默认值：`strategy=percent`（沿用现有 `train_percent/validation_percent`）、`k_fold_folds=5`、`sampling_strategy=random`
- [x] AdvancedDataSplitConfig 与既有 percent 划分的互斥规则（必须写死，避免实现分歧）：
  - 当 `advanced_data_split.strategy=percent`：使用既有 `train_percent/validation_percent`（当前默认行为）
  - 当 `advanced_data_split.strategy=k_fold`：执行阶段忽略 `train_percent/validation_percent`，改用 `k_fold_folds` + `sampling_strategy` 进行交叉验证划分
- [x] 校验要求（建议最小集，避免过度设计）：
  - `conflict_alert_threshold`：整数范围（建议 1..=10）
  - `k_fold_folds`：整数范围（建议 2..=10；仅当 `advanced_data_split.strategy=k_fold` 时生效）
  - `semantic_similarity.threshold_percent`：整数范围（建议 1..=100；仅当 evaluator 为语义相似度时生效）
  - `llm_judge_samples`：整数范围（建议 1..=5；仅当 evaluator 为老师模型评估时生效）

### 任务 2：后端 API（读取+更新任务配置）与类型生成（AC1-AC4）

- [x] 扩展 `backend/src/api/routes/optimization_tasks.rs` 的 `UpdateOptimizationTaskConfigRequest`（沿用“全量更新配置”语义：请求体包含既有字段 + 本 Story 新增字段）
- [x] 更新 `PUT /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}/config`：把新字段写入 `OptimizationTaskConfig` 并通过 `validate_task_config` 校验
- [x] 确保 `GET .../optimization-tasks/{task_id}` 返回的 `config` 永远非空且包含新增字段（规范化默认值补齐）
- [x] 请求体结构示例（仅示意字段形状；保持 snake_case；仍为“全量提交”）：
  ```json
  {
    "initial_prompt": null,
    "max_iterations": 10,
    "pass_threshold_percent": 95,
    "candidate_prompt_count": 5,
    "diversity_injection_threshold": 3,
    "train_percent": 80,
    "validation_percent": 20,
    "output_config": {
      "strategy": "single",
      "conflict_alert_threshold": 3,
      "auto_recommend": true
    },
    "evaluator_config": {
      "evaluator_type": "auto",
      "exact_match": { "case_sensitive": false },
      "semantic_similarity": { "threshold_percent": 85 },
      "constraint_check": { "strict": true },
      "teacher_model": { "llm_judge_samples": 1 }
    },
    "advanced_data_split": {
      "strategy": "percent",
      "k_fold_folds": 5,
      "sampling_strategy": "random"
    }
  }
  ```
- [x] 运行 `cd backend && cargo run --bin gen-types` 同步生成：
  - `frontend/src/types/generated/models/OptimizationTaskConfig.ts`
  - `frontend/src/types/generated/api/UpdateOptimizationTaskConfigRequest.ts`

### 任务 3：前端任务配置页面（高级配置 UI + 重置）（AC1-AC4）

- [x] 在 `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx` 增加“高级配置”折叠区（建议使用 `<details>/<summary>`，样式沿用现有 Tailwind + shadcn 输入组件）
- [x] IterationConfig：把既有字段按分组展示（不改变语义）：`max_iterations`、`pass_threshold_percent`、`candidate_prompt_count`、`diversity_injection_threshold`
- [x] AdvancedDataSplitConfig（高级）：新增数据划分策略选择与配置项：
  - `percent`：沿用既有 Train% / Validation%（当前 UI 已有）
  - `k_fold`：展示 `k_fold_folds` 与 `sampling_strategy`（random/stratified）
- [x] OutputConfig：新增输出策略选择（single/adaptive/multi）+ `conflict_alert_threshold` + `auto_recommend`
- [x] EvaluatorConfig：新增评估器类型选择（auto/精确匹配/语义相似度/约束检查/老师模型评估）与对应字段：
  - 精确匹配：`case_sensitive`（布尔）
  - 语义相似度：`threshold_percent`（1-100）
  - 约束检查：`strict`（布尔）
  - 老师模型评估：`llm_judge_samples`（1-5；使用系统既有“老师模型”配置，不在本 Story 引入新的模型选择 UI）
- [x] evaluator_type 切换的表单策略（必须明确，避免前端实现分歧）：
  - 推荐：保留所有子配置值，但仅显示当前 evaluator_type 对应字段（避免切换时丢失用户输入；保存仍为全量提交）
- [x] “重置为默认值”按钮：
  - 点击后 `window.confirm` 二次确认
  - 仅重置“高级配置”字段（白名单，必须严格按此执行）：
    - OutputConfig：`strategy`、`conflict_alert_threshold`、`auto_recommend`
    - EvaluatorConfig：`evaluator_type` + 各类型子配置（即使未选中也保留并写回默认值）
    - AdvancedDataSplitConfig：`strategy`、`k_fold_folds`、`sampling_strategy`
  - 不重置（黑名单）：`initial_prompt`、`max_iterations`、`pass_threshold_percent`、`candidate_prompt_count`、`diversity_injection_threshold`、`train_percent`、`validation_percent`
  - 确认后调用现有 `PUT .../config` 持久化（payload 仍为全量提交）
- [x] 表单校验：新增字段必须做 `Finite + Integer + Range` 校验；不通过则不发送请求（与 Story 3.3 review 修复保持一致）

### 任务 4：测试与回归保护（AC 全覆盖）

- [x] 后端集成测试（`backend/tests/optimization_tasks_api_test.rs`）：
  - [x] `GET task`：旧 config_json 缺新增字段时，返回 `config` 仍补齐默认值（读取向后兼容）
  - [x] `PUT .../config`：更新新增字段成功，并回显更新后的值
  - [x] `PUT .../config`：校验失败返回 400（覆盖新增字段的边界值）
  - [x] `PUT .../config`：existing config_json 解析失败时返回 400 且不覆盖 DB 原值（防数据丢失）
  - [x] `PUT .../config`：写入时保留 unknown `extra` 字段（防覆盖未来字段）
- [x] 前端单测（`frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx`）：
  - [x] 默认值渲染（含新增高级配置默认值）
  - [x] 切换 evaluator 类型时，仅渲染对应字段；保存成功后回显
  - [x] “重置为默认值”会触发 `window.confirm`；确认后发送请求并回显默认值
  - [x] 本地校验：非法数值（NaN/小数/越界）不发请求

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 明确“重置为默认值”的作用域：仅重置高级配置白名单字段（OutputConfig/EvaluatorConfig/AdvancedDataSplitConfig；不影响基础配置与初始 Prompt）
- [x] [AI-Review] 明确 EvaluatorConfig 的 `auto` 语义：执行引擎阶段按任务模式/测试集信息自动选择评估器；本 Story 仅完成配置承载与 UI

## Dev Notes

### Developer Context（避免实现偏航）

- Epic 3 背景（目标与依赖）：
  - 用户成果：可创建优化任务、配置算法参数、管理多个工作区（见 `docs/project-planning-artifacts/epics.md` 的 Epic 3 概览）。
  - 依赖：Epic 1（API 配置）、Epic 2（测试集）；本 Story 仅扩展“任务配置”能力，不触及执行引擎（Epic 4）。
- 现状：Story 3.2/3.3 已完成“任务配置页 + config_json schema + PUT .../config 更新 + 生成 types + 回归测试”，并已落实两条关键防线：
  - `config` 永远非空：后端 `GET task` 对 `config_json` 做规范化默认值补齐
  - 防数据丢失：更新配置时严格解析 existing `config_json`，失败则拒绝更新（避免“回退默认后覆盖写入”）
- 本 Story 目标：在**不引入新配置系统/不实现执行引擎**的前提下，把“核心算法参数（高级配置）”的**可配置、可持久化、可回显、可一键恢复默认**补齐。
- 明确非目标（防 scope creep）：
  - 不实现评估器/输出策略/交叉验证的真实运行逻辑（属于 Epic 4 执行引擎与后续 Stories）
  - 不升级依赖（Rust/axum/sqlx/ts-rs/React/Vite 等）作为“顺手优化”

### Hard Prerequisites（硬前置：必须已存在/不得改语义）

- 后端：已存在 `GET /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}` 返回 `config` 永远非空（缺字段补默认值），且更新使用 `PUT .../{task_id}/config` 的“全量更新配置”语义（Story 3.2/3.3 既有模式）。
- 后端：更新配置写入必须保留 unknown `extra` 字段；existing `config_json` 解析失败必须拒绝更新（返回 `VALIDATION_ERROR`），避免数据丢失（Story 3.3 review 结论）。
- 前端：已存在 `OptimizationTaskConfigView` 基础配置表单（initial_prompt/max_iterations/pass_threshold_percent/train/validation 等）；本 Story 仅在其上增加高级配置折叠区与重置按钮，不改既有字段含义。

### Technical Requirements（DEV Agent Guardrails）

#### 1) Schema 设计原则（必须遵守）

- 继续使用 `optimization_tasks.config_json`（TEXT）承载配置，不新增表/不引入新配置框架。
- 继续使用 `schema_version=1`；新增字段必须具备默认值与 `#[serde(default)]`，确保读取向后兼容。
- 写入必须保留 unknown 字段（`extra`），并保留 config_json 的 32KB 上限与 prompt 脱敏日志规范。

#### 2) 建议的数据模型（可调整，但需自洽且可生成 TS 类型）

- OutputConfig（新）：
  - `strategy`: `single | adaptive | multi`
  - `conflict_alert_threshold`: `u32`
  - `auto_recommend`: `bool`
- EvaluatorConfig（新，建议带 `auto`）：
  - `evaluator_type`: `auto | exact_match | semantic_similarity | constraint_check | teacher_model`
  - `exact_match`: `{ case_sensitive: bool }`
  - `semantic_similarity`: `{ threshold_percent: u8 }`
  - `constraint_check`: `{ strict: bool }`
  - `teacher_model`: `{ llm_judge_samples: u32 }`
- AdvancedDataSplitConfig（高级，新；不替换既有 `data_split` 百分比配置）：
  - `strategy`: `percent | k_fold`
  - `k_fold_folds`: `u8`
  - `sampling_strategy`: `random | stratified`

> 注意：本 Story 只负责“配置承载 + UI + 校验 + 回显 + 重置”，不要在后端引入任何“真实算法推导”。

### Architecture Compliance（必须对齐项目架构）

- 后端：Axum 路由 + Repo 模式；错误响应沿用 `ApiResponse<T>` + `VALIDATION_ERROR`（见 `backend/src/api/response/*` 与 `backend/src/shared/error_codes.rs`）。
- 前端：沿用现有 React Router 页面结构与 TanStack Query hooks（`frontend/src/features/task-config/hooks/useOptimizationTasks.ts`）。
- 类型：沿用 `ts-rs` 生成链；不要手写 DTO 类型。

### Library & Framework Requirements（不要用错版本/不要顺手升级）

- 后端当前依赖：`axum 0.8`、`sqlx 0.8`、`ts-rs 10`、Rust `1.85`（见 `backend/Cargo.toml`）
- 前端当前依赖：React `19.2.0`、React Router `7.0.0`、TanStack Query `5.x`、Vite `7.x`（见 `frontend/package.json`）
- ⚠️ 本 Story 不做任何依赖升级；尤其不要升级 `ts-rs`（MSRV 变化风险）。

### File Structure Requirements（落点清单）

Backend（核心落点）：
- `backend/src/domain/models/optimization_task_config.rs`（新增字段、默认值、校验、extra 保留、序列化）
- `backend/src/api/routes/optimization_tasks.rs`（更新请求体与 PUT 写入逻辑）
- `backend/tests/optimization_tasks_api_test.rs`（回归测试）
- `backend/src/bin/gen-types.rs`（已有；运行生成类型即可）

Frontend（核心落点）：
- `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`（高级配置 UI、重置、校验与 payload 扩展）
- `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx`（UI/交互回归）
- `frontend/src/types/generated/*`（由 gen-types 生成；不要手改）

### Testing Requirements（Definition of Done 的硬门槛）

- 后端：`cargo test` 通过，且新增用例覆盖：
  - 默认值补齐（读取向后兼容）
  - 写入校验（新增字段边界）
  - 解析失败保护（不覆盖写入）
  - extra 保留（不丢未来字段）
- 前端：`vitest` 通过，且新增用例覆盖：
  - 默认值渲染 + 保存回显
  - 重置确认 + 请求发送 + 回显
  - 本地校验阻断无效请求（NaN/小数/越界）

### Previous Story Intelligence（从 3.2/3.3 继承的关键约束）

- `config` 永远非空：后端必须规范化默认值补齐（读取向后兼容）。
- “全量更新配置”语义：`PUT .../{task_id}/config` 请求体应包含既有字段 + 新增字段（避免 partial update 语义漂移）。
- 安全与可维护性：
  - `config_json` 总大小上限（<= 32KB）
  - 日志不得打印 `initial_prompt` 原文（仅记录长度）
  - existing config_json 解析失败必须拒绝更新（防数据丢失）

### Git Intelligence Summary（近期变更提示）

- 最近相关提交主要集中在 Story 3.3 的“前端数值校验 + 后端解析失败保护 + extra 保留测试”，本 Story 应严格沿用该模式扩展，避免引入新范式。

### Latest Tech Information（本 Story 的“最新情报”结论）

- 本 Story 本质是“配置承载 + UI + 校验 + 类型生成 + 回归测试”，不依赖外部 API 契约；最重要的“最新情报”是：
  - 不要升级 `ts-rs`（当前仓库锁定 Rust `1.85`，而 `ts-rs 11.x` 需要更高 MSRV）

### Project Context Reference

- 未找到 `**/project-context.md`；本 Story 以 `docs/project-planning-artifacts/*` 与 `docs/implementation-artifacts/*` 为主要真相源。

### Story Completion Status（Definition of Done）

- ✅ 任务配置页新增“高级配置”区，并可配置：OutputConfig / EvaluatorConfig / AdvancedDataSplitConfig；基础配置区继续分组展示 IterationConfig（既有字段）
- ✅ “重置为默认值”可用：确认后持久化，回显为默认值
- ✅ `GET task` 返回 `config` 永远非空且包含新增字段（旧 JSON 缺字段时补默认值）
- ✅ 后端/前端新增回归用例覆盖关键风险点；无回归

### References

- [Source: `docs/project-planning-artifacts/epics.md`#Story 3.4]
- [Source: `docs/project-planning-artifacts/architecture.md`#Technical Constraints & Dependencies / Project Structure]
- [Source: `docs/project-planning-artifacts/prd.md`#高级算法特性归属表]
- [Source: `docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md`#4.2.6.1 / default_*]
- [Source: `docs/implementation-artifacts/3-2-initial-prompt-and-iteration-termination-conditions.md`]
- [Source: `docs/implementation-artifacts/3-3-candidate-generation-and-diversity-injection-config.md`]
- [Source: `docs/implementation-artifacts/3-3-candidate-generation-and-diversity-injection-config-reviewed.md`]
- [Source: `backend/src/domain/models/optimization_task_config.rs`]
- [Source: `backend/src/api/routes/optimization_tasks.rs`]
- [Source: `backend/src/infra/db/repositories/optimization_task_repo.rs`]
- [Source: `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`]

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

- 后端：扩展 `OptimizationTaskConfig`（新增 `output_config` / `evaluator_config` / `advanced_data_split`），落实默认值、向后兼容补齐、严格解析失败保护、extra 保留与校验规则。
- 后端：扩展 `PUT .../config` 请求体为“全量更新”，并运行 `gen-types` 生成前端类型。
- 前端：新增“高级配置”折叠区（OutputConfig / EvaluatorConfig / AdvancedDataSplitConfig），支持 evaluator 类型切换、校验与一键重置（仅影响高级配置白名单字段）。
- 测试：补齐后端集成测试与前端单测覆盖默认值/回显/重置/校验/解析失败保护等关键风险点；回归通过。

### File List

- `docs/implementation-artifacts/sprint-status.yaml`
- `backend/src/domain/models/optimization_task_config.rs`
- `backend/src/domain/models/mod.rs`
- `backend/src/api/routes/optimization_tasks.rs`
- `backend/tests/optimization_tasks_api_test.rs`
- `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`
- `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx`
- `frontend/src/types/generated/api/UpdateOptimizationTaskConfigRequest.ts`
- `frontend/src/types/generated/models/OptimizationTaskConfig.ts`
- `frontend/src/types/generated/models/OutputConfig.ts`
- `frontend/src/types/generated/models/OutputStrategy.ts`
- `frontend/src/types/generated/models/EvaluatorConfig.ts`
- `frontend/src/types/generated/models/EvaluatorType.ts`
- `frontend/src/types/generated/models/ExactMatchEvaluatorConfig.ts`
- `frontend/src/types/generated/models/SemanticSimilarityEvaluatorConfig.ts`
- `frontend/src/types/generated/models/ConstraintCheckEvaluatorConfig.ts`
- `frontend/src/types/generated/models/TeacherModelEvaluatorConfig.ts`
- `frontend/src/types/generated/models/AdvancedDataSplitConfig.ts`
- `frontend/src/types/generated/models/AdvancedDataSplitStrategy.ts`
- `frontend/src/types/generated/models/SamplingStrategy.ts`
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [CRITICAL] `gen-types` 生成的新类型文件未纳入 Git：`frontend/src/types/generated/models/*` 存在 untracked，导致 CI/同事环境无法稳定构建。
- [x] [HIGH] “重置为默认值”在请求前先更新 UI 状态，失败时不回滚，可能出现“看起来已重置但未持久化”的一致性问题。
- [x] [MEDIUM] 前端测试缺少对新增高级配置字段的本地校验拦截用例（k_fold_folds / semantic threshold / llm_judge_samples），以及重置后 evaluator 默认值的断言。
- [x] [MEDIUM] AdvancedDataSplitConfig 在 `percent` 策略下仍展示 k-fold 字段，容易误解“字段是否生效”（UI 应仅在 `k_fold` 时展示）。
- [x] [MEDIUM] 读取路径对损坏的 `config_json` 会回退到默认值（不会写回 DB），缺少明确日志提示，排障成本偏高。

### Decisions

- [x] generated types 继续入库（不在 `.gitignore` 忽略），确保 API/类型一致性与可复现构建。
- [x] “重置为默认值”改为以后端回显为准：请求成功后再回填 UI（避免失败时出现假象）。
- [x] 不做大规模重构（如 useReducer），优先修复会影响一致性/可提交/可回归验证的问题。

### Risks / Tech Debt

- [ ] 如果 DB 中已存在损坏的 `config_json`，读路径会回退默认值，但更新会被“解析失败保护”拒绝；后续可考虑提供专用修复/迁移工具（不在本 Story 范围）。

### Follow-ups

- [x] 补充后端 warn 日志：读取到无效 `config_json` 时记录关联 task_id/workspace_id（不打印原文，仅记录长度）。

### Open Questions（生成 Story 时保存的问题，供一次性澄清）

1. （已决策）“重置为默认值”仅重置高级配置白名单字段（OutputConfig/EvaluatorConfig/AdvancedDataSplitConfig），不影响基础配置与初始 Prompt（见任务 3）。
2. 语义相似度评估的“相似度来源”与口径：用 embedding cosine / LLM judge / 其他？（本 Story 先承载阈值字段即可）
3. “约束检查”评估的约束来源：来自测试用例的 constraints 字段？还是来自任务模式（Fixed/Creative）的结构化约束配置？
4. 老师模型评估是否需要暴露更多参数（temperature/samples/置信度阈值）？还是严格复用全局老师模型配置？
