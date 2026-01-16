# Story 5.2: 边动画与数据流可视化

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 5-2-edge-animation-and-data-flow-visualization

## Key Decisions (MVP)

- **不引入新依赖**：优先复用 React Flow 12（`@xyflow/react`）内置的 edge `animated` 能力 + 轻量样式覆盖实现“数据流动”效果。
- **确定性驱动**：在未接入真实 WebSocket 事件前，边动画由现有本地确定性 demo 消息驱动（不出网、可复现、可回归）。
- **并行不混乱**：允许多条边同时动画，但必须遵循“并行降噪规则（最终规范）”（见 Dev Notes / Implementation Notes）。
- **可访问性与减弱动画**：尊重 `prefers-reduced-motion`；在减弱动画时降级为“高亮脉冲/短暂加粗”，仍能表达数据流向但不持续滚动。
- **性能优先**：动画实现不得引入高频 re-render；只更新必要的 edge props（`animated/style/className`），并复用 `PerfNfr3View` 做回归口径。

## Story

As a Prompt 优化用户,
I want 看到边动画展示数据在节点间的流动,
so that 我可以理解优化过程中数据是如何传递的。

## Acceptance Criteria

1. **Given** 数据从一个节点传递到下一个节点  
   **When** 传递发生  
   **Then** 对应的边必须显示动画效果（例如：流动虚线 / 粒子流 / 高亮脉冲，三者择一即可）  
   **And** 动画方向必须与数据流向一致（与箭头方向一致）。  
2. **Given** 多个数据同时在不同边上流动  
   **When** 并行传递发生  
   **Then** 各条边的动画必须独立运行（互不取消/互不串台）  
   **And** 不会造成视觉混乱（需要定义并实现明确的“并行时的视觉降噪规则”）。  
3. **Given** 数据传递完成  
   **When** 目标节点开始处理或进入稳定状态  
   **Then** 边动画必须平滑过渡到静止状态（不允许突然闪烁或直接消失）。  
4. **Given** 当前阶段仍处于 Epic 5 的“纯本地、确定性 demo 数据源”模式  
   **When** 用户在 `RunView` 点击 `回放/模拟运行`  
   **Then** 必须以确定性的方式触发边动画（同样的 demo 序列每次回放效果一致）  
   **And** 不依赖外部服务（不出网）。  
5. **Given** 用户启用了系统级“减弱动画”（`prefers-reduced-motion: reduce`）  
   **When** 发生数据传递  
   **Then** 必须自动降级动画表现（例如：短暂高亮/加粗/闪烁一次）  
   **And** 仍能表达“哪条边在传递/方向是什么”，但不持续滚动播放。  
6. **Given** 发生边动画（含并行）  
   **When** 执行 NFR3 回归测量（`/__perf__/nfr3`）  
   **Then** 必须保持同一口径下的性能可回归对比（不要求 CI 门禁阈值固定，但必须可测且不出现明显退化）。  

## Tasks / Subtasks

- [x] 定义“边 ID + 动画状态”的单一数据模型（AC: 1,2,3,4,5）
  - [x] 复用现有边 ID（`pattern->prompt` / `prompt->quality` / `quality->reflection`）并冻结为常量：从 `frontend/src/components/nodes/IterationGraph.tsx` 提取到 `frontend/src/components/nodes/types.ts`（建议导出 `iterationGraphEdgeIds` / `IterationGraphEdgeId`），全项目只引用该常量
  - [x] 定义 `EdgeFlowState`（例如：`idle | flowing | cooldown`）或 `ActiveEdgeFlows`（带过期时间），确保可表达“并行 + 平滑结束”
  - [x] 实现“并行降噪规则（最终规范）”（见 Dev Notes / Implementation Notes），并为其补齐单测覆盖
- [x] 在 `IterationGraph` 支持 edge 动画渲染（AC: 1,2,3,5）
  - [x] 基于 `@xyflow/react` edge `animated`（内置 dash 动画）实现基础流动效果
  - [x] 为“高亮脉冲/结束过渡”增加最小 CSS（不引入新库；尊重 reduced motion）
  - [x] 保持 edge 更新为局部变更：避免每帧重建 nodes/edges 数组
- [x] 在 demo reducer 中产生“边动画驱动信号”（AC: 1,2,3,4,5）
  - [x] 在不新增 WS 契约前提下：用现有 `DemoWsMessage` 的 `progress/state` 推导边流动（确定性映射）
  - [x] 确保并行边动画互不干扰（例如：同一条消息可同时点亮多条边，但有明确时长/结束逻辑）
  - [x] 保持 Story 5.1 的节点状态逻辑不回退（需要回归测试）
- [x] `RunView` 集成边动画（AC: 4,5）
  - [x] `回放/模拟运行` 同时驱动节点状态与边动画（同序、同一来源、可复现）
  - [x] DEV-only 入口保持不出网、不可随机
- [x] 更新 NFR3 回归入口覆盖“边动画场景”（AC: 6）
  - [x] `PerfNfr3View` 在测量窗口内触发边动画（复用同一 demo 序列/映射）
  - [x] 保持现有结果展示与 `data-testid` 约定
- [x] 测试与回归（AC: 1-6）
  - [x] 单测：`progress/state -> edge flows` 的映射规则必须覆盖并行与结束过渡
  - [x] 组件测试：验证边动画相关 props/类名在回放过程中出现并最终结束（允许扩展 `@xyflow/react` 的 test mock 以可观测 edges）
  - [x] 回归：`frontend` 跑 `npm test` + `npm run build`

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review] 确认 edge IDs 已抽到 `frontend/src/components/nodes/types.ts` 并全量替换引用（`IterationGraph` / demo reducer / tests）
- [x] [AI-Review] 按“Demo 映射（权威）”补齐 `progress/state -> edge flows` 的纯函数与单测（含并行与结束过渡）
- [x] [AI-Review] 按“并行降噪规则（最终规范）”实现并行弱化（opacity/线宽），并补组件测试断言（需要扩展 `@xyflow/react` test mock）
- [x] [AI-Review] 更新 `PerfNfr3View`：在测量窗口内触发边动画（复用同一 demo 序列/映射），确保口径可回归

## Dev Notes

### Developer Context (Read This First)

- **现状基线（已完成）**：Story 5.1 已引入 React Flow 12 并实现 `IterationGraph`（4 节点 + 3 条边）与确定性回放入口：  
  - 图组件：`frontend/src/components/nodes/IterationGraph.tsx`  
  - demo 消息源：`frontend/src/features/ws-demo/demoWsMessages.ts`  
  - demo reducer：`frontend/src/features/visualization/iterationGraphDemoReducer.ts`  
  - 入口页面：`frontend/src/pages/RunView/RunView.tsx`  
  - NFR3 口径页：`frontend/src/pages/PerfNfr3View.tsx`
- **未来接入点提示（真实 WS 承接 Epic 5.3/6）**：
  - 保持“消息 →（纯函数）推导 → `nodeStates/edgeFlowStates` → `IterationGraph` 渲染”的链路不变；仅将 `RunView`/`PerfNfr3View` 中的消息源从 `createDeterministicDemoWsMessages(...)` 替换为真实 WS 订阅流。
  - 契约单一权威入口：`docs/developer-guides/ws-message.schema.json` 与 `backend/src/api/ws/events.rs`（本 Story 不修改 envelope）。
- **Epic 5 全景（便于对齐）**：5.1 节点图基础渲染（done）→ **5.2 边动画数据流动（本 Story）** → 5.3 流式思考过程（NFR2/AR2）→ 5.4 环节标识
- **本 Story 的“数据流动”定义（MVP）**：先把“边动画 = 数据在阶段间传递的可视化提示”做出来；不要求与后端真实语义 100% 精确对齐（真实对齐由 Epic 5.3/6 的 WS 事件与阶段映射承接），但必须确定性、可回归、并能表达方向与并行。
- **禁止事项**：
  - 禁止引入随机动画/随机时序（会破坏回归可复现）。
  - 禁止新增或修改 WebSocket envelope 契约（`WsMessage<T>` 结构冻结）；本 Story 只在前端本地推导或本地扩展 demo 消息（不影响后端契约）。
  - 禁止让动画实现依赖高频 setState（避免 NFR3 退化）。
  - 禁止把“边流动”塞回 demo 消息字段（例如 `flowingEdges?: ...`）：本 Story 要求从现有 `progress/state` 推导，避免双来源漂移。

### Implementation Notes (Suggested Shape)

- **Edge 标识与来源**：
  - Edge IDs 当前已在 `IterationGraph.tsx` 中定义：`pattern->prompt` / `prompt->quality` / `quality->reflection`。请将其抽到 `frontend/src/components/nodes/types.ts`（与 `iterationGraphNodeIds` 同风格），建议形态：
    - `export const iterationGraphEdgeIds = ['pattern->prompt', 'prompt->quality', 'quality->reflection'] as const`
    - `export type IterationGraphEdgeId = (typeof iterationGraphEdgeIds)[number]`
  - Node IDs 冻结：`pattern_extractor` / `prompt_engineer` / `quality_assessor` / `reflection_agent`（来自 Story 5.1）。
- **状态管理位置（强烈建议）**：
  - `nodeStates` 与 `edgeFlowStates` 同层管理（`RunView`/`PerfNfr3View`），由同一个纯函数 reducer/adapter 从消息流推导；`IterationGraph` 只负责“根据 props 渲染边/节点”，避免在图组件内散落定时器与业务推导逻辑。
- **边动画数据模型（两种可选，择一即可）**：
  - 选项 A（三态 + 定时过渡，最直观）：
    - `type EdgeFlowState = 'idle' | 'flowing' | 'cooldown'`
    - `type EdgeFlowStates = Record<IterationGraphEdgeId, EdgeFlowState>`
  - 选项 B（带过期时间，避免 per-edge 定时器；由消息驱动刷新过期）：
    - `type ActiveEdgeFlow = { edgeId: IterationGraphEdgeId; expiresAtMs: number }`
    - reducer 只维护“当前活跃边及其过期时间”，渲染层基于 `now`（低频 tick，例如 100ms）或“消息到达时”推进过期与 `cooldown`
- **动画表达建议（不引入新库）**：
  - 基础：对需要流动的边设置 `animated: true`（React Flow 内置 dash 动画，CSS 在 `@xyflow/react/dist/style.css` 已包含）。
  - 强调：为“正在流动”的边同时设置 `style.stroke`（例如更亮的蓝/紫）或附加 className（便于加粗/发光）。
  - 结束过渡：将“流动”结束时先进入 `cooldown`（短暂保留高亮但 `animated: false`），再回到 `idle`（静止），以满足 AC#3 的“平滑过渡”。
- **动画时序参数建议（用于可复现回归）**：
  - `progress` 触发流动：200–400ms
  - `stream` 触发流动：持续刷新（例如“最后一条 stream 后再保持 150–250ms”），以形成连续流动观感
  - `cooldown`：200–400ms
  - reduced motion 高亮脉冲：150–250ms（单次）
- **Demo 映射（`progress/state` → edge flows，权威规范）**：
  - `payload.kind === 'progress' && payload.state === 'running_tests'`：触发 `pattern->prompt` 流动（短时）
  - `payload.kind === 'progress' && payload.state === 'evaluating'`：触发 `prompt->quality` 流动（短时）；并可同时触发 `pattern->prompt`（用于并行必现与降噪回归）
  - `payload.kind === 'stream'`：触发 `quality->reflection` 流动（可连续刷新以覆盖整段 streaming）
  - `payload.kind === 'progress' && (payload.state === 'waiting_user' || payload.state === 'human_intervention')`：结束所有流动边，进入 `cooldown` 后回到 `idle`
  - `payload.kind === 'progress' && (payload.state === 'completed' || payload.state === 'failed')`：结束所有流动边，进入 `cooldown` 后回到 `idle`
- **并行降噪规则（最终规范，必须实现）**：
  - 同一时刻最多“强高亮” 2 条边（Top2，按触发时间或优先级排序皆可，但必须确定性）。
  - 第 1–2 条：`opacity: 1`，`strokeWidth: 2`，可 `animated: true`
  - 第 3+ 条：`opacity: 0.5`，`strokeWidth: 1`，建议 `animated: false`（仅弱化提示，避免视觉噪音与性能抖动）
- **Reduced Motion**：
  - 当 `prefers-reduced-motion: reduce` 时：禁用 `animated`，但保留短暂高亮（例如 200–400ms）以表达流向。

### Testing Standards Summary

- **单测优先**：把 `progress/state -> edge flows` 的映射写成纯函数并覆盖用例（含并行、结束、reduced motion）。
- **组件测试**：当前 `@xyflow/react` 在 `frontend/src/test/setup.ts` 被 mock（只渲染 node）。本 Story 需要可观测 edge 状态时，允许扩展 mock：渲染 edges 为 `data-edgeid` 元素并暴露 `animated`/className 以便断言。
  - 最小扩展示例（示意）：
    - 在 `frontend/src/test/setup.ts` 的 `ReactFlowMockProps` 增加 `edges?: Array<{ id: string; animated?: boolean; className?: string }>`
    - 在 mock render 中追加：
      - `const edgeEls = (props.edges ?? []).map((e) => React.createElement('div', { key: e.id, 'data-edgeid': e.id, 'data-animated': String(!!e.animated), className: e.className }))`
      - 并把 `edgeEls` 与 `nodeEls` 一起渲染
- **回归命令**：
  - `cd frontend && npm test`
  - `cd frontend && npm run build`

### Project Structure Notes

- 新增/调整代码优先放在既有位置：`frontend/src/components/nodes/`（图渲染相关）与 `frontend/src/features/visualization/`（demo adapter/reducer 纯函数）。
- 若需要新增样式：优先局部 className + Tailwind；只有在确有必要时才新增小型 CSS（并确保不影响全局）。

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 5 / Story 5.2）  
- UX 动画与减弱动画要求：`docs/project-planning-artifacts/ux-design-specification.md`（Design Principles / Accessibility: prefers-reduced-motion）  
- 技术与目录结构基线：`docs/project-planning-artifacts/architecture.md`（React Flow 12 / 前端结构 / NFR3）  
- 本地确定性 demo 数据源约束：`docs/developer-guides/epic-5-demo-data-source.md`  
- NFR3 口径说明：`docs/developer-guides/performance-protocol-nfr2-nfr3.md`

## Git Intelligence Summary

- 最近相关提交：`caf11d6`（Story 5.1 引入/集成 React Flow 12，建立确定性回放入口，并更新 NFR3 口径页）
- 现有 `IterationGraph` 边为 `smoothstep` + `markerEnd` 箭头，但未设置 `animated`：`frontend/src/components/nodes/IterationGraph.tsx`

## Latest Tech Information (Web/Version Snapshot)

- `@xyflow/react`：项目当前锁定 `12.10.0`（来源：`frontend/package.json`）
- React Flow 内置 edge 动画：edge `animated: true` 会启用内置 dash 动画；具体样式定义见 `@xyflow/react/dist/style.css`

## Project Context Reference

- `**/project-context.md`：未发现；以 `docs/project-planning-artifacts/*.md`、`docs/developer-guides/*` 与现有代码为准。

## Story Completion Status

- Status set to `done`
- Completion note: 2026-01-15 完成实现与回归验证并完成 code-review 修复（tests/build/lint 均通过），详见 `docs/implementation-artifacts/validation-report-5-2-dev-2026-01-15_22-46-18.md`

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

- docs/implementation-artifacts/validation-report-5-2-dev-2026-01-15_22-46-18.md

### Completion Notes List

- ✅ create-story：从 `docs/implementation-artifacts/sprint-status.yaml` 自动发现首个 backlog story（Epic 5 / Story 5.2）
- ✅ 生成 ready-for-dev story 文件并提供实现 guardrails、测试与回归口径
- ✅ 实现 edge flow 状态机：`flowing → cooldown → idle`，满足“平滑结束”与并行互不干扰
- ✅ `IterationGraph` 支持边动画渲染与 reduced motion 降级（pulse），并实现并行降噪（Top2 强高亮）
- ✅ `RunView` / `PerfNfr3View` 集成同一确定性 demo 序列驱动边动画，保持 NFR3 口径可回归
- ✅ 补齐单测与组件测试（含 `@xyflow/react` test mock 扩展），并完成 `vitest run` + `build` + `lint` 回归
- ✅ code-review 修复：reduced motion pulse 未生效、flowing 时 `lastActivatedSeq` 未更新、以及 edge setState 无效更新等问题；回归仍通过

### File List

- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/5-2-edge-animation-and-data-flow-visualization.md
- docs/implementation-artifacts/validation-report-5-2-dev-2026-01-15_22-46-18.md
- frontend/src/components/nodes/IterationGraph.test.tsx
- frontend/src/components/nodes/IterationGraph.tsx
- frontend/src/components/nodes/IterationGraphEdges.css
- frontend/src/components/nodes/edgeDenoise.test.ts
- frontend/src/components/nodes/edgeDenoise.ts
- frontend/src/components/nodes/types.ts
- frontend/src/features/visualization/iterationGraphDemoReducer.test.ts
- frontend/src/features/visualization/iterationGraphDemoReducer.ts
- frontend/src/features/visualization/iterationGraphEdgeFlowMachine.test.ts
- frontend/src/features/visualization/iterationGraphEdgeFlowMachine.ts
- frontend/src/hooks/index.ts
- frontend/src/hooks/usePrefersReducedMotion.ts
- frontend/src/pages/PerfNfr3View.tsx
- frontend/src/pages/RunView/RunView.tsx
- frontend/src/test/setup.ts

### Change Log

- 2026-01-15：为 IterationGraph 增加边动画与数据流可视化（含并行降噪与 reduced motion 降级）；RunView/NFR3 集成同一确定性 demo 驱动；补齐测试与 mock 扩展，并通过 test/build/lint 回归。

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] **[HIGH]** edge IDs “冻结为常量”的落点此前不明确，容易导致魔法字符串扩散与重复定义 → 已在本 Story 明确抽到 `frontend/src/components/nodes/types.ts` 并全量复用（见 Implementation Notes）。
- [x] **[HIGH]** “并行降噪规则”此前偏抽象，可能导致实现不一致 → 已补齐“最终规范”（Top2 强高亮，3+ 弱化参数）。
- [x] **[MEDIUM]** `progress/state -> edge flows` 之前缺少权威映射表，难以保证并行必现与回归可复现 → 已补齐 “Demo 映射（权威规范）”与建议时序参数。
- [x] **[MEDIUM]** 组件测试目前无法观测 edge 状态（mock 只渲染 node） → 已补最小 mock 扩展示例，降低实现与测试摩擦。
- [x] **[HIGH]** reduced motion 下 pulse 动画被全局禁用（导致 AC5 失真） → 已收窄 reduced-motion CSS 覆盖范围，禁用 React Flow dash 动画但保留 pulse：`frontend/src/components/nodes/IterationGraphEdges.css`。
- [x] **[HIGH]** edge 已处于 flowing 时未更新 `lastActivatedSeq`（影响 Top2 降噪的“最近触发”语义） → 已在状态机中确保 seq 更新并补断言：`frontend/src/features/visualization/iterationGraphEdgeFlowMachine.ts`、`frontend/src/features/visualization/iterationGraphEdgeFlowMachine.test.ts`。
- [x] **[MEDIUM]** `IterationGraph` 在 edge props 无变化时仍会 setEdges 产生新数组（潜在噪声 re-render） → 已在 `setEdges` 中无变化返回 `prev`：`frontend/src/components/nodes/IterationGraph.tsx`。

### Decisions

- [x] **保持 demo 字段最小化**：不新增 `flowingEdges?: ...` 到 demo 消息；坚持从 `progress/state` 推导边流动，避免“双来源/双写漂移”。
- [x] **推导逻辑上移**：`nodeStates` 与 `edgeFlowStates` 同层（`RunView`/`PerfNfr3View`）由纯函数推导，`IterationGraph` 只做渲染与最小 props→edgeProps 映射，降低性能风险。

### Risks / Tech Debt

- [ ] **定时器/过期机制复杂度**：若选用“三态+定时过渡”，需确保组件卸载/重复回放时清理；若选用“过期时间”，需引入低频 tick 或在消息到达时推进过期（需避免高频 setState）。
- [ ] **并行排序确定性**：Top2 的挑选规则必须确定性（例如按触发时间/固定优先级），否则回归会抖动。

### Follow-ups

- [x] 按本 Story 的“权威映射/降噪规则/测试 mock 示例”完成实现与回归，并在 code-review 中修复关键问题后完成收尾。
