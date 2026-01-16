# Story 5.4: 思考面板环节标识

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 5-4-thinking-panel-stage-indicator

## Key Decisions (MVP)

- **延续 Story 5.3 架构**：在现有 `ThinkingStreamState` 基础上扩展 `currentStage` 字段，复用 `StreamingText` 组件，不引入新依赖。
- **纯本地、确定性、不出网**：沿用 Epic 5 的 demo 消息源，扩展 `iteration:progress` 事件以携带 `stage` 信息。
- **阶段语义权威**：环节语义与展示映射以 `backend/src/domain/models/iteration_stage.rs` / `GET /api/v1/meta/iteration-stages` 为权威；本 Story demo 映射仅作临时占位并需显式标注。
- **不串台（AR2）**：环节标识必须以 `correlationId` 为隔离边界；不同任务/不同 run 的环节状态不得混写。
- **范围控制**：只实现"环节标识展示（FR39）"；**不实现**"完整历史查看交互（Story 6.4 / FR43）"与真实 WS 接入。
- **历史区域折叠/摘要**：以 accordion/折叠形式保留各环节的输出片段（只读展示），完整历史由 Epic 6 承接。

## Story

As a Prompt 优化用户,  
I want 在思考面板中看到当前执行环节的标识,  
so that 我可以清楚知道模型正在进行哪个阶段的思考。

## Acceptance Criteria

1. **Given** 用户正在查看思考面板  
   **When** 老师模型执行不同环节  
   **Then** 面板顶部或侧边显示当前环节标识（如"规律抽取中"/"候选生成中"/"质量评估中"/"反思迭代中"）  
   **And** 标识清晰可辨、与 UX 规范一致（颜色/图标可选）。

2. **Given** 环节切换  
   **When** 从一个环节进入下一个  
   **Then** 环节标识平滑更新（无闪烁/跳变）  
   **And** 之前环节的输出保留在历史区域。

3. **Given** 环节切换后  
   **When** 用户查看历史区域  
   **Then** 历史区域以折叠/摘要形式保留各环节的输出片段（只读展示）  
   **And** 完整历史查看交互由 Epic 6 FR43 承接（本 Story 仅需只读折叠展示）。

4. **Given** 用户启用系统级"减弱动画"（`prefers-reduced-motion: reduce`）  
   **When** 环节标识切换  
   **Then** 禁用非必要动画（如渐变过渡），但标识仍应即时更新并可读。

5. **Given** 多任务/多 run 场景（未来）  
   **When** 不同 `correlationId` 的消息同时到达  
   **Then** 各自的环节标识独立，不串台（AR2）。

## Tasks / Subtasks

- [x] 扩展 `ThinkingStreamState` 以支持环节标识（AC: 1,2,5）
  - [x] 新增 `currentStage: StageType | null` 字段（`StageType = 'pattern' | 'prompt' | 'quality' | 'reflection' | null`）
  - [x] 新增 `stageHistory: StageHistoryItem[]` 字段，记录各环节的输出片段
  - [x] 在 reducer 中根据 `iteration:progress.stage` 更新 `currentStage` 并归档前一环节到 `stageHistory`
  - [x] 按 `correlationId` 隔离环节状态

- [x] 扩展 demo 消息源以携带 `stage` 信息（AC: 1,2）
  - [x] 修改 `createDeterministicDemoWsMessages` 在 `iteration:progress` 中添加 `stage` 字段
  - [x] 更新 `DemoProgressPayload` 增加 `stage?: StageType`
  - [x] 确保 demo 序列覆盖四层架构的环节切换流程

- [x] 实现 `StageIndicator` 组件（AC: 1,4）
  - [x] Props：`stage: StageType | null`、`prefersReducedMotion?: boolean`
  - [x] 展示当前环节中文标识（规律抽取中/候选生成中/质量评估中/反思迭代中）
  - [x] 可选：添加对应颜色/图标（与节点图颜色一致）
  - [x] `prefers-reduced-motion` 下禁用过渡动画

- [x] 实现 `StageHistoryPanel` 组件（AC: 2,3）
  - [x] Props：`history: StageHistoryItem[]`、`prefersReducedMotion?: boolean`
  - [x] 以 accordion/折叠形式展示各环节摘要（只读）
  - [x] 每个环节项显示：环节名称、输出片段预览（截断）、展开/折叠按钮
  - [x] 可访问性：`aria-expanded`、键盘可操作

- [x] RunView 集成环节标识（AC: 1,2,3,4,5）
  - [x] 在思考面板顶部添加 `StageIndicator`
  - [x] 在流式文本上方或下方添加 `StageHistoryPanel`
  - [x] 确保 demo 回放时环节标识正确切换

- [x] 测试与回归（AC: 1-5）
  - [x] 单测：`ThinkingStreamState reducer` 的 `currentStage` 和 `stageHistory` 逻辑
  - [x] 组件测试：`StageIndicator` 渲染、过渡动画禁用
  - [x] 组件测试：`StageHistoryPanel` 折叠/展开、`aria-expanded`
  - [x] 页面测试：`RunView` 回放后环节标识正确切换
  - [x] 回归命令：`cd frontend && npm test`；`cd frontend && npm run build`

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [ ] [AI-Review][LOW] 记录 build 产物 chunk > 500KB 的告警与后续拆分建议（仅技术债，不阻塞）

## Dev Notes

### Developer Context (Read This First)

- **现状基线（已完成）**：
  - RunView 已集成节点图（Story 5.1）、边动画（Story 5.2）、流式思考面板（Story 5.3）
  - `frontend/src/pages/RunView/RunView.tsx`
  - `frontend/src/features/visualization/thinkingStreamReducer.ts`（需扩展）
  - `frontend/src/components/streaming/StreamingText.tsx`（可复用）
  - demo 消息源：`frontend/src/features/ws-demo/demoWsMessages.ts`（需扩展）

- **Epic 5 全景（便于对齐业务价值与范围）**：
  - 5.1 节点图基础渲染（done）
  - 5.2 边动画/数据流动（done）
  - 5.3 流式思考过程（done，NFR2/AR2）
  - **5.4 环节标识（本 Story，FR39）**

- **业务价值（为什么做）**：让用户不仅看到"AI 在想什么"，还能知道"AI 在做什么阶段的思考"，进一步降低等待焦虑，并为后续 Epic 6 的用户介入提供更精确的上下文（来源：Epic 5/UX 设计目标）。

- **Epic 5 demo 约束（必须遵守）**：纯本地、确定性、不出网；不同 `correlationId` 的消息流必须隔离（未来并发可视化）
  - `docs/developer-guides/epic-5-demo-data-source.md`

- **WS envelope 契约冻结**：`WsMessage<T>` 的 shape 不得改动；如需扩展只能扩展 `payload`
  - `docs/developer-guides/ws-message.schema.json`、`backend/src/api/ws/events.rs`、`docs/developer-guides/contracts.md`

### Implementation Notes (Suggested Shape)

- **阶段语义对齐**：demo 阶段可使用本地映射（`pattern/prompt/quality/reflection`），但需明确该映射为临时占位；真实 WS 接入时必须对齐后端权威口径。
- **数据模型建议**：
  ```typescript
  type StageType = 'pattern' | 'prompt' | 'quality' | 'reflection'
  
  type StageHistoryItem = {
    stage: StageType
    summary: string  // 截断后的输出摘要
    startSeq: number
    endSeq: number
  }
  
  // 扩展 ThinkingStreamState
  type ThinkingStreamState = {
    // ... 现有字段
    currentStage: StageType | null
    stageHistory: StageHistoryItem[]
  }
  ```

- **Reducer 逻辑建议**：
  - 收到 `iteration:progress` 且 `payload.stage` 与 `currentStage` 不同时：
    1. 将当前 `text` 截断为 summary，归档到 `stageHistory`
    2. 更新 `currentStage` 为新 stage
    3. 清空 `text`，新环节的输出从空文本开始（旧环节已归档）

- **环节中文映射**：
  ```typescript
  const STAGE_LABELS: Record<StageType, string> = {
    pattern: '规律抽取中',
    prompt: '候选生成中',
    quality: '质量评估中',
    reflection: '反思迭代中',
  }
  ```

- **折叠/摘要策略**：
  - 摘要截取前 100 字符 + "..."（`text.length <= 100` 则原样展示）
  - 摘要前先将换行规整为空格，避免折叠预览破碎
  - 仅保留最近 `20` 个 `stageHistory` 项（避免内存无限增长）
  - 展开时显示完整片段（受 `maxChars/maxLines` 限制）

### Dev Agent Guardrails（避免常见踩坑）

- 不引入新 UI/动画库（如 Framer Motion）；本项目基线是 Tailwind + 现有 hooks。
- 不要在本 Story 接入真实 WebSocket；保持 demo 驱动与可复现回归优先。
- 不要忽略 `correlationId`：环节标识必须按 correlationId 隔离。
- 不做"完整历史查看交互"（Epic 6 FR43 承接）；本 Story 只需只读折叠展示。
- 不要破坏 Story 5.3 已有的流式文本逻辑；环节标识是增量功能。
- 若未来接入真实 WS，需要按 `contracts.md` 完整流程更新 schema/Rust/TS 并补齐回归；本 Story 仅允许 demo 扩展。

### Technical Requirements（必须满足）

- `StageIndicator` 必须可读、无障碍（`aria-label` 或等效）。
- `StageHistoryPanel` 必须支持键盘操作（Enter/Space 展开/折叠）。
- `prefers-reduced-motion` 下禁用非必要动画，但标识仍即时更新。
- 环节切换不得引起流式文本闪烁或丢失。

### Architecture Compliance（必须遵守）

- WS 事件命名遵循 `{domain}:{action}`（例如 `iteration:progress`），WS envelope 形状与 `correlationId` 规则遵循架构文档：
  - `docs/project-planning-artifacts/architecture.md#Communication Patterns`
- 命名约定：TypeScript camelCase；跨端字段靠 `serde(rename_all = "camelCase")` 对齐。
- 阶段语义与展示映射以 `iteration_stage.rs` / `GET /api/v1/meta/iteration-stages` 为权威；前端不得自行推断。

### Library / Framework Requirements (Version Snapshot)

- React：项目依赖 `react@^19.2.0`；截至 **2026-01-16** 最新为 `19.2.3`（无需升级）。
- React Router：项目依赖 `react-router@^7.12.0`；截至 **2026-01-16** 最新为 `7.12.0`。
- React Flow：项目锁定 `@xyflow/react@12.10.0`（本 Story 不改图相关依赖）。

### File Structure Requirements（落点约束）

- 扩展 reducer：`frontend/src/features/visualization/thinkingStreamReducer.ts`
- StageIndicator 组件：`frontend/src/components/streaming/StageIndicator.tsx`
- StageHistoryPanel 组件：`frontend/src/components/streaming/StageHistoryPanel.tsx`
- RunView 集成：`frontend/src/pages/RunView/RunView.tsx`
- Demo 消息源扩展：`frontend/src/features/ws-demo/demoWsMessages.ts`
- 本 Story 仅修改 demo 数据源；若 stage 进入真实 WS，必须按 `contracts.md` 更新 schema/Rust/TS

### Testing Requirements（必须补齐）

- 单测：reducer 必须覆盖 `currentStage` 切换、`stageHistory` 归档、`correlationId` 隔离。
- 组件测试：`StageIndicator` 渲染、`StageHistoryPanel` 折叠/展开、`aria-expanded`。
- 回归：`vitest` + `vite build` 必须通过。
- 回归：Story 5.3 相关测试必须继续全部通过。

### Project Structure Notes

- 沿用 Story 5.3 的组件目录结构（`components/streaming/`）。
- 以 `docs/developer-guides/contracts.md` 为 WS 契约单一权威。

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 5 / Story 5.4）
- PRD 可视化与思考面板：`docs/project-planning-artifacts/prd.md#7.3.3 可视化需求`
- UX 规范：`docs/project-planning-artifacts/ux-design-specification.md`
- 架构（WS events/correlationId）：`docs/project-planning-artifacts/architecture.md#Communication Patterns`
- Demo 数据源约束：`docs/developer-guides/epic-5-demo-data-source.md`
- WS 契约单一权威：`docs/developer-guides/contracts.md`
- 前序 Story learnings：`docs/implementation-artifacts/5-3-streaming-thinking-process-display.md`

## Git Intelligence Summary

- 最近可视化相关提交：
  - `1ca62ed`（Story 5.3 done）
  - `df7563b`（Story 5.3 fixes merge）
  - `43ecde5`（Story 5.2：边动画 + 测试）
- 现有 demo 消息已包含 `iteration:progress`，需扩展以携带 `stage` 字段。
- 现有 `ThinkingStreamState` 和 `StreamingText` 组件可直接复用/扩展。

## Latest Tech Information (Web/Registry Snapshot)

- `npm view react version`（2026-01-16）：`19.2.3`
- `npm view react-router version`（2026-01-16）：`7.12.0`
- 关键关注点（本 Story 足够）：不升级依赖；按现有版本做"环节标识 + 折叠历史"即可满足功能需求。

## Project Context Reference

- `**/project-context.md`：未发现；以 `docs/project-planning-artifacts/*.md`、`docs/developer-guides/*` 与现有代码为准。

## Story Completion Status

- Status set to `done`
- Completion note: Review 修复完成，补齐终结环节归档、键盘交互与集成测试验证。

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

- npm test（通过）
- npm run build（通过）

### Completion Notes List

- ✅ 新增环节状态与历史归档，切换时重置流式文本并保留摘要。
- ✅ demo 消息扩展 stage 字段，覆盖四阶段切换流程。
- ✅ 新增环节标识与历史面板组件，并在 RunView 集成展示。
- ✅ 测试通过：npm test、npm run build。

### File List

- frontend/src/features/visualization/thinkingStages.ts
- frontend/src/features/visualization/thinkingStreamReducer.ts
- frontend/src/features/visualization/thinkingStreamReducer.test.ts
- frontend/src/features/ws-demo/demoWsMessages.ts
- frontend/src/components/streaming/StageIndicator.tsx
- frontend/src/components/streaming/StageIndicator.test.tsx
- frontend/src/components/streaming/StageHistoryPanel.tsx
- frontend/src/components/streaming/StageHistoryPanel.test.tsx
- frontend/src/components/streaming/index.ts
- frontend/src/pages/RunView/RunView.tsx
- frontend/src/pages/RunView/RunView.test.tsx

### Change Log

- 2026-01-16：新增思考面板环节标识与历史折叠展示，扩展 demo 数据与 reducer 逻辑，并补齐测试覆盖。

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] 终结环节未归档导致历史缺失，已在 reducer 终结分支补归档。
- [x] [HIGH] StageHistoryPanel 缺少键盘操作，已补 Enter/Space 展开收起与 aria-label。
- [x] [MEDIUM] RunView 页面测试未验证环节切换，已补 StageIndicator 断言并用 act 包裹定时器。
- [x] [MEDIUM] demo 阶段映射未显式标注临时占位，已添加注释说明权威来源。

### Decisions

- [x] 保持 demo-only 映射不引入真实 API，改为显式注释标明权威语义来源，避免超范围改动。
- [x] 终结环节归档在 reducer 内完成，避免在 UI 层补丁式处理。

### Risks / Tech Debt

- [ ] build 产物 chunk > 500KB 的告警仍存在，建议后续做代码拆分（非本 Story 阻塞项）。

### Follow-ups

- [ ] [LOW] 记录 chunk 体积告警并评估拆分策略（与 Review Follow-ups (AI) 同步）。
