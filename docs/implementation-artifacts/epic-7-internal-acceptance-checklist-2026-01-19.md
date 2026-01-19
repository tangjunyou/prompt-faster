# Epic 7 内部验收清单（Internal Acceptance Checklist）

> 目的：在无外部验收的情况下，提供可执行、可复核的内部验收证据链。
> 规则：不做时间估算；每项验收需记录执行者与证据位置（日志/截图/输出）。

## A. 功能链路（必须全部通过）

### A1. Checkpoint 自动保存
- [x] 运行一次优化任务（包含至少 1 个完整迭代）— 通过集成测试模拟执行
- [x] 验证每个 Layer 完成后产生 Checkpoint
- [x] 验证 Checkpoint 包含：iteration/state/run_control_state/prompt/rule_system/artifacts/user_guidance/checksum
- [x] 验证 checksum 校验为真（完整性 OK）

**证据**：
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/backend-acceptance-tests.log`

### A2. 断点恢复
- [x] 任务运行中强制中断（模拟异常）
- [x] 应用重启后提示“检测到未完成任务，是否恢复？”
- [x] 选择恢复 → 恢复到最近有效 Checkpoint
- [x] checksum 失败时回退至上一个有效 Checkpoint

**证据**：
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/backend-acceptance-tests.log`

### A3. 历史回滚
- [x] 选择任意历史 Checkpoint 回滚
- [x] 回滚后该 Checkpoint 之后的数据被归档（不可作为回滚目标）
- [x] 回滚后继续迭代进入新分支（branch_id 变化）

**证据**：
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/backend-acceptance-tests.log`

### A4. 历史时间线与筛选
- [x] HistoryPanel 列表视图正常展示
- [x] Timeline 视图可切换并展示历史事件
- [x] 筛选（类型/操作者/迭代/时间范围）生效

**证据**：
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/frontend-acceptance-tests.log`

### A5. 历史导出
- [x] 导出 JSON 成功下载（通过 API 导出测试验证）
- [x] 导出内容包含：任务元信息、迭代摘要、Checkpoint 摘要、历史事件、分支信息

**证据**：
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/backend-acceptance-tests.log`

### A6. 权限与隔离
- [x] 非任务所有者访问历史/回滚/恢复接口返回 403 或等效拒绝
- [x] 资源枚举风险被 404/403 抑制

**证据**：
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/backend-acceptance-tests.log`

## B. 质量与稳定性（至少全部执行一次）

### B1. 回归测试
- [x] 后端 `cargo test`
- [x] 前端 `npx vitest --run`
- [x] 前端 `npm run lint`

**证据**：
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/backend-full-cargo-test.log`
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/frontend-full-vitest.log`
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/frontend-lint.log`

### B2. 关键场景演练
- [x] 离线状态下功能降级提示（可查看历史，执行类功能限制）
- [x] 恢复率指标可查询并具备统计意义

**证据**：
- [x] `docs/implementation-artifacts/acceptance-evidence/2026-01-19/backend-acceptance-tests.log`

## C. 部署与发布（Docker Smoke Check）

### C1. Docker Smoke Check
- [x] Docker 构建（含 `--no-cache`）通过
- [x] Docker Compose 启动 + 健康检查通过
- [x] 前端首页可访问（`127.0.0.1:5173`）
- [x] 基础鉴权与恢复接口可用
- [x] Docker Compose 关闭与清理完成

**证据**：
- [x] `docs/implementation-artifacts/docker-smoke-checklist-2026-01-19.md`

## D. 交付证据汇总

- [x] 验收执行人：Codex CLI（GPT-5）
- [x] 验收日期：2026-01-19
- [x] 证据归档位置：`docs/implementation-artifacts/acceptance-evidence/2026-01-19`

---

备注：如发现问题，务必记录为“阻塞/非阻塞”并回填至对应行动项。
