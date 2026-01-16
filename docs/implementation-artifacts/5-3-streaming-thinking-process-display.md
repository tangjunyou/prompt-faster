# Story 5.3: 流式思考过程展示

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 5-3-streaming-thinking-process-display

## Key Decisions (MVP)

- **不引入新依赖**：优先用 React/DOM 原生能力实现流式文本渲染、自动滚动与复制；长文本性能先用“批量 flush +（可选）窗口化/截断”策略兜底。
- **纯本地、确定性、不出网**：沿用 Epic 5 的 demo 消息源 `createDeterministicDemoWsMessages(...)` 驱动流式输出，保证回归可复现（不依赖真实后端 WebSocket）。
- **不串台（AR2）**：Thinking Panel 的流式内容必须以 `correlationId` 为隔离边界；不同任务/不同 run 的消息不得混写到同一面板。
- **性能口径对齐（NFR2）**：沿用 `PerfNfr2View` 的 `firstMessageLatencyMs` 口径；本 Story 重点确保“首条可见反馈”链路稳定且不被 UI 实现拖慢。
- **范围控制**：只实现“流式文本展示（FR38）”；**不实现**“节点图联动/环节标识（Story 5.4）”与真实 WS 接入（留到后续 Story）。

## Story

As a Prompt 优化用户,  
I want 实时查看老师模型的思考过程（流式输出）,  
so that 我可以理解模型的推理逻辑并在需要时介入。

## Acceptance Criteria

1. **Given** 老师模型正在生成响应  
   **When** 用户查看思考面板（Thinking Panel）  
   **Then** 以流式方式持续追加显示模型输出（逐字/逐行均可）  
   **And** 第一条可见反馈到达需可通过既有口径验证（NFR2：`firstMessageLatencyMs < 500ms`）。
2. **Given** 流式输出正在进行  
   **When** 有新内容到达  
   **Then** 平滑追加到已有内容后（不清空、不闪烁）  
   **And** 自动滚动到最新位置（用户手动上滚时允许暂停自动滚动，避免“抢滚动”）。
3. **Given** 后端（未来）使用 WebSocket 推送  
   **When** 建立连接并收到 `WsMessage<T>`  
   **Then** 必须以 `correlationId` 关联请求与响应（AR2）  
   **And** 多任务/多 run 场景下流式内容不得串台（以 `correlationId` 为隔离边界）。
4. **Given** 高 token 频率或长文本（真实场景）  
   **When** 持续流式追加  
   **Then** UI 不应因“每 token setState”导致明显卡顿（需批量 flush / 节流渲染）  
   **And** 提供长文本兜底策略（窗口化/截断/虚拟滚动其一即可），避免内存无限增长。
5. **Given** 用户启用系统级“减弱动画”（`prefers-reduced-motion: reduce`）  
   **When** Thinking Panel 展示流式输出  
   **Then** 仍应持续更新文本  
   **And** 禁用非必要动画（例如光标闪烁、平滑滚动），但不影响可读性与可复制性。

## Tasks / Subtasks

- [x] 定义 Thinking Panel 的单一状态模型（AC: 1,2,3,4,5）
  - [x] 设计 `ThinkingStreamState`（建议字段：`correlationId`、`text`、`status(streaming|complete|error)`、`lastSeq`、`isAutoScrollLocked`）
  - [x] 写成纯函数 reducer：`DemoWsMessage -> ThinkingStreamState`（仅处理 `thinking:stream` 与必要的 "start/end" 信号）
  - [x] **MUST**：按 `correlationId` 隔离；不匹配则丢弃/缓存到对应 run（择一，但必须不串台）
  - [x] **MUST**：使用 `payload.seq` 保序（遇到乱序/重复需幂等处理）
  - [x] 状态机（写死规则，避免实现漂移）：
    - [x] `idle -> streaming`：收到第一条 `thinking:stream`
    - [x] `streaming -> complete`：收到 terminal `iteration:progress`（`state in {completed, failed}`）；`RunView` 回放结束仅作为兜底
    - [x] `* -> error`：收到明确错误信号（未来真实 WS 接入时补齐；本 demo 若无 error 事件则保持 N/A）
  - [x] `seq` 规则（最小可用且可测）：忽略 `seq <= lastSeq`（重复/乱序）；若出现 `seq > lastSeq + 1` 视为"跳跃"，记录 warning（不中断追加）
- [x] 实现可复用 `StreamingText` 组件（AC: 1,2,4,5）
  - [x] Props 覆盖：`text`、`status`、`onCopy?`、`prefersReducedMotion?`、`maxChars?/maxLines?`（兜底策略）
    - [x] 优先级：若同时提供，先按 `maxLines` 再按 `maxChars` 截断（保留末尾）
    - [x] 默认值（MVP）：`maxLines=500`、`maxChars=10000`（可在组件内配置/覆盖）
  - [x] 自动滚动：默认跟随最新；用户手动滚动上移后暂时停用跟随（提供"回到底部"按钮或提示）
    - [x] 判定阈值：距离底部 `bottomThresholdPx=100` 以内视为"在底部"，否则锁定（`isAutoScrollLocked=true`）
  - [x] 可访问性：`aria-live="polite"`（屏幕阅读器友好），复制按钮具备可聚焦与可读 label
    - [x] 复制语义：始终复制"当前已展示文本"（被截断/窗口化时不尝试复制不可见历史）
  - [x] 性能：以 `requestAnimationFrame`/批量 flush 合并多条 chunk，避免每条 chunk 都触发重排/渲染
- [x] RunView 集成 Thinking Panel（AC: 1,2,3,4,5）
  - [x] 用 demo 消息源 `createDeterministicDemoWsMessages(...)` 驱动 `IterationGraph` + Thinking Panel（同一条消息流，同一 `correlationId`）
  - [x] 回放开始时重置 state；回放结束时（兜底）将 `status` 置为 `complete`（若 reducer 未因 terminal `iteration:progress` 置位）
  - [x] 保持现有节点图/边动画链路不回退（Story 5.1/5.2 回归口径保持）
- [x] NFR2 回归入口对齐（AC: 1）
  - [x] 保持 `PerfNfr2View` 的 `firstMessageLatencyMs` 口径不漂移（口径="收到第一条 WS 消息（任意类型）"；不要改成 DOM 首次渲染/ThinkingPanel 首次可见）
  - [x]（可选）在 `PerfNfr2View` 复用 `StreamingText` 展示首条可见反馈，避免"测量口径存在但 UI 不可见"
- [x] 测试与回归（AC: 1-5）
  - [x] 单测：`ThinkingStreamState reducer` 覆盖（保序/幂等/correlationId 隔离/长文本兜底）
  - [x] 组件测试：`StreamingText` 渲染、复制、`aria-live`、自动滚动锁定逻辑（可通过 mock scrollHeight/scrollTop 验证）
  - [x] 页面测试：`RunView` 回放后 Thinking Panel 内容确实追加（fake timers）
  - [x] 回归命令：`cd frontend && npm test`；`cd frontend && npm run build`

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。


## Dev Notes

### Developer Context (Read This First)

- **现状基线（已完成）**：RunView 已集成节点图（Story 5.1）与边动画（Story 5.2），右侧 Thinking Panel 目前为占位 `aside`：
  - `frontend/src/pages/RunView/RunView.tsx`
  - `frontend/src/components/nodes/IterationGraph.tsx`
  - demo 消息源：`frontend/src/features/ws-demo/demoWsMessages.ts`（已包含 `thinking:stream` 事件）
- **Epic 5 全景（便于对齐业务价值与范围）**：5.1 节点图基础渲染（done）→ 5.2 边动画/数据流动（done）→ **5.3 流式思考过程（本 Story，NFR2/AR2）** → 5.4 环节标识（backlog）
- **业务价值（为什么做）**：让用户“看见 AI 在想什么”（玻璃盒体验），降低等待焦虑，并为后续 Epic 6 的用户介入提供可观察的上下文（来源：Epic 5/UX 设计目标）。
- **Epic 5 demo 约束（必须遵守）**：纯本地、确定性、不出网；不同 `correlationId` 的消息流必须隔离（未来并发可视化）  
  - `docs/developer-guides/epic-5-demo-data-source.md`
- **WS envelope 契约冻结**：`WsMessage<T>` 的 shape 不得改动；如需扩展只能扩展 `payload`（并同步 schema/跨端回归，通常不在本 Story 做）  
  - `docs/developer-guides/ws-message.schema.json`、`backend/src/api/ws/events.rs`、`docs/developer-guides/contracts.md`
- **NFR2 口径入口**：`/__perf__/nfr2`（`frontend/src/pages/PerfNfr2View.tsx`）用于回归对比 `firstMessageLatencyMs`（首条 WS 消息到达）  
  - `docs/developer-guides/performance-protocol-nfr2-nfr3.md`

### Implementation Notes (Suggested Shape)

- **数据流建议（单一链路）**：
  - `DemoWsMessage[]`（确定性序列）→（纯函数 reducer/adapter）→ `nodeStates / edgeFlowStates / thinkingStreamState` → `RunView` 渲染
  - **禁止**在 `StreamingText` 内部消费原始消息流；组件只消费“已归一化的 text + status”，降低耦合与性能风险。
- **状态机建议（避免实现漂移）**：
  - `idle -> streaming`：第一条 `thinking:stream`
  - `streaming -> complete`：terminal `iteration:progress`（`completed/failed`）；`RunView` 回放结束只做兜底
  - `* -> error`：未来真实 WS 接入时补齐（本 demo 若无 error 事件则保持 N/A）
- **自动滚动建议**：
  - 维护 `isAutoScrollLocked`：用户滚动到非底部时置为 true；新 chunk 到达时不抢滚动，仅提示“有新内容”
  - 判定阈值：距底部 `bottomThresholdPx=100` 以内视为“在底部”，否则锁定；当用户点击“回到底部”或滚回阈值内时解锁并立即滚动到底部。
- **长文本兜底（至少一种）**：
  1) 纯截断（MVP 推荐）：默认 `maxLines=500`、`maxChars=10000`，只保留末尾并提示“较早内容已省略”；或  
  2) 虚拟滚动：仅渲染可见窗口（更复杂，若无必要不要引入新库）。
- **复制行为（减少实现分歧）**：
  - 复制按钮始终复制“当前已展示文本”（截断/窗口化时不尝试复制不可见历史）

### Dev Agent Guardrails（避免常见踩坑）

- 不引入新 UI/动画库（Framer Motion 等）作为“快实现”；本项目基线是 Tailwind + 现有 hooks。
- 不要在本 Story 接入真实 WebSocket；保持 demo 驱动与可复现回归优先。
- 不要“每 token setState”导致卡顿；必须批量 flush（rAF/节流）。
- 不要忽略 `correlationId`：未来多任务并行时最容易串台，必须现在就把隔离逻辑固化。
- 不要在 UI/日志/WS payload 中回显敏感原文（尤其是 API Key、凭证或用户隐私内容）；Thinking Panel 只展示“允许展示的模型输出/思考文本”。
- 不做“节点图联动/环节标识”（Story 5.4 承接）；本 Story 只保证文本流式追加展示与交互可用。

### Technical Requirements（必须满足）

- `StreamingText` 必须可复制、可滚动、默认自动滚动到底部（UX），并具备 `aria-live="polite"`（A11y）。  
- `prefers-reduced-motion` 下禁用非必要动画（例如光标闪烁、平滑滚动/平滑自动滚动），但文本仍持续更新（自动滚动使用“直接跳转到底部”）。  
- NFR2：需保留并引用 `PerfNfr2View` 作为回归入口；故事验收以该口径为准（< 500ms）。

### Architecture Compliance（必须遵守）

- WS 事件命名遵循 `{domain}:{action}`（例如 `thinking:stream`），WS envelope 形状与 `correlationId` 规则遵循架构文档：  
  - `docs/project-planning-artifacts/architecture.md#Communication Patterns`
- 命名约定：TypeScript camelCase；跨端字段靠 `serde(rename_all = "camelCase")` 对齐（不要引入 snake_case 泄漏）。  

### Library / Framework Requirements (Version Snapshot)

- React：项目依赖 `react@^19.2.0`；截至 **2026-01-16**（npm registry）最新为 `19.2.3`（无需升级，保持兼容即可）。
- React Router：项目依赖 `react-router@^7.12.0`；截至 **2026-01-16** 最新为 `7.12.0`。
- React Flow：项目锁定 `@xyflow/react@12.10.0`；截至 **2026-01-16** 最新仍为 `12.10.0`（本 Story 不改图相关依赖）。

### File Structure Requirements（落点约束）

- Thinking Panel UI：`frontend/src/pages/RunView/RunView.tsx`
- StreamingText 组件：建议 `frontend/src/components/streaming/StreamingText.tsx`（或与团队约定的共用组件目录）
- Thinking reducer/adapter：建议 `frontend/src/features/visualization/` 下新增（与 `iterationGraphDemoReducer.ts` 同风格）
- 不修改 WS contract 文件（`docs/developer-guides/ws-message.schema.json`、`backend/src/api/ws/events.rs`）除非后续 Story 专门处理契约升级

### Testing Requirements（必须补齐）

- 单测：reducer 必须覆盖 `correlationId` 隔离、`seq` 幂等、截断/窗口化策略。
- 组件测试：`aria-live`、复制按钮、自动滚动锁定逻辑（可用 jsdom mock scroll 属性）。
- 回归：`vitest` + `vite build` 必须通过。

### Project Structure Notes

- PRD 中的 WS message 示例为“概念型”，当前仓库的单一权威是 `WsMessage<T>` envelope（见 `docs/developer-guides/contracts.md`）；以仓库契约为准，避免双写漂移。

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 5 / Story 5.3）  
- PRD 可视化与思考面板：`docs/project-planning-artifacts/prd.md#7.3.3 可视化需求`、`#NFR2`  
- UX StreamingText/A11y：`docs/project-planning-artifacts/ux-design-specification.md#StreamingText（流式文本）`、`#Accessibility Strategy`  
- 架构（WS events/correlationId）：`docs/project-planning-artifacts/architecture.md#Communication Patterns`  
- Demo 数据源约束：`docs/developer-guides/epic-5-demo-data-source.md`  
- 性能口径（NFR2）：`docs/developer-guides/performance-protocol-nfr2-nfr3.md`  
- WS 契约单一权威：`docs/developer-guides/contracts.md`、`docs/developer-guides/ws-message.schema.json`、`backend/src/api/ws/events.rs`
- 前序 Story learnings：`docs/implementation-artifacts/5-1-node-graph-basic-rendering.md`、`docs/implementation-artifacts/5-2-edge-animation-and-data-flow-visualization.md`

## Git Intelligence Summary

- 最近可视化相关提交：`caf11d6`（Story 5.1：引入 React Flow 与确定性回放入口）、`43ecde5`（Story 5.2：边动画 + 降噪 + reduced motion + 测试与 mock 扩展）。
- 现有 demo 消息已包含 `thinking:stream`（可直接用于本 Story 的 Thinking Panel 驱动）：`frontend/src/features/ws-demo/demoWsMessages.ts`。

## Latest Tech Information (Web/Registry Snapshot)

- `npm view react version`（2026-01-16）：`19.2.3`
- `npm view react-router version`（2026-01-16）：`7.12.0`
- `npm view @xyflow/react version`（2026-01-16）：`12.10.0`
- 关键关注点（本 Story 足够）：不升级依赖；按现有版本做“批量 flush（rAF）+ 自动滚动锁定 + aria-live”即可满足性能与可用性需求（无需引入额外库或进行跨大版本迁移）。

## Project Context Reference

- `**/project-context.md`：未发现；以 `docs/project-planning-artifacts/*.md`、`docs/developer-guides/*` 与现有代码为准。

## Story Completion Status

- Status set to `done`
- Completion note: 实现与回归已完成，CI 全绿并合入 main（2026-01-16）

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

- docs/implementation-artifacts/validation-report-5-3-2026-01-16_09-44-42.md

### Completion Notes List

- ✅ create-story：从 `docs/implementation-artifacts/sprint-status.yaml` 自动发现首个 backlog story（Epic 5 / Story 5.3）
- ✅ 产出 ready-for-dev story 文件：包含 guardrails、落点约束、测试与回归口径、以及 NFR2/AR2 对齐说明
- ✅ dev-story (2026-01-16)：完成所有 Tasks/Subtasks 实现
  - 创建 `ThinkingStreamState` reducer（correlationId 隔离、seq 保序/幂等、长文本截断）
  - 实现 `StreamingText` 组件（自动滚动、复制、aria-live、prefersReducedMotion 支持）
  - RunView 集成 Thinking Panel（demo 消息源驱动、回放重置/兜底完成）
  - PerfNfr2View 复用 StreamingText 展示首条可见反馈
  - 单测 19 个 + 组件测试 21 个全部通过
  - `npm test` (299 passed) + `npm run build` 全部通过

### File List

- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/5-3-streaming-thinking-process-display.md
- docs/implementation-artifacts/validation-report-5-3-2026-01-16_09-44-42.md
- frontend/src/features/visualization/thinkingStreamReducer.ts (新增)
- frontend/src/features/visualization/thinkingStreamReducer.test.ts (新增)
- frontend/src/components/streaming/StreamingText.tsx (新增)
- frontend/src/components/streaming/StreamingText.test.tsx (新增)
- frontend/src/components/streaming/index.ts (新增)
- frontend/src/pages/RunView/RunView.tsx (修改)
- frontend/src/pages/PerfNfr2View.tsx (修改)

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] 未发现新增问题；实现与测试覆盖均已完整落地。

### Decisions

- [x] 本 Story 无新增关键取舍；按既有方案执行并验证通过。

### Risks / Tech Debt

- [x] 无新增风险或遗留，已由既有测试与回归覆盖。

### Follow-ups

- [x] 无新增后续行动项。
