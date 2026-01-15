# Story 4.8: 核心算法模块解耦与替换演练

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 4-8-core-algorithm-module-decoupling-and-replacement

## Key Decisions (MVP)

- **以“可替换核心算法”为第一原则（NFR15）**：把“核心算法模块”收敛成一个可指认的目录/门面（Facade），调用方只能依赖少量受控接口。
- **不做依赖升级/不引入新依赖**：本 Story 聚焦边界与替换演练；如需引入新 crate / 大规模重构，请单独开 Story/PR（避免回归风险混入）。
- **替换演练必须“可证伪”**：通过双实现（Default vs Alternate）+ 可复现的最小端到端测试证明“替换不影响其他模块”，禁止只写口号/只写文档。
- **注册点单一**：算法选择逻辑集中在 **一个入口点**（工厂/注册表/feature gate）；禁止把选择散落到业务代码或多个 match。
- **确定性与脱敏硬约束继承**：测试/日志/错误/报告不得包含 Prompt 或 TestCase input 全文；必须在 CI 下稳定复现（不出网、无随机/可控随机）。
- **替换范围（本 Story 已决策）**：替换的是 **整个 `OptimizationEngine` Facade**（而非仅替换某个子策略）。
- **切换机制（本 Story 已决策）**：仅使用 **cargo feature** 进行编译期切换，feature 名称固定为：`alt-optimization-engine`（最小影响面，避免触及 domain/db/api/ts）。
- **运行时注册（本 Story 非目标）**：本 Story **不实现** `ModuleRegistry`/`from_registry` 的运行时注册/查找；如未来确需任务配置可选（运行时切换），单独开 Story。
- **验收测试集合（本 Story 已决策）**：DoD 必须覆盖后端测试 **Default + Alternate** 两条路径（含 `backend/tests/`）；前端 Playwright E2E 由 CI 全局兜底，但不作为本 Story 的本地最低验收门槛。

## Story

As a 开发者,
I want 将核心算法实现与调用方解耦并完成一次替换演练,
so that 在需要更换优化算法时，只修改算法模块自身。

## Acceptance Criteria

1. **Given** 项目已经实现四层核心组件（7 Trait）与相关 default 实现  
   **When** 查看代码结构和模块依赖  
   **Then** 可以清晰识别出“核心算法模块”的 crate/模块/目录（建议：`backend/src/core/optimization_engine/`）  
   **And** 该模块对外仅通过少量受控接口（Facade/trait + 工厂）暴露能力  
   **And** 调用方（未来的任务执行/调度/可视化/可靠性等）只依赖这些接口而不直接依赖内部实现细节（对齐 `docs/project-planning-artifacts/architecture.md#架构边界定义`）。
2. **Given** 为验证可替换性，实现一个替代算法实现（Alternate Engine / Alternate Strategy / Mock Engine 均可）  
   **When** 通过工厂 + 编译配置（cargo feature `alt-optimization-engine`）切换到替代实现  
   **Then** 代码变更局限在：
   - 算法模块目录（例如 `backend/src/core/optimization_engine/`）及其测试
   - 用于注册/选择算法实现的单一入口点  
   **And** 其他模块无需修改即可完成编译。
3. **Given** 使用替代算法模块重新运行一套核心端到端测试  
   **When** 执行测试流水线  
   **Then** 所有与算法无关的行为（配置、工作区管理、CRUD API、既有核心模块单测等）测试全部通过  
   **And** 注：Epic 文案中提到的“可视化/断点续跑”等能力属于后续 Epic 的回归范围；本 Story 的“算法无关行为”验收以当前仓库已存在的后端测试集合为准（尤其是 `backend/tests/`）。  
   **And** 在切换回原始算法模块后测试同样全部通过，证明算法模块可以在不影响其他模块的前提下被替换（NFR15）。

## Tasks / Subtasks

- [x] 定义“核心算法模块”的对外门面（AC: 1）
  - [x] 新增 `backend/src/core/optimization_engine/`，作为**唯一**可替换算法模块边界
  - [x] 在该模块内定义对外接口（优先 Facade 风格）：
    - [x] `OptimizationEngine`（最小 API 形状：以技术规格 `#4.2.9 OptimizationEngine 结构定义` 为准，但考虑现有代码需要写入 `ctx.extensions/state`，这里锁死为可变上下文）：
      - [x] `async fn run(&self, ctx: &mut OptimizationContext) -> Result<OptimizationResult, OptimizationEngineError>;`
      - [x] `async fn resume(&self, checkpoint: Checkpoint, ctx: &mut OptimizationContext) -> Result<OptimizationResult, OptimizationEngineError>;`
      - [x] `fn name(&self) -> &str;`（用于诊断/日志/测试断言；风格与 7 Trait 保持一致）
    - [x] 统一错误类型（例如 `OptimizationEngineError`），用 `thiserror` 聚合子错误但不泄露敏感信息
  - [x] 在 `backend/src/core/mod.rs` 导出 `optimization_engine`（使模块对外可被稳定引用）

- [x] 实现单一注册/选择入口点（AC: 1,2）
  - [x] 参考既有模式（`backend/src/core/execution_target/mod.rs::create_execution_target`）提供 `create_optimization_engine(...)`
  - [x] 选择策略（本 Story 锁死为 **编译配置**，最小影响面）：
    - [x] 在 `backend/Cargo.toml` 增加 `[features] alt-optimization-engine = []`
    - [x] 工厂根据 `#[cfg(feature = "alt-optimization-engine")]` 选择实现
  - [x] 强约束：选择逻辑只能存在于一个文件/函数内（禁止扩散）

- [x] Default Engine：把既有核心组件“组装”成可调用单元（AC: 1）
  - [x] 复用既有实现（避免造轮子）：
    - [x] RuleEngine：`backend/src/core/rule_engine/DefaultRuleEngine`
    - [x] PromptGenerator：`backend/src/core/prompt_generator/DefaultPromptGenerator`
    - [x] Evaluator：`backend/src/core/evaluator/*`（含 `create_evaluator_for_task_config` 与同序契约）
    - [x] FeedbackAggregator：`backend/src/core/feedback_aggregator/DefaultFeedbackAggregator`
    - [x] Optimizer：`backend/src/core/optimizer/DefaultOptimizer`
    - [x] ExecutionTarget：`backend/src/core/execution_target/create_execution_target`
  - [x] 复用/封装已有“最小端到端编排入口”（如存在）：`backend/src/core/iteration_engine/orchestrator.rs` 中的纯内存可测函数（避免重复实现口径）
  - [x] 反模式（禁止）：在 `OptimizationEngine` 内重新实现迭代编排逻辑（先复用既有 `IterationEngine`/orchestrator；需要新增“缺失的编排入口”时，优先补到 orchestrator 并复用）
  - [x] 输出必须结构化、可观测：关键决策原因写入 `ctx.extensions` 或返回结构（禁止只打日志）

- [x] Alternate Engine：替换演练实现（AC: 2,3）
  - [x] 在算法模块内提供一个替代实现（例如 `AlternateOptimizationEngine` / `MockOptimizationEngine`）：
    - [x] 行为必须确定性（不出网、不依赖真实 LLM/Dify）
    - [x] 必须实现与 Default **同一个 `OptimizationEngine` 接口形状**（用于验证可替换性）
    - [x] 建议策略：内部使用 Example/Mock 依赖（如 `ExampleExecutionTarget`、`ExampleTeacherModel`）走通最小闭环；返回值/副作用稳定可断言（例如产出非空 prompt、更新 `ctx.state`、写入必要的 `ctx.extensions`）
    - [x] 目的不是“更聪明”，而是证明**可替换边界**与**最小变更面**成立
  - [x] 切换机制必须可复现：用 `cargo test --features alt-optimization-engine` 明确演练步骤

- [x] 测试与回归门禁（AC: 3）
  - [x] 新增/更新后端测试，覆盖：
    - [x] `create_optimization_engine` 在 Default/Alternate 下都能构建并运行最小闭环（至少断言：返回 `OptimizationResult`、行为可复现、`engine.name()` 区分实现）
    - [x] “替换不影响其他模块”的证据：必须复跑 `backend/tests/` 全量集成测试 + 关键 core 单测
    - [x] CI 门禁：在 `.github/workflows/ci.yml` 增加对 `alt-optimization-engine` 的测试覆盖（例如新增一步 `cargo test --all --features alt-optimization-engine` 或以矩阵方式跑两套）
  - [x] 测试硬约束：
    - [x] 禁止网络、禁止真实外部服务；使用 Trait Mock/Example 实现
    - [x] 不得泄露敏感信息（Prompt/TestCase input 全文、API key）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][CRITICAL] 明确“核心算法模块”的边界与对外 API：对外只能依赖 `OptimizationEngine`（或等价 Facade），禁止直接依赖 `core/*` 具体实现
- [x] [AI-Review][HIGH] 替换演练必须给出可复现命令序列（Default ↔ Alternate），并在 CI 下可验证（不出网、确定性）
- [x] [AI-Review][MEDIUM] 给出“最小闭环测试”参考实现位置（复用既有 `IterationEngine`/orchestrator 的纯内存入口），避免开发者另起炉灶

### Review Follow-ups (AI) — 2026-01-15 Senior Dev Review

- [x] [AI-Review][CRITICAL] 在 `DefaultOptimizationEngine` 调用 evaluator 前写入 `EXT_TASK_EVALUATOR_CONFIG`（避免非 Example evaluator 路径直接失败） `backend/src/core/optimization_engine/default_impl.rs:82`
- [x] [AI-Review][CRITICAL] 让 Alternate Engine 成为“可证伪”的替换实现：不应仅包装 Default + 打标，需具备实际差异且仍满足接口契约 `backend/src/core/optimization_engine/alternate_impl.rs:1`
- [x] [AI-Review][HIGH] CI 增加 alt feature 下的 clippy：`cargo clippy --features alt-optimization-engine -- -D warnings` `.github/workflows/ci.yml:45`
- [x] [AI-Review][MEDIUM] 强化 resume 测试断言：至少校验 iteration/state/prompt/rule_system 与 checkpoint 一致 `backend/tests/optimization_engine_smoke.rs:182`
- [x] [AI-Review][MEDIUM] 增加脱敏断言：确保 `TOPSECRET_DO_NOT_ECHO` 不会出现在 out/ctx.extensions/错误字符串 `backend/tests/optimization_engine_smoke.rs:96`

## Dev Notes

### Developer Context（读我：避免踩坑）

- 本仓库“可替换核心组件”的既有落点：`backend/src/core/traits.rs`（7 Trait 定义，已经写明“支持 Mock 替换”）。
- 既有“单一入口点工厂”模式（可直接复用做 OptimizationEngine 工厂）：
  - `backend/src/core/execution_target/mod.rs::create_execution_target`
  - `backend/src/core/evaluator/mod.rs::create_evaluator_for_task_config`（或等价工厂）
- 既有“最小端到端（纯内存可测）”入口示例：
  - `backend/src/core/execution_target/mod.rs#tests::example_execution_target_can_run_minimal_end_to_end_loop`
  - `backend/src/core/iteration_engine/orchestrator.rs::run_failure_archive_and_diversity_injection_step`（与 Story 4.6 绑定）
- 技术规格（权威）：`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md`  
  - 里面明确提出 `OptimizationEngine`（Facade）与 `ModuleRegistry`（运行时注册/查找）用于“核心算法可替换”。

### Technical Requirements（DEV AGENT GUARDRAILS）

1) **边界纪律（最重要）**
- `api/` → `core/`：只通过受控接口调用（对齐 `docs/project-planning-artifacts/architecture.md#架构边界定义`）。本 Story 的目标是让未来调用方只依赖 `OptimizationEngine`（或等价 Facade），而不是直接 new 各种 default_impl。
- **算法选择只能在一个入口点发生**：禁止散落 `match/if`（对齐 Story 4.7 的“注册点单一”原则）。

2) **替换演练必须“可回滚 + 可复跑”**
- 必须提供 Default/Alternate 双实现，并用测试/命令证明可切换且不影响其他模块测试通过。
- 本 Story **仅允许** 编译期 feature（`alt-optimization-engine`，最小影响面）；运行时配置/注册表能力另起 Story。

3) **禁止破坏既有口径与序列化形状**
- 不得随意改动已导出的领域模型 JSON shape（前端依赖 `ts-rs` 生成类型）。
- 如必须新增字段/类型：优先“新增不破坏”，并补齐生成/前端验证路径（仅在确有跨端需求时）。  
  - 仅当本 Story 选择“运行时配置可选”并需要新增 domain 枚举/字段时，才需要执行 `cd backend && cargo run --bin gen-types` 并校验前端类型（否则不应引入跨端变更）。

4) **安全与确定性**
- 错误/日志/测试输出不得包含 `current_prompt`、TestCase `input` 全文或任何 API key。
- 所有演练测试必须不出网（Mock/Example 实现），并在 CI 稳定复现。

### Architecture Compliance

- “核心算法模块”建议落点：`backend/src/core/optimization_engine/`（作为可指认边界）。  
  - 内部可以组合/复用 `core/rule_engine`、`core/prompt_generator`、`core/evaluator`、`core/feedback_aggregator`、`core/optimizer`、`core/iteration_engine` 与 `core/execution_target`。
  - 对外只暴露 Facade + 工厂（例如 `OptimizationEngine` + `create_optimization_engine`）。
- 遵守项目结构与边界（见 `docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries` 与 `#架构边界定义`）：
  - `core/` 只放算法与可替换实现；HTTP/DB/外部调用在 `api/` 与 `infra/`。

### Dependency → Gap → DoD（实现证据链）

> 目的：把 Epic/前置 Story 的“依赖关系”落成开发者可执行的缺口清单与验收证据，避免脑补与重复造轮子。

| 依赖/既有资产（应复用） | 本 Story 的复用方式 | 本 Story 需要补齐的缺口 | DoD 证据（必须可复现） |
|---|---|---|---|
| `backend/src/core/traits.rs`（7 Trait） | `OptimizationEngine` 内持有/注入这些 trait 实例（不改 trait 形状） | 新增 `OptimizationEngine` Facade（接口在 Story Tasks 已锁死） | Default/Alternate 均能编译并通过最小闭环测试 |
| `backend/src/core/iteration_engine/orchestrator.rs`（`IterationEngine` + 纯内存入口） | **复用既有编排/执行逻辑**，禁止在 Engine 内另起炉灶 | 若 Engine 需要的“最小闭环入口”不存在，优先补到 orchestrator 再复用 | 最小闭环测试必须走复用路径（避免重复实现口径） |
| `backend/src/core/execution_target/mod.rs`（工厂 + 示例测试） | 复用 `create_execution_target` 与 ExampleExecutionTarget 的确定性行为 | Engine 工厂与测试应遵循同样的“单一入口点 + 可诊断”模式 | `cd backend && cargo test --all` |
| `backend/src/core/evaluator/mod.rs::create_evaluator_for_task_config` | 复用 Evaluator 选择/装配逻辑（同序契约保持不变） | Engine 内部组装时不得绕开同序契约/统计口径 | 相关 core 单测继续通过（不得引入回归） |
| `backend/src/core/teacher_model/*`（`ExampleTeacherModel`） | 测试/Alternate 路径用 ExampleTeacherModel（不出网） | 明确避免真实外部调用路径（LLM/Dify） | 最小闭环测试不出网且可重复 |
| `backend/src/domain/types/optimization_context.rs`（`OptimizationContext`） | `run/resume` 接收 `&mut OptimizationContext` 并写入 `ctx.state/extensions` | 结构化可观测输出写入 `ctx.extensions`（禁止只打日志） | 最小闭环测试断言：`ctx.state` 与关键 `ctx.extensions` 键存在 |
| `backend/src/domain/models/algorithm.rs`（`Checkpoint`、`OptimizationResult` 等） | `resume(checkpoint, ctx)` 复用现有模型（不新增跨端字段） | 仅当确需新增领域字段/枚举才触发 TS types 生成（本 Story 默认不触及） | 若未触及 domain：无需 `gen-types`；若触及：必须 `cargo run --bin gen-types` 并通过前端检查 |
| `.github/workflows/ci.yml`（现有 CI） | 增加 alt feature 覆盖以防路径腐烂 | 新增 CI 步骤：`cargo test --all --features alt-optimization-engine`（或矩阵） | CI 必须同时覆盖 Default + Alternate 两条路径 |

### Epic 4 全景（位置与依赖）

- Epic 4 目标：四层算法工程化（可运行、可测试、可替换）。  
- 已完成相关基石：
  - 4.1 `core/rule_engine`
  - 4.2 `core/prompt_generator`（确定性模板变体）
  - 4.3 `core/evaluator`（同序契约 + 统计口径）
  - 4.4 `core/feedback_aggregator` + `core/optimizer`（注意：当时明确“未实现 OptimizationEngine”）
  - 4.5 `core/iteration_engine` + 执行调度（串行/并行）
  - 4.6 失败档案 + 多样性注入（已有纯内存可测编排入口）
  - 4.7 扩展模板与文档（ExecutionTarget/Evaluator/TeacherModel）+ 单一入口点模式
- 4.8（本 Story）：把“核心算法可替换”从分散的 Trait/模块，收敛为一个可替换算法模块边界，并完成替换演练（NFR15）。

### Library / Framework Requirements（版本与依赖边界）

> 本 Story 不要求升级依赖；以 lockfile 为准（`backend/Cargo.lock` / `frontend/package-lock.json` 或等价）。

- 后端：Rust Edition 2024（`backend/Cargo.toml`），`axum 0.8`、`tokio 1`、`sqlx 0.8`、`reqwest 0.12`、`ts-rs 10`、`utoipa 5`。
- 若需要“注册表/单例”能力：优先使用标准库（本仓库 `rust-version = 1.85`，可用 `std::sync::OnceLock`），避免引入 `once_cell` 等新依赖。
- 提示：crates.io 可能已有更新版本，但本 Story 以 lockfile 为准（升级依赖请单独开 Story/PR）。

### File Structure Requirements（建议改动清单）

**Backend（Rust）**

- `backend/src/core/optimization_engine/`（新增，算法模块边界）
  - `mod.rs`：对外导出 + 单一 `create_optimization_engine(...)` 入口
  - `default_impl.rs`：Default 实现（组装既有模块）
  - `alternate_impl.rs`（或等价）：替代实现（用于替换演练）
  - `error.rs`：统一错误类型（脱敏）
- `backend/src/core/mod.rs`：导出 `optimization_engine`
- 测试（择一即可，按仓库习惯）：
  - `backend/src/core/optimization_engine/*` 的 `#[cfg(test)]` 单测
  - 或 `backend/tests/optimization_engine_*` 集成测试（用于覆盖 feature 切换与跨模块回归）

### Testing Requirements

- 必须提供“替换演练”的可复现命令（feature 名固定）：
  - Default：`cd backend && cargo test --all`
  - Alternate：`cd backend && cargo test --all --features alt-optimization-engine`
- 覆盖范围最低要求：
  - 复跑 `backend/tests/`（确保与算法无关的 CRUD/API 行为不受影响）
  - 至少一个“最小闭环”测试：构造 `OptimizationContext` + `TestCase`，走一条确定性路径（不出网）
- CI 门禁最低要求：必须覆盖 Default 与 Alternate 两条测试路径（避免 feature 路径腐烂）。

## Previous Story Intelligence

- 来自 Story 4.4：当时明确“未实现 OptimizationEngine”；本 Story 要把它补齐为**可替换边界**（但仍应避免把 checkpoint/WS/可靠性一口气全塞进来造成 scope creep）。
- 来自 Story 4.6：已存在“纯内存可测”的编排入口与脱敏约束；应复用该入口做最小闭环测试，避免重复实现口径。
- 来自 Story 4.7：坚持“单一入口点 + 不引入新依赖 + 确定性测试 + 脱敏硬约束”的工程风格；算法替换同样必须遵守。
- 来自 Story 4.7 的具体 review 修复点（避免复犯）：`execute_batch`/`evaluate_batch` 的同序契约与脱敏边界最终是靠单测+证据链写死的；本 Story 也应把“替换演练/feature 路径”写进测试与 CI 门禁，避免长期腐烂。

## Git Intelligence Summary

- 近期 core 模块模式已稳定：`mod.rs + default_impl.rs + error.rs (+ #[cfg(test)])`，并倾向把“硬契约”写进单测防回归（参考最近提交：`acc68d6`、`782b1a6`、`8172a61`、`90161ae`）。
- 工厂/选择逻辑已有成熟范式（ExecutionTarget/Evaluator），本 Story 应复用同样的“单入口点”结构，避免到处散落 `match`。

## Project Context Reference

- `**/project-context.md`：未发现（以规划/架构/技术规格与已完成 Story 的 Dev Notes + 现有代码为准）。

## References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md#Story 4.8: 核心算法模块解耦与替换演练`
- NFR15 与模块化目标：`docs/project-planning-artifacts/prd.md#可量化指标 (Measurable Outcomes)`、`docs/project-planning-artifacts/prd.md#7.4.2 模块化设计（Trait 概述）`
- 架构边界与目录结构：`docs/project-planning-artifacts/architecture.md#架构边界定义`、`docs/project-planning-artifacts/architecture.md#技术规格 7 Trait → 后端 core/ 子模块`
- 技术规格（权威）：`docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#4.2.9 OptimizationEngine 结构定义`
- 既有工厂与最小闭环示例：`backend/src/core/execution_target/mod.rs`、`backend/src/core/iteration_engine/orchestrator.rs`
- 测试体系与 Mock 约束：`docs/test-design-system.md#Mock 服务需求`

## Decisions (resolved)

1. 替换范围：替换 **整个 `OptimizationEngine` Facade**（对齐 NFR15）。  
2. 切换机制：使用编译期 feature（`alt-optimization-engine`）完成替换演练；运行时配置/注册表能力延期。  
3. 验收测试：本 Story 的最低门槛为后端 Default+Alternate 两条路径测试通过（含 `backend/tests/`）；前端 E2E 由 CI 全局兜底，保持绿即可。

## Story Completion Status

- Status set to `done`
- Completion note: 2026-01-15 已合并至 main，main 分支 CI 全绿。

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

- `cd backend && cargo test --all`
- `cd backend && cargo test --all --features alt-optimization-engine`
- `cd backend && cargo clippy -- -D warnings`

### Completion Notes List

- 新增 `core/optimization_engine` 作为可替换算法边界，并提供 Facade + 单一工厂入口 `create_optimization_engine(...)`
- 通过 cargo feature `alt-optimization-engine` 实现编译期替换（Default ↔ Alternate），并在 CI 增加双路径测试门禁
- 增加最小闭环集成测试（确定性、不出网）覆盖 run/resume 与 feature 切换

### File List

- `.github/workflows/ci.yml`
- `backend/Cargo.toml`
- `backend/src/core/mod.rs`
- `backend/src/core/optimization_engine/mod.rs`
- `backend/src/core/optimization_engine/default_impl.rs`
- `backend/src/core/optimization_engine/alternate_impl.rs`
- `backend/src/core/optimization_engine/error.rs`
- `backend/tests/optimization_engine_smoke.rs`
- `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md`
- `docs/implementation-artifacts/sprint-status.yaml`

### Change Log

- 引入 `OptimizationEngine` 可替换 Facade 与 feature gate（`alt-optimization-engine`），并在 CI + tests 覆盖 Default/Alternate 双路径
- 2026-01-15 Senior Dev Review：Changes Requested（修复 evaluator 配置注入、增强 Alternate 可证伪性、补齐 CI clippy 与测试断言）
- 2026-01-15 Follow-up Review：Approve（Ready to merge；Status 保持 review）
- 2026-01-15 Shipped：已合并 main 且 main CI 通过；Story 状态更新为 done
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

#### RESOLVED (2026-01-15)

1) ✅ DefaultEvaluator 路径“隐藏炸弹”已修复：评估前注入 `EXT_TASK_EVALUATOR_CONFIG`，并新增 smoke 覆盖 DefaultEvaluator 路径。  
   - 修复：`backend/src/core/optimization_engine/default_impl.rs:83`、`backend/tests/optimization_engine_smoke.rs:105`

2) ✅ Alternate Engine 已变为可证伪替换实现：引入 fast-path（达标跳过 rule/reflect/optimize）与 full pipeline 两条路径，且不再包装 DefaultEngine。  
   - 修复：`backend/src/core/optimization_engine/alternate_impl.rs:57`

3) ✅ CI 已覆盖 alt feature 的 clippy。  
   - 修复：`.github/workflows/ci.yml:45`

4) ✅ resume 测试断言已补强（覆盖 task_id/prompt/iteration/state）。  
   - 修复：`backend/tests/optimization_engine_smoke.rs:139`

5) ✅ 脱敏证据链已补齐：新增断言，确保敏感 input 不出现在 `out` 与 `ctx.extensions` 序列化结果中。  
   - 修复：`backend/tests/optimization_engine_smoke.rs:91`

#### MEDIUM (remaining)

6) **OptimizationContext 文档与现实不一致**：注释声明“只有编排层能更新 Context”，但当前 `OptimizationEngine` 本身就是编排入口并直接写 `ctx.*`。建议更新注释表述（把 Engine 纳入“编排层/入口层”定义），避免误导后续贡献者。  
   - 证据：`backend/src/domain/types/optimization_context.rs:8`

7) **Default/Alternate 代码重复度高**：两份实现存在大量重复（执行→评估→写 extensions），未来容易 drift（契约键名/写入口径不一致）。建议抽出共享 helper（仅内部模块私有）降低维护成本。  
   - 证据：`backend/src/core/optimization_engine/default_impl.rs:60`、`backend/src/core/optimization_engine/alternate_impl.rs:57`

#### LOW (remaining)

8) **extensions 的 JSON key 顺序可能不稳定**：将 `HashMap` 直接 `to_value` 可能带来序列化顺序差异；当前测试不依赖顺序，但若未来引入快照/对比测试需注意。  
   - 证据：`backend/src/core/optimization_engine/default_impl.rs:91`、`backend/src/core/optimization_engine/alternate_impl.rs:89`

### Decisions

- **Outcome**：Approve（Ready to merge）。  
- **Status**：维持 `review`（等待合并/发布流程）。  
- **AC 口径**：AC3 以两条后端测试路径（Default/Alternate）均通过为最低门槛；Alternate 现在具备可观察差异，可作为“可证伪替换演练”的证据链一部分。

### Risks / Tech Debt

- Default/Alternate 仍有较高重复度，未来若新增 extensions 键或评估口径，可能出现两实现不一致的 drift（建议后续小重构消除重复）。

### Follow-ups

- 已同步到 `### Review Follow-ups (AI)`（见下方新增条目，均带 file:line）。

## Senior Developer Review (AI)

**Reviewer:** 耶稣  
**Date:** 2026-01-15  
**Epic/Story:** 4.8  
**Decision:** Changes Requested  

**Validated commands (local):**
- `cd backend && cargo test --all`
- `cd backend && cargo test --all --features alt-optimization-engine`

**Summary:**
- Git vs Story File List：一致（无差异）
- 结论：当前实现“结构与门面”方向正确，但替换演练与默认评估器路径存在关键缺口，需修复后再标记 done。

### Fix Verification (AI) — 2026-01-15

已完成修复并本地验证通过（Default + Alternate 两条路径）：
- 注入 `EXT_TASK_EVALUATOR_CONFIG`，覆盖非 Example evaluator 路径
- Alternate Engine 引入可观察差异：达标时走 fast-path（跳过 rule/reflect/optimize），未达标时走 full pipeline（仍保持确定性/不出网）
- CI 增加 alt feature 下的 clippy
- 补强 `resume` 测试断言与脱敏断言

**Recommended next status:** `review`（等待再次 code review / 合并）

### Final Review (AI) — 2026-01-15

**Decision:** Approve（Ready to merge）  
**Validated commands (local):**
- `cd backend && cargo fmt --all`
- `cd backend && cargo clippy -- -D warnings`
- `cd backend && cargo clippy --features alt-optimization-engine -- -D warnings`
- `cd backend && cargo test --all`
- `cd backend && cargo test --all --features alt-optimization-engine`

### Code Review Checklist Traceability (AI) — 2026-01-15

对照：`_bmad/bmm/workflows/4-implementation/code-review/checklist.md`

- [x] Story file loaded from `{{story_path}}`  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:1`
- [x] Story Status verified as reviewable (review)  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:3`
- [x] Epic and Story IDs resolved ({{epic_num}}.{{story_num}})  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:1`、`docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:7`、`docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:337`
- [x] Story Context located or warning recorded  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:9`、`docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:110`
- [x] Epic Tech Spec located or warning recorded  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:121`、`docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:232`
- [x] Architecture/standards docs loaded (as available)  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:33`、`docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:231`
- [x] Tech stack detected and documented  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:183`
- [x] MCP doc search performed (or web fallback) and references captured  
  Evidence: 本次未使用 MCP；以仓库内权威文档作为 fallback 并在 References 列出：`docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:227`
- [x] Acceptance Criteria cross-checked against implementation  
  Evidence: AC1-3：`docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:27`；工厂/单一入口点：`backend/src/core/optimization_engine/mod.rs:40`；feature 路径测试：`backend/tests/optimization_engine_smoke.rs:35`
- [x] File List reviewed and validated for completeness  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:265`；git status 已对照（无差异）
- [x] Tests identified and mapped to ACs; gaps noted  
  Evidence: `backend/tests/optimization_engine_smoke.rs:35`；CI 双路径：`.github/workflows/ci.yml:71`
- [x] Code quality review performed on changed files  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:362`
- [x] Security review performed on changed files and dependencies  
  Evidence: 错误脱敏：`backend/src/core/optimization_engine/error.rs:15`；脱敏断言：`backend/tests/optimization_engine_smoke.rs:91`
- [x] Outcome decided (Approve/Changes Requested/Blocked)  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:321`、`docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:360`
- [x] Review notes appended under "Senior Developer Review (AI)"  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:333`
- [x] Change Log updated with review entry  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:278`
- [x] Status updated according to settings (if enabled)  
  Evidence: `docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:3`；`docs/implementation-artifacts/4-8-core-algorithm-module-decoupling-and-replacement.md:242`
- [x] Sprint status synced (if sprint tracking enabled)  
  Evidence: `docs/implementation-artifacts/sprint-status.yaml:99`
- [x] Story saved successfully  
  Evidence: 本文件与相关代码/CI 变更均已落盘（见 git status）
