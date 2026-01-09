# Story 4.3: 质量评估层（Layer 3: Quality Assessor）

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 4-3-quality-assessment-layer

## Key Decisions (MVP)

- **对齐技术规格的 Evaluator Trait**：以 `Evaluator::evaluate(ctx, test_case, output)` / `evaluate_batch` 为核心边界，避免继续沿用“(prompt, test_cases)”的占位签名。
- **评估结果结构复用领域模型**：复用 `backend/src/domain/models/algorithm.rs::EvaluationResult`（`passed/score/dimensions/failure_points/confidence/reasoning/extra`），用 `HashMap<test_case_id, EvaluationResult>` 承载“逐用例评估详情”。
- **先保证确定性评估可用**：MVP 优先落地 `ExactMatch` / `ConstraintCheck` /（轻量）`SemanticSimilarity`；`TeacherModel` 评估器按“依赖注入可选项”设计（没有 TeacherModel 时必须给出可诊断错误）。
- **EnsembleEvaluator 与 confidence 门控遵循算法规格**：算法级门控参数以 `OptimizationContext.config.evaluator` 为准；任务级 evaluator_type/阈值以 `task_evaluator_config`（形状复用 `OptimizationTaskConfig.evaluator_config`）为准。在无多评估器/无 TeacherModel 时允许降级，但必须显式、可观测（日志/extra 字段）。
- **严格防 scope creep**：本 Story 只负责 Layer 3（Evaluator）契约与默认实现；不在本 Story 落地 ExecutionTarget/IterationEngine/FeedbackAggregator/Optimizer/TeacherModel 的完整实现（这些在后续 Story 推进）。

## Story

As a 系统,
I want 评估候选 Prompt 在测试集上的效果,
so that 可以判断候选 Prompt 是否优于当前 Prompt。

## Acceptance Criteria

1. **Given** ExecutionTarget 已产出可关联的 `(test_case, output)` 列表（`results: &[(TestCase, String)]`；以 `TestCase.id` 为唯一对齐键），且与 `OptimizationContext.test_cases` 一致  
   **When** 调用 Layer 3 `Evaluator.evaluate_batch(ctx, results)`（results 不应在 Evaluator 内被过滤/重排）  
   **Then** 必须为每个输入对生成一条 `EvaluationResult`，并且：
   - `passed` 表示该用例是否通过
   - `score` ∈ `[0.0, 1.0]`，可用于候选排序
   - `failure_points` 在失败时给出可被 Layer 4 消费的结构化原因（至少包含 `dimension/description/severity`）
   - `evaluator_type` 必须可追溯到具体评估器实现（用于 debug / UI 展示 / 统计）
   - 返回的 `Vec<EvaluationResult>` **顺序必须与 `results` 输入顺序一致**（MVP 通过 index 对齐来组装 `test_case_id -> EvaluationResult`，避免并行/重排导致错位）
2. **Given** 任务配置指定了评估策略（精确匹配 / 语义相似度 / 约束检查 / 老师模型）  
   **When** 执行评估  
   **Then** 必须使用配置指定的评估器或组合评估器（Ensemble），且选择逻辑必须可诊断（例如写入 `EvaluationResult.extra["selected_evaluators"]`；如发生降级/兜底，写入 `EvaluationResult.extra["evaluator_fallback_reason"]`；关键阈值写入 `EvaluationResult.extra["thresholds"]`）。
3. **Given** `OptimizationContext.config.data_split.enabled = true` 且 TestCase `split` 字段已标注  
   **When** Layer 3 参与“通过率阈值判断与候选排名”  
   **Then** 必须支持按 split 过滤评估用例（至少支持 Validation；Holdout 仅用于最终验证，不参与 Racing/排序）；过滤仅影响通过率/排名统计，不得改变 `evaluate_batch` 的逐用例返回数量与顺序（见 AC1）。
4. **Given** 已得到某候选 Prompt 的逐用例评估结果（`HashMap<test_case_id, EvaluationResult>`）  
   **When** 计算候选通过率与选择最优候选  
   **Then** 必须：
   - 通过率计算规则清晰且可复现（`passed_count / total_count`）
   - 选择通过率最高者为本轮最优；若通过率相同，按 `score` **简单平均值**决胜（MVP：`mean(score)`；如仍相同，保持候选原始顺序/稳定排序）
   - 记录“候选排名与逐用例结果”以支持 UI 展示与 NFR21 统计
5. **Given** 候选 Prompt 的通过率达到 `OptimizationContext.config.iteration.pass_threshold`（默认 0.95）  
   **When** 评估完成  
   **Then** 编排层可以据此标记优化成功；Layer 3 必须提供足够的统计字段，支持后续生成“成功率报表”来验证 NFR21（整体优化成功率基准 ≥ 90%）。
6. **Given** 输入不完整（例如 results 为空、TestCase 缺失、输出为空但引用要求非空）  
   **When** 执行评估  
   **Then** 必须返回 `EvaluatorError::InvalidInput`（不得 silent fallback），并包含可诊断信息（缺失字段/索引/用例 id）。

## Tasks / Subtasks

- [x] 对齐 Layer 3 的输入/输出契约（AC: 1,2,6）
  - [x] 将 `backend/src/core/traits.rs` 的 `Evaluator` 签名对齐技术规格（增加 `ctx/test_case/output`、`evaluate_batch`、`name()`；返回 `Result<_, EvaluatorError>`）
  - [x] 在 `backend/src/core/evaluator/error.rs` 定义 `EvaluatorError`（`thiserror`；至少包含 `InvalidInput/Timeout/ModelFailure/Internal`）
  - [x] 明确“任务级评估器选择配置”的注入方式（建议：编排层写入 `ctx.extensions["task_evaluator_config"]`，形状复用 `domain/models/optimization_task_config.rs::EvaluatorConfig`；并在编排层将 `llm_judge_samples` 同步到 `OptimizationContext.config.evaluator.llm_judge_samples`，避免运行时出现双来源冲突）
- [x] 实现核心评估器与组合评估器（AC: 1,2,4）
  - [x] 新增 `backend/src/core/evaluator/` 模块与默认实现入口（`mod.rs` / `default_impl.rs`）
  - [x] 实现 `ExactMatchEvaluator`：支持 `TaskReference::Exact` 与 `Hybrid.exact_parts`（最小规则：trim；可选 case_sensitive 来自 task_evaluator_config）
  - [x] 实现 `ConstraintCheckEvaluator`：覆盖 FR14 的最小约束集（复用 Story 2.6 的编码约定：`length/must_include/must_exclude/format` + `params.{minChars,maxChars,keywords,format}`）；失败时产出结构化 `failure_points`（dimension 需可被 Layer1 聚合使用，建议与约束类型对应）
  - [x] 实现轻量 `SemanticSimilarityEvaluator`：不引入重依赖即可工作（允许后续替换为更强实现）；阈值来自 `task_evaluator_config.semantic_similarity.threshold_percent`；MVP 推荐：对 `TaskReference::Constrained.core_request` 与 `output` 做 token Jaccard 相似度（缺少 `core_request` 且 evaluator_type 显式为 SemanticSimilarity 时返回 `EvaluatorError::InvalidInput`）
  - [x] 实现 `TeacherModelEvaluator`：当注入 `Arc<dyn TeacherModel>` 时可工作；未注入时返回 `EvaluatorError::ModelFailure`（信息明确）
  - [x] 实现 `EnsembleEvaluator`：按 `OptimizationContext.config.evaluator` 执行组合与 confidence 门控；按 `task_evaluator_config.evaluator_type` 选择“基础评估器集合”（Auto/单一/多评估器）；最小可观测性：`extra["selected_evaluators"]`（string[]）、`extra["thresholds"]`（object）、`extra["evaluator_fallback_reason"]`（string，可选）
- [x] 评估结果与排序/统计（AC: 3,4,5）
  - [x] 提供辅助函数（可放在 `core/evaluator/default_impl.rs`）将 `Vec<EvaluationResult>` 组织为 `HashMap<test_case_id, EvaluationResult>`（以 `results[i].0.id` 绑定 `evaluations[i]`，要求 `evaluate_batch` 同序返回，见 AC1）
  - [x] 明确 pass-rate/score 汇总与边界：若参与统计的用例数为 0，返回 `EvaluatorError::InvalidInput`；`score` 的 MVP 汇总为简单平均值（见 AC4）
  - [x] 明确 split 过滤策略（Train/Validation/Holdout/Unassigned）：过滤仅用于统计/排名；Holdout 仅用于最终验证；并加单测
- [x] 测试与质量保障（AC: 1-6）
  - [x] 单测覆盖：Exact/Constrained/Hybrid 三类 TaskReference；通过/失败与 failure_points；TaskReference 不匹配返回 `EvaluatorError::InvalidInput`
  - [x] 单测覆盖：`evaluate_batch` 顺序保持（输入与输出一一对应、不可重排）
  - [x] 单测覆盖：split 过滤与 pass_threshold 计算（阈值边界：0.0/1.0/0.95）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 将本 Story 的“6+4+3”审查结论沉淀到 `## Review Notes`（含风险/遗留与后续 Story 承接点）

## Dev Notes

### Developer Context (What exists today)

- `backend/src/core/evaluator/` 目录已存在但为空（当前缺实现）。
- `backend/src/core/traits.rs` 的 `Evaluator` 仍是占位形态：`evaluate(prompt, test_cases) -> anyhow::Result<EvaluationResult>`，与技术规格不一致（本 Story 需要对齐）。
- Layer 1 默认实现已假设存在逐用例评估结果映射（见 `backend/src/core/rule_engine/default_impl.rs` 测试扩展点 `layer1_test_results.evaluations_by_test_case_id`），因此 Layer 3 的产出形态必须与其可对接。

### Contract Clarifications (MVP - Must read)

- **输入对齐键**：`results: &[(TestCase, String)]` 中的 `TestCase.id` 是唯一对齐键；不得引入“额外的 test_case_ids 列表”等第二事实来源。
- **顺序约束**：`evaluate_batch` 返回的 `Vec<EvaluationResult>` 必须与输入 `results` 同序；`HashMap<test_case_id, EvaluationResult>` 由 `results[i].0.id` 与 `evaluations[i]` 绑定生成（见 AC1）。
- **输入合法性（防错位/防静默覆盖）**：`evaluate_batch` 必须拒绝：
  - `results[*].TestCase.id` 为空
  - `results` 中出现重复 `test_case_id`
  - `results` 中出现 `ctx.test_cases` 未知 `test_case_id`
- **配置分层与优先级**：
  - 任务级：`ctx.extensions["task_evaluator_config"]`（形状复用 `OptimizationTaskConfig.evaluator_config`）决定 evaluator_type 与各子配置（case_sensitive / threshold_percent / strict / llm_judge_samples）。
  - 算法级：`ctx.config.evaluator` 承载 Ensemble/confidence 门控参数；`llm_judge_samples` 允许作为兜底（任务级优先）。
- **可观测性（最小 schema）**：建议在 `EvaluationResult.extra` 中使用固定 key：`selected_evaluators`（string[]）、`thresholds`（object）、`evaluator_fallback_reason`（string，可选）。
- **TaskReference 不匹配**：当评估器策略与 `TaskReference` 不兼容时，统一返回 `EvaluatorError::InvalidInput`（不新增错误枚举分支，避免与技术规格偏离）。
- **split 过滤职责**：Evaluator 不过滤输入；过滤仅作用于统计/排名集合（见 AC3）。

### Cross-Story Context (Epic 4)

- Layer 1/2 已完成（`docs/implementation-artifacts/4-1-pattern-extraction-layer.md`、`docs/implementation-artifacts/4-2-prompt-generation-layer.md`），Layer 3 的主要输入来自编排层：
  - ExecutionTarget 产出的 `(test_case, output)`（后续 Story 实现执行与并行调度）
  - `OptimizationContext.config.*` 的算法级配置（含 `iteration.pass_threshold`、`evaluator.*`）
  - “任务级评估器配置”（建议通过 `ctx.extensions["task_evaluator_config"]` 注入，避免改动 Context 结构）
- Layer 3 的输出将被后续层消费：
  - 通过率/score 用于候选排序、终止判断（Layer 4/编排层）
  - `failure_points` 用于失败模式抽取与反思输入（Layer 1/4）

### Previous Story Intelligence (What to reuse / not repeat)

- 从 Story 4.2 继承的关键约定：
  - 编排层负责“多候选循环”（FR22），Layer 3 只做评估，不做候选生成与调度（避免职责漂移）。
  - `OptimizationContext.extensions` 是“向前兼容”的扩展点：需要新输入时优先走 extensions 注入，而不是给 Context 加字段（与 4.2 的 `candidate_index/optimization_goal` 做法一致）。
- 从 Story 4.1 继承的关键约定：
  - Layer 1 已在测试里使用 `evaluations_by_test_case_id` 形状；Layer 3 的输出必须能被组织为该形状（避免后续对接返工）。

### Git Intelligence (Recent patterns)

- 最近一次与核心算法相关的大改动是 `feat(core): complete Layer1+Layer2 and close stories 4.1/4.2`：建立了 `core/<module>/{mod.rs,error.rs,default_impl.rs}` 的实现形态与“仅导出必要模块”的习惯（本 Story 复用同样结构）。
- 最近 3 次提交集中在前端工作区删除体验修复，对 Layer 3 实现影响较小；本 Story 不应顺带改动前端（除非评估结果结构确需联动）。

### Disaster Prevention (Must-follow guardrails)

- **禁止重复建模**：不得在 `core/` 新建 `EvaluationResult/TestCase/Constraint` 等重复结构；统一复用 `domain/models`（见 Project Structure Notes）。
- **禁止 silent fallback**：评估缺输入/配置时必须报 `EvaluatorError::InvalidInput`，不要“给个默认通过/默认 0 分”掩盖问题（AC: 6）。
- **禁止破坏序列化形状**：`EvaluationResult` 会被 ts-rs 导出给前端；如需扩展字段，优先写入 `extra`（AC: 1 + 回归保护）。
- **TeacherModel 评估的安全边界**：若引入 LLM judge，必须避免泄露敏感输入/密钥类信息到评估 prompt；并对超时/预算做防护（对应 `EvaluatorError::Timeout/ModelFailure`）。
  - MVP 实现要求：judge prompt 仅包含 `test_case_id + reference + output`（不序列化 `input/metadata`）；并对 `TeacherModel.generate` 增加超时保护（默认 60s，可用 `ctx.config.budget.max_duration_secs` 覆盖）；错误信息仅保留 `raw_excerpt`（截断）避免泄露/日志爆炸。

### Architecture & Compliance

- **模块边界**：评估器实现必须落在 `backend/src/core/evaluator/`；对外通过 `core::traits::Evaluator` 暴露。  
- **错误处理**：核心模块（core/）应以 `thiserror` 的类型安全错误为主；应用层（api/）才使用 `anyhow` 兜底包装（见 `docs/project-planning-artifacts/architecture.md` 的错误处理决策）。
- **测试位置**：优先同文件 `#[cfg(test)]` 单测；仅在需要端到端时使用 `backend/tests/`（本 Story 以单测为主）。

### UX Implications (for later Epics)

- UX 期望“评估节点”展示分数跳动与结果色块（见 `docs/project-planning-artifacts/ux-design-specification.md` 的评估节点描述），因此评估结果必须：
  - 可快速汇总为 `pass_rate` + `score_summary`
  - 可提供逐用例结果（用于“为什么更好”的结构化解释入口）

### Latest Tech Information (as of 2026-01-09)

> 目的：避免“看了最新文档但仓库版本不匹配”导致的实现偏差。除非单独开 Story/PR，不在本 Story 里升级依赖大版本。

- 当前 `backend/Cargo.toml` 关键依赖：`axum = 0.8`、`sqlx = 0.8`、`reqwest = 0.12`、`utoipa = 5`、`ts-rs = 10`、`thiserror = 2`
- 最新版本（参考 crates.io，可能高于当前锁定）：`axum 0.8.8`、`sqlx 0.8.6`、`reqwest 0.13.1`、`utoipa 5.4.0`、`ts-rs 11.1.0`、`thiserror 2.0.17`

### Project Structure Notes

- 目标落点（后端）
  - `backend/src/core/evaluator/`：新增 Layer 3 默认实现（`mod.rs`, `default_impl.rs`, `error.rs`）
  - `backend/src/core/mod.rs`：补充导出 `pub mod evaluator;`（保持 “7 Trait + IterationEngine” 的模块边界）
  - `backend/src/core/traits.rs`：仅改 Evaluator 部分以对齐技术规格（避免 scope creep 触碰其他 Trait）
- 一致性检查
  - `EvaluationResult` 与 `TaskReference/Constraint/QualityDimension` 必须复用 `backend/src/domain/models/algorithm.rs`
  - “任务级配置”结构（如 `EvaluatorType/threshold_percent/llm_judge_samples`）优先复用 `backend/src/domain/models/optimization_task_config.rs::EvaluatorConfig`，并通过 `ctx.extensions` 注入（不要在 core/ 新建重复配置结构）
  - 回归保护：不得修改 `EvaluationResult` 的字段语义与序列化形状（该结构会被 ts-rs 导出并被前端消费）；如确需调整，必须单独开 Story 并同步更新前端与生成脚本

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md#Story 4.3: 质量评估层（Layer 3: Quality Assessor）`
- PRD（Trait 概述与扩展性目标）：`docs/project-planning-artifacts/prd.md#7.4.2 模块化设计（Trait 概述）`、`docs/project-planning-artifacts/prd.md#7.4.3 扩展性保障`
- 架构（模块边界/错误处理/项目结构）：`docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`、`docs/project-planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules`
- 技术规格（Evaluator Trait/错误类型/评估器配置）：`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#Evaluator Trait`、`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#EvaluatorError`、`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#9.7 评估器配置`
- 现有 core Trait 占位实现：`backend/src/core/traits.rs`
- 评估结果领域模型：`backend/src/domain/models/algorithm.rs#EvaluationResult`

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

### Completion Notes List

- 对齐 `core::traits::Evaluator` 到技术规格：引入 `EvaluatorError`、补齐 `evaluate/evaluate_batch/name`
- 新增 `core/evaluator` 默认实现：ExactMatch/ConstraintCheck/SemanticSimilarity/TeacherModel + Ensemble（含可观测性 extra）
- 提供统计与排序辅助：split 过滤、pass-rate/mean(score)、按 `test_case_id` 组织逐用例结果
- 新增单测覆盖：Exact/Constrained/Hybrid、TaskReference 不匹配、`evaluate_batch` 同序、split+阈值（0.0/0.95/1.0）、缺少 task_evaluator_config、未知/重复 test_case_id、TeacherModel timeout/解析；`cargo test` 通过

### File List

- backend/src/core/mod.rs
- backend/src/core/traits.rs
- backend/src/core/evaluator/mod.rs
- backend/src/core/evaluator/default_impl.rs
- backend/src/core/evaluator/error.rs
- docs/implementation-artifacts/4-3-quality-assessment-layer.md
- docs/implementation-artifacts/sprint-status.yaml

## Change Log

- 2026-01-09: 落地 Layer 3 Evaluator Trait + 默认评估器实现与单测；支持逐用例评估、可观测性 extra、split 过滤统计与候选排序基础能力
- 2026-01-09: Code Review Fixes：Ensemble 路径接入 TeacherModel（可选注入、可诊断降级）；`evaluate_batch`/`build_evaluations_by_test_case_id` 增强输入校验（未知/重复 id）；TeacherModel 加 timeout + prompt 脱敏（仅 reference）+ 解析更稳（支持 fenced JSON / raw_excerpt 截断）；补齐回归测试
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [CRITICAL] 明确 `evaluate_batch` 必须同序返回，避免逐用例结果错位（`EvaluationResult` 本身不携带 `test_case_id`）。
- [CRITICAL] 明确任务级配置（`task_evaluator_config`）与算法级配置（`ctx.config.evaluator`）的职责边界与优先级，避免实现期读取混乱。
- [HIGH] 补齐 ConstraintCheck 最小约束集的 `name/params` 编码约定（复用 Story 2.6），避免评估器实现“猜 schema”。
- [HIGH] 明确 SemanticSimilarity 的轻量算法与阈值来源，避免引入重依赖或出现不可复现实现。
- [MEDIUM] 明确 split 过滤仅影响统计/排名，不影响 `evaluate_batch` 的逐用例输出。
- [HIGH] `evaluate_batch` 与 `test_case_id -> EvaluationResult` 组装必须拒绝重复/未知 id，避免静默覆盖与统计失真。
- [HIGH] Ensemble(Auto) 评估路径必须与 TeacherModel 依赖注入对齐：可用则纳入、不可用则显式降级（可诊断）。
- [MEDIUM] TeacherModel 必须具备最小安全护栏：timeout、prompt 脱敏、错误信息截断与更稳的 JSON 解析。

### Decisions

- MVP 保持 `EvaluatorError` 与技术规格一致：不新增细分错误分支；`TaskReference` 不匹配归类为 `InvalidInput`。
- 运行时避免双来源冲突：`llm_judge_samples` 以任务级配置为准，算法级作为兜底；编排层仍可选择同步以统一观测口径。
- 先保证确定性/可测：SemanticSimilarity 采用无依赖轻量算法；TeacherModel/Ensemble 的“更强版本”留到后续 Story。

### Risks / Tech Debt

- SemanticSimilarity 目前为轻量启发式实现；后续若需要更强语义判定，可在独立 Story 升级为 embedding/更可靠算法。
- TeacherModelEvaluator 的“更完整预算控制/系统性脱敏/稳健采样一致性策略”仍需后续 Story 细化与回归验证（本 Story 只落地最小安全护栏）。
- `core::traits` 其余 Trait 仍存在占位签名/anyhow 返回；按 Story 4.2 的 follow-up 在后续 Stories 逐步对齐技术规格（避免一次性大改）。

### Follow-ups

- [ ] 在后续 Story 引入更强 SemanticSimilarity（embedding/更稳定语义）并补齐回归用例
- [ ] 在后续 Story 落地真实 TeacherModel judge（含脱敏/预算/超时/采样一致性）
- [ ] 按技术规格逐步对齐 `core::traits` 其余 Trait 的签名与 typed errors（避免长期 contract 漂移）
