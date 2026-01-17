# Epic 5 行动项跟踪（Action Items Tracker）

- 来源：`docs/implementation-artifacts/epic-5-retro-2026-01-16.md`
- Epic：5（可视化与实时反馈）
- 日期：2026-01-16

> 说明：本文件用于“可勾选跟踪”；不新增范围、不做工期预估。若需要截止日期，请用明确日期（而非预估时长）。

## A) 开工门槛（Epic 6 前置）

- [x] A1 明确"暂停/编辑/继续"的状态一致性与权限边界
  - Owner：Winston（Architect）+ Charlie（Senior Dev）
  - 成功标准：形成一份状态与权限边界说明（包含状态机、允许的操作与触发条件），并在实现前完成评审确认。
  - 证据：`docs/implementation-artifacts/epic-6-run-control-state-design.md`
  - 完成日期：2026-01-16

- [x] A2 用户介入的可追踪证据链
  - Owner：Dana（QA Engineer）+ Amelia（Dev Agent）
  - 成功标准：定义并落地"日志/状态回放入口"的最小证据链要求，确保用户介入操作可追踪与可回放。
  - 证据：`docs/implementation-artifacts/epic-6-traceability-verification.md`
  - 完成日期：2026-01-16

- [ ] A3 构建体积告警拆分评估
  - Owner：Winston（Architect）
  - 成功标准：形成可执行拆分方案清单，并在后续 Story 中落实为具体任务。
  - 证据：`docs/implementation-artifacts/epic-5-build-size-warning-2026-01-16.md`
