# 契约（Contracts）单一权威入口

本文件的目标：把“会反复对齐、且一旦漂移就会引发大范围返工/回归”的工程契约集中到一个可检索入口，作为 **单一事实来源（Single Source of Truth）**。

> 适用范围：后端 / 前端 / WS / 类型生成 / 测试门禁。  
> 原则：宁可“明确且少”，也不要“分散且多份”。

---

## 1) 权威入口（Source of Truth）

### 1.1 核心 Trait 契约（算法/执行/评估）

- 权威文件：`backend/src/core/traits.rs`
- 关键不变量：
  - `ExecutionTarget::execute_batch` 默认同序返回（并行由编排层负责调度，但**必须保序**）
  - `Evaluator::evaluate_batch` **不得过滤/重排**（必须与输入同序）

### 1.2 `OptimizationContext.extensions` 约定键（跨模块协作）

- 权威文件：`backend/src/domain/types/extensions.rs`
- 规则：
  - 禁止在 core 模块内新增“同义 key”
  - 新增 key 必须写进此文件并补齐回归测试（避免魔法字符串扩散）

### 1.3 WebSocket 消息外形（AR2 / 可视化前置）

- 权威文件：`backend/src/api/ws/events.rs`
- Schema（冻结，单一权威）：`docs/developer-guides/ws-message.schema.json`
- `WsMessage<T>` 形状（TypeScript 对齐 `docs/project-planning-artifacts/architecture.md`）：
  - `type: string`
  - `payload: T`
  - `timestamp: string`（RFC3339 / ISO 8601）
  - `correlationId?: string`（AR2，全链路追踪）

> 冻结说明：如需修改该 envelope（新增/改名字段），必须同步更新 schema、Rust 类型与前端消费方，并补齐跨端回归测试后再合入。

### 1.4 `correlationId` 全链路规则

- 约束来源：`docs/project-planning-artifacts/architecture.md#Communication Patterns`
- 后端入口生成/复用实现参考：`backend/src/api/middleware/correlation_id.rs`

### 1.5 Story / 文档 DoD（证据链门禁）

- 权威脚本：`scripts/verify_story_dod.py`
- CI 门禁：`.github/workflows/ci.yml` 的 `Docs DoD Gate`

### 1.6 状态/阶段口径（后端权威）

- 权威类型：`backend/src/domain/models/algorithm.rs` 的 `IterationState`
- 权威映射（阶段/展示口径）：`backend/src/domain/models/iteration_stage.rs`
- 权威 API：`GET /api/v1/meta/iteration-stages`（用于前端展示映射；前端不得自行推断阶段语义）

---

## 2) 修改契约的流程（必须）

当你需要改动上述任何契约（新增字段、改名、改变语义）：

1. **先改权威入口**（traits / extensions / ws message / docs）
2. **补齐回归测试**（Rust 单测 / 前端 Vitest / E2E，按影响面选择）
3. **如涉及跨端类型**：运行 `cd backend && cargo run --bin gen-types` 并提交生成产物
4. **跑门禁**：至少 `python3 scripts/verify_story_dod.py` + 相关测试集合

---

## 3) 反模式（禁止）

- 在多个地方复制同一契约（“这里也写一份、那里也写一份”）
- 新增魔法字符串 key（尤其是 `ctx.extensions[...]`）
- 破坏同序契约（返回过滤/重排/并发不保序）
- 在错误/日志/WS payload/UI 中回显敏感原文（prompt/testcase input/token）
