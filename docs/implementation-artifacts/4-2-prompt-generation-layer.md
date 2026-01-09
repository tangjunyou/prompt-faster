# Story 4.2: Prompt 生成层（Layer 2: Prompt Engineer）

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 4-2-prompt-generation-layer

## Key Decisions (MVP)

- **方案 A：Trait 单候选，编排层多次调用聚合多候选（对齐技术规格 `#4.2.7`）**
  - Layer 2：`#[async_trait] async fn generate(&self, ctx: &OptimizationContext) -> Result<String, GeneratorError>`
  - 编排层（不在本 Story 实现）：按 `OptimizationTaskConfig.candidate_prompt_count` 循环调用 `generate()`，并在每次调用前写入不同的 `ctx.extensions["candidate_index"]`
  - `all_passed` 终止：编排层在调用前短路处理（Layer 2 不承担 “skip 信号” 的返回类型设计）
- **确定性优先**：MVP 只做确定性/可测试启发式（不调用 TeacherModel），并用 `candidate_index` 驱动模板变体（同输入可复现）

## Story

As a 系统,
I want 基于 Layer 1 输出的成功/失败规律生成候选 Prompt,
so that 能用更少的试错快速提升测试集通过率，并尽量不破坏已成功用例的特性。

## Acceptance Criteria

1. **Given** `OptimizationContext.rule_system.rules` 中包含来自 Layer 1 的规律（`Rule.tags.extra["polarity"]` ∈ `success|failure|all_passed`，且 `source_test_cases` 可追溯证据）  
   **When** Layer 2 执行候选生成（`PromptGenerator.generate(ctx)`，或默认实现的候选生成入口）  
   **Then** 必须生成候选 Prompt，并确保每个候选都：
   - 显式尝试解决至少 1 条 failure 规律（且优先覆盖“失败维度”更集中的失败模式；见下文“失败维度映射说明”）
   - 显式保留 success 规律所代表的“成功特性”（例如输出格式/结构/关键概念/约束满足点）。
2. **Given** 优化任务配置了候选生成数量（FR22，对应 `OptimizationTaskConfig.candidate_prompt_count`）  
   **When** 编排层生成候选 Prompt 集合（调用 Layer 2）  
   **Then** 编排层必须按配置次数循环调用 `PromptGenerator.generate(ctx)`，并在每次调用前写入不同的 `ctx.extensions["candidate_index"]`（通常为 `0..candidate_prompt_count-1`）；不得 silently fallback。
3. **Given** `Rule.tags.extra["polarity"] = "all_passed"` 的规则存在（来自 Layer 1 all-pass 信号）  
   **When** 编排层准备进入候选生成阶段  
   **Then** 编排层必须短路为 no-op：不调用 Layer 2（不生成新候选），并返回“跳过候选生成”的可观测信号（例如状态/日志），供编排层直接终止或跳过后续层。
4. **Given** 用户初始 Prompt 为空（等价于 `ctx.current_prompt` 为空/仅空白）  
   **When** 首次执行 Layer 2  
   **Then** 必须基于“优化目标 + 测试集信息”生成初始候选 Prompt；且所需信息必须来自 `ctx.extensions`（不得擅自给 `OptimizationContext` 加字段）。  
   **And** 若必需字段缺失（例如 `optimization_goal`），必须返回包含缺失 key 列表的可诊断错误。
5. **Given** Layer 1 规律缺失/为空，或 polarity 不符合约定  
   **When** Layer 2 执行候选生成  
   **Then** 必须返回可诊断错误（不得生成“拍脑袋 Prompt”），以避免无依据的迭代导致回归。

## Tasks / Subtasks

- [x] 对齐 Layer 2 的输入/输出契约（AC: 1,2,3,4,5）
  - [x] 对齐 `core::traits::PromptGenerator` 的签名到技术规格 `#4.2.7 PromptGenerator Trait`
    - [x] 形状：`#[async_trait] async fn generate(&self, ctx: &OptimizationContext) -> Result<String, GeneratorError>; fn name(&self) -> &str;`
    - [x] 移除 `rules: &[Rule]` 入参（直接从 `ctx.rule_system.rules` 读取）
    - [x] 禁止 `anyhow::Result<Vec<String>>` 形状继续扩散（避免后续 Story 返工）
  - [x] 明确编排层的“多候选生成”职责（FR22，不在本 Story 实现）
    - [x] 编排层读取 `OptimizationTaskConfig.candidate_prompt_count`，循环调用 `PromptGenerator.generate(ctx)` 生成多候选
    - [x] 编排层每次调用前写入 `ctx.extensions["candidate_index"]`（通常 `0..candidate_prompt_count-1`）
    - [x] 若 `candidate_prompt_count` 缺失/不可用，必须返回可诊断错误（不得 silently fallback）
  - [x] 明确 Layer 2 的“优化目标”来源：由编排层写入 `ctx.extensions["optimization_goal"]`（string，必需用于初始 Prompt）；缺失必须报错
  - [x] 明确 all-pass 信号：编排层在进入候选生成前检查 `polarity=all_passed` 并短路（不调用 Layer 2）
- [x] 实现 `core/prompt_generator` 的默认候选生成（AC: 1,2,3,4,5）
  - [x] 只使用“确定性/可测试”的启发式生成（MVP 不依赖 TeacherModel 调用，避免引入不可控输出与额外成本）
  - [x] 失败规律消费：按 `Rule.tags.extra["polarity"]="failure"` 筛选并聚合，将“触发条件/失败表现/建议修复方向”转为 Prompt 中的显式指令
  - [x] 成功规律保护：按 `polarity="success"` 抽取“必须保持项”，写入 Prompt 的不可破坏约束区
  - [x] 初始 Prompt：当 `ctx.current_prompt` 为空时，从 `optimization_goal` + `ctx.test_cases[*].reference`（尤其是 `TaskReference::Constrained` 的 constraints/quality_dimensions）推导输出约束与任务意图
  - [x] 候选多样性（FR22）：通过“模板变体 + `candidate_index`”驱动单次 `generate()` 的输出形态（不使用随机数；同输入可复现）
- [x] 测试与可维护性（AC: 1,2,3,4,5）
  - [x] 单元测试覆盖：混合 success/failure、缺失 `candidate_index`、缺失 `optimization_goal`、初始 Prompt 生成、polarity 非法/规律为空报错、不同 `candidate_index` 下模板变体可复现
  - [x] 不引入新依赖（优先复用现有 `serde_json/uuid/thiserror/tracing`）；如确需新增，必须在 Story 中记录动机与替代方案

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][HIGH] `core::traits` 目前的 `PromptGenerator/Evaluator/Optimizer` 签名与技术规格不一致：在本 Story 以 PromptGenerator 为主先对齐（其余 Trait 放到后续对应 Story 统一校准，避免 scope creep）
- [ ] [AI-Review][MEDIUM] 明确编排层对 `all_passed` 的短路语义（不调用 Layer 2），并在编排层对应 Story/测试中固定下来，避免误判

## Dev Notes

### What Exists Today (to reuse, not reinvent)

- `OptimizationContext` 已按技术规格（2025-12-14）落地在 `backend/src/domain/types/optimization_context.rs`，并包含 `extensions` 扩展点（本 Story 必须复用该扩展点，不得加字段）
- Layer 1（RuleEngine）默认实现已在 `backend/src/core/rule_engine/` 落地，输出 `Rule.tags.extra["polarity"]`（`success|failure|all_passed`）供 Layer 2 消费
- 用户侧候选数量配置已在 `backend/src/domain/models/optimization_task_config.rs`（`candidate_prompt_count`）；编排层读取该配置并控制调用次数（不要求写入 `ctx.extensions`，避免让 Layer 2 依赖候选数量）

### Non-goals (to avoid scope creep)

- 不实现 TeacherModel 驱动的候选生成（本 Story 只做确定性启发式 + 可测试）
- 不实现 Layer 3/4、也不实现完整编排引擎（仅定义 Layer 2 的契约与默认实现；编排层写入 `extensions` 的约定在本 Story 说明即可）
- 不引入 Racing/预算控制等 Phase 2 机制（即使 `OptimizationContext.config.racing` 已存在，也只作为“候选多样性/数量”的潜在 fallback 讨论，不作为强依赖）

### Implementation Notes (MVP)

- 建议把 `ctx.extensions` 的 key 提取为常量（避免魔法字符串扩散）：`EXT_OPTIMIZATION_GOAL`、`EXT_CANDIDATE_INDEX`
- 聚合/排序规则时保持确定性（例如按维度名排序或使用 `BTreeMap`），避免“同输入不同输出”的 flaky 测试
- `candidate_index` 仅用于选择模板变体：不要把索引文本写进最终 Prompt（避免影响模型行为）

## Technical Requirements

- **输入契约（只读）**
  - 规律来源：`ctx.rule_system.rules`（复用 `backend/src/domain/models/algorithm.rs` 的 `Rule/RuleTags`；不得重复建模）
  - 极少量必需扩展（由编排层写入，Layer 2 只读）：
    - `ctx.extensions["optimization_goal"]`：string，必需（仅在 `ctx.current_prompt` 为空时使用）
    - `ctx.extensions["candidate_index"]`：number（u32），必需（用于确定性变体；多候选由编排层循环调用并写入不同 index）
    - `candidate_index` 取值范围：`0..10`（对应 10 个确定性模板变体）；超出范围必须返回可诊断错误，避免静默重复候选
- **输出契约**
  - 生成的候选 Prompt 必须为“可直接替换 current_prompt”的完整 Prompt 文本（非 delta patch）
  - 同一输入 + 同一 `candidate_index` 必须产生相同输出（可复现）；不同 `candidate_index` 应尽量触发不同模板变体（减少编排层去重压力）
- **错误处理（不得静默）**
  - 缺少必需扩展 key：返回 `GeneratorError::MissingContext { keys: Vec<String> }`
  - 规律为空/不符合 polarity 约定：返回 `GeneratorError::InvalidRules { reason: String }`
  - `candidate_index` 越界：返回 `GeneratorError::InvalidContext { reason: String }`
  - 检测到 `polarity=all_passed`：返回 `GeneratorError::AllPassed { rule_ids: Vec<String> }`（编排层应短路避免触发）
- **可观测性**
  - 必须用 `tracing` 输出关键信息：`candidate_index`、`rules` 数量/分类（success/failure/all_passed）、missing keys、失败规律覆盖情况（避免“生成了但不知道为什么”）
  - 日志不得输出完整 prompt / test_case input 原文（仅输出计数、ID、摘要/维度名），避免泄露用户数据

### Deterministic Template Variants (candidate_index)

> 用于保证 “多候选” 可复现且不依赖随机数；编排层通常写入 `candidate_index = 0..candidate_prompt_count-1`（`candidate_prompt_count` 最大为 10）。

- `0`：基础版（规则优先：success 保持区 + failure 修复区）
- `1`：结构优先版（更强调输出格式/结构的“不可破坏”约束）
- `2`：失败聚焦版（按失败维度聚合后逐条列出修复指令）
- `3`：示例驱动版（在不泄露真实输入的前提下，给出抽象示例/反例结构）
- `4`：检查清单版（把关键约束转为可执行 checklist）
- `5`：最小改动版（对当前 Prompt 做最小增量强化，降低回归风险）
- `6`：强约束版（“必须/不得”更明确，适合格式类失败）
- `7`：解释优先版（要求输出前先做自检/解释，但保持最终格式不变）
- `8`：维度权重版（对 `TaskReference::Constrained.quality_dimensions` 加强强调）
- `9`：冲突仲裁版（若 success/failure 约束冲突，要求显式优先级与折中方案）

> **失败维度映射说明**：当前实现把 failure 规律的“维度”映射为 `Rule.tags.semantic_focus[0]`（为空则归类为 `unknown`），用于按维度聚合与排序。

## Architecture Compliance

- **模块边界**
  - `core/` 只放算法模块（Trait + 默认实现），不把“业务编排/状态机/持久化”逻辑塞进 Layer 2
  - `domain/models/` 为权威 DTO；Layer 2 只能复用，不能新建 `Rule/TestCase` 的同名/近似结构
- **Context 纪律**
  - Layer 2 只能读 `&OptimizationContext`；需要额外输入一律通过 `ctx.extensions` 约定（由编排层写入）
  - 不允许在 Layer 2 内部“偷偷写 ctx”（如果需要候选索引，由编排层注入 `candidate_index` 到 `extensions`）
- **与 Layer 1 对齐**
  - 只消费 Layer 1 产物的既定约定：`Rule.tags.extra["polarity"]` 的三态 `success|failure|all_passed`
- 不重算测试结果、不读取 `ctx.extensions["layer1_test_results"]`（那是 Layer 1 的输入；Layer 2 应消费规则而不是 raw results）

## Library / Framework Requirements

- **Rust / 后端依赖基线（以仓库为准）**
  - Rust edition：`2024`；`rust-version = 1.85`（见 `backend/Cargo.toml`）
  - 主要依赖：`axum 0.8`、`tokio 1`、`sqlx 0.8`、`serde 1`、`async-trait 0.1`、`uuid 1`、`utoipa 5`、`ts-rs 10`
- **禁止隐式升级**
- 本 Story 不要求升级任何依赖；不允许“顺手升级”来解决编译/风格问题（若未来必须升级，请单独开 Story/PR 并跑全量测试）

## File Structure Requirements

- **目标落点（后端）**
  - `backend/src/core/prompt_generator/`
    - `mod.rs`：导出默认实现与错误类型
    - `default_impl.rs`：`DefaultPromptGenerator`（确定性启发式候选生成）
    - `error.rs`：`GeneratorError`（用 `thiserror`；至少包含 `MissingContext { keys: Vec<String> }`、`InvalidRules { reason: String }`）
  - `backend/src/core/traits.rs`
    - 对齐 `PromptGenerator` 签名到技术规格（并引入/使用 `GeneratorError`，避免继续使用 `anyhow::Result<Vec<String>>`）
  - `backend/src/core/mod.rs`
    - 增加 `pub mod prompt_generator;`（保持“7 Trait + IterationEngine”的模块边界；不要顺手引入其它空模块）
- **严禁**
- 新建/复制 `Rule/TestCase/OptimizationContext` 的 DTO（必须复用 `backend/src/domain/models/algorithm.rs` 与 `backend/src/domain/types/optimization_context.rs`）

## Testing Requirements

- 测试位置：优先在 `backend/src/core/prompt_generator/default_impl.rs` 内用 `#[cfg(test)]` 写单元测试（与 Layer 1 一致）
- 测试必须是 **纯内存、确定性**（禁止网络、禁止依赖真实 LLM、禁止随机数；如需“多样性”，用候选索引驱动模板变体）
- 覆盖矩阵（至少）：
  - 混合 success/failure：候选中同时包含“修复项”和“保持项”
  - 缺失 `candidate_index`：报错且包含缺失 key
  - 初始 Prompt（current_prompt 为空）：
    - 有 `optimization_goal`：能生成非空候选
    - 缺失 `optimization_goal`：报错且包含缺失 key
  - 不同 `candidate_index`：至少覆盖 `0/1/2` 三种变体的“可复现 + 结构差异”断言（避免实现退化成完全相同的 Prompt）

## Previous Story Intelligence

- **来自 Story 4.1（Layer 1：规律抽取）**
  - Layer 1 的输出约定已经落地：`Rule.tags.extra["polarity"]` 区分 `success|failure|all_passed`；Layer 2 必须直接复用该约定（不要再发明新的字段或枚举）
  - Layer 1 对“不完整数据”采取强约束：缺失结果不允许静默忽略；Layer 2 同样要对“缺失关键上下文”强约束（缺 key 直接报错），避免生成不可解释的候选
- **来自 Epic 3（任务配置/候选数量）**
  - `OptimizationTaskConfig.candidate_prompt_count` 已有校验边界（见 `backend/src/domain/models/optimization_task_config.rs`）；编排层按该值控制调用次数，缺失/不可用必须可诊断（Layer 2 不重复校验范围）

## Git Intelligence Summary

- 最近与本 Epic 直接相关的落地：
  - `de4fd0b` — Story 4.1（RuleEngine / Pattern Extractor）已实现并进入 review
  - `3e2e35d` — Story 3.3（候选数量等配置项）已落地并通过硬化
  - `a1c1fd9` / `c66862e` — CI/clippy 与构建约束更严格：本 Story 实现应避免引入 clippy 触发点与无用依赖

## Latest Tech Information (Non-binding)

> 仅用于避免“用错库/版本”的信息补全；本 Story **不**要求升级依赖。

- crates.io（截至 2026-01-08）显示的部分最新稳定版（供对照）：
  - `tokio`：1.47.1（仓库依赖为 `tokio = "1"`）
  - `axum`：0.8.5（仓库依赖为 `axum = "0.8"`）
  - `sqlx`：0.8.6（仓库依赖为 `sqlx = "0.8"`）
  - `async-trait`：0.1.89（仓库依赖为 `async-trait = "0.1"`）
  - `serde`：1.0.228（仓库依赖为 `serde = "1"`）
  - `uuid`：1.19.0（仓库依赖为 `uuid = "1"`）
  - `utoipa`：5.4.0（仓库依赖为 `utoipa = "5"`）
  - `ts-rs`：11.1.0（仓库依赖为 `ts-rs = "10"`）

## Project Context Reference

- `**/project-context.md`：未发现（本 Story 以项目规划/技术规格文档与现有代码为准）

## References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md#Story 4.2`
- PRD（四层与高层 Trait 概述、FR25/FR22）：`docs/project-planning-artifacts/prd.md#7.4.2 模块化设计（Trait 概述）`、`docs/project-planning-artifacts/prd.md#Functional Requirements`
- 架构一致性与目录结构：`docs/project-planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules`、`docs/project-planning-artifacts/architecture.md`（core 模块映射表）
- 技术规格（权威 Trait/数据结构）：`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#4.2.7 核心 Trait`、`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#4.2.6 OptimizationContext`
- 现有领域模型（Rule/TestCase/TaskReference）：`backend/src/domain/models/algorithm.rs`
- 现有任务配置（候选数量）：`backend/src/domain/models/optimization_task_config.rs`
- Layer 1 输出契约（polarity）：`docs/implementation-artifacts/4-1-pattern-extraction-layer.md`
- 依赖版本基线：`backend/Cargo.toml`

## Change Log

- 2026-01-09: 落地 Layer 2 PromptGenerator Trait + 默认实现与单测（确定性启发式、多候选由 candidate_index 驱动）
- 2026-01-09: 补齐 0-9 共 10 个确定性模板变体；candidate_index 严格范围校验；增强 failure 规律的“可执行修复指令”输出；补充关键单测

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

（无）

### Completion Notes List

1. 对齐 `core::traits::PromptGenerator` 签名到技术规格 `#4.2.7`，并引入 `GeneratorError`（强约束缺失上下文/非法规则）
2. 实现 `core/prompt_generator::DefaultPromptGenerator`：从 `ctx.rule_system.rules` 生成候选 Prompt；current_prompt 为空时用 `optimization_goal` + 测试集摘要生成初始 Prompt；用 `candidate_index` 驱动确定性模板变体
3. 补齐 `candidate_index=0..9` 的 10 个确定性模板变体，并对越界 index 返回可诊断错误（避免静默重复候选）
4. 新增单元测试覆盖：10 变体互异与可复现、candidate_index 越界报错、缺失 polarity key、Constrained/Hybrid 摘要解析；并通过 `cargo test` 与 `cargo fmt`

### File List

- backend/src/core/mod.rs
- backend/src/core/traits.rs
- backend/src/core/prompt_generator/default_impl.rs
- backend/src/core/prompt_generator/error.rs
- backend/src/core/prompt_generator/mod.rs
- docs/implementation-artifacts/4-2-prompt-generation-layer.md
- docs/implementation-artifacts/sprint-status.yaml

## Review Notes

### Findings

- [HIGH] `candidate_index` 驱动的 0-9 共 10 个确定性模板变体已实现，并对越界 index 返回可诊断错误，避免静默重复候选
- [MEDIUM] 多候选（FR22）由编排层按 `OptimizationTaskConfig.candidate_prompt_count` 多次调用 `generate()` 聚合；Layer 2 仅依赖 `candidate_index` 驱动确定性变体（避免把候选数量强耦合进 `OptimizationContext`）

### Decisions

- MVP 采用确定性启发式候选生成（不调用 TeacherModel），以保证单测稳定与可复现
- 优化目标与 `candidate_index` 通过 `ctx.extensions` 注入；候选数量由编排层使用 `OptimizationTaskConfig.candidate_prompt_count` 控制（保持 `OptimizationContext` 结构稳定，不加字段）

### Risks / Tech Debt

- 没有 TeacherModel 驱动时，候选质量受启发式上限影响；后续可在独立 Story 引入 TeacherModel 生成并用回归测试兜底

### Follow-ups

- [ ] 将 `core::traits` 其余 Trait（Evaluator/Optimizer/FeedbackAggregator/ExecutionTarget/TeacherModel）的签名按技术规格分 Story 逐步统一（避免一次性大改）
