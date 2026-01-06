# Story 2.6: 创意任务配置（核心诉求与结构化约束）

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## 用户故事

**As a** Prompt 优化用户，  
**I want** 在“创意任务”模式下为每条测试用例填写“核心诉求”（无标准答案），并配置结构化约束（长度/必含/禁止/格式），  
**So that** 系统后续可以据此评估创意任务输出是否符合我的期望。

## 验收标准（Acceptance Criteria）

### AC1：核心诉求（FR13）

**Given** 测试用例 `reference` 为 `Constrained`（表示创意任务）  
**When** 用户在测试集编辑页编辑该测试用例  
**Then** UI 提供“核心诉求”输入控件（替代固定任务的标准答案）  
**And** 核心诉求为自然语言描述（string）  
**And** 保存测试集后该字段与测试用例一起持久化并可回显  
**And** 用户切换到 `cases` JSON 编辑/JSONL 导入视图时，该字段应在 `cases[*].reference.Constrained.core_request` 中直接可见（保持单一事实来源）。

### AC2：结构化约束（FR14）

**Given** 测试用例 `reference` 为 `Constrained`  
**When** 用户点击“添加/编辑结构化约束”  
**Then** 至少支持配置以下约束类型：
- 长度限制（最小/最大字符数）
- 必含关键词列表（string[]）
- 禁止内容列表（string[]）
- 格式要求（例如 `json` / `markdown` / `plain_text` / 自定义字符串）

**And** 约束以结构化方式存入 `cases[*].reference.Constrained.constraints[*]`  
**And** 存储结构可被后续评估引擎直接读取（避免把结构化信息塞进纯文本 `description` 导致二次解析/歧义）。

**建议的约束编码（MVP 可落地且向后兼容）：**

- 每个约束使用 `Constraint`（算法 DTO）承载：
  - `name`: 固定枚举风格字符串（例如：`length` / `must_include` / `must_exclude` / `format`）
  - `params`: **可选**结构化参数（`Option<serde_json::Value>`；通常为 JSON 对象；允许任意 JSON 值以便后续扩展；评估/解析时按 `name` 解释其结构）
  - `description`: 人类可读说明（用于 UI 展示/调试，不作为唯一数据源）
  - `weight`: 允许缺失或 `null`（本 Story 不强制引入权重体系；与 Rust `Option<f64>` 语义对齐）

```json
{
  "id": "case-1",
  "input": { "prompt": "写一段面向新人的欢迎文案" },
  "reference": {
    "Constrained": {
      "core_request": "友好、简洁、鼓励探索，避免说教。",
      "constraints": [
        { "name": "length", "description": "长度限制", "params": { "minChars": 30, "maxChars": 120 }, "weight": null },
        { "name": "must_include", "description": "必含关键词", "params": { "keywords": ["欢迎", "一起"] }, "weight": null },
        { "name": "must_exclude", "description": "禁止内容", "params": { "keywords": ["政治", "敏感"] }, "weight": null },
        { "name": "format", "description": "格式要求", "params": { "format": "markdown" }, "weight": null }
      ],
      "quality_dimensions": []
    }
  }
}
```

### AC3：兼容性与约束（必须遵守）

- `cases` 仍为最终数据源（JSON 编辑 / JSONL 导入仍可用）；新增 UI 只是更友好的编辑方式。
- 不破坏既有固定任务：标准答案编辑仍写入 `cases[*].reference.Exact.expected`（Story 2.5）。
- 导入/校验链路需要兼容新增字段（避免因“可选字段的缺失/null”误拦导入）：
  - `Constrained.core_request`：允许缺失或 `null`（向后兼容）；若存在且非 `null`，必须为 string
  - `Constraint.params`：完全可选；允许缺失或 `null`；若存在且非 `null`，允许为任意 JSON 值
  - `Constraint.weight`：允许缺失 / `null` / number（与 Rust `Option<f64>` 语义一致）
- 不引入新依赖也能完成 MVP（避免为“标签输入”拉入重量级组件库）。

## 任务拆分（Tasks / Subtasks）

### 任务 1：数据结构与 DTO（AC1/AC2）

- [x] 扩展算法 DTO（向后兼容）：
  - `backend/src/domain/models/algorithm.rs`：
    - `TaskReference::Constrained` 增加 `core_request: Option<String>`（字段名保持 `snake_case`，与 `quality_dimensions` 一致）
    - `Constraint` 增加 `params: Option<serde_json::Value>`（用于承载结构化约束参数；缺省为 `None`）
    - 推荐：对 `Option` 字段使用 `#[serde(skip_serializing_if = "Option::is_none")]`（后端回显时省略 `None`，更贴近“缺失即无”语义）
- [x] 同步类型生成链路（保持前后端一致）：
  - 运行/更新既有 `backend/src/bin/gen-types.rs` 类型生成（对齐现有流程；**DTO 修改后必须执行**）：
    - `cd backend && cargo run --bin gen-types`

**参考代码片段（避免误改结构）：**

```rust
// backend/src/domain/models/algorithm.rs
pub enum TaskReference {
    Exact { expected: String },
    Constrained {
        #[serde(skip_serializing_if = "Option::is_none")]
        core_request: Option<String>,
        constraints: Vec<Constraint>,
        quality_dimensions: Vec<QualityDimension>,
    },
    Hybrid { exact_parts: HashMap<String, String>, constraints: Vec<Constraint> },
}

pub struct Constraint {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}
```

### 任务 2：JSONL 导入与最小深度校验（AC3）

- [x] 前端 JSONL 解析器更新：
  - `frontend/src/features/test-set-manager/services/parseTestCasesJsonl.ts`
    - `reference.Constrained.core_request`：允许缺失或 `null`；若存在且非 `null`，必须为 string
    - `Constraint.params`：完全可选；允许缺失或 `null`；若存在且非 `null`，允许为任意 JSON 值（不做深度校验）
    - `weight` 字段放宽：允许缺失 / `null` / number（与 Rust `Option<f64>` 语义一致；避免用户手写 JSONL 因缺字段被拦）
- [x] 补齐/调整单测：
  - `frontend/src/features/test-set-manager/services/parseTestCasesJsonl.test.ts`
    - 合法：含 `core_request` + 含 `params`
    - 合法：`weight` 缺失或 `null`
    - 非法：`core_request` 存在且非 `null` 时不是 string

### 任务 3：测试集编辑页 UI（核心交付）（AC1/AC2/AC3）

- [x] 在 `frontend/src/pages/TestSetsView/TestSetsView.tsx` 增加“创意任务配置（Constrained）”区块：
  - 仅当 `casesJson` 可解析且至少存在 `reference.Constrained` 用例时展示（与“标准答案（固定任务）”同级）
  - 对每个 `Constrained` 用例展示：
    - `core_request` 文本框（多行）
    - 结构化约束编辑器（长度 min/max、必含关键词、禁止内容、格式）
    - 核心诉求输入建议：trim 后非空提示（不强制阻止用户手写 JSON 模式）；可选给出最大长度提示（例如 2000 字符）
  - 写回策略（必须做到“单一事实来源”）：
    - 所有编辑都通过更新 `casesJson` 完成（类似 `updateExactExpectedAtIndex`）
    - 更新时应保留用户在 JSON 中手写的“未知约束”（避免 UI 覆盖丢数据）
    - 更新时应保留已知约束对象上的“未知字段”（避免通过重建对象导致静默丢字段）
    - 若 `constraints` 中存在同名 `name` 的多条约束：UI 仅编辑第一条，其余原样保留并随数组写回（避免误删用户手写数据）
    - 当某类约束被清空时，从 `constraints` 中移除对应 `name` 的项（保持 JSON 干净）
  - 关键词输入建议：多行输入（每行一个关键词），写回到 `params.keywords: string[]`
- [x] 更新 JSONL 示例文案（降低学习成本）：
  - `frontend/src/pages/TestSetsView/TestSetsView.tsx` 的 `JSONL_FORMAT_HELP.example` 增加 `core_request`/`params` 示例
  - 追加一条可复制的 JSONL 单行示例（含 `core_request` + 1 个带 `params` 的约束）

```txt
{"id":"case-1","input":{"prompt":"写一段欢迎文案"},"reference":{"Constrained":{"core_request":"友好、简洁、鼓励探索","constraints":[{"name":"length","description":"长度限制","params":{"minChars":30,"maxChars":120},"weight":null}],"quality_dimensions":[]}}}
```

### 任务 4：后端合约/持久化回归保护（AC3）

- [x] 后端集成测试补齐（或在既有用例中扩展）：
  - `backend/tests/test_sets_api_test.rs`
    - create/update 测试集时，`cases` 内包含 `Constrained.core_request` 与 `constraints[*].params` 能被正确保存并在 `GET` 中回显
    - 跨用户/跨 workspace 权限边界不变（仍应 404）

### 任务 5：前端交互测试（AC1/AC2/AC3）

- [x] `frontend/src/pages/TestSetsView/TestSetsView.test.tsx`
  - 核心诉求编辑：输入 → 写回 `casesJson` → 创建/保存请求体中包含 `core_request`
  - 结构化约束编辑：填写 min/max、关键词、格式 → 写回 `constraints`（含 `params`）→ 保存请求体正确
  - 兼容性：当某用例不是 `Constrained`，该用例不被此编辑器修改；固定任务“标准答案编辑”不回归

## Dev Notes

### Developer Context（给 Dev 的最小上下文）

- 现状（已完成）：
  - Story 2.2：JSONL 导入（逐行校验+行号），`reference` 支持 `Exact/Constrained/Hybrid`
  - Story 2.5：固定任务“标准答案”编辑器已落地（写入 `reference.Exact.expected`），并明确“cases JSON 为单一事实来源”
- 本 Story 的目标：把“创意任务”从“只能手写 JSON”提升为“有明确 UI 输入控件”，同时保证数据结构对后续评估引擎友好（结构化、可扩展、可回显）。

### 技术要求（不可违背）

- 合约与兼容性：
  - `reference` 仍为单 key 变体对象（`Exact`/`Constrained`/`Hybrid`），不要改为内联 `type` 字段。
  - 新增字段必须向后兼容：旧数据没有 `core_request/params` 时也能正常解析与编辑。
- 单一事实来源：
  - UI 任何编辑必须写回 `casesJson`；不得引入“额外状态字段”导致保存时两边不一致。
- 容错：
  - 用户可能直接编辑 `cases` JSON：UI 需要在解析失败时自然降级（不崩溃，不误写）。
  - 最小降级行为：当 `casesJson` 解析失败或结构校验失败时，隐藏“标准答案/创意任务配置”编辑器，仅展示 JSON 编辑区与错误提示；不做自动修复。

### 架构对齐（必须遵守）

- 继续沿用现有测试集实现结构（不要回到架构文档里过时的 `test_cases` 命名）：
  - Backend：`backend/src/api/routes/test_sets.rs` + `backend/src/infra/db/repositories/test_set_repo.rs`
  - Frontend：`frontend/src/features/test-set-manager/*` + `frontend/src/pages/TestSetsView/TestSetsView.tsx`
- 前端错误展示：继续遵守“不直接展示 `error.details`”的约束（见 `docs/project-planning-artifacts/architecture.md#Error-Handling-Layers`）。

### Library / Framework Requirements

- 不新增依赖完成 MVP：
  - 关键词列表可用“多行输入（每行一个）/逗号分隔”实现；后续再演进为 tag input。

### File Structure Requirements（建议落点）

- DTO：`backend/src/domain/models/algorithm.rs`
- 类型生成：`backend/src/bin/gen-types.rs`
- 前端 JSONL 校验：`frontend/src/features/test-set-manager/services/parseTestCasesJsonl.ts`
- 前端 UI：`frontend/src/pages/TestSetsView/TestSetsView.tsx`
- 前端测试：`frontend/src/pages/TestSetsView/TestSetsView.test.tsx`、`frontend/src/features/test-set-manager/services/parseTestCasesJsonl.test.ts`
- 后端测试：`backend/tests/test_sets_api_test.rs`

### Testing Requirements（与 CI 门禁一致）

- Backend：`cargo test`
- Frontend：`npm run lint`、`npm test -- --run`、`npm run build`

### References

- [Source: docs/project-planning-artifacts/epics.md#Story-2.6-创意任务配置（核心诉求与结构化约束）]（FR13/FR14 验收）
- [Source: docs/project-planning-artifacts/prd.md#能力区域-2-测试集管理]（FR13/FR14）
- [Source: docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md#决策-D4-双任务模式统一]（`TaskReference::Constrained` 语义）
- [Source: backend/src/domain/models/algorithm.rs]（`TaskReference::Constrained` / `Constraint` 当前结构）
- [Source: frontend/src/pages/TestSetsView/TestSetsView.tsx]（标准答案编辑器模式、`reference` 变体约束、JSONL 示例）
- [Source: frontend/src/features/test-set-manager/services/parseTestCasesJsonl.ts]（JSONL 深度校验口径）

### Previous Story Intelligence（可复用/避免回归）

- **“写回 cases JSON”模式**：复用 Story 2.5 的 `updateExactExpectedAtIndex` 思路，避免引入第二份事实来源。
- **JSONL 校验是第一道门**：Story 2.2 已形成“逐行 + 行号”体验，新增字段必须更新该解析器与单测，否则用户会在导入阶段被误拦。
- **约束命名必须稳定**：`constraints[*].name` 一旦确定（length/must_include/must_exclude/format），后续评估引擎与 UI 都会依赖；不要在实现中随意改名。

### Git Intelligence Summary（最近提交的工程约定）

- 近期与测试集管理最相关的提交：
  - `181610c` Story 2.5: generic variables + fixed task answers
  - `63295f8` Story 2.4: Dify variables config
  - `12cf8b0` Story 2.3: test set templates + review fixes
  - `debd9b4` Story 2.2: harden JSONL batch import
  - `cbab13d` Story 2.1: test set CRUD + list summary

## Dev Agent Record

### Agent Model Used

GPT-5.2 (Codex CLI)

### Completion Notes

- Backend DTO：`TaskReference::Constrained` 增加 `core_request`，`Constraint` 增加 `params`（均向后兼容）
- Frontend：JSONL 校验放宽（core_request/params/weight 缺失或 null），并新增“创意任务配置（Constrained）”编辑区块（写回 `casesJson` 单一事实来源）
- Types：已运行 `cd backend && cargo run --bin gen-types` 更新前端生成类型
- Review fixes：当约束 `params` 不是对象时，编辑前提示并允许用户选择是否覆盖；强化 `cases` 结构校验（Constrained/Hybrid 关键字段）并补齐前端测试覆盖
- Tests：新增/更新前端单测与后端集成测试，`cargo test` + `npm run lint` + `npm test -- --run` + `npm run build` 全部通过

### Senior Developer Review (AI)

Date: 2026-01-06

- 结论：Changes Requested → Resolved
- 修复要点：
  - 避免 UI 在 `params` 为非对象（数组/字符串等）时静默覆盖：改为弹窗确认“是否覆盖”
  - 强化前端 `cases` 结构校验，避免“编辑器可见但写不回/保存后端才报错”
  - 约束 `description` 已存在时保留，避免无意覆盖用户手写描述
  - 增加前端测试覆盖上述边界行为

## File List

- backend/src/domain/models/algorithm.rs
- backend/tests/test_sets_api_test.rs
- frontend/src/features/test-set-manager/services/parseTestCasesJsonl.ts
- frontend/src/features/test-set-manager/services/parseTestCasesJsonl.test.ts
- frontend/src/pages/TestSetsView/TestSetsView.tsx
- frontend/src/pages/TestSetsView/TestSetsView.test.tsx
- frontend/src/types/generated/models/Constraint.ts
- frontend/src/types/generated/models/TaskReference.ts
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/2-6-creative-task-configuration.md

## Change Log

- 2026-01-05：实现 Story 2.6（核心诉求 + 结构化约束），更新前后端 DTO/校验/UI，并补齐测试与回归保护
- 2026-01-06：Senior code review 修复：params 非对象覆盖确认 + cases 结构校验强化 + 增补前端测试
