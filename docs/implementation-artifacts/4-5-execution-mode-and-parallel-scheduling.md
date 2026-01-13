# Story 4.5: 执行模式与并行调度（Layer 5: Execution Mode & Parallel Scheduling）

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 4-5-execution-mode-and-parallel-scheduling

## Key Decisions (MVP)

- **严格顺序契约**：无论串行/并行，执行结果都必须与输入 `TestCase` 顺序一致（或至少可由 `test_case_id` 稳定对齐），以满足技术规格与 Layer 3 的同序要求（`zip(batch, exec_results, eval_results)`）。
- **并行仅解决“用例并发”**：本 Story 仅实现“同一候选 Prompt 下的多条测试用例并发执行 + 限流 + 汇总”；不引入 Racing（多候选早淘汰，Phase 2），不改 Layer 1-4 的算法语义。
- **并行不牺牲质量**：并行模式必须做到“统计口径一致 + 逐用例对齐一致”，并提供对比工具/报告，确保并行 vs 串行差异 < 5%（NFR22）。
- **可观测但不泄露敏感信息**：执行期错误与日志中不得包含 Prompt/TestCase 输入的全文；需要可诊断信息时使用 `*_len`、hash、截断片段或结构化摘要。

## Story

As a Prompt 优化用户,  
I want 选择串行或并行模式执行测试集,  
so that 我可以根据需求在速度和资源消耗之间权衡。

## Acceptance Criteria

1. **Given** 任务配置中 `execution_mode = "serial"`  
   **When** IterationEngine 执行测试集（进入 `IterationState::RunningTests`）  
   **Then** 必须按输入顺序逐条执行测试用例（严格串行），并产出 `Vec<ExecutionResult>`，且每个结果包含：
   - `test_case_id`（必须与输入 `TestCase.id` 一致）
   - `output`（执行输出）
   - `latency_ms`（本次调用耗时，单位 ms；允许为 0 但必须可观测）
2. **Given** 任务配置中 `execution_mode = "parallel"` 且 `max_concurrency = N (N>=1)`  
   **When** IterationEngine 执行测试集  
   **Then** 必须在并发上限内同时执行多条测试用例，并保证：
   - **并发上限可验证**：任意时刻 in-flight 执行数量不超过 `max_concurrency`
   - **顺序契约可验证**：返回的 `Vec<ExecutionResult>` 顺序必须与输入 `TestCase` 顺序一致（技术规格 8.2 的 hard contract）
3. **Given** `Evaluator.evaluate_batch(ctx, results)` 的契约要求“results 不得被过滤/重排，返回 Vec 同序”（见 Story 4.3）  
   **When** 并行执行完成并进入评估阶段  
   **Then** 编排层必须能按技术规格进行稳定对齐：`[(tc, er.output) for tc, er in zip(batch, exec_results)]`，并且不得出现 test_case 错位。
4. **Given** 同一组测试用例、同一 Prompt、同一 ExecutionTarget（Mock/可控）  
   **When** 分别用串行与并行模式运行一次  
   **Then** 必须提供“对比工具或可复现测试”来证明两种模式的结果差异 < 5%（NFR22，MVP 以 `passed` 与 `mean_score` 为主口径；允许 latency 差异）。
5. **Given** 执行目标返回错误（上游 4xx/5xx、超时、响应解析失败等）  
   **When** 执行测试集  
   **Then** 必须返回可诊断错误（`ExecutionError` 或等价 typed error），且错误信息不得包含敏感原文（Prompt/TestCase input 的全文）；错误需包含至少：
   - 失败用例的 `test_case_id`
   - 上游类型（例如 `timeout`/`upstream_error`/`invalid_credentials`/`parse_error`）
6. **Given** 并行调度实现需要满足 NFR1（调度开销 < 100ms，不含模型调用时间）与 NFR4（并行接近线性加速）  
   **When** 使用 MockExecutionTarget 进行基准验证  
   **Then** 必须产出一份可复现的基准/压测报告（Markdown），说明：
   - 串行 vs 并行耗时对比（至少覆盖 N=1/2/4/8 并发）
   - 估算的调度开销定义与测量方法（明确“不含模型调用时间”的口径）

## Tasks / Subtasks

- [x] 任务配置支持“串行/并行 + 并发数”（AC: 1,2）
  - [x] 在 `OptimizationTaskConfig` 增加 `execution_mode` 与 `max_concurrency`（默认串行；并发默认值需明确且校验范围固定）
  - [x] 更新 `UpdateOptimizationTaskConfigRequest`/前端配置页，支持用户选择与校验
- [x] 对齐技术规格中的 `ExecutionTarget` 契约并落地默认实现骨架（AC: 1,2,5）
  - [x] `ExecutionTarget.execute/execute_batch/name`：返回 `ExecutionResult`；批量执行保证同序返回
  - [x] 新增 `ExecutionError`（typed error，脱敏）
- [x] 实现 IterationEngine 的测试执行与并发调度（AC: 1,2,3）
  - [x] `serial_execute(prompt, batch)`：严格串行
  - [x] `parallel_execute(prompt, batch, max_concurrency)`：并发限流 + 同序收集
  - [x] 汇总输出为 `Vec<ExecutionResult>`，供 Layer 3 评估与 Layer 4 反思输入使用
- [x] 结果对比工具与基准报告（AC: 4,6）
  - [x] 提供可运行的对比入口（建议：`backend/src/bin/compare_execution_modes.rs`）输出差异摘要
  - [x] 写入基准/压测报告到 `docs/implementation-artifacts/`（文件名包含 story key + 时间戳）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 已将关键审查结论沉淀到 `## Review Notes`（含修复、取舍与遗留边界）

## Dev Notes

### Developer Context (What exists today)

- 领域模型已具备 `TestCase` / `ExecutionResult` / `EvaluationResult`：`backend/src/domain/models/algorithm.rs`
- 算法层 7 Trait 已落地并有默认实现（Layer 1-4 已实现并硬化）：`backend/src/core/{rule_engine,prompt_generator,evaluator,feedback_aggregator,optimizer}/`
- `backend/src/core/execution_target/` 与 `backend/src/core/iteration_engine/` 已落地：包含 `ExecutionError/ExecutionTarget` 骨架、调度原语（serial/parallel）与最小编排入口（IterationEngine.run_tests）
- 前端任务配置页已具备基础配置（max_iterations、pass_threshold_percent 等）：`frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`

### Technical Requirements (必须遵守)

- **执行顺序契约（硬约束）**：`ExecutionTarget.execute_batch()` 与并行调度的返回顺序必须与输入用例顺序一致（技术规格 8.2）；禁止在执行层做过滤/排序。
- **与 Layer 3 的对齐（硬约束）**：Layer 3 `evaluate_batch` 依赖“同序返回”；并行调度必须维持“test_case_id ↔ output”的稳定映射（见 Story 4.3 AC1）。
- **错误必须 typed + 脱敏**：参考 `core/evaluator/error.rs`、`core/optimizer/error.rs` 的模式，新增 `ExecutionError`（或同等 typed error）；错误不得包含 Prompt/TestCase input 全文。
- **并发必须可控**：并行模式必须具备 `max_concurrency` 上限；不得“无界 spawn”导致资源耗尽。
- **默认策略保守**：MVP 默认串行（质量优先）；并行作为用户显式选择项（对齐 PRD “先保证串行质量，再开放并行”）。

#### Contract Clarifications (MVP - Must Read)

- **Batch 失败语义（写死，避免实现发散）**：`execute_batch`/`parallel_execute` 采用“全有或全无”策略：任一用例执行失败（超时/上游错误/解析失败/无效请求等）→ **直接返回 `Err(ExecutionError::*)`，不产出 partial results**，并且编排层不得进入 Layer 3 评估（避免错位/污染统计口径）。
- **错误信息必须可诊断但不得泄露原文**：`ExecutionError` 的 message 必须包含 `test_case_id` 与错误类别（例如 `timeout/upstream_error/parse_error/invalid_request`），但不得包含 Prompt/TestCase input 全文；推荐使用 `*_len`、hash、截断片段（≤ 200 chars）或结构化摘要替代。
- **NFR1 口径（调度开销 < 100ms，不含模型调用）**：基准验证必须使用 `MockExecutionTarget`（每个用例固定 sleep `T_ms`，例如 20ms），并定义：`expected_ms = ceil(batch_size / max_concurrency) * T_ms`，`scheduling_overhead_ms = wall_clock_ms - expected_ms`；验收建议在 `batch_size ∈ {64, 256}`、`max_concurrency ∈ {1, 2, 4, 8, 16}` 下验证 `scheduling_overhead_ms < 100ms`（允许少量 OS 抖动，但必须可复现并写入报告）。

### Architecture / Compliance (必须遵守)

- 项目结构与边界：遵循 `docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`
  - `api/` 只做路由/鉴权/请求校验/调用编排入口；核心调度逻辑放在 `core/iteration_engine/`
  - 外部 HTTP 调用封装在 `infra/external/`，core 通过 Trait/组件组合使用，不把 HTTP 细节散落到 core 之外
- 命名与序列化：
  - 领域模型（ts-rs 生成物）字段保持 `snake_case`（例如 `test_case_id/latency_ms/max_concurrency`）
  - 不要为领域模型引入 `serde(rename_all = "camelCase")` 或改变既有序列化形状
- WebSocket 事件命名（本 Story 仅预留，不强制实现）：`{domain}:{action}`（结构复用 `backend/src/api/ws/events.rs::WsMessage<T>`）

### Library / Framework Requirements（版本与依赖边界）

- Rust 依赖以 `backend/Cargo.toml` / `backend/Cargo.lock` 为准；本 Story **不升级/替换依赖**
- 并发实现优先使用 Tokio 原生工具（`tokio::task::JoinSet` / `tokio::sync::Semaphore` / `tokio::time::Instant`），避免引入新依赖

### File Structure Requirements（建议改动清单）

**后端（Rust）**

- `backend/src/core/mod.rs`：导出 `execution_target` 与 `iteration_engine`
- `backend/src/core/traits.rs`：对齐技术规格中的 `ExecutionTarget`（新增 `execute_batch`；返回 `ExecutionResult`；新增 `name()`）
- `backend/src/core/execution_target/`
  - `mod.rs`（对外导出 + 工厂/选择逻辑的骨架）
  - `dify_impl.rs`（Dify 执行实现骨架；可先 stub 但必须有清晰 TODO 边界）
  - `direct_api_impl.rs`（OpenAI 兼容 API 执行实现骨架；可先 stub 但必须有清晰 TODO 边界）
  - `error.rs`（`ExecutionError`；脱敏）
- `backend/src/core/iteration_engine/`
  - `mod.rs`（对外导出）
  - `executor.rs`（建议：聚焦 Step 2.1 串行/并行执行与限流/同序收集）
  - （可选）`error.rs`（若需要区分 Orchestrator vs Execution errors）
- `backend/src/domain/models/optimization_task_config.rs`：
  - 增加 `execution_mode` / `max_concurrency`（并维护 storage 结构的向后兼容）
  - 增加校验边界（例如 `max_concurrency` 仅允许 `1..=64`，范围写死）
- `backend/src/api/routes/optimization_tasks.rs`：更新 `UpdateOptimizationTaskConfigRequest` 与校验/回显（前端可用）
- `backend/src/bin/gen-types.rs`：如新增枚举/结构体未被自动导出，补齐导出，保持前端类型同步

**前端（React）**

- `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`：新增执行模式选择（串行/并行）与并发数输入（仅在并行时显示），并复用既有表单校验/Toast 模式
- `frontend/src/types/generated/`：通过现有 `gen-types` 流程更新生成物（不手写）

### Testing Requirements（建议）

- 单元测试（Rust）建议位置：
  - `backend/src/core/iteration_engine/executor.rs`：覆盖并发上限、同序返回、空输入、重复 id 防护（如有）
  - `backend/src/core/execution_target/*`：覆盖错误脱敏、超时/上游错误映射（可用 mock/wiremock）
- “并行 vs 串行差异 < 5%”建议以 **可控 MockExecutionTarget** 做确定性测试（避免真实 LLM 不可复现导致 flaky）
- 基准/压测报告：在本 Story 完成时新增一份 Markdown 报告到 `docs/implementation-artifacts/`（纳入 PR/Review 的可见产物）

## Previous Story Intelligence

- **来自 Story 4.3（Layer 3：Quality Assessor）**
  - `Evaluator.evaluate_batch` 必须同序返回；`EvaluationResult` 本身不携带 `test_case_id`，编排层依赖 index 对齐构建 `test_case_id -> EvaluationResult` 映射
  - split 过滤只影响统计/排序，不影响逐用例输出数量与顺序
- **来自 Story 4.4（Layer 4：Reflection Agent）**
  - Context 只由编排层写：Layer 4 只读 `&OptimizationContext` 并输出 `OptimizationResult`；本 Story 的 IterationEngine 作为编排层必须遵守该原则

## Git Intelligence Summary

- 近期 core 模块落地模式已稳定：`mod.rs + default_impl.rs + error.rs (+ #[cfg(test)])`，并倾向把“硬约束”写进单测防回归（参考：`fix(core): harden evaluator contract...`、`fix(core): harden Layer4 contracts and tests`）。
- 本 Story 落地并行调度时，必须把“顺序契约/并发上限/脱敏错误”落到可自动化回归的测试中，避免后续引入 Racing/断点续跑时出现隐性退化。

## Latest Tech Information (Non-binding)

> 仅用于避免“用错 API/用错并发原语”；本 Story **不**要求升级依赖。

- 后端并发运行时：Tokio（以 `backend/Cargo.toml` 为准，`tokio = "1"`，已启用 `features = ["full"]`）
- HTTP 客户端：reqwest 0.12（以 `backend/Cargo.toml` 为准）

## Project Context Reference

- `**/project-context.md`：未发现（本 Story 以规划/技术规格文档与现有代码为准）

## References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md#Story 4.5: 执行模式与并行调度`
- PRD（并行作为可选、质量差异约束）：`docs/project-planning-artifacts/prd.md#8.2 MVP Feature Set (Phase 1)`
- UX（Run View 作为主运行舞台，非阻塞监控）：`docs/project-planning-artifacts/ux-design-specification.md#Run View（核心体验主舞台）`
- 架构/边界/结构：`docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`、`docs/project-planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules`
- 技术规格（权威）：`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#ExecutionTarget Trait`、`#8.2 并行测试实现`
- Layer 3 同序契约：`docs/implementation-artifacts/4-3-quality-assessment-layer.md`
- 领域模型：`backend/src/domain/models/algorithm.rs#ExecutionResult`

## Story Completion Status

- Status set to `done`
- Completion note: execution_mode 调度、同序契约、并发上限与可复现基准/对比均已落地并通过回归

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

### Completion Notes List

- 完成 `execution_mode/max_concurrency` 全链路（domain + API request + 前端表单 + TS types 生成）
- 落地 `ExecutionTarget` 新契约与脱敏 `ExecutionError`，并提供 Dify/Direct API 骨架实现与工厂函数
- 实现测试执行调度：`serial_execute`（严格串行）与 `parallel_execute`（限流 + 同序收集 + 全有或全无）
- 补齐 IterationEngine 编排入口：`IterationEngine.run_tests`（进入 `IterationState::RunningTests`，按 execution_mode 调度；强校验 `test_case_id` 对齐；提供 `build_evaluation_pairs` 用于对接 Layer 3）
- 提供可运行对比工具 `compare_execution_modes`（串行 vs 并行差异摘要）
- 输出/更新基准报告（batch_size=64/256，N=1/2/4/8；含调度开销口径与 NFR 结论）

### File List

- backend/src/api/routes/optimization_tasks.rs
- backend/src/bin/compare_execution_modes.rs
- backend/src/core/execution_target/dify_impl.rs
- backend/src/core/execution_target/direct_api_impl.rs
- backend/src/core/execution_target/error.rs
- backend/src/core/execution_target/mod.rs
- backend/src/core/iteration_engine/executor.rs
- backend/src/core/iteration_engine/mod.rs
- backend/src/core/iteration_engine/orchestrator.rs
- backend/src/core/mod.rs
- backend/src/core/traits.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/models/optimization_task_config.rs
- docs/implementation-artifacts/4-5-execution-mode-and-parallel-scheduling-benchmark-2026-01-10.md
- docs/implementation-artifacts/4-5-execution-mode-and-parallel-scheduling.md
- docs/implementation-artifacts/sprint-status.yaml
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx
- frontend/src/types/generated/api/UpdateOptimizationTaskConfigRequest.ts
- frontend/src/types/generated/models/ExecutionMode.ts
- frontend/src/types/generated/models/OptimizationTaskConfig.ts
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] `execution_mode/max_concurrency` 仅在配置链路落地、缺少编排层接入：已补齐 `IterationEngine.run_tests`，并在单测中验证 state=RunningTests、串行不重叠/并行限流与同序契约。
- [x] [MEDIUM] `ExecutionTarget` 骨架用 `Internal` 表达“未实现”易误导：已新增 `ExecutionError::NotImplemented` 并用于 Dify/Direct API 骨架实现。
- [x] [MEDIUM] `parallel_execute` 对 `max_concurrency=0` 静默 clamp 容易掩盖配置错误：已改为 `InvalidRequest` 并补齐单测。
- [x] [MEDIUM] 前端缺少“执行模式切换”用例：已补齐切换显示/隐藏并发输入、并行模式并发数越界本地拦截、请求 payload/回显验证。
- [x] [LOW] 基准报告缺少 NFR/差异结论与 batch_size=64 数据：已更新基准报告补齐结论与对比输出。

### Decisions

- [x] 保持 “全有或全无” 的 batch 失败语义（与 Story Dev Notes 一致），避免 partial results 造成后续 `zip` 对齐与统计口径污染。
- [x] 在编排入口强校验 `test_case_id` 对齐：宁可 fail-fast，也不允许 silent mismatch 进入 Layer 3/4。

### Risks / Tech Debt

- [x] Dify/Direct API 的真实执行仍为骨架（NotImplemented）：待后续 Story 落地真实上游调用与错误映射（Network/Timeout/Upstream/Parse/Credentials）。

### Follow-ups

- [ ] [LOW] 将 `IterationEngine.run_tests` 串到真实 “任务运行入口”（API/WS/Job runner）以实现端到端可运行优化（非本 Story 必需）。
