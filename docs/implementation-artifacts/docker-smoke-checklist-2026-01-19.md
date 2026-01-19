# Docker Smoke Check 清单

> 目的：为并行部署线提供最小可执行验证路径，确保 Docker 版本可构建、可启动、可访问。
> 规则：不做时间估算；每次执行需记录环境信息与输出日志。

## 1) 构建

- [ ] 清理旧镜像/容器（如有）（未执行）
- [x] 运行构建命令：

```bash
docker compose build --no-cache
```

实际执行：
- `docker compose build`（未加 `--no-cache`）
- `docker compose build --no-cache`（已补跑）

**证据**：
- [x] 构建日志归档：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-build.log`
  - 结果：构建成功（此前一次因 Docker daemon 未运行失败，已恢复）
- [x] `--no-cache` 构建日志：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-build-no-cache.log`
  - 结果：构建成功

## 2) 启动

- [x] 启动服务：

```bash
docker compose up -d
```

- 已执行

- [x] 确认容器状态为 healthy/running

**证据**：
- [x] `docker compose up -d` 输出：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-compose-up.log`
- [x] `docker compose ps` 输出：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-compose-ps.log`

## 3) 基本连通性

- [x] 健康检查接口可访问：

```bash
curl -s http://localhost:3000/api/v1/health
```

实际执行：`curl -s http://127.0.0.1:3000/api/v1/health`（首次连接 reset，重试成功）

- [ ] Swagger/Docs 页面可加载（如启用）（未验证）
- [x] 前端首页可访问（使用 `http://127.0.0.1:5173/`，避免本机 ::1 端口冲突）

**证据**：
- [x] 健康检查输出：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-health-check.log`
- [x] 前端首页响应：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-frontend-ipv4-check.log`
- [x] 本机 5173 监听说明：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/host-port-5173.log`

## 4) 核心链路快速抽检

- [x] 登录/鉴权路径可用（register/login 返回 200）
- [x] History API 返回正常（占位 task_id，返回 403，权限校验生效）
- [x] Recovery/Checkpoint 接口可访问（`/recovery/unfinished-tasks` 返回 200，空列表）

**证据**：
- [x] 认证状态：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-auth-status.log`
- [x] 注册：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-auth-register.log`
- [x] 登录：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-auth-login.log`
- [x] History 占位校验：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-history-placeholder.log`
- [x] Recovery 列表：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-recovery-unfinished.log`

## 5) 关闭与清理

- [x] 停止并清理：

```bash
docker compose down
```

**证据**：
- [x] 关闭日志：`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-compose-down.log`

## 6) 环境记录

- [x] Docker 版本：29.1.3（`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-env.log`）
- [x] OS：macOS 15.6.1（`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-env.log`）
- [x] CPU/架构：arm64（`docs/implementation-artifacts/acceptance-evidence/2026-01-19/docker-env.log`）
- [x] 备注：本机另有 Node 进程占用 ::1:5173，`curl http://localhost:5173` 命中本机服务；Docker 前端使用 `127.0.0.1:5173` 验证通过
