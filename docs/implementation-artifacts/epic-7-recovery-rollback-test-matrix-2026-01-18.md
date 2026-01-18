# Epic 7 开工门槛 P3：恢复/回滚测试矩阵与验证脚本

- 日期：2026-01-18
- Owner：Dana（QA Engineer）
- 参与：Amelia（Dev Agent）
- 关联：`docs/implementation-artifacts/epic-7-planning-review-2026-01-18.md`

## 1) 测试矩阵（核心场景）

| 场景 | 触发方式 | 预期结果 | 验证点 |
| --- | --- | --- | --- |
| 断电/强制退出 | 运行中强制 kill 进程 | 最新 Checkpoint 可恢复 | checkpoints 表存在最新记录；checksum 通过；恢复不丢用户介入数据 |
| 崩溃模拟 | panic/abort | 恢复到最近 Checkpoint | checksum 校验通过；恢复流程不中断 |
| 断网 | 断网后继续执行 | 不影响本地 checkpoint 保存 | WAL + FULL 同步仍生效；日志含错误但不阻塞 |
| IO 错误 | 模拟 DB 不可写 | 记录错误并降级继续 | ERROR 级别日志；迭代继续执行 |
| 跨版本恢复 | 升级后重启 | 恢复到旧版本 checkpoint | schema 兼容；checksum 校验通过 |

## 2) 验证脚本（已准备）

脚本位置：`scripts/epic-7/`

- `verify_wal_and_schema.sh`
  - 验证 SQLite WAL + FULL synchronous
  - 校验 checkpoints 表与索引存在

- `checkpoint_smoke_query.sh`
  - 按 task_id 查询 checkpoint
  - 校验 checksum 非空且按 created_at 降序

> 注：部分灾难场景（断电/崩溃）需人工触发，脚本用于复核落盘状态与表结构。

## 3) 执行说明（示例）

```bash
# WAL + schema 验证
DB_PATH=data/prompt_faster.db scripts/epic-7/verify_wal_and_schema.sh

# 查询某任务 checkpoints
DB_PATH=data/prompt_faster.db TASK_ID=<task_id> scripts/epic-7/checkpoint_smoke_query.sh
```

## 4) 评审记录

- [ ] Dana（QA Engineer）评审通过
- [ ] Winston（Architect）确认场景覆盖

