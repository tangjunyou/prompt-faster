# Story 4.7: 扩展模板与文档（ExecutionTarget / Evaluator / TeacherModel）

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 4-7-extension-templates-and-docs

## Key Decisions (MVP)

- **目标是“让扩展变成可复制的套路”**：提供统一的扩展指南 + 可编译的模板/示例，实现“复制→改名→实现→注册→配置启用→跑通用例”的闭环。
- **不做依赖升级/不引入新依赖**：本 Story 聚焦扩展点工程化；若扩展需要新增 crate / npm 包，请单独开 Story/PR（避免混入回归风险）。
- **示例实现必须确定性、可测试、默认不出网**：`Example*` 用于演示扩展流程与回归防护，不依赖真实 LLM/Dify，避免 flaky。
- **注册点单一**：新增实现只允许改“一个入口点”（工厂/注册表），禁止把选择逻辑散落到业务代码与多处 match。
- **安全与脱敏硬约束继承**：日志/错误/文档示例不得包含 API Key/Prompt/TestCase input 全文；必要时只用 `*_len`、截断片段或结构化摘要。

## Story

As a 开发者,
I want 通过统一的扩展模板和文档快速新增执行目标、评估器和老师模型实现,
so that 可以在不修改核心框架的前提下扩展支持新的执行/评估逻辑。

## Acceptance Criteria

1. **Given** 项目已经实现基础的 `ExecutionTarget` / `Evaluator` / `TeacherModel` trait 定义  
   **When** 开发者查阅项目文档  
   **Then** 能找到一节专门介绍如何新增执行引擎、评估器和老师模型的扩展点  
   **And** 文档中包含从复制模板、实现必要方法到在配置中启用新实现的完整步骤（含“需要改哪些文件/为什么”与最小回归清单）。
2. **Given** 开发者按文档为 ExecutionTarget 新增一个示例实现（例如 `ExampleExecutionTarget`）  
   **When** 仅在示例模块中实现必要接口并在工厂/注册表中注册该实现  
   **Then** 不需要修改现有调用 `ExecutionTarget` 的业务代码即可在任务配置中选择该执行目标  
   **And** 使用该执行目标可以完成一轮“端到端优化任务”的最小闭环验证（至少覆盖：创建最小 `OptimizationContext` → 执行一批 `TestCase` → 进入评估并产出统计摘要）  
   **And** 实际人力投入时间不超过 4 小时（NFR12）。
3. **Given** 开发者按文档为 Evaluator 新增一个示例实现（例如 `ExampleEvaluator`）  
   **When** 仅在示例模块中实现必要接口并在配置中声明可用  
   **Then** 用户可以在任务配置中选择该评估器并成功运行一轮评估（至少覆盖：`evaluate_batch` 同序契约 + 逐用例结果可追溯）  
   **And** 实际人力投入时间不超过 2 小时（NFR13）。
4. **Given** 开发者按文档为 TeacherModel 新增一个示例实现（例如 `ExampleTeacherModel`）  
   **When** 仅在示例模块中实现必要接口并在配置中声明可用  
   **Then** 可以将该老师模型用于一轮最小闭环验证（至少覆盖：被 `DefaultEvaluator` 的 `teacher_model` 路径使用，并通过确定性单测证明 timeout/解析/脱敏边界仍成立）  
   **And** 实际人力投入时间不超过 2 小时（NFR14）。
5. **Given** 团队需要验证上述扩展耗时指标  
   **When** 以具备 Prompt Faster 项目上下文的开发者为基准，从复制官方扩展模板开始到跑通文档中的示例用例结束计时  
   **Then** 计时范围仅包含编码与本地验证时间，不包含依赖下载、CI 排队等等待时间（在文档中明确计时口径与复现步骤）。

## Tasks / Subtasks

- [x] 扩展指南与入口（AC: 1,5）
  - [x] 新增开发者文档：`docs/developer-guides/extensions.md`（或等价路径），必须包含：
    - [x] 3 类扩展点概览（ExecutionTarget / Evaluator / TeacherModel）与它们在系统中的职责边界
    - [x] 概念澄清（避免误解）：
      - [x] `TeacherModel`（trait，可被 `DefaultEvaluator` 注入用于 `EvaluatorType::TeacherModel` 路径）≠ 任务配置里的 `teacher_llm.model_id`（仅“选择模型 ID”，不等于新增 TeacherModel 实现）
      - [x] PRD 提到的 `generate_structured` 与当前代码的 trait 签名可能不一致：以 `backend/src/core/traits.rs::TeacherModel` 的 `generate/generate_stream` 为准；结构化 JSON 解析与护栏由 Evaluator 路径负责
    - [x] “最小改动清单”：每类扩展新增实现时需要改哪些文件（含原因）
    - [x] “反模式清单”：避免新依赖/错误目录/泄露敏感信息/破坏同序契约/未更新 TS types
    - [x] TS 类型生成与验证（强制，避免跨端漂移）：
      - [x] 生成命令：`cd backend && cargo run --bin gen-types`
      - [x] 生成结果：检查 `frontend/src/types/generated/models/` 中对应类型（如 `ExecutionTargetType.ts` / `EvaluatorType.ts`）是否更新
      - [x] 最小验证：`cd frontend && npm run build`（包含 `tsc -b`，用于类型校验）；必要时再跑 `npm run test`
    - [x] “计时口径”与复现命令（AC5）
  - [x] 在 `README.md#📖 文档` 与 `docs/implementation-artifacts/index.md` 增加该扩展指南入口链接（让新同学可发现）。

- [x] ExecutionTarget 扩展示例与模板（AC: 2）
  - [x] 在 `backend/src/core/execution_target/` 添加可复制模板（建议：`example_template.rs` + `README` 小节或 doc 注释），明确必须实现的方法与错误处理边界（脱敏）。
  - [x] 新增 `ExampleExecutionTarget`（确定性、默认不出网）并在单一入口点注册/选择：
    - [x] 后端：在 `backend/src/core/execution_target/mod.rs` 的工厂/注册表增加选择分支（或改造为注册表，但必须保持注册点单一）
    - [x] Domain/TS：如需新增枚举项，更新 `ExecutionTargetType`（Rust + `backend/src/bin/gen-types.rs`）并生成 `frontend/src/types/generated/` 对应变更
    - [x] API/DB（必改，否则“前端可选但后端拒绝/落库失败”）：
      - [x] API 入参解析白名单：`backend/src/api/routes/optimization_tasks.rs::parse_execution_target_type`
      - [x] DB 序列化/反序列化映射：`backend/src/infra/db/repositories/optimization_task_repo.rs`（parse/serialize）
    - [x] 前端：在 `frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx` 的下拉选项中暴露该执行目标（并补齐测试）
  - [x] 增加最小闭环验证（建议优先“确定性测试/工具”而非真实 LLM）：
    - [x] 新增后端单测：验证 `ExampleExecutionTarget.execute_batch` 同序契约 + `test_case_id` 对齐
    - [x] 新增一个最小 smoke（可为 `#[cfg(test)]` 或 bin）：构造 `OptimizationContext` + `TestCase`，跑一轮“执行→评估→摘要输出”。

- [x] Evaluator 扩展示例与模板（AC: 3）
  - [x] 在 `backend/src/core/evaluator/` 添加可复制模板（建议：`example_template.rs`），明确：
    - [x] `evaluate_batch` **不得过滤/重排**输入；输出必须同序
    - [x] 必须填充 `EvaluationResult.evaluator_type` 与 `extra`（用于可观测性）
  - [x] 新增 `ExampleEvaluator` 并在“单一入口点”声明可用：
    - [x] 后端：将选择逻辑集中在 1 个位置（建议在 `DefaultEvaluator` 的“选择层”或新增 `create_evaluator(...)` 工厂；禁止在多处散落 match）
    - [x] Domain/TS：如需新增枚举项，更新 `EvaluatorType`（Rust + 生成 TS）
    - [x] 前端：在 `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx` 下拉中暴露该评估器（并补齐测试）
  - [x] 增加确定性单测：覆盖 `ExampleEvaluator` 的 `evaluate` 与 `evaluate_batch`（含同序契约、未知/重复 id 防护、阈值/extra 字段存在性）。

- [x] TeacherModel 扩展示例与模板（AC: 4）
  - [x] 对齐架构文档的目录约定：补齐 `backend/src/core/teacher_model/`（当前目录可能为空/缺失实现文件）
    - [x] 让模块被 crate 编译/导出：在 `backend/src/core/mod.rs` 增加 `pub mod teacher_model;`
    - [x] `backend/src/core/teacher_model/mod.rs`：统一导出与构造入口（可先只导出模板/示例）
    - [x] `backend/src/core/teacher_model/example_impl.rs`：`ExampleTeacherModel`（确定性、默认不出网）
  - [x] 在扩展指南中明确 TeacherModel 的安全护栏：
    - [x] 必须支持超时（参考 `core/evaluator/default_impl.rs` 的 teacher_model timeout 模式）
    - [x] 输出解析失败要可诊断且不泄露原文（raw_excerpt 截断）
    - [x] 流式接口约定（若暂不实现，写清楚 stub/NotSupported 的策略）
  - [x] 增加单测：证明 `ExampleTeacherModel` 可被注入 `DefaultEvaluator::new(Some(tm))` 并触发 `teacher_model` 路径（不出网、稳定可复现）。

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][CRITICAL] 将本次审查结论沉淀到 `## Review Notes`（含取舍与风险）
- [x] [AI-Review][CRITICAL] 补齐 `ExampleExecutionTarget.execute_batch` 的显式单测（避免“Story 声称有但代码无证据”）
- [x] [AI-Review][HIGH] 修复 `ExampleEvaluator` 错误信息可能泄露 `TaskReference` 内容的问题（脱敏约束）
- [x] [AI-Review][MEDIUM] 补齐 AC5 的可复制“计时复现命令序列”

## Dev Notes

### Developer Context（读我：避免踩坑）

- 扩展点 trait 定义：`backend/src/core/traits.rs`（`ExecutionTarget` / `Evaluator` / `TeacherModel`）。  
- ExecutionTarget 工厂入口：`backend/src/core/execution_target/mod.rs::create_execution_target`（当前为 match；本 Story 的目标是让“新增实现的改动点”尽量集中、可复制）。  
- Evaluator 默认实现与契约：`backend/src/core/evaluator/default_impl.rs`（同序契约、extra 可观测性、TeacherModel 可选注入与 timeout）。  
- 任务配置与枚举（会影响 TS/前端下拉）：`backend/src/domain/models/optimization_task.rs`（ExecutionTargetType）与 `backend/src/domain/models/optimization_task_config.rs`（EvaluatorType）。  
- TS 类型生成入口：`backend/src/bin/gen-types.rs` → 输出到 `frontend/src/types/generated/`（新增枚举/字段必须同步生成，否则前端类型/运行时会漂移）。  

### Technical Requirements（DEV AGENT GUARDRAILS）

- **禁止**把“如何选择实现”的逻辑散落到多处：ExecutionTarget/Evaluator/TeacherModel 的选择必须集中在单一入口点（工厂/注册表）。  
- 示例实现必须满足 **确定性、无网络依赖**：只用于演示与回归防护；真实集成（新 provider/新协议）请单独开 Story。  
- 所有新增错误/日志/文档示例必须脱敏：不得包含 API Key、Prompt/TestCase input 全文；必要时使用截断/长度/结构化摘要。  

### Architecture Compliance

- 遵守项目边界：`core/` 只放核心算法与可替换实现；HTTP/DB/外部调用在 `api/` 与 `infra/`（见 `docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`）。  
- 遵守既定契约：  
  - `ExecutionTarget.execute_batch` 默认串行，同序返回；并行由编排层/调度层负责（见 `backend/src/core/traits.rs` 与 Story 4.5）。  
  - `Evaluator.evaluate_batch` 输入/输出必须同序，且不得过滤/重排（见 Story 4.3）。  

### Epic 4 全景（位置与依赖）

- Epic 4 目标：把“自动迭代优化”四层架构落到可运行、可测试、可扩展的工程形态（Layer1-4 + 执行调度 + 可靠性/可观测性）。  
- Epic 4 相关 Stories（按流水线顺序）：  
  - 4.1 规律抽取（RuleEngine）→ 4.2 候选生成（PromptGenerator）→ 4.3 质量评估（Evaluator）→ 4.4 反思迭代（FeedbackAggregator/Optimizer）  
  - 4.5 执行模式与并行调度（ExecutionTarget + scheduler）→ 4.6 失败档案与多样性注入（避免重复失败 + 跳出局部最优）  
  - 4.7（本 Story）：把 3 个关键可替换点（ExecutionTarget/Evaluator/TeacherModel）工程化为“可复制模板 + 文档 + 示例闭环”，降低后续扩展成本  
  - 4.8 核心算法模块解耦与替换演练（NFR15，验证“替换算法不影响其他模块”）  

### Library / Framework Requirements（版本与依赖边界）

> 本 Story 不要求升级依赖；以 lockfile 为准（`backend/Cargo.lock` / `frontend/package-lock.json` 或等价）。

- 后端：Rust Edition 2024（`backend/Cargo.toml`），`axum 0.8`、`tokio 1`、`sqlx 0.8`、`reqwest 0.12`、`ts-rs 10`、`utoipa 5`。  
- 前端：React 19、React Router 7、Vite 7、Vitest 2（见 `frontend/package.json`）。  

### File Structure Requirements（建议改动清单）

**Docs**

- `docs/developer-guides/extensions.md`（新增）
- `README.md`（补链接）
- `docs/implementation-artifacts/index.md`（补链接）

**Backend（Rust）**

- `backend/src/core/execution_target/`：新增模板/示例实现（如 `example_template.rs` / `example_impl.rs`）
- `backend/src/core/execution_target/mod.rs`：注册/选择入口点（单一改动点）
- `backend/src/core/evaluator/`：新增模板/示例实现或工厂（单一选择入口点）
- `backend/src/core/teacher_model/`：补齐 `mod.rs` 与示例实现（目录与架构文档对齐）
- `backend/src/core/mod.rs`：导出 `teacher_model` 模块（避免“文件存在但未编译进 crate”）
- `backend/src/domain/models/optimization_task.rs`、`backend/src/domain/models/optimization_task_config.rs`：如新增枚举项，需要同步 TS types
- `backend/src/bin/gen-types.rs`：如新增导出类型，需要更新 export 列表

**Frontend（React/TS）**

- `frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx`：ExecutionTarget 下拉选项（建议改为使用生成的 `ExecutionTargetType`，避免手写 union 漂移）
- `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`：Evaluator 下拉选项
- 对应测试文件：`*.test.tsx`（新增/更新断言）

### Testing Requirements

- 后端（Rust）：
  - 必须新增确定性单测覆盖：ExecutionTarget 同序契约、Evaluator 同序契约、TeacherModel 注入路径（不出网）。
  - 推荐在 `backend/src/core/*/` 内用 `#[cfg(test)]` 就地加测试，避免新增测试 harness 复杂度。
- 前端（Vitest）：
  - 新增/更新下拉选项的 UI 测试，确保新增枚举值可选择并能正确回填到 payload。

## Previous Story Intelligence

- 来自 Story 4.5（ExecutionTarget + 并行调度）：ExecutionTarget 的 **同序契约** 与 **脱敏错误** 已写死；新增实现必须遵守，否则会导致 Layer 3/4 错位与回归。见 `docs/implementation-artifacts/4-5-execution-mode-and-parallel-scheduling.md`。  
- 来自 Story 4.3（Evaluator）：`evaluate_batch` 必须同序返回；选择逻辑必须可诊断（`extra` 字段），TeacherModel 路径必须有 timeout/解析护栏。见 `docs/implementation-artifacts/4-3-quality-assessment-layer.md`。  
- 来自 Story 4.6（Failure Archive）：坚持“不引入新依赖 + 确定性测试 + 脱敏硬约束”的工程风格，避免扩展点变成回归入口。见 `docs/implementation-artifacts/4-6-failure-archive-and-diversity-injection.md`。  

## Git Intelligence Summary

- 近期 core 模块模式已稳定：`mod.rs + default_impl.rs + error.rs (+ #[cfg(test)])`，并倾向把“硬契约”写进单测防回归（参考最近提交：`e2d2f44`、`90161ae`、`8172a61`）。  
- TS 类型生成是跨端一致性的关键：新增/变更 Rust 枚举或 API shape 后，必须同步 `gen-types` 输出与前端 UI 下拉，否则会出现“后端支持但前端不可选/类型不匹配”的隐性故障。  

## Latest Tech Information (Non-binding)

> 仅用于避免“用错 API / 用错版本”；本 Story **不**要求升级依赖（升级请单独开 Story）。

- Rust `OnceLock` 已稳定可用（本仓库 rustc >= 1.85），若实现注册表/单例工厂，优先使用标准库避免引入 `once_cell`。  
- 版本提示（以 2026-01-14 为参考点）：`axum` 0.8.8、`tokio` 1.49.0、`sqlx` 0.8.6、`reqwest` 已出现 0.13.x（仓库仍为 0.12）、`ts-rs` 已出现 11.x（仓库仍为 10）、`utoipa` 5.4.0。  
- 若扩展涉及外部服务（新 Provider/新协议）：优先复用 `backend/src/infra/external/` 的“唯一外部调用点”约束（见 `backend/src/infra/external/llm_client.rs` 的注释与白名单策略），并补齐 SSRF/超时/错误截断等护栏。  

## Project Context Reference

- `**/project-context.md`：未发现（以规划/架构/已完成 Story 的 Dev Notes 与现有代码为准）。

## References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md#Story 4.7: 扩展模板与文档（ExecutionTarget / Evaluator / TeacherModel）`  
- PRD（Trait 概述与扩展性保障）：`docs/project-planning-artifacts/prd.md#7.4.2 模块化设计（Trait 概述）`、`docs/project-planning-artifacts/prd.md#7.4.3 扩展性保障`  
- 架构/边界/目录结构：`docs/project-planning-artifacts/architecture.md#Project Structure & Boundaries`、`docs/project-planning-artifacts/architecture.md#技术规格 7 Trait → 后端 core/ 子模块`  
- ExecutionTarget 契约与落点：`backend/src/core/traits.rs`、`backend/src/core/execution_target/mod.rs`、`docs/implementation-artifacts/4-5-execution-mode-and-parallel-scheduling.md`  
- Evaluator 契约与 TeacherModel 注入：`backend/src/core/evaluator/default_impl.rs`、`docs/implementation-artifacts/4-3-quality-assessment-layer.md`  
- 测试与 Mock 策略：`docs/test-design-system.md#Mock 服务需求`  
- TS 类型生成：`backend/src/bin/gen-types.rs`  

## Story Completion Status

- Status set to `review`
- Completion note: 已补齐实现与回归测试；本文件处于“审查/验收”阶段（见下方 Review Notes）。

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

- 本地验证：`cd backend && cargo test`
- 类型生成：`cd backend && cargo run --bin gen-types`
- 前端验证：`cd frontend && npm run test -- --run`

### Completion Notes List

- 新增扩展点开发指南：`docs/developer-guides/extensions.md`，覆盖 ExecutionTarget/Evaluator/TeacherModel 的职责边界、最小改动清单、反模式与 TS 类型同步流程。
- ExecutionTarget：新增 `ExampleExecutionTarget`（确定性、不出网），并通过 `ExecutionTargetType::Example` 在单一工厂入口注册；同步 API/DB 解析映射与前端下拉/测试。
- Evaluator：新增 `ExampleEvaluator`（确定性）+ `create_evaluator_for_task_config` 工厂，支持通过 `EvaluatorType::Example` 选择实现；补齐同序/可追溯单测。
- TeacherModel：补齐 `core/teacher_model` 模块并提供 `ExampleTeacherModel`；将 DefaultEvaluator 的 TeacherModel 单测切换为示例实现，覆盖 timeout 与 fenced-json 解析护栏。

### File List

- README.md
- backend/src/api/routes/optimization_tasks.rs
- backend/src/core/evaluator/default_impl.rs
- backend/src/core/evaluator/example_impl.rs
- backend/src/core/evaluator/mod.rs
- backend/src/core/execution_target/example_impl.rs
- backend/src/core/execution_target/mod.rs
- backend/src/core/mod.rs
- backend/src/core/teacher_model/example_impl.rs
- backend/src/core/teacher_model/mod.rs
- backend/src/domain/models/optimization_task.rs
- backend/src/domain/models/optimization_task_config.rs
- backend/src/infra/db/repositories/optimization_task_repo.rs
- docs/developer-guides/extensions.md
- docs/implementation-artifacts/4-7-extension-templates-and-docs.md
- docs/implementation-artifacts/index.md
- docs/implementation-artifacts/sprint-status.yaml
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx
- frontend/src/pages/OptimizationTasksView/OptimizationTasksView.test.tsx
- frontend/src/pages/OptimizationTasksView/OptimizationTasksView.tsx
- frontend/src/types/generated/models/EvaluatorType.ts
- frontend/src/types/generated/models/ExecutionTargetType.ts

### Change Log

- 2026-01-14：完成 Story 4.7（扩展模板/示例/文档），新增 Example* 实现与端到端最小闭环单测；同步 TS types 与前端下拉/测试。

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [CRITICAL] 修复 Story 勾选与内容不一致：此前 `Review Follow-ups (AI)` 标记完成但 `Review Notes` 仍是 placeholder，容易造成“已审查/已完成”的误导。
- [x] [CRITICAL] 补齐 `ExampleExecutionTarget.execute_batch` 的显式单测：原先 Story 声称覆盖，但代码层缺少直接调用证据（仅覆盖 `execute`/最小闭环）。
- [x] [HIGH] 修复 `ExampleEvaluator` 的错误信息可能通过 `Debug` 输出泄露 `TaskReference`（可能包含 expected/core_request 等敏感/长文本）。
- [x] [MEDIUM] 补齐 AC5：在扩展指南中增加“可复制的计时复现命令序列”（以 `time` 为基准，避免引入新工具/依赖）。
- [ ] [MEDIUM]（非阻塞）运行时入口若仍直接使用 `DefaultEvaluator`，当 `evaluator_type=example` 时会拒绝执行；当前通过工厂/测试与文档证明“可选=可跑”的最小闭环，但后续如接入真实任务执行链路需确保统一使用 `create_evaluator_for_task_config`。

### Decisions

- [x] 选择“补齐证据链”而非删减 Story 目标：对文档/测试的“已完成声明”要求可核验，优先补测试与审查记录，避免后续维护成本。
- [x] `execute_batch` 继续复用 trait 默认实现（保持实现简单、避免重复逻辑），但增加显式单测锁定同序与对齐契约。
- [x] 对 `ExampleEvaluator` 的输入不支持分支仅输出“引用类型名称/枚举 tag”，不输出原文，优先满足脱敏约束。

### Risks / Tech Debt

- [x] 若未来出现“真实执行链路”直接实例化 `DefaultEvaluator` 且用户在配置中选择了 `example`，会出现运行时拒绝；触发条件：任务执行入口未统一走 `create_evaluator_for_task_config`。缓解：在接入执行入口时强制使用工厂并加回归测试。
- [x] AC5 的命令序列可能随项目结构/脚本演进而漂移；缓解：保持步骤短且指向稳定入口（`cargo test` / `gen-types` / `npm test`），必要时在变更时同步更新文档。

### Follow-ups

- [ ]（可选）当“任务执行 API/CLI”落地时：在真实入口处接入 `create_evaluator_for_task_config`，并添加一条集成测试覆盖 `evaluator_type=example` 的端到端运行。
