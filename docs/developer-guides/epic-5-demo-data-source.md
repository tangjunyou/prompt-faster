# Epic 5 可视化 Demo 数据源（纯本地、确定性、不出网）

目标：在未接入真实后端 WebSocket 的情况下，前端可用 **纯本地、确定性** 的事件序列完成节点图/动画/Thinking Panel 的开发与回归验证。

## 使用方式（Frontend）

- Demo 生成器：`frontend/src/features/ws-demo/demoWsMessages.ts`
- 单测覆盖（保证不串台/不乱序）：`frontend/src/features/ws-demo/demoWsMessages.test.ts`

示例（伪代码）：

- `createDeterministicDemoWsMessages({ correlationId, iterations, tokensPerIteration })` 返回 `WsMessage[]`
- 每条消息都带 `correlationId`，并且 `payload.seq` 严格递增（用于验证“保序/不乱序”）

## 契约对齐

- WS envelope schema（冻结单一权威）：`docs/developer-guides/ws-message.schema.json`
- Rust 权威类型：`backend/src/api/ws/events.rs`

## 约束（必须遵守）

- 不出网：不得依赖任何外部 API / WebSocket 服务
- 确定性：同输入参数必须产出同样的消息序列（用于 CI 回归）
- 不串台：不同 `correlationId` 的消息流必须严格隔离（用于后续并发可视化）

