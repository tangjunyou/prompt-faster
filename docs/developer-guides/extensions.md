# 扩展点开发指南（ExecutionTarget / Evaluator / TeacherModel）

本项目的目标是“扩展可复制”：你应该能够 **复制模板 → 改名 → 实现接口 → 在单一注册点注册 → 配置启用 → 跑通确定性测试**，而不需要修改核心业务代码。

> 约束继承：
> - **不引入新依赖**（新增 crate/npm 包请单独开 Story/PR）。
> - **示例实现必须确定性、可测试、默认不出网**（避免 flaky）。
> - **安全与脱敏硬约束**：日志/错误/结果/文档示例不得包含 API Key / Prompt 原文 / TestCase input 全文。

---

## 1. 三类扩展点概览与职责边界

### 1.1 `ExecutionTarget`（执行目标）

定义：`backend/src/core/traits.rs` 的 `ExecutionTarget` trait。

职责：
- 给定 `prompt + input + test_case_id` 执行并返回 `ExecutionResult`。
- **同序契约**：`execute_batch` 必须与输入 batch **同序返回**，且 `ExecutionResult.test_case_id` 必须与输入测试用例 id 对齐。

边界（不该做的事）：
- 不在这里做评估/排序/反思；这些属于 Evaluator/Aggregator/Optimizer。
- 不把选择逻辑散落到业务代码：新增实现应只改一个入口点（工厂/注册表）。

当前入口点：
- 工厂：`backend/src/core/execution_target/mod.rs::create_execution_target`
- 类型枚举：`backend/src/domain/models/optimization_task.rs::ExecutionTargetType`

示例实现：
- `backend/src/core/execution_target/example_impl.rs::ExampleExecutionTarget`（确定性、不出网）

---

### 1.2 `Evaluator`（评估器）

定义：`backend/src/core/traits.rs` 的 `Evaluator` trait。

职责：
- 给定 `OptimizationContext + TestCase + output` 产出 `EvaluationResult`。
- `evaluate_batch` 必须 **同序返回**，不得在 Evaluator 内过滤/重排（映射由编排层保证）。

边界：
- TeacherModel 的调用/解析/超时护栏由评估路径负责（见 DefaultEvaluator）。
- 新增评估器实现不应改动业务调用点：应通过统一工厂选择实现。

当前入口点：
- 工厂：`backend/src/core/evaluator/mod.rs::create_evaluator_for_task_config`
- 任务配置枚举：`backend/src/domain/models/optimization_task_config.rs::EvaluatorType`

示例实现：
- `backend/src/core/evaluator/example_impl.rs::ExampleEvaluator`（确定性、不出网）

---

### 1.3 `TeacherModel`（老师模型）

定义：`backend/src/core/traits.rs` 的 `TeacherModel` trait（`generate` / `generate_stream`）。

职责：
- 作为能力提供者，被 `DefaultEvaluator` 在 `EvaluatorType::TeacherModel` 路径注入并调用。

重要澄清（避免误解）：
- `TeacherModel`（trait，可注入 DefaultEvaluator） **≠** 任务配置里的 `teacher_llm.model_id`（仅“选择模型 ID”，不等于新增 TeacherModel 实现）。
- PRD 中提到的 `generate_structured` 与当前代码签名可能不一致：以 `backend/src/core/traits.rs::TeacherModel` 的 `generate/generate_stream` 为准；结构化 JSON 解析与护栏由 Evaluator 路径负责（见 `backend/src/core/evaluator/default_impl.rs`）。

当前入口点：
- 工厂：`backend/src/core/teacher_model/mod.rs::create_teacher_model`

示例实现：
- `backend/src/core/teacher_model/example_impl.rs::ExampleTeacherModel`（确定性、不出网）

---

## 2. 最小改动清单（复制模板到跑通）

> 目标：新增实现时“选择逻辑只改一个入口点（工厂/注册表）”，避免散落 match。

### 2.1 新增 `ExecutionTarget`

最小修改文件：
- 新增实现文件：`backend/src/core/execution_target/<your_impl>.rs`
- 注册入口：`backend/src/core/execution_target/mod.rs`（只在 `create_execution_target` 增加分支）
- 类型枚举：`backend/src/domain/models/optimization_task.rs::ExecutionTargetType`（增加枚举值）
- API 解析（若前端可选）：`backend/src/api/routes/optimization_tasks.rs` + `backend/src/infra/db/repositories/optimization_task_repo.rs`

回归清单（最少）：
- Rust 单测：同序契约 + 不泄露敏感信息（见 ExampleExecutionTarget 单测）
- TS types：运行 `cd backend && cargo run --bin gen-types`
- 前端：下拉选项可选择、payload 回填正确（含 Vitest）

### 2.2 新增 `Evaluator`

最小修改文件：
- 新增实现文件：`backend/src/core/evaluator/<your_impl>.rs`
- 注册入口：`backend/src/core/evaluator/mod.rs::create_evaluator_for_task_config`
- 类型枚举：`backend/src/domain/models/optimization_task_config.rs::EvaluatorType`

回归清单（最少）：
- Rust 单测：`evaluate_batch` 同序契约 + 逐用例可追溯（建议写入 `extra.test_case_id`）
- TS types + 前端下拉（若暴露给 UI）：同上

### 2.3 新增 `TeacherModel`

最小修改文件：
- 新增实现文件：`backend/src/core/teacher_model/<your_impl>.rs`
- 注册入口：`backend/src/core/teacher_model/mod.rs::create_teacher_model`

回归清单（最少）：
- Rust 单测：确保可被 `DefaultEvaluator` 的 TeacherModel 路径注入使用，并验证：
  - timeout 仍生效（预算上限）
  - JSON 解析与 fenced-json 提取仍生效
  - 错误/日志不回显 prompt 原文（示例实现不应 echo prompt）

---

## 3. 反模式清单（常见坑）

- **新增依赖**：把新 crate/npm 包塞进同一个 Story → 回归风险上升（请单独开 Story/PR）。
- **选择逻辑散落**：在多个模块/业务代码里加 match → 未来扩展必然失控（统一工厂/注册表）。
- **破坏同序契约**：`execute_batch/evaluate_batch` 过滤/重排/并发不保序 → Layer 3/4 错位回归。
- **泄露敏感信息**：错误/日志/结果里输出 prompt/input 全文、API key、原始响应全文。
- **忘记同步 TS 类型**：Rust enum 增值但前端 union 未更新 → “后端支持但前端不可选/类型不匹配”。

---

## 4. TS 类型生成与验证（强制）

生成命令：

```bash
cd backend
cargo run --bin gen-types
```

检查要点：
- `frontend/src/types/generated/models/` 内对应枚举文件是否更新（如 `ExecutionTargetType.ts`、`EvaluatorType.ts`）
- 相关页面下拉是否使用生成类型而非手写 union（避免漂移）

---

## 5. NFR 计时口径（用于验证扩展耗时）

计时范围（包含）：
- 从复制官方模板开始，到本地跑通文档示例用例结束
- 编码 + 本地测试/验证时间

计时范围（不包含）：
- 依赖下载、CI 排队、网络波动导致的等待

建议复现步骤：
1. 复制示例实现（Example*）到新文件并改名
2. 仅修改实现与单一注册点（工厂/注册表）
3. 运行 Rust 单测 + `gen-types` + 前端 Vitest

### 5.1 可复制的计时复现命令序列（AC5）

> 说明：以下命令只使用系统自带的 `time`，不引入新工具；计时范围仅包含“编码 + 本地验证”。

```bash
# 1) 后端：运行单测（建议先跑目标模块相关用例，再全量）
cd backend
time cargo test

# 2) 生成 TS types（Rust enum / API shape 变更后必须做）
time cargo run --bin gen-types

# 3) 前端：跑 Vitest（验证下拉可选 + payload 回填）
cd ../frontend
time npm test -- --run
```

如果你要严格复现“复制 → 改名 → 注册 → 验证”的全过程，建议把“复制/改名/注册”作为计时起点，
并在完成上述 3 组命令且全部通过后停止计时（不要把依赖下载/CI 等等待时间算进去）。
