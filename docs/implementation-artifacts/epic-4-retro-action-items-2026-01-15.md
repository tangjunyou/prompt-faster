# Epic 4 行动项跟踪（Action Items Tracker）

- 来源：`docs/implementation-artifacts/epic-4-retro-2026-01-15.md`
- Epic：4（自动迭代优化执行）
- 日期：2026-01-15

> 说明：本文件用于“可勾选跟踪”；不新增范围、不做工期预估。若需要截止日期，请用明确日期（而非预估时长）。

## A) 流程改进（Process）

- [x] A1 把“证据链 DoD”前移为门禁  
  Owner：Charlie（Senior Dev）+ Dana（QA Engineer）  
  Success：任何 Story 标记为 `done` 前必须满足最小单测覆盖 + 可复现命令序列 + 关键脱敏断言；review 中“已完成但无证据”显著下降

- [x] A2 建立“契约单一权威”清单  
  Owner：Winston（Architect）+ Amelia（Dev Agent）  
  Success：`ctx.extensions` key / 同序契约 / 脱敏边界 / WS 事件字段 / 状态枚举均有单一权威入口；变更时同步更新并有测试/校验

- [x] A3 补齐 Epic 3 复盘（形成连续性）  
  Owner：耶稣（Project Lead）  
  Success：补一份最小复盘文档（行动项 + 风险 + 下一步），使 Epic 4/5 可做连续性对照

## B) 技术债/遗留（Technical debt / carry-overs）

- [x] B1 WS/可视化链路的脱敏回归门禁  
  Owner：Dana（QA Engineer）  
  Success：对 WS payload 与前端展示新增回归断言：不得出现 prompt/testcase 原文或敏感 token，仅允许结构化摘要/长度/hash/必要截断（且经过脱敏规则）

- [x] B2 真实执行链路（Dify/Direct）完善与错误映射  
  Owner：Amelia（Dev Agent）  
  Success：NotImplemented 骨架替换为真实调用与可回归的错误分类；同序契约与脱敏边界不被破坏
  Evidence：`backend/src/core/execution_target/dify_impl.rs`、`backend/src/core/execution_target/direct_api_impl.rs`、`backend/src/infra/external/llm_client.rs`（含 wiremock 单测）

- [x] B3 降低 Default/Alternate 引擎重复度，避免 drift  
  Owner：Winston（Architect）+ Charlie（Senior Dev）  
  Success：尽可能共享公共逻辑且保留可证伪差异；CI 继续覆盖 Default 与 `--features alt-optimization-engine` 双路径
  Evidence：`backend/src/core/optimization_engine/common.rs`（公共执行/评估管线）、Default/Alternate 引擎继续通过 `cargo test --all` 与 `cargo test --all --features alt-optimization-engine`

## C) 下一 Epic 开工前 P0（强制门槛）

- [x] P0-1 冻结后端→前端 WS 事件契约（schema + 必带字段含 `correlationId`），并作为单一权威  
  Evidence：`docs/developer-guides/ws-message.schema.json`、`backend/src/api/ws/events.rs`、`docs/developer-guides/contracts.md`
- [x] P0-2 状态/阶段口径由后端提供，前端只映射，不推断  
  Evidence：`backend/src/domain/models/iteration_stage.rs`、`backend/src/api/routes/meta.rs`（`GET /api/v1/meta/iteration-stages`）、`docs/developer-guides/contracts.md`
- [x] P0-3 提供“纯本地、确定性、不出网”的可视化 demo 数据源闭环（可验证不串台/不乱序）  
  Evidence：`frontend/src/features/ws-demo/demoWsMessages.ts`、`frontend/src/features/ws-demo/demoWsMessages.test.ts`、`docs/developer-guides/epic-5-demo-data-source.md`
- [x] P0-4 脱敏边界升级为门禁（WS/UI/错误字符串均不泄露）  
  Evidence：`frontend/src/App.routes.test.tsx`、`backend/tests/error_handling_test.rs`、`backend/src/api/ws/events.rs`、ExecutionTarget wiremock 单测（`backend/src/core/execution_target/*.rs`）
- [x] P0-5 性能验证口径预置（NFR2/NFR3），形成可回归对比  
  Evidence：`docs/developer-guides/performance-protocol-nfr2-nfr3.md`、`frontend/src/pages/PerfNfr2View.tsx`、`frontend/src/pages/PerfNfr3View.tsx`
