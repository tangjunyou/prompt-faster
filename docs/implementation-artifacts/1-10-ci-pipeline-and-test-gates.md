# Story 1.10: CI 流水线与测试门禁

状态: review

<!-- 注：校验可选。在 dev-story 之前可运行 validate-create-story 做质量检查。 -->

## 用户故事

作为 维护 Prompt Faster 代码质量的工程负责人，
我想 使用 GitHub Actions + Docker Compose 建立最小可用 CI 流水线并设置测试门槛，
以便 每次提交都能自动验证核心流程和模块回归。

## 验收标准

1. **Given** 代码仓库托管在 GitHub  
   **When** 查看 `.github/workflows` 目录  
   **Then** 至少存在一个 CI workflow 文件，覆盖 lint、单元测试和构建三个基本步骤

2. **Given** 开发者本地装有 Docker  
   **When** 在仓库根目录执行 `docker compose up -d`（或文档中约定的等价命令）  
   **Then** 可以启动包含后端与前端的开发环境  
   **And** 数据库为 SQLite 文件库（由后端容器持有，并通过 Compose volume 持久化），为端到端测试提供基础运行环境  
   **And** 可用以下最小检查验证环境就绪：  
   - `docker compose ps` 显示 `backend` 与 `frontend` 处于 running/healthy（或 running）  
   - `curl -fsS http://localhost:3000/api/v1/health` 返回 200  
   - `curl -fsS http://localhost:5173` 返回 HTML

3. **Given** 已配置针对核心用户旅程的端到端测试用例  
   **When** CI 在主分支或 Pull Request 上运行 E2E 测试  
   **Then** 报告中显示的“核心用户旅程集合被自动化 E2E 用例覆盖的比例”不低于 80%（以旅程数量计）  
   **And** 当该比例低于阈值时 CI 标记为失败（NFR19）

4. **Given** 后端和前端模块均有相应的单元/集成测试  
   **When** CI 在每次 Pull Request 上运行回归测试任务  
   **Then** 所有被标记为“必需通过”的测试全部通过，否则 CI 状态为失败且不允许合并（NFR20）

5. **Given** 一次完整的 PR 流程已经执行  
   **When** 查看 GitHub Actions 的执行记录  
   **Then** 可以看到构建成功、测试通过/失败状态以及必要时的覆盖率报告链接

## 任务 / 子任务

- [x] 任务 1：CI 基线流水线核验与补齐（AC: #1, #4, #5）
  - [x] 1.1 校验/完善 `.github/workflows/ci.yml` 的后端 lint/test/build 与前端 lint/test/build 步骤
  - [x] 1.2 确认 CI 输出包含测试结果与覆盖率报告的可访问入口（建议以 GitHub Actions artifacts + Job Summary 呈现）
    - [x] 前端单测覆盖率：`npm run test:coverage` 生成 `frontend/coverage/`，在 CI 中上传为 artifact（建议命名：`frontend-coverage`）
    - [x] 在 `$GITHUB_STEP_SUMMARY` 中写明获取方式：Actions Run → Artifacts → `frontend-coverage`
  - [x] 1.3 明确 required checks（在仓库设置中配置）以阻止未通过测试的合并（NFR20）
    - [x] 在仓库 Settings → Branches → Branch protection rules 中，为 `main`/`develop`（如适用）配置 Required status checks
    - [x] required checks 必须包含（以 GitHub 显示的 check 名称为准；名称/前缀可能随触发事件或 workflow 名称略有变化，建议直接从 PR Checks 列表复制粘贴）：
      - `CI / Backend Lint`
      - `CI / Backend Test`
      - `CI / Backend Build`
      - `CI / Frontend Lint`
      - `CI / Frontend Test`
      - `CI / Frontend Build`
      - `CI / Frontend E2E`
      - `CI / Security Audit`（若启用安全门禁）
- [x] 任务 2：Docker Compose 作为 E2E 基础环境（AC: #2）
  - [x] 2.1 校验根目录 `docker-compose.yml` 能启动后端 + 前端（SQLite 文件库 + volume 持久化）
    - [x] `docker compose up -d`
    - [x] `docker compose ps`（确认服务 running/healthy）
    - [x] `curl -fsS http://localhost:3000/api/v1/health`
    - [x] `curl -fsS http://localhost:5173`
  - [x] 2.2 补充/核对文档中的启动命令与环境变量说明（以 `docker-compose.yml` 为准，避免引入多套 compose 入口）
- [x] 任务 3：核心用户旅程 E2E 覆盖门禁（AC: #3）
  - [x] 3.1 定义“核心用户旅程集合”清单（与 E2E 用例一一映射）
    - [x] 旅程清单文件：`frontend/tests/e2e/core-journeys.yml`
    - [x] 口径：覆盖率按“旅程数量”计（而非测试用例数量）；一个旅程可由多个 spec 覆盖
    - [x] 最小示例（可按实际 PRD/UX 旅程扩充）：
      ```yaml
      # frontend/tests/e2e/core-journeys.yml
      journeys:
        - id: auth
          title: 用户登录/认证
          covered_by:
            - file: frontend/tests/e2e/auth.spec.ts
        - id: workspace-access
          title: 工作区访问控制
          covered_by:
            - file: frontend/tests/e2e/workspaces.spec.ts
        - id: view-switcher
          title: 三视图切换
          covered_by:
            - file: frontend/tests/e2e/view-switcher.spec.ts
        - id: health
          title: 系统健康检查
          covered_by:
            - file: frontend/tests/e2e/health.spec.ts
      ```
  - [x] 3.2 在 CI 中统计覆盖比例（≥80%）并在未达标时失败（NFR19）
    - [x] 覆盖率统计脚本：`frontend/tests/e2e/scripts/check-core-journeys-coverage.mjs`
    - [x] 输入：`frontend/tests/e2e/core-journeys.yml`
    - [x] 输出：
      - 机器可读：`frontend/test-results/core-journeys-coverage.json`
      - 人类可读：写入 `$GITHUB_STEP_SUMMARY`（列出 covered/total/percentage + 未覆盖旅程列表）
    - [x] 失败策略：当 `covered/total < 0.8` 时脚本 `exit 1`，使 CI 失败
  - [x] 3.3 确保 Playwright E2E 在 CI 中可重复稳定运行（必要时修复不稳定用例）
    - [x] 遵循既有稳定性经验：避免“刷新导致内存态登录丢失”等场景回归；对路由跳转与登录态依赖敏感的用例优先稳定化
- [x] 任务 4：回归测试门禁（AC: #4）
  - [x] 4.1 后端 `cargo test --all` 通过且作为 required check
  - [x] 4.2 前端 `npm run test -- --run` 与 `npm run test:coverage` 通过且作为 required check
- [x] 任务 5：安全审计门禁（可选但推荐，若已存在则核验）（AC: #5）
  - [x] 5.1 运行 `cargo audit` 与 `npm audit --audit-level=high`
  - [x] 5.2 确保审计失败会阻止合并（将 `CI / Security Audit` 纳入 required checks）

## 开发备注

- **Developer Context**
  - 目标不是“新增一个 CI”，而是保证 **lint/test/build + E2E + 覆盖率门禁** 真正起作用
  - 以现有 CI 配置为基线（如已存在，则只做缺口补齐与门禁强化）

### 技术要求（Technical Requirements）
- CI 必须运行于 GitHub Actions（架构约束）
- E2E 测试以 Playwright 为标准执行方式（现有配置在前端）
- 本地 E2E 基础环境必须可由 `docker compose up` 启动
- 保持 Rust 与 Node 工具链版本与项目锁定版本一致

### 架构合规（Architecture Compliance）
- CI 最小流水线：lint + test + build（三段齐全）
- 数据库为 SQLite 文件库（Compose 场景：由后端容器持有 + volume 持久化）
- 端到端测试覆盖率目标与回归测试门禁为 **强制**（NFR19/NFR20）

### 库/框架要求（Library & Framework）
- GitHub Actions：同一类官方 Action（如 checkout/setup-node/cache/upload-artifact）保持一致的主版本，避免混用；本 Story 不强制升级到“最新版本”
- Playwright 作为前端 E2E 测试框架，按现有 `frontend/playwright.config.ts` 执行（需要机器可读报告时，可在 CI 增加输出文件/reporter，但避免改变本地开发体验）
- 安全审计：`cargo-audit`（RustSec）与 `npm audit`

### 文件结构要求（File Structure Requirements）
- CI：`.github/workflows/ci.yml`（单一入口，避免碎片化）
- E2E：`frontend/tests/e2e/*.spec.ts`
- 核心旅程清单：`frontend/tests/e2e/core-journeys.yml`
- 覆盖率统计脚本：`frontend/tests/e2e/scripts/check-core-journeys-coverage.mjs`
- 前端单测：`frontend/src/**/*.test.ts(x)`
- 后端测试：`backend/tests/*.rs` + `#[cfg(test)]` 单元测试
- 本地环境：`docker-compose.yml`（根目录）

### 测试要求（Testing Requirements）
- Rust：`cargo fmt --all -- --check`、`cargo clippy -- -D warnings`、`cargo test --all`
- Frontend：`npm run lint`、`npm run test -- --run`、`npm run test:coverage`
- E2E：`npm run test:e2e`（Playwright）
- 覆盖率门禁：核心用户旅程 E2E 覆盖率 ≥ 80%

### 前序故事情报（Previous Story Intelligence）
- Story 1.9 已建立前端三视图路由与 ViewSwitcher，E2E 已覆盖视图切换与登录流程
- 近期针对 E2E 稳定性进行了修复（避免刷新导致内存态登录丢失）
- 前端测试使用 Vitest + Playwright，CI 中已存在 E2E 任务框架

### Git 近期提交情报（Git Intelligence Summary）
- `fix: keep auth e2e in-memory navigation`：说明 E2E 对路由跳转/内存态依赖敏感
- `fix: stabilize auth e2e`：E2E 稳定性仍在持续维护
- `feat: complete story 1.9 frontend architecture`：已补齐前端测试/路由/类型生成

### 版本策略（Version Policy）
- CI 与本地开发以仓库现有锁定版本为基线，避免在本 Story 中引入“顺手升级”导致的不稳定
  - GitHub Actions：以 `.github/workflows/ci.yml` 当前使用的主版本为准；若确需升级，单独开变更并验证 CI 全绿
  - Node.js：以 CI 与 Docker 镜像当前使用版本为准（目前 CI 与前端镜像均为 22.x 系列）
  - Playwright：以 `frontend/package-lock.json` 锁定版本为准；升级 Playwright 属于独立风险变更（浏览器版本/用例稳定性），不纳入本 Story 必做项
- Runner 镜像与依赖可能随时间变化；若出现不稳定，再考虑显式固定 `runs-on` 或关键依赖版本

### 项目结构备注

- 与统一项目结构保持一致（路径、模块、命名）
- 现有 CI 相关文件已集中于 `.github/workflows/` 与根目录 `docker-compose.yml`，避免新增分散入口
- E2E 与测试路径沿用现有结构（无需新增目录层级）

### 参考

- [Source: docs/project-planning-artifacts/epics.md#Story-1.10] — 验收标准原文
- [Source: docs/project-planning-artifacts/architecture.md#Infrastructure-&-Deployment] — CI/CD 与 Docker Compose 约束
- [Source: .github/workflows/ci.yml] — 现有 CI 工作流
- [Source: docker-compose.yml] — 本地环境基础
- [Source: frontend/playwright.config.ts] — E2E 配置
- [Source: frontend/tests/e2e] — 现有 E2E 用例目录
- [Source: backend/tests] — 后端测试目录
- [Source: frontend/package.json] — 前端脚本与依赖约定
- [Source: docs/implementation-artifacts/1-9-frontend-architecture-and-type-safe-api-client.md] — 前序故事情报

## 故事完成状态

- 状态：review
- 完成说明：已完成 CI 流水线与测试门禁实现与本地验证，可进入代码评审

## 开发代理记录

### 使用的代理模型

GPT-5 (Codex CLI)

### 调试日志引用

N/A

### 完成备注列表

- 已补齐 CI Job Summary 与覆盖率 artifact 输出（`frontend-coverage`）
- 已新增“核心用户旅程”清单与覆盖率门禁脚本，并接入 CI 的 E2E Job（阈值：80%）
- 已修复 `npm run test:coverage` 在 CI 中可能进入 watch 模式的问题（改为 `vitest run --coverage`）
- 已补充 README 中的 `docker compose up -d` 启动检查与 Compose 环境变量说明
- 已验证 `docker compose up -d` 可启动前后端，且健康检查可用（`/api/v1/health` 与前端首页返回 200）
- 已运行 `cargo audit` 与 `npm audit --audit-level=high`（本次本机执行未触发 high 级别告警，命令返回成功）
- 已在 `main` 分支启用分支保护并配置 required checks（阻止未通过测试/审计的合并）
- 已忽略 `coverage/`、`playwright-report/`、`test-results/` 以避免前端 lint 受本地产物影响

待处理（需要你在 GitHub 仓库设置中完成或受外部网络影响）：
- `develop` 分支当前不存在，因此未配置分支保护

### 文件清单

实际修改/新增：
- `.github/workflows/ci.yml`
- `docker-compose.yml`
- `README.md`
- `backend/Cargo.toml`
- `frontend/package.json`
- `frontend/eslint.config.js`
- `frontend/src/lib/core-journeys-coverage.js`
- `frontend/src/lib/core-journeys-coverage.test.ts`
- `frontend/tests/e2e/core-journeys.yml`
- `frontend/tests/e2e/scripts/check-core-journeys-coverage.mjs`

## 变更日志

- 2026-01-03：CI 增加覆盖率 artifact + Job Summary；新增核心旅程覆盖门禁（含 JSON 输出 + Summary）；修复 `test:coverage` 在 CI 中可能卡住；补齐 Compose 启动文档
- 2026-01-03：修复 Docker Compose 后端启动（设置默认运行二进制）；本机验证 compose 启动与健康检查；补齐本机安全审计执行结果
- 2026-01-03：启用 `main` 分支 required checks；前端 lint 忽略本地测试产物目录
- 2026-01-03：修复前端 TypeScript build 被测试文件阻塞（为 core journeys 覆盖模块补齐 `.d.ts` 类型声明）
