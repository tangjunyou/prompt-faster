# Epic 5 构建体积告警记录

- 项目：Prompt Faster
- Epic：5（可视化与实时反馈）
- 日期：2026-01-16
- 记录人：Winston（Architect）

## 现状

- 构建产物出现 chunk > 500KB 告警（来源：Story 5.4 Review Notes）。

## 初步判断（原因假设）

1. 可视化与流式展示相关组件集中在单一入口页，代码分割不足。
2. React Flow 相关依赖与 demo 数据驱动逻辑集中加载，导致初始包体偏大。
3. 复用组件与 feature 的边界尚未形成可按路由/场景拆分的粒度。

## 候选拆分策略（后续评估）

1. **按路由拆分**：将 RunView 相关可视化模块拆到独立 chunk（懒加载）。
2. **按功能拆分**：将 StreamingText/Stage 组件与 IterationGraph 组件拆分为独立动态模块。
3. **按环境拆分**：demo 数据驱动与性能回归页仅在 DEV/性能路由加载。

## 后续行动（可执行任务）

- 在 Epic 6 或后续 Story 中新增拆分任务：
  - 明确目标阈值（例如单 chunk ≤ 500KB）。
  - 明确验证方式（构建输出截图/日志）。
  - 明确影响范围（RunView、PerfNfr2View、PerfNfr3View）。
  - 跟踪文件：`docs/implementation-artifacts/epic-5-retro-action-items-2026-01-16.md`

## 证据与关联

- 关联复盘：`docs/implementation-artifacts/epic-5-retro-2026-01-16.md`
- 关联 Story：`docs/implementation-artifacts/5-4-thinking-panel-stage-indicator.md`
