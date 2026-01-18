# Epic 7 开工门槛 P2：iterations ↔ checkpoints 历史口径统一策略

- 日期：2026-01-18
- Owner：Winston（Architect）
- 参与：Dana（QA Engineer）、Amelia（Dev Agent）
- 关联：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`

## 1) 背景与问题

当前系统已有 **iterations 历史展示**，Epic 7 引入 **checkpoints**。若两者同时作为历史入口，会造成：
- 双轨数据漂移
- UI/接口歧义
- 回滚/恢复语义混乱

目标是 **统一为单一历史入口**，并明确过渡期策略。

## 2) 统一口径策略（Single Entry）

**统一入口：** `历史查询 = 单一入口（History API）`

**Phase 策略：**

- **Phase 1（7.1）**：
  - iterations 仍作为历史展示入口（沿用现有 History UI）。
  - checkpoints 仅用于可靠性与恢复，不直接进入历史 UI。

- **Phase 2（7.2–7.3）**：
  - 引入统一 History API（服务端聚合层）。
  - **入口仍只有一个**：UI 只调用统一 History API。
  - checkpoints 在 API 内可作为“恢复/回滚节点”附加项，但不破坏主时间线。

- **Phase 3（7.4）**：
  - History API 改为以 checkpoints 为主线（SSOT），iterations 作为补充或迁移来源。
  - 最终 UI/外部接口仍保持单一入口。

## 3) 数据语义边界

| 数据源 | 角色定位 | 用途 |
| --- | --- | --- |
| iterations | 运行过程与评估历史 | UI 历史展示（现状）、评估结果回放 |
| checkpoints | 可靠性快照/恢复 | 恢复/回滚入口、数据安全 |

**统一原则：**
- 历史入口只暴露 **一种“历史事件”结构**，内部再区分来源（iteration/checkpoint）。
- 不允许前端直接混用两套 API。

## 4) History API 统一结构（示意）

```json
{
  "items": [
    {
      "id": "...",
      "source": "iteration | checkpoint",
      "iteration": 3,
      "createdAt": "2026-01-18T12:00:00Z",
      "summary": "Layer 2 completed",
      "metadata": { "checkpointId": "..." }
    }
  ]
}
```

## 5) 迁移与兼容

- **短期（7.1）**：仅新增 checkpoints API；历史 UI 继续走 iterations。
- **中期（7.2–7.3）**：新增 History API，并将历史 UI 切到单入口（推荐：`/api/v1/tasks/{task_id}/history`）。
- **长期（7.4）**：History API 以 checkpoints 为主线，必要时从 iterations 回溯补齐。

## 6) 验证与测试

- 确保 History API 始终是唯一入口（FE 不直接调用两个历史源）。
- 校验排序规则：统一按 created_at 降序。
- 校验跨版本一致性：同一 task_id 的历史不会出现重复/漂移。

## 7) 评审记录

- [x] Winston（Architect）评审通过
- [x] Dana（QA Engineer）确认验证范围
