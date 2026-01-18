# Epic 7 规划复核准备（Prep）

- 日期：2026-01-18
- 关联复盘：`docs/implementation-artifacts/epic-6-retro-2026-01-18.md`
- 目标 Epic：Epic 7（可靠性与断点续跑）

## 1) 复核目的

- 统一 Epic 7 的技术路径与历史口径，避免双轨实现与数据漂移。
- 在开工前明确收敛策略与验证方案，减少返工与回滚风险。

## 2) 已确认事实（来自 Epic 6 复盘）

- pause_state 为 Epic 6 关键依赖，必须在 Epic 7 被 Checkpoint 机制收敛或替换。
- 历史口径当前存在 iterations 与未来 checkpoints 双轨风险。
- A3 构建体积拆分评估尚未闭环，需纳入 Epic 7 计划。

## 3) 关键决策点（必须在复核中达成）

1. **Checkpoint ↔ pause_state 收敛策略**
   - 是否替换 / 兼容期如何处理
   - 数据结构与迁移路径

2. **历史口径统一（iterations vs checkpoints）**
   - 历史面板数据源优先级
   - 过渡期数据合并/回溯策略

3. **恢复/回滚验证策略**
   - 必测场景清单（断电/断网/崩溃/跨版本）
   - 验证方式与可复现路径

4. **A3 构建体积拆分评估落地方式**
   - 拆分方案清单是否进入 Epic 7 story 或独立任务

## 4) 复核需要产出的结论

- Epic 7 路径更新说明（写入 `docs/project-planning-artifacts/epics.md` 的必要变更）
- 收敛设计落点与迁移路径（文档或技术设计稿）
- 历史口径统一方案与接口/数据源调整清单
- 恢复/回滚测试矩阵与验证流程
- A3 拆分方案落地清单

## 5) 参与角色

- Alice（Product Owner）
- Bob（Scrum Master）
- Winston（Architect）
- Charlie（Senior Dev）
- Dana（QA Engineer）
- Amelia（Dev Agent）

## 6) 参考资料

- `docs/project-planning-artifacts/epics.md`（Epic 7 现有定义）
- `docs/implementation-artifacts/epic-6-retro-2026-01-18.md`
- `docs/implementation-artifacts/epic-5-build-size-warning-2026-01-16.md`
- Epic 6 相关 story 记录（6.1–6.5）

