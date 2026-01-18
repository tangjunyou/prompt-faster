# Epic 7 开工门槛 P1：Checkpoint ↔ pause_state 收敛设计

- 日期：2026-01-18
- Owner：Winston（Architect）+ Charlie（Senior Dev）
- 参与：Dana（QA Engineer）、Amelia（Dev Agent）
- 关联：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`

## 1) 目标与范围

**目标：**
- 形成 Checkpoint 与 pause_state 的**单一路径**，确保用户介入状态（RunControlState + IterationArtifacts + UserGuidance）被完整保留。
- 明确“过渡期兼容 → 最终收敛”的迁移路径，避免双轨漂移。

**范围：**
- Epic 7 全周期（7.1–7.4）。
- 仅涉及状态持久化与恢复路径，不改动现有 pause_state 的交互逻辑与用户体验。

**非目标：**
- 本文不实现恢复/回滚功能细节（由 Story 7.2/7.3 承接）。
- 本文不更改前端 UI。

## 2) 现状基线

- pause_state：文件系统持久化 JSON（最小暂停快照），包含 `run_control_state` 与 `context_snapshot`（含 artifacts/user_guidance）。
- Checkpoint：领域结构已存在，但存储与校验机制未落地。
- 迭代引擎编排层负责写入 `OptimizationContext` 并进行状态推进。

## 3) 收敛原则（Hard Rules）

1. **用户介入状态必须被 Checkpoint 覆盖**（RunControlState + IterationArtifacts + UserGuidance）。
2. **Checkpoint 成为可靠性与恢复的单一权威来源（SSOT）**；pause_state 仅作为交互层“临时缓存/控制器”。
3. **过渡期允许双写，但不得双读**：读路径保持单一入口，避免数据漂移。
4. **日志与追踪遵循 AR2**：所有保存/恢复操作必须携带 correlationId。

## 4) 数据映射（pause_state → Checkpoint）

| pause_state / Context | Checkpoint 字段 | 说明 |
| --- | --- | --- |
| task_id | task_id | 任务维度一致 |
| run_control_state | run_control_state | Pause/Resume 需要可恢复 |
| context_snapshot.artifacts | artifacts | 复用已有结构，不新增并行字段 |
| context_snapshot.user_guidance | user_guidance | 复用已有结构 |
| ctx.state | state | IterationState 全量保存 |
| ctx.current_prompt | prompt | 当前 prompt |
| ctx.rule_system | rule_system | RuleSystem JSON |
| ctx.iteration | iteration | 当前迭代轮次 |
| ctx.branch_id / lineage | branch_id / parent_id / lineage_type | 分支治理字段 |

## 5) 迁移与收敛路径（Phase Plan）

**Phase 0（当前）**
- pause_state 文件系统持久化为主；Checkpoint 仅为结构预留。

**Phase 1（Story 7.1）**
- 在每个 Layer 完成后自动保存 Checkpoint（含 pause_state 用户介入数据）。
- pause_state 继续作为“暂停/继续/编辑”的状态控制器。
- **读路径仍以 pause_state 为准**，Checkpoint 仅用于可靠性与恢复准备。

**Phase 2（Story 7.2）**
- 恢复逻辑以 Checkpoint 为首选；pause_state 作为兼容回退源。
- 若检测到 pause_state 存在但 Checkpoint 缺失：生成“补偿 Checkpoint”。

**Phase 3（Story 7.3）**
- 回滚基于 Checkpoint lineage，暂停/介入操作不再依赖 pause_state 持久化文件。

**Phase 4（Story 7.4）**
- 历史展示与恢复入口统一为 Checkpoint（单入口）。
- pause_state 仅保留内存态或被移除持久化。

## 6) 兼容与降级策略

- **双写策略：**
  - Phase 1/2 期间：checkpoint 保存时读取 pause_state 数据并写入 DB；pause_state 仍写文件。
- **单读策略：**
  - 运行态读取优先 pause_state（Phase 1），恢复态读取优先 Checkpoint（Phase 2 起）。
- **降级：**Checkpoint 保存失败不阻塞迭代（记录 ERROR + 继续）。

## 7) 风险与缓解

- **风险：** pause_state 与 Checkpoint 数据不一致。
  - **缓解：** 单读原则 + 补偿 Checkpoint + 只在恢复路径切换时迁移。

- **风险：** 用户介入数据字段漂移。
  - **缓解：** 复用已有 `context_snapshot` 结构，不新增并行字段。

## 8) 验收标准

- Checkpoint 保存包含用户介入状态（RunControlState + IterationArtifacts + UserGuidance）。
- pause_state 仍保持原行为，兼容性无破坏。
- 恢复路径优先 Checkpoint，pause_state 仅作为回退。
- 全链路日志包含 correlationId。

## 9) 评审记录

- [x] Winston（Architect）评审通过
- [x] Charlie（Senior Dev）评审通过
- [x] Dana（QA Engineer）确认测试路径覆盖
