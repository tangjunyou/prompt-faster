# Story 5.1: 节点图基础渲染

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 5-1-node-graph-basic-rendering

## Key Decisions (MVP)

- **先打通“可渲染 + 可更新状态”**：本 Story 只实现 4 个核心节点（四层架构）与状态颜色变更，数据源先以本地/模拟输入驱动，后续 Story 再接入真实 WebSocket 流（Epic 5.3/6）。
- **高性能优先**：优先选择 React Flow（架构/PRD/UX 一致）并在实现中避免不必要的 re-render；NFR3 的 FPS 回归入口复用现有 `PerfNfr3View`。
- **扩展友好**：PRD 提到的“更新节点”等更细粒度节点，本 Story 不强行一次性铺开；但组件/类型设计需支持后续扩展为 5+ 节点而不推翻结构。

## Story

As a Prompt 优化用户,
I want 通过节点图形式查看迭代流程，并看到节点状态颜色变化,
so that 我可以直观了解优化进度和各层的执行状态。

## Acceptance Criteria

1. **Given** 用户进入 `Run View`（可视化面板）  
   **When** 页面加载完成  
   **Then** 必须渲染一个节点图画布，包含四层架构节点：
   - Pattern Extractor
   - Prompt Engineer
   - Quality Assessor
   - Reflection Agent  
   **And** 节点之间存在清晰的连接线表示数据流向（至少覆盖主链路 Pattern → Prompt → Quality → Reflection）。  
2. **Given** 节点图已渲染  
   **When** 某个节点状态变化（由输入数据驱动：真实运行数据或本地模拟数据）  
   **Then** 节点颜色必须按以下规则即时更新（颜色值/样式可由 UI 实现决定，但语义不可变）：
   - 灰色：未开始 / 待执行
   - 蓝色：执行中（可选：轻量脉冲效果）
   - 绿色：成功完成
   - 红色：失败
   - 黄色：需要用户介入 / 已暂停  
3. **Given** 节点图处于持续更新状态（例如：状态变更、缩放/拖拽）  
   **When** 进行 NFR3 性能回归测量  
   **Then** 必须提供一个可重复运行的测量入口（复用 `PerfNfr3View` 或等价页面），用于验证“节点图渲染 + 基础交互”在合理节点规模下可达 60fps（NFR3）。  
4. **Given** 本 Story 尚未接入真实 WebSocket 流（Epic 5.3/6 承接）  
   **When** 开发/测试需要驱动节点状态变化  
   **Then** 必须提供确定性的本地驱动方式：复用 `frontend/src/features/ws-demo/demoWsMessages.ts` 的确定性消息序列，并在 `DEV` 下提供“回放/模拟运行”触发入口，以保证单测与手工验收可复现（不出网）。

## Tasks / Subtasks

- [x] 引入 React Flow 12 并建立基础样式（AC: 1）
  - [x] 前端依赖新增 `@xyflow/react@12.x`（建议锁到 12.10.0 或同系列 patch）
  - [x] 在全局入口 `frontend/src/main.tsx` 引入基础样式 `@xyflow/react/dist/style.css`（确保节点/边基础样式正常）
- [x] 定义“节点图模型 + 状态语义”并保持可扩展（AC: 1,2,4）
  - [x] 定义 4 个节点的稳定 ID（用于后续 WS/状态更新对齐）：`pattern_extractor`/`prompt_engineer`/`quality_assessor`/`reflection_agent`
  - [x] 明确 `IterationGraph` 的输入契约：以“节点级状态”作为唯一输入（例如 `Record<NodeId, NodeStatus>`），本 Story 不要求从 `IterationState` 推断节点状态
  - [x] 定义节点状态枚举与颜色语义（灰/蓝/绿/红/黄），并用纯函数实现 `status -> className/style` 映射
  - [x] 预留扩展位：未来可追加 PRD 的“更新节点”等，不需要推翻现有类型/渲染结构
- [x] 实现节点图组件（React Flow）与最小交互（AC: 1,2）
  - [x] 新增 `IterationGraph`（或等价命名）组件：接收 “节点状态输入” 并渲染 4 节点 + 边
  - [x] 交互最小集：画布可缩放/可拖拽；节点样式清晰；无需实现编辑弹窗/聚焦模式（留给后续 Story）
- [x] 在 `RunView` 集成节点图并提供确定性本地驱动（AC: 1,2,4）
  - [x] `RunView` 布局：为后续右侧思考面板预留区域（可以先放占位卡片）
  - [x] 本地驱动（必须确定性、不出网）：复用 `frontend/src/features/ws-demo/demoWsMessages.ts` 的 `createDeterministicDemoWsMessages(...)` 作为事件源；在 `frontend/src/features/visualization/`（或等价目录）实现纯函数 adapter/reducer，将事件流归一为“4 节点状态输入”；`DEV` 下只提供“回放/模拟运行”触发入口（仅触发回放，不允许另起一套不可复现的随机序列）
- [x] 更新 NFR3 性能回归入口为“真实节点图场景”（AC: 3）
  - [x] 将 `PerfNfr3View` 的“盒子网格”替换为：渲染 `IterationGraph` + 触发一段确定性的状态/交互更新（用于测 FPS）
  - [x] 保持现有 `data-testid` 约定与结果口径输出
- [x] 测试与回归（AC: 2,4）
  - [x] 单测：状态映射纯函数（颜色语义）必须覆盖全量状态枚举
  - [x] 组件测试：验证 4 节点渲染与状态变化可驱动（允许 mock React Flow 的布局依赖）
  - [x] 最小构建验证：`frontend` 的 `vitest` 与 `vite build` 通过

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][CRITICAL] 修复 `PerfNfr3View` 停止/卸载/重复启动的生命周期问题（`frontend/src/pages/PerfNfr3View.tsx`）
- [x] [AI-Review][MEDIUM] 补齐 demo reducer 的最小单测覆盖（`frontend/src/features/visualization/iterationGraphDemoReducer.test.ts`）
- [x] [AI-Review][MEDIUM] 扩展确定性 demo 消息与 reducer 映射，覆盖 `paused`/`error` 语义（`frontend/src/features/ws-demo/demoWsMessages.ts`、`frontend/src/features/visualization/iterationGraphDemoReducer.ts`）
- [x] [AI-Review][HIGH] 修正文档中的 placeholder/过时内容，使 Story 与实现一致（本文件）

## Dev Notes

### Developer Context (现状扫描)

- `RunView` 已集成节点图与确定性回放入口：`frontend/src/pages/RunView/RunView.tsx`
- `PerfNfr3View` 已作为真实节点图场景的回归入口：`frontend/src/pages/PerfNfr3View.tsx`
- 已引入 React Flow 12（`@xyflow/react`）并在全局入口注入基础样式：`frontend/package.json`、`frontend/src/main.tsx`
- 确定性 WS 消息生成器可复用作本 Story 的“本地驱动”输入：`frontend/src/features/ws-demo/demoWsMessages.ts`
- 后端已定义权威的迭代状态分组/标签映射（后续 Thinking Panel / Stage Indicator 可复用）：`backend/src/domain/models/iteration_stage.rs`

### Dev Agent Guardrails（避免常见踩坑）

- **不要引入第二套 UI 框架**（MUI/AntD/Chakra 等）；本项目 UI 基线是 Tailwind + shadcn/ui（见现有 `frontend/src/components/ui`）。
- **不要在前端“脑补”状态含义**：如果需要“阶段口径/阶段文案/分组”，必须以后端权威 API `GET /api/v1/meta/iteration-stages` 为唯一来源（单一事实入口见 `docs/developer-guides/contracts.md`），禁止自建推断/硬编码文案导致漂移。
- **不要一次性实现 Epic 5.2/5.3/5.4/6 的功能**：本 Story 只交付“节点图基础渲染 + 状态颜色变更 + 可复现性能测量入口”。

### Technical Requirements（必须满足）

- 节点状态语义：灰/蓝/绿/红/黄（见 AC#2；与 PRD 7.3.3 的“待执行/执行中/成功/失败/已暂停”一致）
- 节点图至少包含四层节点与主链路边（见 AC#1）
- 需要“可复现”的本地驱动方式（见 AC#4），用于单测与验收（推荐：确定性消息序列 + 纯函数 reducer）

### Architecture Compliance（必须遵守）

- 路由入口：`/run` 对应 `RunView`（见 `frontend/src/App.tsx`）
- 组件分层：Page 负责布局与组合；节点图与节点样式应落在可复用组件目录（建议：`frontend/src/components/nodes/`），避免与 `RunView` 布局强耦合（后续 Focus/Workspace 复用）
- 性能验证：复用 `PerfNfr3View` 作为 NFR3 口径入口，并替换其“纯盒子”场景为“真实节点图渲染 + 基础交互”

### Library / Framework Requirements（版本与用法）

- React：`react@19.2.0` / `react-dom@19.2.0`（已存在）
- 节点图库：使用 `@xyflow/react`（React Flow 12.x）。截至 2026-01-15，`npm view @xyflow/react version` 返回 `12.10.0`（建议锁定到 12.x，避免跨大版本破坏）。  
  - 必须引入其基础样式（按官方文档）：`@xyflow/react/dist/style.css`
- 样式：继续使用 Tailwind（不要在节点图里塞入大量 inline style，避免难以统一主题）

### File Structure Requirements（落点约束）

- 新增 React Flow 相关组件：`frontend/src/components/nodes/`（例如：`IterationGraph.tsx`、`nodeStyles.ts`、`types.ts`）
- `RunView` 只负责布局与组合：`frontend/src/pages/RunView/RunView.tsx`
- NFR3 入口更新：`frontend/src/pages/PerfNfr3View.tsx`
- 如需本地驱动状态：复用 `frontend/src/features/ws-demo/` 的确定性消息序列；adapter/reducer 建议落在 `frontend/src/features/visualization/`（避免把 demo/转换逻辑散落在 Page 里）

### Testing Requirements（必须补齐）

- 单元测试（Vitest + RTL）：至少覆盖
  - “输入状态 → 节点颜色语义”的映射纯函数（强制确定性）
  - `RunView` 能渲染节点图容器与 4 个节点的最小集成验证（允许对 React Flow 做最小 mock，以避免 jsdom/布局限制）
- 回归入口：确保 `PerfNfr3View` 仍可运行且输出 fps 指标（可使用 `data-testid="nfr3-run"`/`nfr3-result"` 现有约定）

### Project Structure Notes

- 架构文档给出的“理想目录树”与当前 `frontend/src/` 实际结构存在偏差（例如：当前以 `pages/` + `features/*/services` 为主）。以现有仓库结构为准，在不破坏现有约定的前提下落点即可。
- `IterationStageDescriptor` 的权威映射已提供后端 API：`GET /api/v1/meta/iteration-stages`（单一事实入口见 `docs/developer-guides/contracts.md`）。前端如需展示“阶段文案/分组”，必须以该 API 为唯一来源，禁止自建推断/硬编码文案，避免双写漂移。

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md#Story 5.1: 节点图基础渲染`
- PRD 可视化需求与状态语义：`docs/project-planning-artifacts/prd.md#7.3.3 可视化需求`
- UX 节点图模式与设计系统选型：`docs/project-planning-artifacts/ux-design-specification.md#Transferable UX Patterns`、`#Design System Choice`
- 架构（技术栈/边界/目录约束）：`docs/project-planning-artifacts/architecture.md#Frontend Architecture`、`#Project Structure & Boundaries`
- 现有代码入口：`frontend/src/pages/RunView/RunView.tsx`、`frontend/src/pages/PerfNfr3View.tsx`、`frontend/src/features/ws-demo/demoWsMessages.ts`
- 契约单一权威入口：`docs/developer-guides/contracts.md`
- Epic 5 本地确定性 demo 数据源：`docs/developer-guides/epic-5-demo-data-source.md`
- 性能口径（NFR2/NFR3）：`docs/developer-guides/performance-protocol-nfr2-nfr3.md`

## Git Intelligence Summary

- 已引入 React Flow 12：`frontend/package.json`（`@xyflow/react@12.10.0`）与 `frontend/src/main.tsx`（全局样式注入）
- `RunView` 已集成 `IterationGraph` 并提供 DEV 下确定性回放入口：`frontend/src/pages/RunView/RunView.tsx`
- `PerfNfr3View` 已替换为“真实节点图 + 确定性状态更新”的测量口径入口，并修复停止/卸载等生命周期问题：`frontend/src/pages/PerfNfr3View.tsx`
- demo WS 消息源为本地确定性序列（覆盖成功/暂停/失败）：`frontend/src/features/ws-demo/demoWsMessages.ts`

## Latest Tech Information (Web/Version Snapshot)

- React Flow（React 生态节点图）：使用 `@xyflow/react`（React Flow 12）。截至 **2026-01-15**：
  - `npm view @xyflow/react version` = `12.10.0`
  - 样式导入路径（按官方文档）：`@xyflow/react/dist/style.css`
- 兼容性注意：项目当前 React 为 `19.2.0`；请按 React Flow 12 的官方示例/文档用法接入，并在 CI 中跑 `vitest` 与 `vite build` 作为最小兼容验证。

## Project Context Reference

- `**/project-context.md`：未发现（本 Story 以 `docs/project-planning-artifacts/*.md` 与现有代码为准）

## Story Completion Status

- Status set to `done`
- Completion note: 4 节点图 + 状态语义映射 + 确定性回放入口 + NFR3 口径页面已实现，并通过 `vitest`/`vite build`/`cargo test` 回归验证

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

N/A

### Completion Notes List

- ✅ 引入 `@xyflow/react@12.10.0` 并在 `frontend/src/main.tsx` 注入基础样式
- ✅ 新增 `IterationGraph`：4 节点 + 主链路边，支持缩放/拖拽；状态 → 颜色语义映射为纯函数
- ✅ `RunView` 集成节点图 + DEV 环境“回放/模拟运行”入口（复用确定性 demo WS 消息 + 纯函数 reducer）
- ✅ `PerfNfr3View` 替换为真实节点图场景，并在测量窗口内进行确定性状态更新
- ✅ 测试与回归：`frontend` 运行 `vitest --run`、`vite build` 通过；`backend` 运行 `cargo test` 通过
- ✅ Code Review 修复已应用，并补齐 `## Review Notes` 与 `Review Follow-ups (AI)`

### File List

- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/5-1-node-graph-basic-rendering.md
- frontend/package.json
- frontend/package-lock.json
- frontend/src/main.tsx
- frontend/src/components/nodes/IterationGraph.tsx
- frontend/src/components/nodes/nodeStyles.ts
- frontend/src/components/nodes/types.ts
- frontend/src/components/nodes/IterationGraph.test.tsx
- frontend/src/components/nodes/nodeStyles.test.ts
- frontend/src/features/visualization/iterationGraphDemoReducer.ts
- frontend/src/features/visualization/iterationGraphDemoReducer.test.ts
- frontend/src/features/ws-demo/demoWsMessages.ts
- frontend/src/pages/PerfNfr3View.tsx
- frontend/src/pages/RunView/RunView.tsx
- frontend/src/pages/RunView/RunView.test.tsx
- frontend/src/test/setup.ts

## Change Log

- 2026-01-15：实现 Story 5.1（节点图基础渲染 + 状态颜色更新 + 可复现本地回放 + NFR3 回归入口），并补齐测试/构建验证
- 2026-01-15：Code Review 修复：补齐 `PerfNfr3View` 的停止/卸载稳定性、扩展 demo 状态覆盖并补充 reducer 单测、清理本 Story 的占位/过时段落
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] **[CRITICAL]** `PerfNfr3View` 的停止按钮会导致一次测量永远悬挂（Promise 不会结束），且卸载/快速重复启动存在风险 → 已修复（加入 abort + 卸载清理 + 防重复启动）。
- [x] **[HIGH]** 本 Story 的 review 内容为 placeholder（`Review Follow-ups` 被勾选但 `Review Notes` 未填写），且存在“开发前扫描”段落未更新 → 已补齐并修正。
- [x] **[MEDIUM]** demo reducer 缺少单测，且确定性回放对 `paused/error` 语义覆盖不足 → 已补齐（消息序列 + reducer 映射 + 单测）。

### Decisions

- [x] `PerfNfr3View` 用 `AbortController` 统一处理 stop/卸载/中断，避免 Promise 悬挂与卸载后 setState。
- [x] demo 状态覆盖以“确定性消息序列 + 纯函数 reducer”为唯一入口，不引入随机性与额外网络依赖。

### Risks / Tech Debt

- [x] NFR3 的“合理节点规模”口径目前仍以 4 节点为基线示例；若后续 Story 引入更多节点/边与动画，需要在 `PerfNfr3View` 中扩展为更贴近真实规模的压力场景。

### Follow-ups

- [x] 本 Story 当前无剩余阻塞项；后续如扩展节点规模/边动画（Story 5.2），需同步更新 NFR3 口径场景与阈值解释。
