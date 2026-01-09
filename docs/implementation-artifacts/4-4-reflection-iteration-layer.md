# Story 4.4: 反思迭代层（Layer 4: Reflection Agent）

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 4-4-reflection-iteration-layer

## Key Decisions (MVP)

- **Trait/Type 对齐技术规格优先**：以 `docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md` 中的 `FeedbackAggregator/Optimizer` 签名与 `ReflectionResult/UnifiedReflection/OptimizationResult/TerminationReason` 为权威；允许对 `backend/src/core/traits.rs` 做破坏性调整（当前 Layer 4 仍是占位，不会破坏已实现 Layer 1-3）。
- **MVP 先做确定性反思/聚合/优化**：`aggregate/arbitrate/optimize_step/should_terminate` 先用可测试的确定性规则（投票/去重/优先级/阈值）；TeacherModel 仅作为未来可选注入，避免 LLM 输出不可复现导致 flaky 测试。
- **复用 Layer 3 统计/排序口径**：通过 `backend/src/core/evaluator/default_impl.rs` 的 `summarize_for_stats` / `rank_candidates`（以及 split 过滤策略）得到 pass_rate/mean_score；禁止在 Layer 4 重新实现排名/阈值口径，避免“同一轮不同模块口径漂移”。
- **终止口径写死**：阈值判断以 Validation 口径为主（当 `OptimizationContext.config.data_split.enabled=true` 时：Validation + Unassigned；Holdout 不参与每轮判断），对齐 Story 3.2 与 Story 4.3；终止原因用 `TerminationReason::*` 结构化输出（WS/UI/Checkpoint 可消费）。
- **Context 只由编排层写**：Layer 4 只读 `&OptimizationContext`，输出 `OptimizationResult`；由编排层决定是否写回 `ctx.current_prompt/ctx.rule_system/ctx.checkpoints`（避免未来引入分支/断点续跑时状态被模块私自修改）。

## Story

As a 系统,
I want 对本轮迭代进行反思并决定下一步行动,
so that 可以持续改进 Prompt 直到达成目标或触发终止条件。

## Acceptance Criteria

1. **Given** Layer 3 已产生“候选排序/统计”与“逐用例评估明细”（至少可得到 best candidate 的 `pass_rate/mean_score`，并能追溯每个失败用例的 `EvaluationResult.failure_points`）  
   **When** Layer 4 执行反思（进入 `IterationState::Reflecting`）  
   **Then** 必须生成可被机器消费的结构化反思输出：至少包含
   - `ReflectionResult`（面向单个失败聚类/失败用例集合的分析与建议）
   - `UnifiedReflection`（对多个 `ReflectionResult` 的聚合/去重/排序/冲突仲裁结果）
   - `OptimizationResult`（本轮下一步行动与是否终止）
2. **Given** 本轮 best candidate 的表现优于当前 Prompt（优先比较 `pass_rate`，再比较 `mean_score`；同分保持稳定）  
   **When** 反思完成并进入“下一步行动决策”  
   **Then** Layer 4 必须产出“采用 best candidate 作为下一轮 current_prompt”的可观测信号（例如 `OptimizationResult.primary.source` + `improvement_summary`），并且由编排层据此更新 `ctx.current_prompt`。
3. **Given** best candidate 的通过率达到 `ctx.config.iteration.pass_threshold`（默认 0.95；口径：split 开启时按 Validation + Unassigned 统计，Holdout 不参与每轮）  
   **When** 反思完成  
   **Then** 必须建议终止迭代：`OptimizationResult.should_terminate=true`，并填充 `termination_reason`：
   - `AllTestsPassed`：当口径内 `pass_rate == 1.0`（全部通过）
   - `PassThresholdReached { threshold, actual }`：当 `pass_rate >= threshold` 且未达到 1.0
4. **Given** 已达到最大迭代轮数（`ctx.config.iteration.max_iterations`；与任务配置 `OptimizationTaskConfig.max_iterations` 对齐）  
   **When** Layer 4 判断是否继续  
   **Then** 必须终止：`TerminationReason::MaxIterationsReached { max }`，并给出明确的 `improvement_summary`（包含“历史最佳候选/当前候选”的关键信息），供 UI 与报告展示。
5. **Given** best candidate 未优于当前 Prompt，且连续多轮没有提升（震荡/停滞）  
   **When** Layer 4 判断策略  
   **Then** 必须给出可执行的下一步行动建议（`UnifiedReflection.recommended_action`），至少支持：
   - `InjectDiversity`：当达到 `ctx.config.iteration.diversity_inject_after`（或等价阈值）
   - `RequestHumanIntervention`：当建议冲突无法仲裁、或置信度长期低于阈值（参考 `ctx.config.evaluator.confidence_*`）  
   **And** 所有“为什么要切换策略”的理由必须结构化记录在 `UnifiedReflection.extra` 或 `OptimizationResult.extra`（禁止只写日志）。
6. **Given** 输入不完整或不一致（例如：`reflections` 为空、冲突仲裁失败、history 状态不一致）  
   **When** Layer 4 执行聚合/优化  
   **Then** 必须返回可诊断错误（`AggregatorError`/`OptimizerError`），不得 silent fallback；错误信息不得包含敏感原文（Prompt/TestCase input 的全文）。
7. **Given** 多个终止条件在同一轮同时满足（例如：`pass_rate == 1.0` 且已到达 `max_iterations`）  
   **When** Layer 4 需要返回 `termination_reason`  
   **Then** 必须按“终止优先级（MUST）”选择唯一的 `TerminationReason`（避免 UI/报告出现互相矛盾的终止原因）。

## Tasks / Subtasks

- [x] 落地 Layer 4 的领域结构（AC: 1,3,4,5）
  - [x] 在 `backend/src/domain/models/` 中补齐/对齐技术规格类型：`TerminationReason`、`CandidateSource`、`ReflectionResult`、`UnifiedReflection`、`OptimizationResult`（以及必要的子结构）
  - [x] 明确哪些结构需要 `ts-rs` 导出给前端（建议：`UnifiedReflection` 与 `OptimizationResult` 需要，`ReflectionResult` 可选）
- [x] 对齐 `core::traits` 的 Layer 4 Trait 形状（AC: 1,6）
  - [x] 将 `FeedbackAggregator` 改为：`aggregate(ctx, reflections) -> Result<UnifiedReflection, AggregatorError>` + `arbitrate(ctx, conflicts) -> Result<ArbitrationResult, AggregatorError>` + `name()`
  - [x] 将 `Optimizer` 改为：`optimize_step(ctx, unified_reflection) -> Result<OptimizationResult, OptimizerError>` + `should_terminate(ctx, history) -> Option<TerminationReason>` + `name()`
  - [x] 移除/替换 `AggregatedFeedback` 占位类型（避免继续扩散）
- [x] 实现 `core/feedback_aggregator` 默认实现（AC: 1,5,6）
  - [x] `aggregate`：投票决定 `primary_failure_type`；建议去重/合并/排序形成 `unified_suggestions`
  - [x] 冲突检测：至少识别 `AddRule` vs `RemoveRule` / “同一规则修改互斥” 等直接矛盾，填充 `has_conflicts/conflicts`
  - [x] `arbitrate`：MVP 用确定性仲裁（Voting 或 KeepAll），并把理由写入 `ArbitrationResult.reasoning`
- [x] 实现 `core/optimizer` 默认实现（AC: 2,3,4,5,6）
  - [x] `optimize_step`：根据 `UnifiedReflection.recommended_action` 与统计结果产出 `OptimizationResult`（primary prompt 由编排层提供或通过 extensions 提供）
  - [x] `should_terminate`：统一封装终止条件（pass_threshold/max_iterations/用户停止/震荡）并返回 `TerminationReason`
- [x] 编排层对接约束（本 Story 只写死契约，不要求完成 IterationEngine）（AC: 2,3,4）
  - [x] 编排层必须负责：更新 `ctx.current_prompt`、推进 `ctx.state`、写入 checkpoints、注入 `ctx.extensions`（例如：candidate stats、best candidate 内容/索引、当前与历史指标）
  - [x] 编排层不得绕过 Layer 3 的排序口径（复用 `rank_candidates` 结果）
- [x] 测试（与 CI 门禁对齐）（AC: 1-6）
  - [x] 单元测试：`aggregate`（空输入/去重/冲突）与 `arbitrate`（可复现仲裁）
  - [x] 单元测试：`should_terminate`（pass_threshold=0.95/1.0、max_iterations、震荡阈值）  
  - [x] 回归保护：不得改动 Layer 3 的通过率口径与 `EvaluationResult` 序列化形状（前端依赖）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][HIGH] 明确 Layer 4 需要从编排层拿到哪些“候选对比数据”（current vs best candidate 的 stats）以及注入路径（建议固定 extensions key 常量，避免魔法字符串扩散）。
- [x] [AI-Review][HIGH] 明确 TerminationReason 的判定优先级（AllPassed > PassThresholdReached > MaxIterationsReached > Oscillation/UserStopped/HumanIntervention），避免 UI/报告出现互相矛盾的终止原因。
- [x] [AI-Review][MEDIUM] 明确 “置信度门控” 与 “可自动更新的建议类型” 的关系（参考技术规格 confidence 表格）：低置信度下禁止自动 Add/Remove/ModifyRule。

## Dev Notes

### What Exists Today (to reuse, not reinvent)

- Layer 1-3 已落地：
  - Layer 1 `core/rule_engine`（含 `polarity=all_passed` 信号）  
  - Layer 2 `core/prompt_generator`（确定性模板变体 + `candidate_index`）  
  - Layer 3 `core/evaluator`（逐用例 `EvaluationResult` + `failure_points` + `rank_candidates/summarize_for_stats`）
- `OptimizationContext`（含 `config.iteration.max_iterations/pass_threshold/diversity_inject_after` 与 `extensions` 扩展点）在 `backend/src/domain/types/optimization_context.rs`，并且“仅编排层可写”的纪律已在 Story 4.1/4.2/4.3 中写死。
- 任务级配置（max_iterations / pass_threshold_percent / candidate_prompt_count / data_split）在 `backend/src/domain/models/optimization_task_config.rs`；编排层负责把 percent/ratio 映射到 `OptimizationContext.config.*`（Layer 4 不承担映射逻辑）。

### Non-goals (防 scope creep)

- 不在本 Story 实现完整 IterationEngine/OptimizationEngine（编排、checkpoint 持久化、WS 推送会在后续 Story 统一推进）。
- 不在本 Story 设计或实现“Racing/并行调度”细节（见 Story 4.5）。
- 不强制引入 TeacherModel 驱动的反思（MVP 先确定性实现；TeacherModel 反思属于 Phase 2/增强项）。

### Developer Context（实现护栏：避免偏航/回归）

1) **口径一致性是第一优先级**：pass_rate/mean_score 的计算、候选排名、split 过滤必须复用 Layer 3 已实现逻辑（避免 Layer 4 重复实现造成口径漂移）。  
2) **结构化输出优先于日志**：UI/报告/断点续跑都依赖结构化数据；日志只能作为补充，且必须遵循脱敏规则。  
3) **禁止改动已对外导出的序列化形状**：`EvaluationResult/TestCase/Rule/RuleSystem` 已被 `ts-rs` 导出并被前端消费；本 Story 只能新增类型/字段（或新建文件），避免破坏前端。  
4) **错误必须可诊断但不得泄露**：错误信息可包含 ID/计数/阈值/状态，但不得包含 `current_prompt`、TestCase `input` 原文等敏感内容（对齐既有 log_sanitizer 方向）。

### Technical Requirements（实现建议）

#### 1) 数据结构与模块边界（建议落点）

- 领域结构（建议在 `backend/src/domain/models/algorithm.rs` 同域集中管理，或新增 `backend/src/domain/models/reflection.rs` 并在 `domain/models/mod.rs` 导出）：
  - `TerminationReason`
  - `CandidateSource`
  - `ReflectionResult/FailureType/Suggestion/SuggestionType`
  - `UnifiedReflection/UnifiedSuggestion/SuggestionConflict/ConflictType/ArbitrationResult/ArbitrationMethod/RecommendedAction`
  - `OptimizationResult/PromptCandidate`
- core 层模块：
  - `backend/src/core/feedback_aggregator/`：`mod.rs`、`default_impl.rs`、`error.rs`
  - `backend/src/core/optimizer/`：`mod.rs`、`default_impl.rs`、`error.rs`

#### 2) 与 Layer 3 的数据对接（extensions 契约：MUST）

为避免实现期“字段名来回调整 / 魔法字符串扩散”，编排层（Orchestrator）**必须**在进入 Layer 4 前写入以下 `ctx.extensions` 字段；Layer 4 必须读取并校验它们：

- 缺失/类型不匹配：必须返回可诊断错误（`AggregatorError` / `OptimizerError`），不得 silent fallback。
- key 必须稳定：**必须**以 `pub const` 形式集中定义并复用（推荐：`backend/src/core/optimizer/mod.rs`，必要时再由其他模块引用）。

| Key | JSON shape（示例形状，不要在代码里直接使用 Rust 类型名） | Producer | Consumer |
|---|---|---|---|
| `layer4.candidate_ranking` | `[{ "candidate_index": number, "pass_rate": number, "mean_score": number }]` | Orchestrator（来自 Layer 3 `rank_candidates` 的结果） | `Optimizer::optimize_step` / `Optimizer::should_terminate` |
| `layer4.best_candidate_index` | `number` | Orchestrator | `Optimizer::optimize_step` |
| `layer4.best_candidate_prompt` | `string` | Orchestrator | `Optimizer::optimize_step` |
| `layer4.current_prompt_stats` | `{ "pass_rate": number, "mean_score": number }` | Orchestrator（复用 Layer 3 `summarize_for_stats` 口径） | `Optimizer::optimize_step` / `Optimizer::should_terminate` |
| `layer4.best_candidate_stats` | `{ "pass_rate": number, "mean_score": number }` | Orchestrator（复用 Layer 3 `summarize_for_stats` 口径） | `Optimizer::optimize_step` / `Optimizer::should_terminate` |
| `layer1_test_results.evaluations_by_test_case_id` | `{ [test_case_id: string]: EvaluationResult }` | Orchestrator（由 Layer 3 输出组装；对齐 Story 4.1/4.3 既有约定） | `FeedbackAggregator::aggregate`（用于追溯 failure_points） |

> 兼容性约束：`layer1_test_results.evaluations_by_test_case_id` 为既有约定；本 Story 不引入新的“同义 key”来替换它。如后续确需迁移，必须在编排层实现向后兼容（双写/映射）并加测试。

#### 2.1) Checkpoint / WS 最小兼容字段集（forward contract）

本 Story 仍不实现 checkpoint 持久化与 WS 推送，但为避免后续 UI/导出/断点续跑对接返工，编排层在“落盘/推送”时至少要能携带并保持以下字段语义稳定（可扩展，但不要删改语义）：

- `OptimizationResult`：`iteration`、`should_terminate`、`termination_reason`、`improvement_summary`、`primary.content`、`primary.source`
- `UnifiedReflection`：`primary_failure_type`、`recommended_action`、`has_conflicts`

#### 3) 终止与策略切换（优先级：MUST）

当同一轮出现多个终止条件同时满足时，必须按以下优先级选择唯一 `TerminationReason`（避免互相矛盾）：

1. `AllTestsPassed`
2. `PassThresholdReached`
3. `MaxIterationsReached`
4. `OscillationDetected`
5. `UserStopped`
6. `HumanInterventionRequired`

#### 4) 置信度门控（对齐技术规格与 Layer 3）

- 当 `EvaluationResult.confidence` 可用时，按 `ctx.config.evaluator.confidence_high_threshold/low_threshold` 做门控：
  - `>= high`：允许产生规则层建议（Add/Modify/RemoveRule）
  - `[low, high)`：只允许表达层建议（ChangeFormat/Rephrase/AddExample/AddConstraint）
  - `< low`：禁止自动化建议；推荐 `RequestHumanIntervention` 或 `InjectDiversity`

### Architecture / Compliance (必须遵守)

- 项目结构与边界：遵循 `docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`（core 只通过 Trait 边界被 API 层调用；domain 模型复用；不得把 core 逻辑塞进 api/routes）。
- 命名与序列化：对齐现有 `ts-rs` 生成物：字段名保持 `snake_case`（例如 `test_case_id/latency_ms`），**不要**为这些结构引入 `serde(rename_all = "camelCase")` 或改变既有序列化形状。
- WebSocket 事件命名：`{domain}:{action}`（例如未来可能的 `iteration:reflecting` / `iteration:completed`），结构复用 `backend/src/api/ws/events.rs::WsMessage<T>`。

### Library / Framework Requirements（版本与依赖边界）

- Rust：以 `Cargo.lock` / `Cargo.toml` 为准；本 Story **不**升级/替换依赖（避免无关 breaking changes）。
- Frontend：本 Story 不要求动前端依赖；保持 `react-router` 版本不降级（见 `frontend/package.json`）。

### File Structure Requirements（建议改动清单）

**后端（Rust）**

- `backend/src/core/traits.rs`：仅改 `FeedbackAggregator/Optimizer` 签名与相关导入（避免触碰已稳定的 Layer 1-3 Trait）。
- `backend/src/core/mod.rs`：新增 `pub mod feedback_aggregator; pub mod optimizer;`（与已有 `rule_engine/prompt_generator/evaluator` 对齐）。
- `backend/src/core/feedback_aggregator/mod.rs`、`backend/src/core/feedback_aggregator/default_impl.rs`、`backend/src/core/feedback_aggregator/error.rs`
- `backend/src/core/optimizer/mod.rs`、`backend/src/core/optimizer/default_impl.rs`、`backend/src/core/optimizer/error.rs`
- `backend/src/domain/models/algorithm.rs`（或新增 `backend/src/domain/models/reflection.rs`）：补齐/导出技术规格要求的结构（并按需增加 `TS` 导出）

### Testing Requirements（建议）

- 单元测试（Rust）建议位置：
  - `backend/src/core/feedback_aggregator/default_impl.rs`：`#[cfg(test)]` 覆盖 merge/冲突/仲裁可复现
  - `backend/src/core/optimizer/default_impl.rs`：`#[cfg(test)]` 覆盖终止判定优先级、阈值边界、震荡判定
- 不需要前端测试变更（除非本 Story 额外暴露了新 API/WS payload —— 这不在本 Story 目标内）。

### Previous Story Intelligence (4.3 → 4.4)

- Layer 3 已明确：`failure_points` 是 Layer 4 的结构化输入（至少包含 `dimension/description/severity`），且排序/统计口径（pass_rate/mean_score/split 过滤）已写死在 `core/evaluator`。Layer 4 必须复用该口径，避免“评分看起来提升但阈值判断不一致”的灾难。
- `EvaluationResult` 本身不携带 `test_case_id`；Layer 3 通过“同序返回 + 由编排层 index 对齐”构建 `test_case_id -> EvaluationResult`。Layer 4 如果需要追溯失败用例，必须依赖该映射而不是猜测。

### Git Intelligence (recent patterns)

- 近期 core 模块落地模式稳定：每个 Layer 一个目录（`mod.rs + default_impl.rs + error.rs`），Trait 在 `core/traits.rs` 聚合，对外 module 在 `core/mod.rs` 导出；同时更新对应 story 文档与 `docs/implementation-artifacts/sprint-status.yaml`（见最近提交：`feat(core): complete Layer1+Layer2`、`fix(core): harden evaluator contract...`）。

### Latest Technical Information (dependency guardrails)

- 本 Story 不做依赖升级：实现期如遇“需要新增/升级依赖”的冲动，先回到需求与现有实现检查是否真必要；必须升级时再做单独 Story/PR。

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md#Story 4.4: 反思迭代层（Layer 4: Reflection Agent）`
- PRD（Layer 4 职责与反思报告）：`docs/project-planning-artifacts/prd.md#7.4 后端架构`、`docs/project-planning-artifacts/prd.md#7.4.2 模块化设计（Trait 概述）`
- 技术规格（权威）：`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#4.2.3 ReflectionResult 结构定义`、`#4.2.4 UnifiedReflection 结构定义`、`#4.2.2 OptimizationResult 结构定义`、`#FeedbackAggregator Trait`、`#Optimizer Trait`、`#AggregatorError`、`#OptimizerError`
- 架构/结构/一致性约束：`docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`、`docs/project-planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules`
- 现状代码（Layer 1-3 与统计口径）：`backend/src/core/rule_engine/`、`backend/src/core/prompt_generator/`、`backend/src/core/evaluator/`（尤其 `backend/src/core/evaluator/default_impl.rs`）
- 任务配置口径（max_iterations / pass_threshold_percent）：`backend/src/domain/models/optimization_task_config.rs`
- UX（反思报告承载位置）：`docs/project-planning-artifacts/ux-design-specification.md#Run View（核心体验主舞台）`

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

Backend：

- `cd backend && cargo fmt --check`
- `cd backend && cargo clippy -- -D warnings`
- `cd backend && cargo test`

Frontend：

- `cd frontend && npm test -- --run`
- `cd frontend && npm run lint`

### Completion Notes List

1. 领域模型：新增 Layer 4 结构（`ReflectionResult/UnifiedReflection/OptimizationResult/TerminationReason/...`），对齐技术规格。
2. Trait 契约：对齐 `core::traits::FeedbackAggregator/Optimizer`，移除 `AggregatedFeedback` 占位并引入结构化错误。
3. 默认实现：落地确定性 `DefaultFeedbackAggregator`（投票/去重/冲突/仲裁/推荐行动）与 `DefaultOptimizer`（采用信号、终止优先级、震荡判定）。
4. 测试：新增单测覆盖聚合空输入/冲突/多样性注入、终止阈值与震荡；全量回归通过。

### File List

- backend/src/core/evaluator/default_impl.rs
- backend/src/core/feedback_aggregator/default_impl.rs
- backend/src/core/feedback_aggregator/error.rs
- backend/src/core/feedback_aggregator/mod.rs
- backend/src/core/mod.rs
- backend/src/core/optimizer/default_impl.rs
- backend/src/core/optimizer/error.rs
- backend/src/core/optimizer/mod.rs
- backend/src/core/traits.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/models/reflection.rs
- backend/src/domain/types/extensions.rs
- backend/src/domain/types/mod.rs
- docs/implementation-artifacts/4-4-reflection-iteration-layer.md
- docs/implementation-artifacts/sprint-status.yaml

## Change Log

- 2026-01-09: 创建 Story 4.4（ready-for-dev），补齐 Layer 4 的 Trait/类型对齐、MVP 确定性实现策略、终止口径与对接护栏
- 2026-01-09: 落地 Layer 4 FeedbackAggregator/Optimizer（领域模型、Trait 对齐、默认实现与单测），并通过后端/前端回归，状态推进至 `review`
- 2026-01-09: 根据代码审查修复：统一 extensions 契约与 CandidateStats、减少 core 模块耦合、补齐仲裁/置信度/震荡相关单测与终止信号一致性

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] `core/feedback_aggregator` 依赖 `core/optimizer` 的 extensions 常量，削弱模块可替换性；已将 extensions 约定集中到 `domain/types/extensions.rs`，避免 core 模块互相依赖。
- [x] [HIGH] `optimize_step` 与 `should_terminate` 的终止信号不一致风险（尤其是震荡判定）；已提取基础终止规则复用，并支持编排层注入 `layer4.recent_primary_scores` 让 `optimize_step` 也能给出一致的震荡终止信号。
- [x] [MEDIUM] `CandidateStats` 在多个模块重复定义；已抽为共享类型 `CandidateStats`。
- [x] [MEDIUM] 置信度门控/仲裁缺少边界与直接覆盖；已补齐关键单测（仲裁空输入/等置信度、confidence low/high 边界）。
- [x] [LOW] Layer 3 统计函数存在 `#[allow(dead_code)]` 导致误解；已移除。

### Decisions

- [x] 扩展键常量集中到 `domain/types/extensions.rs`：以“编排层注入/消费契约”为中心，避免将常量放在某个具体模块导致其他模块反向依赖。
- [x] 震荡终止在 `optimize_step` 中采用“可选注入”方案：不改变 Trait 签名、不强制编排层立即实现，但提供一致终止信号的升级路径。

### Risks / Tech Debt

- [x] `ReflectionResult` 的生成（Reflection Agent 本体）仍未实现：当前 Story 主要落地 Aggregator/Optimizer 的契约与确定性实现；后续需要在编排层（或未来模块）产出 ReflectionResult 并串起来才能形成端到端“反思→优化”闭环。
- [x] `OptimizationResult.primary.source` 目前仍是从 `recommended_action` 推导的启发式映射，可能与“候选真实来源”不完全一致；如后续要做报表/归因，建议由编排层注入真实来源字段或新增扩展键。

### Follow-ups

- [ ] [AI-Review][MEDIUM] 编排层注入 `layer4.recent_primary_scores`（历史 primary.score 序列）以便 `optimize_step` 输出一致的震荡终止信号。
- [ ] [AI-Review][LOW] 明确 `OptimizationResult.primary.source` 的真实来源注入策略（避免启发式映射在报表中造成误读）。
