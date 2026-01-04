# Story 2.2: 测试集批量导入

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 批量导入测试集（txt 格式）,
so that 我可以快速导入大量测试用例而无需逐条手动输入。

## Acceptance Criteria

1. **Given** 用户在测试集页面（测试集管理）  
   **When** 用户上传符合格式的 txt 文件  
   **Then** 系统解析文件内容并显示预览（总数 + 示例项）  
   **And** 用户确认后，将解析结果应用为测试集的 `cases`（用于创建/更新 TestSet）

2. **Given** 用户上传格式不正确的文件  
   **When** 解析失败（任意行 JSON 无法解析 / 缺少字段 / 字段类型不正确 / id 重复）  
   **Then** 显示友好的错误提示  
   **And** 明确说明“正确格式”并给出可复制的示例  
   **And** 错误列表应包含行号并可定位（MVP：列表高亮即可）

3. **Given** 文件包含 100+ 条测试用例  
   **When** 批量导入  
   **Then** 解析过程显示进度（至少显示“已解析/总行数”或“已解析条数”）  
   **And** 解析完成后显示成功/失败统计（例如：成功解析 N 条、失败 M 条）  
   **And** 保存时显示保存成功/失败（若保存失败，给出失败原因文案）

## Tasks / Subtasks

- [x] 任务 1：定义并固化导入 txt 格式（AC: #1, #2, #3）
  - [x] 格式约定（MVP）：**JSON Lines**（`.txt` / `text/plain`，UTF-8）
    - [x] 每行一个 `TestCase` JSON 对象；允许空行（空行跳过）
    - [x] 必填字段：`id`（非空字符串且全文件唯一）、`input`（对象）、`reference`（见下：**必须是单 key 的变体对象**）
    - [x] 可选字段：`split`、`metadata`（若提供，必须符合 `TestCase` 结构）
    - [x] 解析成功输出：`Vec<TestCase>`；解析失败输出：按行聚合的错误列表（含行号、错误原因）
  - [x] 在 Story 中提供“正确格式示例”（至少覆盖 `Exact`；可选再给 `Constrained` 示例）

- [x] 任务 2：前端批量导入 + 预览 + 应用（AC: #1, #2, #3）
  - [x] 在 `frontend/src/pages/TestSetsView/TestSetsView.tsx` 增加“批量导入（txt）”区块
    - [x] 文件选择 +（可选但推荐）拖拽上传
    - [x] 读取文件文本（MVP：最大文件大小 **5MB**；超限给出提示并禁止解析）
    - [x] 逐行解析 JSONL，生成预览与错误列表（含行号）
    - [x] 解析成功：显示预览（总数 + 前 N 条展示/折叠）；提供“应用到 cases(JSON)”按钮，将 `casesJson` 替换为 pretty JSON 数组
    - [x] 解析失败：显示“正确格式说明 + 示例”与错误列表；禁止“应用/保存”
  - [x] “100+ 条用例”体验：解析过程不阻塞 UI（实现上可按批处理让出事件循环），并显示进度

- [x] 任务 3：后端保持合约稳定（AC: #1）
  - [x] 不新增导入 API：复用既有 `POST/PUT /api/v1/workspaces/{workspace_id}/test-sets`（`cases` 仍为 `TestCase[]`）
  - [x] 后端仍以 `parse_cases` 做最终校验，失败返回 400 + `VALIDATION_ERROR`（前端仅展示 `error.message`，不展示 details）

- [x] 任务 4：测试与门禁（AC: #1, #2, #3）
  - [x] 前端：为 JSONL 解析器新增单测（空行、合法/非法 JSON、缺字段、重复 id、`reference` 变体结构、`split/metadata`、100+ 行性能/进度的基本行为）
  - [x] 本地预检：`npm run lint`、`npm test -- --run`、`npm run build`

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [x] [AI-Review][LOW] 评估是否需要“追加导入”（append）模式（覆盖 vs 追加）以及默认值
- [x] [AI-Review][LOW] 若真实使用场景出现 1000+ 行，评估是否需要后端 multipart + 流式解析/分段提交以改善性能与错误定位

## Dev Notes

### Developer Context（给 Dev 的最小上下文）

- 本 Story 在 **Story 2.1（测试集 CRUD）** 的基础上新增“批量导入（txt）→ 预览 → 应用到 cases”的前端能力（FR7）。
- 当前数据模型里 **TestSet 以 `cases_json` 内嵌 `TestCase[]`** 存储；本 Story 不新增 test_case 表、不引入新的持久化形态。
- 后端 API 已支持创建/更新时提交 `cases: TestCase[]` 并做反序列化校验；本 Story 的“批量创建测试用例”体现在：用户确认后一次性创建/更新 TestSet 的 `cases` 数组。
- UX 目标来自 `ux-design-specification.md`：MVP 仅要求 txt 导入 + 基本预览 + 错误高亮（后续 Growth 可扩展 CSV/JSON/YAML）。

### 导入 txt 格式（MVP：JSON Lines / JSONL）

> 选择 JSONL 的原因：与现有 `TestCase` DTO 完全对齐，避免在 2.2 额外发明一套“半结构化 txt 语法”导致返工。

- 文件要求：`text/plain` / `.txt`，**UTF-8**。
- 解析规则：
  - 逐行读取，空行跳过。
  - 每个非空行必须是一个 JSON 对象，可反序列化为 `TestCase`（见 `backend/src/domain/models/algorithm.rs:TestCase`）。
  - `id`：非空字符串；**全文件唯一**（若重复，视为格式错误）。
  - `input`：对象（`Record<string, JsonValue>`）。
  - `reference`：**必须是单 key 的变体对象**（与 ts-rs/serde 的枚举形态一致）：
    - `{"Exact":{"expected": string}}`
    - `{"Constrained":{"constraints": Constraint[],"quality_dimensions": QualityDimension[]}}`
    - `{"Hybrid":{"exact_parts": Record<string,string>,"constraints": Constraint[]}}`
  - `split`（可选）：若提供，只允许 `"unassigned" | "train" | "validation" | "holdout"`。
  - `metadata`（可选）：对象（`Record<string, JsonValue>`）。
  - 解析失败策略（MVP）：继续扫描整文件并汇总错误；错误列表最多保留前 50 条（超出只提示“还有更多错误”）。
- “正确格式示例（可复制）”：

```txt
{"id":"case-1","input":{"question":"你好，帮我写一段自我介绍"},"reference":{"Exact":{"expected":"（此处填写期望输出）"}}}
{"id":"case-2","input":{"question":"用 JSON 输出一个用户对象"},"reference":{"Constrained":{"constraints":[{"name":"format","description":"必须是 JSON","weight":1.0}],"quality_dimensions":[{"name":"correctness","description":"字段合理且可解析","weight":1.0}]}}}
```

### Frontend UX Micro-spec（MVP）

- 入口：`/workspaces/:id/test-sets`（已存在 `TestSetsView`）。
- 交互：
  - 提供文件选择按钮与（推荐）拖拽上传区域；提示“仅支持 txt（JSONL，一行一个 TestCase JSON）”。
    - 文件约束（MVP）：`accept=".txt,text/plain"`；大小上限 5MB（超限提示并禁止解析）。
  - 上传后立即解析并显示：
    - 解析进度（至少：已处理行数/总行数 或 已解析条数）
    - 预览（总条数 + 前 N 条摘要；可折叠显示原始 JSON）
    - 解析结果统计：成功 N 条、失败 M 条（失败为“按行错误”，不进入保存）
  - 解析成功：显示“应用到 cases”按钮，一键把解析结果写入现有 JSON 编辑器（**本 Story 默认行为：覆盖 `casesJson`**）。
    - 覆盖保护（MVP）：若当前 `casesJson` 解析为非空数组（或存在未保存编辑），点击“应用”前二次确认。
  - （明确不做）追加导入（append）不在本 Story 范围内；若需要在 review 后作为增强项追加。
  - 解析失败：显示友好错误（不要直接吐出原始异常堆栈），并展示错误列表（行号 + 原因），同时给出“正确格式示例”。
- 100+ 条用例：解析过程避免长时间阻塞 UI（实现上允许按批次让出事件循环），并提供最小可感知进度。
- 进度更新建议（MVP）：每处理 100 行更新一次进度；每处理 200 行让出一次事件循环（例如 `await new Promise(r => setTimeout(r, 0))`）。
  - 进度更新建议（MVP）：每处理 100 行更新一次进度；每处理 100 行让出一次事件循环（确保 100+ 行至少让出一次）。

### API Contract（本 Story 不新增端点）

- 复用：
  - `POST /api/v1/workspaces/{workspace_id}/test-sets`（创建）
  - `PUT /api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}`（更新）
- 约束：
  - 后端失败：400 + `VALIDATION_ERROR`；前端只展示 `error.message`，不要展示 `error.details`（见 `docs/project-planning-artifacts/architecture.md` 的前端错误展示约束）。

### Library / Framework Requirements（以仓库 lock 为准）

- Frontend：React 19、React Router 7、TanStack Query v5（见 `frontend/package-lock.json`）。
- Backend：axum 0.8、sqlx 0.8（见 `backend/Cargo.lock`）。
- 版本策略：**本 Story 不做依赖升级**，按现有代码模式补功能。

### Previous Story Intelligence（来自 Story 2.1 的既有落地）

- 既有页面 `frontend/src/pages/TestSetsView/TestSetsView.tsx` 已包含 `casesJson` 的本地最小校验（`id/input/reference`）与保存流程；批量导入建议复用同一套校验口径（但导入需要“逐行 + 行号”错误信息）。
- 既有后端 `backend/src/api/routes/test_sets.rs` 会把 `cases` 反序列化为 `Vec<TestCase>`，失败返回 `VALIDATION_ERROR`；前端应保持“只展示 message，不展示 details”的原则。
- 列表接口返回 summary（`cases_count`），编辑时再 `GET /{test_set_id}` 拉全量 cases；批量导入的“预览”应基于本地解析结果，不依赖 list 接口。

### Git Intelligence Summary（最近一次相关实现）

- 最近一次与测试集最相关的提交为 `cbab13d`（Story 2.1），新增/修改了：`backend/src/api/routes/test_sets.rs`、`frontend/src/pages/TestSetsView/TestSetsView.tsx`、`frontend/src/features/test-set-manager/*` 等；本 Story 应沿用这些现成结构追加功能，避免新建平行的 import 页面/服务层。

### Project Structure Notes

### 目录结构澄清（重要）

- `docs/project-planning-artifacts/architecture.md` 中“测试集管理”相关的目录/命名（`test_cases`、`test-case-manager`）已与实际实现产生偏差。
- 本项目当前已落地的实现以 `test_sets` / `test-set-manager` 为准：`backend/src/api/routes/test_sets.rs`、`frontend/src/features/test-set-manager/*`、`frontend/src/pages/TestSetsView/TestSetsView.tsx`。

### File Structure Requirements（建议落点）

- Frontend 页面：继续在 `frontend/src/pages/TestSetsView/TestSetsView.tsx` 完成 UI 入口与状态管理（避免拆散导致复杂度上升）。
- Frontend 解析器：新增纯函数工具（便于单测），例如：
  - `frontend/src/features/test-set-manager/services/parseTestCasesJsonl.ts`（或同级 `utils/` 目录）
  - 返回结构建议：`{ cases: TestCase[]; errors: { line: number; message: string }[] }`
- Frontend 测试：`frontend/src/features/test-set-manager/services/parseTestCasesJsonl.test.ts`

### Testing Requirements（与 CI 门禁一致）

- Frontend：`npm run lint`、`npm test -- --run`、`npm run build`

### References

- [Source: docs/project-planning-artifacts/epics.md#Story-2.2-测试集批量导入] — 原始验收标准
- [Source: docs/project-planning-artifacts/prd.md#能力区域-2-测试集管理] — FR7（txt 批量导入）与 TestSet 持久化形态
- [Source: docs/project-planning-artifacts/ux-design-specification.md#Interaction-Scenarios-Matrix] — “测试集导入与预览（txt + 预览 + 错误高亮）”
- [Source: docs/implementation-artifacts/2-1-test-set-data-model-and-basic-crud.md] — 既有 TestSet CRUD 合约与前端页面入口
- [Source: backend/src/domain/models/algorithm.rs] — `TestCase` / `TaskReference` 结构（导入格式对齐）
- [Source: docs/project-planning-artifacts/architecture.md] — 命名/错误展示/目录结构约束（注意 `test_cases` 映射已与实现产生偏差，以现实现 `test_sets` 为准）

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Debug Log References

- `frontend`: `npm run lint`
- `frontend`: `npm test -- --run`
- `frontend`: `npm run build`
- `backend`: `cargo test`

### Implementation Plan

- 新增纯函数/可测试的 JSONL 解析器：逐行解析、空行跳过、行号错误聚合、id 去重、reference 变体结构校验、split/metadata 校验；支持进度回调与让出事件循环。
- 在 TestSetsView 增加 txt 导入区块：文件选择/拖拽、5MB 限制、解析进度与统计、成功预览与“应用到 cases(JSON)”覆盖写入（带二次确认）、失败时展示格式说明+示例并禁用应用/保存。

### Completion Notes List

- 已创建 Story 2.2 文档并补全为可直接开发的实现指南（含导入格式、UX micro-spec、文件落点、测试门禁）
- 新增 JSONL 解析器与单测：覆盖空行、非法 JSON、缺字段、重复 id、reference 变体、split/metadata、100+ 行进度/让出。
- TestSetsView 增加“批量导入（txt）”区块：预览/错误高亮/进度，支持应用到 cases(JSON)；解析失败时禁用应用与保存。
- 保存反馈补全：创建/更新成功提示；失败继续仅展示 `error.message`。
- 门禁通过：`npm run lint`、`npm test -- --run`、`npm run build`、`cargo test`。
- 评估结论（AI-Review）：MVP 保持“覆盖导入”，不做 append；1000+ 行场景后续可考虑 Worker 或后端流式/分段提交。

### File List

- `docs/implementation-artifacts/2-2-test-set-batch-import.md`
- `docs/implementation-artifacts/sprint-status.yaml`
- `docs/implementation-artifacts/validation-report-2026-01-04_22-56-29.md`
- `frontend/src/features/test-set-manager/services/parseTestCasesJsonl.ts`
- `frontend/src/features/test-set-manager/services/parseTestCasesJsonl.test.ts`
- `frontend/src/pages/TestSetsView/TestSetsView.tsx`

## Change Log

- 2026-01-04：实现测试集 txt(JSONL) 批量导入（解析/预览/应用）、新增解析器单测、补充保存成功提示并通过 lint/test/build 门禁。
- 2026-01-05：补齐 JSONL 导入阶段的最小深度校验（避免保存阶段才失败且无法行号定位）、增加导入取消/竞态保护、重构解析器重复逻辑，并补全 File List 可追溯性。

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- ✅（Added / HIGH）导入格式对齐真实 DTO：`reference` 必须是单 key 变体对象；补充 `split/metadata` 可选字段边界（避免解析器实现错误）。
- ✅（Fixed / HIGH）补齐 `Constrained/Hybrid` 的最小深度校验（约束/维度元素结构与 `Hybrid.exact_parts` 值类型），尽量把“保存阶段失败”前移到“导入阶段并带行号定位”。
- ✅（Added / HIGH）覆盖保护：默认覆盖 `casesJson` 前加入二次确认（避免误操作丢数据）。
- ✅（Added / MEDIUM）拆分“解析统计 vs 保存结果”语义（保存是原子操作，避免误做“部分保存成功”）。
- ✅（Fixed / MEDIUM）导入流程增加取消/竞态保护：允许清除并忽略旧解析结果，避免快速切换文件导致状态被覆盖。
- ✅（Added / MEDIUM）显式澄清架构文档与实现偏差：以 `test_sets` / `test-set-manager` 为准，避免在不存在目录中开发。
- ✅（Added / LOW）MVP 文件约束更明确：5MB 上限 + 超限禁止解析；错误列表上限 50 条；给出进度更新节奏建议。

### Decisions

- 选择 JSONL（而非自定义 txt 语法）：与 `TestCase` DTO 直接对齐，且便于逐行错误定位与进度显示。
- 解析失败策略：继续扫描整文件并汇总错误（上限 50 条），有错误则禁止“应用/保存”。
- 交互默认覆盖：覆盖前做确认；不做 append（追加导入）以控制 MVP 复杂度。

### Risks / Tech Debt

- 若真实用户常见 1000+ 行且单行 JSON 很长：主线程解析可能卡顿（触发条件：明显掉帧/卡顿时，考虑 Worker/分段解析）。
- 后端 `parse_cases` 只做整体反序列化校验：保存失败无法提供“行号级”定位（通过前端解析阶段尽量拦截并提供行号来缓解）。

### Follow-ups

- 同步到 `### Review Follow-ups (AI)`（见上方）。
