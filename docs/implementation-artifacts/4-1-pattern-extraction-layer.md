# Story 4.1: 规律抽取层（Layer 1: Pattern Extractor）

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 4-1-pattern-extraction-layer

## Story

As a 系统,
I want 从测试集结果中抽取成功/失败规律,
so that 后续层可以基于规律生成更优的候选 Prompt。

## Acceptance Criteria

1. **Given** 优化任务已启动且测试集已执行  
   **When** Layer 1（`RuleEngine.extract_rules(ctx, test_cases)`）接收到测试结果（来自 `ctx.extensions["layer1_test_results"]`，至少包含每条用例的 passed/failed 与失败点信息，且可由 `test_case_id` 关联回 `TestCase`）  
   **Then** 能对成功用例抽取“什么情况下成功”的共性特征，并输出为结构化 `Rule` 列表（自然语言描述 + tags + source_test_cases）。  
2. **Given** 测试结果中既有成功也有失败  
   **When** 执行规律抽取  
   **Then** 输出的规律必须同时包含成功规律与失败规律，并以自然语言形式输出（供 Layer 2 直接消费），且可通过 `Rule.tags.extra["polarity"]`（`success|failure`）区分。  
3. **Given** 所有测试用例都成功  
   **When** 执行规律抽取  
   **Then** 必须生成 1 条 `polarity=all_passed` 的 `Rule`（`description` 明确为“当前 Prompt 已满足所有测试用例”），供编排层直接终止/跳过后续迭代。  
4. **Given** 某些测试用例缺少执行/评估结果（不完整数据）  
   **When** 执行规律抽取  
   **Then** 必须产生可诊断的错误或降级输出（不得 silently 忽略），并在日志/返回错误中包含缺失的 `test_case_id` 列表。  

## Tasks / Subtasks

- [x] 定义 Layer 1 的输入/输出契约与最小上下文 (AC: 1,2,3,4)
  - [x] 落地/对齐 `OptimizationContext`（以技术规格 `#4.2.6 OptimizationContext` 为准；不要额外往 struct 里塞字段）并明确“只读访问、仅编排层可写”的纪律
  - [x] 明确 Layer 1 所需“测试结果”的来源与结构：由编排层写入 `ctx.extensions`（Layer 1 只读），至少提供每条用例的 `passed/failed` 与 `failure_points`，并可通过 `test_case_id` 关联回 `TestCase`
    - 建议约定（MVP 固定）：`ctx.extensions["layer1_test_results"]` 为对象，包含：
      - `evaluations_by_test_case_id: { [test_case_id: string]: EvaluationResult }`（必需）
      - `executions_by_test_case_id: { [test_case_id: string]: ExecutionResult }`（可选，用于更具体的描述）
  - [x] 对齐 `core::traits::RuleEngine` 到技术规格 `#4.2.7`：至少对齐 `extract_rules(ctx, test_cases) -> Result<Vec<Rule>, RuleEngineError>` 与 `name()`；其余 `detect_conflicts/resolve_conflict/merge_similar_rules` 方法可先提供占位实现，但 Trait 形状必须对齐，避免后续 Story 返工
- [x] 实现 `core/rule_engine` 的默认规律抽取实现 (AC: 1,2,3,4)
  - [x] 从 `ctx.extensions["layer1_test_results"]` 读取 `evaluations_by_test_case_id`，并建立 `test_case_id → (TestCase, EvaluationResult)` 的映射与完整性校验（缺失任一用例结果都不得 silently 忽略）
  - [x] 将测试用例按 `EvaluationResult.passed` 分为 success/failure 两组（证据用例必须写入 `source_test_cases`）
  - [x] 失败规律：按 `failure_points.dimension` 聚合（必要时结合 `TaskReference` / `Constraint`），生成可执行的失败模式描述（含“触发条件/失败表现/建议修复方向”）
  - [x] 成功规律：抽取成功用例的共性（输出格式/结构/关键概念/约束满足点），生成“保持项”规律（避免 Layer 2 破坏已成功特性）
  - [x] 生成 `Rule`：`id=uuid`、`description` 为中文自然语言、`tags` 填充最小可用字段（`output_format/output_structure/output_length/semantic_focus/key_concepts` + `tags.extra["polarity"]`）、`source_test_cases` 记录证据
  - [x] all-pass 信号：实现并在单测覆盖（固定实现：生成 1 条 `polarity=all_passed` 的 `Rule`）
- [x] 测试与可维护性 (AC: 1,2,3,4)
  - [x] 为典型场景添加单元测试：混合成功/失败、全成功、缺失结果、重复/冲突 failure_points
  - [x] 明确不引入新依赖（优先复用现有 `uuid/serde/ts-rs`）；如确需新增，需在 story 中记录动机与替代方案

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][HIGH] 补齐“重复/冲突 failure_points”聚合行为的单测证据（Story 已勾选但原先缺失）
- [x] [AI-Review][MEDIUM] Layer1 success 规律在缺少 executions 时给出可诊断提示（避免静默降级）
- [ ] [AI-Review][MEDIUM] 后续 Story 再增强 failure 规律：结合 `TaskReference/Constraint` 提供更可执行的修复方向（避免 scope creep）
- [ ] [AI-Review][HIGH] 在对应 Story 中统一对齐 `core::traits` 里其余 Trait 的签名与技术规格 `#4.2.7`（本 Story 仅要求 RuleEngine）

## Dev Notes

### Scope / Non-Goals

- **In scope**：后端 Layer 1 `RuleEngine` 的默认规律抽取实现（从测试结果抽取 success/failure 规律，输出 `Vec<Rule>` 供 Layer 2 使用；`RuleSystem` 的更新/合并由编排层负责）。
- **Non-goals**：本 Story 不要求实现 Layer 2/3/4、`iteration_engine` 编排、WebSocket 推送、前端节点图展示、DB 持久化（除非为满足最小可运行链路而必要，必要性需在实现时以 AC 证明）。

### Inputs → Outputs（开发者必须明确并写进代码契约）

- **输入（最小集合）**
  - `ctx: &OptimizationContext`：按技术规格 `#4.2.6`；Layer 1 只读访问
  - `test_cases: &[TestCase]`：测试用例（含输入变量与 `TaskReference` 约束/期望）
  - `ctx.extensions["layer1_test_results"]`：本 Story 的“测试结果”输入（由编排层写入），至少包含：
    - `evaluations_by_test_case_id`：每条用例的通过/失败与失败点（`failure_points`）
    - （可选）`executions_by_test_case_id`：输出文本与延迟（用于更具体的规律描述）
- **输出**
  - `Vec<Rule>`：自然语言规律（必须可读、可直接被 Layer 2 消费）
  - `RuleSystem`：由编排层负责更新/合并（rules + coverage_map + version + log 等；本 Story 不要求在 `RuleEngine.extract_rules` 中返回/构建 `RuleSystem`）
  - **all-pass 信号**：全成功时固定产出 1 条 `polarity=all_passed` 的 `Rule`（供编排层判定终止）

### “共性特征”抽取建议（MVP 可落地的确定性策略）

- failure 侧：按 `failure_points.dimension` 聚合，抽取“触发条件/失败表现/修复方向”（描述中必须包含维度名与典型失败点摘要）
- success 侧：抽取“必须保留的成功特性”（例如输出格式/结构约束、关键概念词、约束满足点），防止 Layer 2 破坏已有成功
- 输出描述必须遵循：一句话结论 + 2-3 条要点（避免过长、也避免含糊）

### 技术要求（DEV Agent Guardrails）

- **禁止重复建模**：`Rule/RuleTags/RuleSystem/RuleConflict/...` 已存在于 `backend/src/domain/models/algorithm.rs`，不得再创建同名/近似 DTO。
- **确定性与可测性**：同一输入必须产出稳定可预测的规则集合（排序、聚合键、拼接顺序要固定）。
- **错误与诊断**：任何“不完整测试结果”必须显式报错或产出降级信号；不得静默忽略导致“假成功”。
- **无隐式依赖升级**：不得在本 Story 中升级 `axum/sqlx/reqwest/tokio` 等依赖版本（除非出现编译/安全阻断且能用证据证明必要）。
- **输出面向 LLM 消费**：规则描述必须短、明确、可执行；避免“泛泛而谈”的总结句。

### 架构一致性（必须遵循）

- 模块归属：Layer 1 实现在 `backend/src/core/rule_engine/`，并通过 `backend/src/core/mod.rs` 暴露（符合“7 Trait → core/ 子模块”约定）。
- 命名与格式：Rust 使用 snake_case；跨语言结构体序列化遵循现有 `serde`/`ts-rs` 约定（不新增自定义序列化格式）。
- 时间字段：如新增时间戳字段，遵循 INTEGER (Unix ms) 约定（不要引入混用 TEXT/INTEGER）。

### 依赖与框架要求

- 以 `backend/Cargo.toml` 为准：本 Story 默认不新增依赖、不升级依赖版本。
- 规则 ID：使用现有 `uuid` crate（v4）。
- 异步：沿用 `async-trait` 与 `tokio`（不要引入新的 async runtime）。

### 测试要求（必须满足）

- 单元测试优先：`backend/src/core/rule_engine/` 内部用 `#[cfg(test)]` + `tokio::test` 覆盖 AC 1-4。
- 测试数据构造：显式构造 `TestCase` + `EvaluationResult`（含 `failure_points`），避免依赖 DB/HTTP。
- 断言策略：对规则数量、`polarity` 分类、`source_test_cases` 覆盖、all-pass 信号进行精确断言；对自然语言描述可采用“包含关键字/维度名”的断言（避免脆弱的全字符串匹配）。
- 本地验证：`cargo test -p prompt_faster`（至少保证新增测试通过）。

### 最新技术信息（用于避免“拍脑袋升级”）

- 本项目当前依赖基线以 `backend/Cargo.toml` 为准（例如：`axum=0.8`、`sqlx=0.8`、`tokio=1`、`reqwest=0.12`、`utoipa=5`、`ts-rs=10`）。
- 2026-01（查询时点）上游已存在更新版本（示例：`axum 0.8.8`、`sqlx 0.8.6`、`tokio 1.49.0`、`reqwest 0.13.x`、`utoipa 5.4.x`、`ts-rs 11.x`）。本 Story 不要求升级；若后续必须升级，请先在单独 PR/Story 里处理并跑全量测试。

### Project Structure Notes

- **目标落点（后端）**
  - `backend/src/core/rule_engine/`：新增 Layer 1 默认实现（`mod.rs`, `default_impl.rs`，必要时 `error.rs`）
  - `backend/src/core/mod.rs`：导出 `pub mod rule_engine;` 并保持 “7 Trait + IterationEngine” 的模块边界
  - `backend/src/core/traits.rs`：对齐 `RuleEngine` 的签名与错误类型（避免继续使用占位 `struct Rule; struct TestCase;`）
  - `backend/src/domain/types/`：补齐 `OptimizationContext`（若当前尚未存在），以满足 `ctx: &OptimizationContext` 的调用约定
- **一致性检查**
  - 当前仓库已存在 `backend/src/domain/models/algorithm.rs` 的 `Rule*` 结构；实现时必须复用，不得重复建模
  - `backend/src/core/*` 子目录已建立但大多为空/占位；本 Story 允许首次填充 `rule_engine/`，但不要顺手填充其他模块（避免 scope creep）

### References

- `**/project-context.md`：未发现（本 Story 以项目规划/技术规格文档与现有代码为准）
- Epic/Story 定义：`docs/project-planning-artifacts/epics.md#Epic 4: 自动迭代优化执行`
- PRD（四层与 Trait 映射、数据库/Checkpoint 约束）：`docs/project-planning-artifacts/prd.md#7.4 后端架构`
- 架构一致性与目录结构：`docs/project-planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules`、`docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`
- 技术规格（权威算法/数据结构/Trait 签名）：`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#4.2.6 OptimizationContext`、`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#4.2.7 核心 Trait`、`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#6.2 数据结构定义`、`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#6.3.1 规律提炼算法`
- 现有领域模型（已落地的 Rule*）：`backend/src/domain/models/algorithm.rs`
- 依赖版本基线：`backend/Cargo.toml`

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

### Completion Notes List

1. 实现 Layer 1 规律抽取：从 `ctx.extensions["layer1_test_results"]` 生成 success/failure/all_passed 规则
2. 对齐 `OptimizationContext` 与 `RuleEngine` Trait（含占位 detect/resolve/merge）并补齐 `RuleEngineError`
3. 新增单元测试覆盖：混合成功/失败、全成功 all_passed、缺失结果报错；`cargo test` 全量通过

### File List

- backend/src/core/mod.rs
- backend/src/core/traits.rs
- backend/src/core/rule_engine/default_impl.rs (新增)
- backend/src/core/rule_engine/error.rs (新增)
- backend/src/core/rule_engine/mod.rs (新增)
- backend/src/domain/types/mod.rs
- backend/src/domain/types/optimization_context.rs (新增)
- docs/implementation-artifacts/4-1-pattern-extraction-layer.md (新增)
- docs/implementation-artifacts/sprint-status.yaml

## Change Log

- 新增 `DefaultRuleEngine` 与 `RuleEngineError`，实现 Layer 1 规律抽取与 all-pass 信号
- 对齐 `RuleEngine` Trait 与新增 `OptimizationContext` 领域类型（含配置结构与 extensions 扩展点）

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [HIGH] Story 勾选“重复/冲突 failure_points”测试，但实现中缺少对应单测证据；已在本次 review 补齐（见 `DefaultRuleEngine` 单测）。
- [MEDIUM] `executions_by_test_case_id` 为空/缺失时，success 规律只基于 `TestCase` 推断而无提示，容易造成“静默降级”；已补充 `warn!` 诊断日志。
- [LOW] `"layer1_test_results"` 存在魔法字符串；已提取为常量，降低未来改动风险。

### Decisions

- 保持 `detect_conflicts/resolve_conflict/merge_similar_rules` 为占位实现：本 Story 目标是对齐 Trait 形状与 Layer1 抽取主流程，避免 scope creep。
- `infer_output_structure_tags` 仍采用轻量启发式：只做低风险增强（扩大扫描范围、支持多标签），不引入复杂优先级规则。

### Risks / Tech Debt

- failure 规律目前主要基于 `failure_points.dimension/description` 聚合，尚未结合 `TaskReference/Constraint` 输出更细的修复指引；若 Layer2 需要更强可执行性，再在后续 Story 增强。
- `core::traits` 其余 Trait 的签名是否完全对齐技术规格需在对应 Story 统一校准（避免在本 Story 扩大改动面）。

### Follow-ups

- [ ] [AI-Review][MEDIUM] failure 规律增强：引入 `TaskReference/Constraint` 的信号，让修复方向更具体（在 Layer2 开始消费规则前完成）。
- [ ] [AI-Review][HIGH] 对齐 `core::traits` 其余 Trait 的方法签名与错误类型（按技术规格 `#4.2.7` 分 Story 推进）。
