# Story 3.3 Code Review（Reviewed）

Story Key: `3-3-candidate-generation-and-diversity-injection-config`

Review Date: 2026-01-06

## Outcome

- 结论：Changes Applied（已按审查落实修复与回归测试补齐）
- 目标：在不过度扩展范围的前提下，补齐真实存在的风险点（前端输入绕过、后端解析失败导致潜在数据丢失），并保持与 Story AC/Tasks 一致。

## What Was Fixed (Evidence-Based)

### 1) 前端：防止 NaN/小数绕过本地校验（必须）

- 修复点：对数值字段做 `Finite + Integer + Range` 校验；不通过则不发送请求。
- 位置：`frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`
- 回归：新增本地校验单测（小数、越界不发请求）。
  - `frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx`

### 2) 后端：避免“existing config_json 解析失败 → 回退默认 → 覆盖写入”导致潜在数据丢失（必须）

- 修复点：更新配置（保留 `extra`）时，对 existing `config_json` 改为严格解析；若解析失败则拒绝更新并返回 `VALIDATION_ERROR`，避免覆盖写入。
- 位置：`backend/src/domain/models/optimization_task_config.rs`
- 回归：新增测试确保解析失败时返回 400 且不覆盖 DB 中原值。
  - `backend/tests/optimization_tasks_api_test.rs`

### 3) 文案：页面说明覆盖新增字段（应做）

- 位置：`frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`

## Notes on Other Review Suggestions

- “后端 initial_prompt 标准化不一致”结论不成立：后端已调用 `.normalized()`，空字符串会归一化为 `null`，属于预期行为。
- “需要手动 invalidateQueries 避免旧数据”结论不成立：`useUpdateOptimizationTaskConfig` 已在 `onSuccess` 做了 invalidate。
- “缺少后端更新成功用例”结论不成立：已存在成功更新断言用例（含非边界值与边界值）。

