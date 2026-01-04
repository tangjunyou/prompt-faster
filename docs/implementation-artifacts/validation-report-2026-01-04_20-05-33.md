# 审查报告（Story Context Quality Review）

**被审查文档（Story）：** `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md`  
**审查日期：** 2026-01-04 20:05:33  
**对照输入：** 你提供的 4 份“审查建议”（以下简称 R1/R2/R3/R4）

## 0. 结论摘要（TL;DR）

- **谁审得最好：R4**。原因：有明确证据链（具体文件/行号），且指出的问题对“能否落地/是否会返工”最关键。  
- R1 次之（思路系统，但有少量“事实不成立/已在 story 中说明”的指控）。  
- R2 有价值但偏“产品/体验偏好 + 预设计”，部分建议不适合作为 Must Fix。  
- R3 结论有方向但缺少可核验证据，且存在“凭空断言已给出完全修正版”的成分。

**与我最初这份报告（旧版 8/9 PASS）不一致之处：**  
我当时漏掉了 2 个高风险点（迁移编号冲突、`cases_json` 最小校验与 `TestCase` 不一致）。本次已据证据纠正，并已把对应修复写回 Story。

---

## 1. 逐条核实：四份审查建议的真实性

### R1（“6 Critical / 4 Enhancement / 3 Optimization”）核实

**✅ 需要采纳（真实且重要）**

1) **API 必须按 workspace 归属建模（不能“建议”）**  
证据：PRD 数据表设计包含 `test_sets` 且带 `workspace_id`（`docs/project-planning-artifacts/prd.md:634`-`641`）；架构把“能力区域 2”定位为独立模块（`docs/project-planning-artifacts/architecture.md:392`-`405`），因此从 API 设计上应明确归属边界，避免“全局资源”引发隔离漏洞。  
处置：已在 Story 中把“建议”改为“强制”，并固定为 workspace 嵌套路径（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:40`）。

2) **前端入口点需要更明确**  
证据：目前前端只有 `/workspace` 路由（`frontend/src/App.tsx:89`），没有 `workspaces/:id/*` 的现成页面；Story 若不写清入口，dev 容易做成“全局测试集”。  
处置：已在 Story 中明确入口落点（列表每行按钮/链接 + 跳转路径）（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:47`、`docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:48`）。

3) **权限语义需要收敛（跨用户访问不要 403/404 混用）**  
证据：现有 Workspace 访问模式是“查不到就 NotFound”，不会返回 403（`backend/src/infra/db/repositories/workspace_repo.rs:53`-`81` 返回 `NotFound`；路由层把它映射为 404，`backend/src/api/routes/workspaces.rs:66`-`72`）。  
处置：已在 Story 中强制写死：跨用户访问一律 404（不泄露存在性），未登录 401，参数校验 400（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:43`、`docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:123`）。

4) **测试覆盖需要更具体（happy path + 校验错误场景）**  
处置：已补充到 Story 的 Testing Requirements（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:123`）。

5) **“关联数据同步清理”要明确 MVP 含义**  
处置：已补充“当前仅删除 test_sets 记录，未来再扩展”的说明（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:69`）。

**❌ 不需要采纳 / 事实不成立**

- “Story 没有明确说明不能修改 `001_initial_schema.sql`”：**不成立**。Story 的 File Structure 已明确“不要改 001”（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:113`）。

---

### R2（“系统审查 12 点”）核实

**✅ 需要采纳（但多为 Should Fix / Nice to Have）**

1) **TestCase 结构需明确（避免 dev 自行脑补）**  
证据：`TestCase.reference` 是必填（`backend/src/domain/models/algorithm.rs:11`-`23`）。  
处置：已在 Story 的 `cases_json` 最小校验里加入 `reference` 必填（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:71`-`79`）。

2) **错误响应示例（可选但很有帮助）**  
现状：Story 已引用 `ApiResponse<T>`，但没有示例 JSON。  
建议：作为 Should Improve（不会阻塞实现，但能显著降低返工）。

**❌ 不建议作为 Must Fix（偏“预设计/偏好”或与 PRD/现状不强相关）**

- “预留批量接口（/bulk 或 /batch）”：Story 2.2 的导入可以用独立端点（例如 `.../test-sets/import`），没必要在 2.1 强行预留。此项属于 **Nice to Have**。  
- “原始 JSON 编辑器体验差，至少要语法高亮”：这属于 UX 体验提升，不是 2.1 的阻塞点；可作为后续增强项。  
- “删除必须事务”：若当前只删一张表一条记录，事务不是必须；未来涉及多表清理再引入事务即可。  
- “必须加 E2E 覆盖”：现有 E2E 门禁是“核心旅程覆盖率”，新增 test-set 旅程会影响门禁口径；是否纳入核心旅程需要产品/QA决策，本 Story 不强制。
- “分页 / i18n / 嵌套路由”：均不在 Epic 2 Story 2.1 的必需范围，可作为未来演进建议。

---

### R3（“Final Analysis Summary”）核实

**✅ 真实点（但表述过于笼统）**

- “Migration 编号冲突风险”：确实存在（见下方 R4 的证据）。  
- “需要明确 TestSet 领域模型/CRUD 规格”：方向正确，但 R3 没给出可核验的文件/行号与具体落点，难以直接执行。

**❌ 不采纳的部分**

- “我已经提供 fully corrected story version”：无法核验（你给的文本里没有提供具体修正版内容与文件变更），属于不可验证主张。

---

### R4（“3 Critical / 5 High-Medium / 4 优化”）核实

**✅ 必须采纳（关键且证据充分）**

1) **Migration 编号冲突（会直接卡 migrations/tests）**  
证据：仓库已存在 `backend/migrations/002_api_credentials_and_teacher_settings.sql`（`backend/migrations/002_api_credentials_and_teacher_settings.sql:1`）。  
处置：Story 的迁移文件已顺延为 `backend/migrations/003_create_test_sets.sql`（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:113`）。

2) **`cases_json` 最小校验与 `TestCase` 结构不一致**  
证据：`TestCase.reference` 必填（`backend/src/domain/models/algorithm.rs:11`-`18`）。  
处置：已把最小校验改为至少包含 `id`/`input`/`reference`（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:71`-`79`）。

3) **前端路由现状不包含 `/workspaces/:id/*`，Story 必须写清最小闭环**  
证据：当前路由仅有 `/workspace`（`frontend/src/App.tsx:89`）。  
处置：已把“入口位置/跳转路由”写死（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:47`-`48`）。

**✅ 建议采纳（HIGH/MEDIUM）**

- “路径命名收敛（避免 `/test_sets` / `/test-sets` 分裂）”：合理。处置：Story 统一为 `/test-sets`（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:40`）。  
- “删除或改写‘最新版本号’断言”：合理。处置：已改为“以仓库锁定版本为准，不升级”（见 Story `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md:106`）。  
- “错误语义收敛为 404”：已采纳（见上文 R1/R4）。

**⚠️ 需要澄清后再决定**

- “SQLite foreign_keys 未启用所以 FK/ON DELETE CASCADE 不可靠”：  
SQLx（sqlite）在上游实现中倾向默认开启 foreign key enforcement，但为了避免争议，**建议在连接池显式设置**（例如 `SqliteConnectOptions::foreign_keys(true)`）。这是“可选但强建议”的工程化措施，是否做取决于你们对“显式配置”的偏好。

---

## 2.1 额外发现（四份审查都没点出，但值得补充）

- **命名一致性风险：`test_sets` vs `test_cases`**  
  证据：架构映射中能力区域 2 使用 `test_cases.rs` / `test_case_repo.rs` 命名（`docs/project-planning-artifacts/architecture.md:397`），而 PRD 数据表是 `test_sets`（`docs/project-planning-artifacts/prd.md:640`）。  
  建议：本 Story 以 PRD 为准实现 `test_sets`（测试集），并在代码/文档中明确“TestSet（集合）包含 TestCase（用例）”；后续可同步修正文档映射，避免新人按架构表去建 `test_cases` 表导致模型分裂。

---

## 2. 最终采纳清单（按优先级）

### 🚨 Must Fix（会导致实现发散/返工/直接卡住）

1) 迁移文件顺延到下一个序号：`backend/migrations/003_create_test_sets.sql`（原因：仓库已存在 002）  
2) `cases_json` 最小校验与 `TestCase` 对齐：至少 `id` + `input` + `reference`  
3) API 路径必须按 workspace 嵌套：`/api/v1/workspaces/{workspace_id}/test-sets`（并统一命名风格）  
4) 权限错误语义写死：跨用户访问 404；未登录 401；校验失败 400  
5) 前端入口闭环写死：`/workspace` 列表每行入口 → `/workspaces/:id/test-sets`，并在 `frontend/src/App.tsx` 增加路由

### ⚡ Should Fix（不阻塞但强烈建议，能显著减少实现成本）

6) 明确 API 合约清单（list/create/get/update/delete 的路径、DTO、状态码、错误码）  
7) 增加错误响应示例 JSON（至少 400/401/404 三类）  
8) 明确字段约束（name 为空/长度上限；`cases_json` 最大体积/格式校验错误信息）

> 说明：以上 6-8 已作为 “MVP 写死防发散” 写入 Story 的 `### API Contract（MVP）` 与 `### Frontend UX Micro-spec（MVP）`（见 `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md`）。

### ✨ Nice to Have（体验/演进建议，不应阻塞 Story 2.1）

9) JSON 编辑器的语法高亮/格式化（后续优化）  
10) 为 Story 2.2 的导入预留路径命名空间（例如 `.../test-sets/import`）但不在 2.1 强行实现  
11) 分页 / i18n / 嵌套路由等：放入后续 Story 或 tech debt 列表

---

## 3. 建议的“最终修改建议”（可直接照做）

> 说明：下列建议已经反映进当前 Story 文档中（见 `docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md`）。如果你希望把这些建议再“精炼成更短的 dev-friendly 版本”，我可以再做一轮 token 优化，但不牺牲约束清晰度。

1) **后端 DB**
   - 新增迁移：`backend/migrations/003_create_test_sets.sql`（严禁修改 `001_initial_schema.sql`；002 已存在）
   - 表建议：`test_sets(id, workspace_id, name, description, cases_json, created_at, updated_at)`；`cases_json` 存 `TestCase[]` 的 JSON 字符串

2) **后端 API（workspace 子资源）**
   - 路由文件：`backend/src/api/routes/test_sets.rs`
   - 路径（统一 kebab-case）：`/api/v1/workspaces/{workspace_id}/test-sets`
   - 必备端点：list/create/get/update/delete（并保证跨用户访问 404）

3) **前端最小闭环**
   - 在 `/workspace` 的列表每行加入口，导航到 `/workspaces/:id/test-sets`
   - 在 `frontend/src/App.tsx` 增加对应 Route，并使用 `ProtectedRoute`

4) **数据校验**
   - `cases_json`：必须是 JSON 数组；每个元素至少包含 `id`、`input`、`reference`；否则 400 + `VALIDATION_ERROR`

---

## 5. 落地状态（截至 2026-01-04）

- Story 文档已按 Must Fix + 高性价比 Should Fix 完成修订（包含 API Contract 与最小 UX micro-spec）。
- 已完成实现并通过门禁：
  - Backend：`cargo fmt --all`、`cargo clippy --all -- -D warnings`、`cargo test --all`
  - Frontend：`npm run lint`、`npm test -- --run`、`npm run build`
- Sprint 状态已更新为 `done`：`docs/implementation-artifacts/sprint-status.yaml`

---

## 4. 附：我对“谁正确”的回答

- **结论：R4 在关键事实点上最正确**（迁移编号冲突、`TestCase.reference` 必填、前端路由现状），且给出可核验证据。  
- **我最初那份校验报告（旧版）在关键性上不如 R4**：我漏掉了上述两处高风险点；本文件已纠正。  
- **R1 的大方向是对的**，但其中“Story 没写不要改 001”的指控与事实不符。  
- **R2 更像“增强建议清单”**，其中若干建议不应上升为 Must Fix。  
- **R3 表述偏结论先行**，缺乏证据与可执行落点。
