# Story 4.6: 失败档案与多样性注入（Failure Archive & Diversity Injection）

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 4-6-failure-archive-and-diversity-injection

## Key Decisions (MVP)

- **失败档案是“可执行的去重/避坑输入”**：用于防止重复候选与重复失败模式，不是“全量日志系统”；记录“足够用来避免重蹈覆辙”的最小信息集。
- **不引入新依赖**：本 Story 不新增第三方 crate（例如 sha/embedding/simhash）；指纹使用确定性字符串指纹即可（避免升级/新增依赖引发回归）。  
- **连续失败语义要“真的连续”**：FR32 的触发条件是“连续 N 轮无提升”，不能用 `iteration >= N` 近似替代；计数由编排层维护并注入 `ctx.extensions`，Layer 4 只读使用。
- **脱敏是硬约束**：错误/日志/报告/档案中不得包含 Prompt 或 TestCase input 的全文；一律用 `*_len`、截断片段（≤200 chars）或结构化摘要替代。
- **指纹必须是“不可读/不可逆的标识”**：`failure_fingerprint` 允许使用 prompt 参与计算，但 **fingerprint 本身不得包含任何可读的 prompt/testcase 原文片段**（因为它可能进入错误/日志/报告链路）。

## Story

As a 系统,  
I want 记录失败档案并在连续失败时触发多样性注入,  
so that 可以避免重蹈覆辙并跳出局部最优。

## Acceptance Criteria

1. **Given** 某个候选 Prompt 在测试用例上失败  
   **When** 记录失败档案  
   **Then** 必须记录最小可用的失败档案条目，至少包含：
   - `prompt_excerpt`（≤200 chars，脱敏/可截断）
   - `prompt_len`
   - `test_case_id`
   - `failure_reason`（可由 `EvaluationResult.failure_points` 生成摘要）
   - `failure_fingerprint`（确定性、可复现，用于去重/避坑）
   **And** 后续候选生成/选择时可基于 `failure_fingerprint` 避免重复失败。
2. **Given** 连续 N 次迭代未能提升通过率（N = 用户配置的多样性注入阈值，FR23）  
   **When** 检测到连续失败  
   **Then** 必须触发多样性注入策略：`UnifiedReflection.recommended_action = InjectDiversity`  
   **And** 触发依据必须结构化记录（写入 `UnifiedReflection.extra`），不得只打日志。
3. **Given** 所有测试用例都通过（口径与 Layer 3/4 既有定义一致）  
   **When** 系统判断优化状态  
   **Then** 必须自动标记“优化成功”：`TerminationReason::AllTestsPassed`（FR33），并终止迭代循环。
4. **Given** 生成候选 Prompt 的流程支持多候选（FR22，候选由 `candidate_index` 驱动的确定性模板变体产生）  
   **When** 生成候选 Prompt  
   **Then** 若候选的 `failure_fingerprint` 命中失败档案（或候选内容完全重复），必须可诊断地拒绝该候选（例如返回 `GeneratorError::DuplicateCandidate`），以避免进入“重复候选→重复失败”的死循环。
5. **Given** 本 Story 引入任何新字段/扩展 key/错误变体  
   **When** 序列化/日志/错误信息输出  
   **Then** 必须遵守既定安全规范：不得包含 Prompt/TestCase input 全文；证据仅包含 ID/计数/摘要。  
6. **Given** 需要验证上述行为  
   **When** 运行单元测试  
   **Then** 必须提供纯内存、确定性测试覆盖（禁止网络、禁止真实 LLM、禁止随机数），并在 CI 下可稳定复现。

## Tasks / Subtasks

- [x] 定义失败档案领域结构与扩展 key（AC: 1,5）
  - [x] 在 `backend/src/domain/models/algorithm.rs` 新增 `FailureArchiveEntry`（或新文件并在 `mod.rs` 导出），字段满足 AC1 的最小集合（含 `failure_fingerprint`）
    - 最小字段建议（仅供对齐；不强制额外字段）：
      - `prompt_excerpt: String`（≤200 chars）
      - `prompt_len: u32`
      - `test_case_id: String`
      - `failure_reason: String`（结构化摘要；不得包含 prompt/testcase input 全文）
      - `failure_fingerprint: String`（见下方“指纹规则”）
  - [x] 在 `backend/src/domain/types/extensions.rs`（或等价位置）集中定义扩展 key 常量：
    - `EXT_FAILURE_ARCHIVE = "layer4.failure_archive"`（类型固定为 `Vec<FailureArchiveEntry>`；避免 `Vec<String>` 与结构列表混用导致反序列化/口径漂移）
    - `EXT_CONSECUTIVE_NO_IMPROVEMENT = "layer4.consecutive_no_improvement"`（u32，由编排层维护并注入）
  - [x] 约定档案大小上限（例如 200 条；超过则 FIFO 丢弃最旧），避免无界增长（写入 Story 的实现细则并单测）
  - [x] 写死编排层“何时/如何写入失败档案”（并单测）：
    - [x] 写入时机：当某个候选 Prompt 完成评估后，对所有失败用例生成 `FailureArchiveEntry` 并写入 `ctx.extensions[EXT_FAILURE_ARCHIVE]`
    - [x] 去重键：`(failure_fingerprint, test_case_id)`（相同 Prompt 在同一用例上的重复失败不重复写入）
    - [x] 上限策略：全局上限 200 条；追加新条目后若超限则 FIFO 丢弃最旧（保证有界）
- [x] 实现“连续失败→InjectDiversity”的正确语义（AC: 2）
  - [x] 更新 `backend/src/core/feedback_aggregator/default_impl.rs`：
    - [x] 优先读取 `ctx.extensions[EXT_CONSECUTIVE_NO_IMPROVEMENT]`；当 `>= ctx.config.iteration.diversity_inject_after` 且本轮 `best_is_better == false` 时返回 `RecommendedAction::InjectDiversity`
    - [x] 在 `UnifiedReflection.extra` 写入结构化原因（例如 `reason=no_improvement_and_consecutive_threshold_reached`，并包含 threshold/current 值）
    - [x] 若未注入扩展 key，则保持现状但必须写入可诊断 reason（避免 silent fallback）
  - [x] 为上述分支补齐单测（不依赖编排层）：覆盖 `consecutive_no_improvement` 的边界值（N-1/N/N+1）
  - [x] 写死编排层维护 `EXT_CONSECUTIVE_NO_IMPROVEMENT` 的规则（并单测）：
    - [x] 当本轮 `best_is_better == true`（采用 best candidate）时：计数重置为 `0`
    - [x] 当本轮 `best_is_better == false` 时：计数 `+1`
    - [x] 计数只由编排层写入；Layer 4 只读（对齐 `OptimizationContext` 只由编排层更新的原则）
- [x] 为候选去重/避坑提供可复用的指纹规则（AC: 1,4,5）
  - [x] 实现一个确定性的 `failure_fingerprint` 生成策略（不引入新依赖）
    - 要求：同输入同指纹；不同 Prompt 尽量不同；**fingerprint 本身不得包含任何可读原文片段**
    - 约定：`failure_fingerprint` 以 **Prompt 内容**为输入生成（便于在候选生成阶段预先计算并与失败档案比对）；失败原因/用例维度信息由 `failure_reason`/`test_case_id` 承载（不参与 fingerprint）
    - 建议：实现稳定的 64-bit FNV-1a（或等价自实现），输出 `v1:fnv1a64:{hex}`；输入为规范化后的 `prompt` 全文（仅用于 hash，不输出任何片段）
  - [x] 明确：指纹用于“去重/避坑”，不是安全散列；不可用于安全用途；允许极低概率碰撞
  - [x] 补齐单测：相同 Prompt 同指纹、不同 Prompt 指纹不同（确定性 + 区分度）
- [x] 在 Layer 2 的候选生成阶段支持“拒绝重复候选”（AC: 4）
  - [x] 扩展 `backend/src/core/prompt_generator/error.rs` 增加可诊断错误（例如 `DuplicateCandidate { fingerprint: String }`；fingerprint 为不可读标识，不得携带 prompt 片段）
  - [x] 在 `backend/src/core/prompt_generator/default_impl.rs`：
    - [x] 读取 `ctx.extensions[EXT_FAILURE_ARCHIVE]`（`Vec<FailureArchiveEntry>`）
    - [x] 若当前 `candidate_index` 生成的候选命中失败档案/或候选内容与历史重复，则返回 `DuplicateCandidate`
  - [x] 单测：构造 `ctx.extensions` 注入失败档案后，验证重复候选会被拒绝；且不同 `candidate_index` 仍保持确定性输出
  - [x] 写死编排层的恢复策略（并单测）：捕获 `DuplicateCandidate` 后继续尝试下一个 `candidate_index`；当候选空间耗尽时必须给出可诊断终态（例如记录 `UnifiedReflection.extra.reason=candidate_space_exhausted` 并转入 InjectDiversity）
- [x] 明确 FR33 的落点并补齐“证据链”（AC: 3）
  - [x] 复核 `backend/src/core/optimizer/default_impl.rs` 的 `AllTestsPassed` 终止条件（目前基于 `best_pass_rate == 1.0`）是否与 Layer 3 split 口径一致（见 Story 4.3/4.4）
  - [x] 若需要：在 `OptimizationResult.extra` 写入 `termination_source`/`stats_scope`，便于 UI/报告展示（本 Story 评估为不需要，保持既有序列化形状）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][MEDIUM] 若后续要做“失败档案持久化 + 断点续跑”，优先把 `failure_archive` 放到 `Checkpoint`（或其 `extra`）而不是散落在多个表/文件中（已记录为后续 Story/PR；本 Story 不做持久化落地）。

## Dev Notes

### Developer Context (What exists today)

- 领域模型已具备：
  - `EvaluationResult.failure_points`（结构化失败点，用于“失败原因摘要”）：`backend/src/domain/models/algorithm.rs`  
  - Layer 4 推荐行动 `RecommendedAction::InjectDiversity` 与终止原因 `TerminationReason::AllTestsPassed`：`backend/src/domain/models/reflection.rs`
- Layer 4 已有“无提升→InjectDiversity”的雏形，但当前实现更接近 `iteration >= threshold` 的近似，**不满足** FR32 的“连续失败 N 次”语义：`backend/src/core/feedback_aggregator/default_impl.rs`
- Layer 2 候选生成是**纯确定性**（10 个模板变体，candidate_index 驱动，不用随机数）：`backend/src/core/prompt_generator/default_impl.rs`（见 Story 4.2）

### Technical Requirements (必须遵守)

- **编排层写 Context，算法层只读**：`OptimizationContext` 只能由编排层更新；Layer 2/4 只能通过 `ctx.extensions` 读入额外输入（见 `backend/src/domain/types/optimization_context.rs` 顶部注释与 Story 4.2/4.4 的约定）。
- **连续失败计数由编排层维护**：本 Story 仅定义扩展 key 与读取逻辑，不要求在 `FeedbackAggregator` 内部自增（因为 `aggregate(&OptimizationContext, ...)` 只读）。
- **错误必须 typed + 可诊断**：新增的“重复候选”必须是显式错误分支，禁止静默 fallback 为另一种候选（否则会掩盖编排层 bug）。
- **日志/错误脱敏**：不得打印 prompt/test_case input 全文；如需定位，用 `test_case_id` + 摘要（len/截断）。
- **EXT_RECENT_PRIMARY_SCORES vs 连续失败计数**：
  - `EXT_RECENT_PRIMARY_SCORES`：历史分数序列，用于震荡/停滞检测（现有机制）
  - `EXT_CONSECUTIVE_NO_IMPROVEMENT`：连续无提升计数，用于多样性注入触发（本 Story 新增；由编排层维护）

### Architecture / Compliance (必须遵守)

- 项目结构与边界：遵循 `docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`
  - `core/`：只放算法模块（Trait + 默认实现），不把“持久化/业务编排”塞进这里
  - `domain/models/`：权威 DTO；新增领域结构优先放这里
  - `domain/types/`：运行期 Context 与 config；扩展 key 统一集中维护

### Library / Framework Requirements（版本与依赖边界）

- Rust edition：`2024`；`rust-version = 1.85`（`backend/Cargo.toml`）
- 本 Story **不新增/不升级依赖**；如必须引入 hash/embedding 等能力，请单独开 Story/PR 并跑全量回归（见 Story 4.5 既有约束）。

### File Structure Requirements（建议改动清单）

- `backend/src/domain/models/algorithm.rs`：新增失败档案领域结构（或拆文件并在 `mod.rs` re-export）
- `backend/src/domain/types/extensions.rs`：新增扩展 key 常量（失败档案、连续失败计数）
- `backend/src/core/feedback_aggregator/default_impl.rs`：修正 InjectDiversity 触发语义（读连续失败计数）
- `backend/src/core/prompt_generator/{default_impl.rs,error.rs}`：候选去重/避坑（可诊断错误）

### Testing Requirements（必须覆盖）

- 单元测试必须是纯内存、确定性：
  - `feedback_aggregator`：覆盖连续失败计数阈值边界 + extra reason 写入
  - `prompt_generator`：覆盖命中失败档案时返回 `DuplicateCandidate`，并确保不泄露敏感信息
  - `fingerprint`：覆盖稳定性与区分度（同输入同指纹，不同关键输入不同指纹）

### Previous Story Intelligence (4.5 learnings)

- 并发/执行相关 Story 已把“脱敏 + 可诊断”作为硬约束：错误不得包含 Prompt/TestCase input 全文（参见 `docs/implementation-artifacts/4-5-execution-mode-and-parallel-scheduling.md##Dev Notes`）。
- 既有模式倾向于把“关键口径/契约”写死并配套单测，避免编排层口径漂移（同上 + `backend/src/core/iteration_engine/orchestrator.rs` 的 hard contract 校验）。

### Git Intelligence (recent work)

- 最近提交集中在 Story 4.5 + Layer 3/4 hardening（契约/测试/脱敏）：`git log -5 --oneline`
  - `8172a61` Story 4.5: execution mode + parallel scheduling
  - `90161ae` fix(core): harden Layer4 contracts and tests
  - `e2d2f44` fix(core): harden evaluator contract and safety guards

### Latest Tech Information (do-not-upgrade note)

- 本 Story 不需要追逐“最新版本”；以仓库 lockfile 为准（`backend/Cargo.lock` / `frontend/package-lock` 或等价）。
- 版本参考（2026-01-14，通过 `cargo search` 查看 crates.io 最新条目；仅作信息，不做升级要求）：
  - `axum`：仓库 `0.8.x`；crates.io `0.8.8`
  - `tokio`：仓库 `1.x`；crates.io `1.49.0`
  - `sqlx`：仓库 `0.8.x`；crates.io 显示最新为 `0.9.0-alpha.1`（预发布）
  - `reqwest`：仓库 `0.12.x`；crates.io `0.13.1`
  - `utoipa`：仓库 `5.x`；crates.io `5.4.0`
- 若后续发现安全公告/破坏性变更需要升级，请新开 Story，避免把“依赖升级风险”混入算法语义 Story。

### Project Structure Notes

- 当前仓库尚未生成 `project-context.md`（`**/project-context.md` 未找到）；因此以 `docs/project-planning-artifacts/architecture.md` 与已完成 Story 的 Dev Notes 为准。

### References

- `docs/project-planning-artifacts/epics.md`：Story 4.6（失败档案、多样性注入、优化成功）  
- `docs/project-planning-artifacts/prd.md`：FR31/FR32/FR33 定义  
- `docs/analysis/brainstorming-session-2025-12-12.md#4.1`：失败档案最小信息集与目的  
- `docs/analysis/brainstorming-session-2025-12-12.md#4.2`：多样性注入触发条件  
- `docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`：模块边界与约束  
- `docs/implementation-artifacts/4-4-reflection-iteration-layer.md`：`InjectDiversity`/`AllTestsPassed` 既有语义与口径  
- `docs/implementation-artifacts/4-5-execution-mode-and-parallel-scheduling.md`：脱敏/可诊断硬约束与既有实现风格  

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

### Completion Notes List

- 新增失败档案领域模型 `FailureArchiveEntry`，并提供确定性指纹 `failure_fingerprint_v1`（FNV-1a 64）与单测（不引入新依赖）。
- 新增扩展 key：`EXT_FAILURE_ARCHIVE` / `EXT_CONSECUTIVE_NO_IMPROVEMENT`，并约定失败档案上限 `FAILURE_ARCHIVE_MAX_ENTRIES=200`。
- 修正 Layer 4 多样性注入：优先读取连续无提升计数并写入结构化诊断信息；未注入时保留旧近似但显式标注 fallback reason。
- Layer 2 候选生成支持避坑：命中失败档案时返回 `GeneratorError::DuplicateCandidate`（仅包含 fingerprint/candidate_index，不泄露 prompt 原文）。
- 编排层契约（helper）落地并单测：失败档案写入（去重+FIFO）、连续计数维护、DuplicateCandidate 重试与候选空间耗尽诊断。
- 回归通过：`backend` 运行 `cargo test`；`frontend` 运行 `npm test -- --run`。

### File List

- backend/src/core/feedback_aggregator/default_impl.rs
- backend/src/core/iteration_engine/orchestrator.rs
- backend/src/core/prompt_generator/default_impl.rs
- backend/src/core/prompt_generator/error.rs
- backend/src/domain/models/algorithm.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/types/extensions.rs
- backend/src/domain/types/mod.rs
- docs/implementation-artifacts/4-6-failure-archive-and-diversity-injection.md
- docs/implementation-artifacts/sprint-status.yaml
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] `failure_reason` 摘要链路可能泄露上游评估器生成的原文片段：已将 `summarize_failure` 调整为仅输出结构化信息（不包含 `FailurePoint.description`）。见 `backend/src/core/iteration_engine/orchestrator.rs`。
- [x] [MEDIUM] 失败档案写入顺序存在非确定性（HashMap 迭代顺序导致 FIFO 行为不稳定）：已按 `test_case_id` 排序后写入。见 `backend/src/core/iteration_engine/orchestrator.rs`。
- [x] [MEDIUM] `prompt_excerpt` 可能包含敏感信息（尤其短 prompt 时等同全文）：已增加保守脱敏（Bearer/常见 key=value/长 token/sk- 形态）。见 `backend/src/domain/models/algorithm.rs`。

### Decisions

- [x] `failure_reason` 只保留结构化摘要（dimension/severity/数量），不保留任何可能包含输入/输出片段的自由文本：以安全与可复现为优先。
- [x] `prompt_excerpt` 采用“保守脱敏”而非复杂检测：不引入新依赖，避免回归；必要时以 `prompt_len` + `failure_fingerprint` 完成诊断闭环。

### Risks / Tech Debt

- [x] 编排层 helper（连续计数维护/失败档案写入/候选重试）已在最小编排入口中接入，并以纯内存、确定性端到端单测验证（不依赖网络/真实 LLM）。见 `backend/src/core/iteration_engine/orchestrator.rs`。

### Follow-ups

- [ ] （可选增强）将 `candidate_space_exhausted` 的结构化 reason key 从 `strategy_reason` 统一为 `reason`（如果后续报告/前端统一口径需要）。
