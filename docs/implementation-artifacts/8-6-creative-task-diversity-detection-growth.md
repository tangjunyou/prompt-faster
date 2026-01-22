# Story 8.6: 创意任务多样性检测（Growth）

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 8-6-creative-task-diversity-detection-growth

## Epic 8 概述

> **Epic 8: 结果输出与元优化** - 用户成果：用户可以查看、导出优化结果，查看诊断报告，并使用元优化功能优化老师模型 Prompt。

**Epic 8 Story 列表**：
- 8.1 结果查看与导出（FR60, FR61，NFR18）- ✅ done
- 8.2 诊断报告（FR63）- ✅ done
- 8.3 元优化基础（FR56, FR57, FR58）- ✅ done
- 8.4 高级用户直接编辑老师模型 Prompt（FR59）- ✅ done
- 8.5 Prompt 版本对比（FR62）- ✅ done
- **8.6 创意任务多样性检测（本 Story，FR34）** - Growth

## Key Decisions (Growth)

- **检测时机**：在质量评估（Layer 3）完成后追加计算多样性分数，**后台异步计算并回写迭代产物**，不阻塞主流程。
- **计算策略**：仅对"创意任务"模式启用，读取 `optimization_tasks.task_mode`（取值 `fixed|creative`）判断。
- **指标体系**：
  - 词汇多样性（Lexical Diversity）：使用 Type-Token Ratio（TTR）或 MTLD。
  - 结构多样性（Structural Diversity）：输出格式/长度变化度。
  - 语义多样性（Semantic Diversity）：使用 embedding 余弦距离（本 Story **不引入 embedding 能力**，默认关闭）。
- **基准线**：启用多样性检测后 **首次评估自动记录基准线**；`POST /diversity/baseline` 用于**手动重置/更新**基准线（幂等 upsert）。
- **告警阈值**：多样性分数低于 0.3（可配置）时发出警告；**告警等级**按整体分数区间划分（见「Diversity Computation Logic」）。
- **展示位置**：在评估结果页面新增"多样性分析"卡片。
- **AR2 遵循**：所有操作记录 correlationId，支持全链路追踪。
- **可选功能**：用户可在任务配置中开启/关闭多样性检测。

## Story

As a 创意任务用户,
I want 系统检测创意任务输出的多样性分数,
so that 我可以确保优化后的 Prompt 不会导致输出过于单一。

## Acceptance Criteria

1. **Given** 用户创建的优化任务标记为"创意任务"
   **When** 执行质量评估
   **Then** 除通过率外，额外计算多样性分数

2. **Given** 多样性分数计算完成
   **When** 显示评估结果
   **Then** 展示多样性指标（如：词汇多样性、结构多样性、语义多样性）
   **And** 与基准线对比显示改进/退化

3. **Given** 多样性分数过低
   **When** 系统检测到问题
   **Then** 发出警告"优化可能导致输出过于单一"
   **And** 建议用户考虑调整优化目标

## Tasks / Subtasks

- 文件落点以 **File Structure Requirements** 为准；本节只描述职责，避免重复写路径。

- [x] 后端：多样性分析 DTO 定义（AC: 1-3）
  - [x] 创建 `diversity_analysis.rs` 模块
    - `DiversityMetrics`: lexical_diversity, structural_diversity, semantic_diversity, overall_score
    - `DiversityBaseline`: metrics, recorded_at, iteration
    - `DiversityAnalysisResult`: metrics, baseline_comparison, warnings, suggestions
    - `DiversityConfig`: enabled, warning_threshold, compute_lexical, compute_structural, compute_semantic
  - [x] 注册 gen-types（见 FSR）
  - [x] 运行 `cd backend && cargo run --bin gen-types` 并提交前端生成类型

- [x] 后端：多样性计算服务逻辑（AC: 1, 2）
  - [x] 创建 `diversity_analyzer` 模块
    - `DiversityAnalyzer` trait 定义
    - `DefaultDiversityAnalyzer` 实现
      - `compute_lexical_diversity(outputs: &[String]) -> f64`：基于 TTR（Type-Token Ratio）
      - `compute_structural_diversity(outputs: &[String]) -> f64`：长度方差 + 格式检测
      - `compute_semantic_diversity(outputs: &[String], embeddings: Option<&[Vec<f64>]>) -> f64`：余弦距离平均值
      - `analyze(outputs: &[String], baseline: Option<&DiversityBaseline>) -> DiversityAnalysisResult`
  - [x] 集成到 **评估流程**（`core/optimization_engine/common.rs::run_tests_and_evaluate`）在 `evaluate_batch` 之后执行
    - 仅在 `task_mode == creative` 且 `diversity_config.enabled == true` 时执行
    - 语义多样性本 Story 默认关闭：`compute_semantic=false` 时 `embeddings=None`

- [x] 后端：多样性配置扩展（AC: 1）
  - [x] 扩展 `OptimizationTaskConfig`
    - 新增 `diversity_config: DiversityConfig` 字段
    - 默认 `enabled: false`（Growth 功能，默认关闭）
  - [x] 同步扩展 `OptimizationTaskConfigStorage`（保持 `schema_version = 1`，保留 `extra` 未知字段）
  - [x] 扩展 `EvaluationResult`
    - 明确扩展位置：`backend/src/domain/models/algorithm.rs::EvaluationResult`
    - 新增 `diversity_analysis: Option<DiversityAnalysisResult>` 字段

- [x] 后端：多样性基准线存储（AC: 2）
  - [x] **新增** `diversity_baselines` 表（不扩展 `optimization_tasks`）
    - `id`, `task_id`（UNIQUE）, `metrics_json`, `recorded_at`, `iteration`
  - [x] 添加 SQLx Migration
  - [x] 实现基准线 CRUD 服务
  - [x] 首次评估自动记录基准线（若已存在则保持不变；手动 POST 用于重置）
    - 判断首次：`diversity_baselines` 中不存在该 `task_id`
    - 自动记录触发点：评估完成后（编排层写入 `EvaluationResult.diversity_analysis` 后）

- [x] 后端：多样性分析 API（AC: 1-3）
  - [x] 新增 `GET /api/v1/tasks/{task_id}/diversity` 获取多样性分析
  - [x] 新增 `POST /api/v1/tasks/{task_id}/diversity/baseline` 记录基准线
  - [x] 权限校验：需登录（`CurrentUser` 提取 user_id）
  - [x] correlationId：从 headers 提取并写入 tracing 日志
  - [x] 路由注册（见 FSR）
    - 在 `backend/src/main.rs` 中 `nest("/api/v1/tasks/{task_id}/diversity", ...)` 挂载受保护路由
  - [x] 添加 OpenAPI 文档描述
  - [x] 在 `docs.rs` 注册新增 path/schema

- [x] 前端：多样性配置入口（AC: 1）
  - [x] 扩展任务配置表单
    - 新增"多样性检测"开关（仅创意任务可见）
  - 新增"告警阈值"输入框（0.1-0.9，默认 0.3）

- [x] 前端：多样性分析展示组件（AC: 2, 3）
  - [x] 创建 `DiversityAnalysisCard.tsx`
    - 多样性评分展示（雷达图或条形图）
    - 基准线对比指示器（改进/退化）
    - 告警消息展示
    - 优化建议展示
  - [x] 创建 `DiversityMetricsChart.tsx`
    - 使用 **recharts** 绘制雷达图（项目当前无图表库依赖，统一选型）
    - 展示三维多样性指标

- [x] 前端：多样性告警组件（AC: 3）
  - [x] 创建 `DiversityWarningBanner.tsx`
    - 告警级别：低/中/高
    - 展示具体告警原因
    - 提供优化建议链接

- [x] 前端：集成到评估结果页面（AC: 2）
  - [x] 扩展 `ResultView.tsx`
    - 创意任务模式下显示多样性分析卡片
    - 与通过率卡片并列展示

- [x] 前端：多样性服务层封装（AC: 1-3）
  - [x] 创建 `diversityService.ts`
    - `getDiversityAnalysis(taskId): Promise<DiversityAnalysisResult>`
    - `recordBaseline(taskId): Promise<DiversityBaseline>`
  - [x] 创建 `hooks/useDiversityAnalysis.ts` TanStack Query hook

- [x] 测试与回归（AC: 1-3）
  - [x] 按 **Testing Requirements** 表执行
  - [x] 新增/覆盖测试文件
    - `backend/tests/diversity_analysis_test.rs`
    - `frontend/src/features/diversity/components/DiversityAnalysisCard.test.tsx`
    - `frontend/src/features/diversity/components/DiversityWarningBanner.test.tsx`

### Hard Gate Checklist

> 必填：跨 Story 硬门禁清单（若不适用请标注 N/A 并说明原因）。

- [x] correlationId 全链路透传（HTTP/WS/日志）
- [x] A2 日志字段齐全（correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp；不适用则标注 N/A）
- [x] 新增/变更类型已运行 gen-types 并提交生成产物
- [x] 状态一致性与幂等性已校验（基准线记录为幂等操作）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review] 已将本 Story review 结论沉淀到 `## Review Notes`（含风险/遗留）
- [x] [AI-Review] 已同步 Story 8.5 完整 File List（见 `## Review Notes / Findings`）
- [x] [AI-Review][HIGH] 多样性分析不应阻塞主评估流程；改为异步或后台任务并回写结果（`backend/src/core/optimization_engine/common.rs:507`）
- [x] [AI-Review][MEDIUM] 基准线读取 50ms 超时后将跳过对比且不记录首笔基准线，需提升可靠性（`backend/src/core/optimization_engine/common.rs:630`）
- [x] [AI-Review][MEDIUM] `compute_semantic` 打开但 embeddings 始终为 None → 语义多样性恒为 0，应显式禁用或标注不可用（`backend/src/core/optimization_engine/common.rs:659`）
- [x] [AI-Review][MEDIUM] 多样性告警阈值在禁用状态仍校验，存在“无法保存以关闭功能”的阻塞风险（`frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx:301`）
- [x] [AI-Review][LOW] GET 多样性分析 404 属于“暂无数据”，前端应走空态而非错误态（`frontend/src/features/diversity/services/diversityService.ts:18`）
- [x] [AI-Review][MEDIUM] 补齐 Dev Agent Record File List（至少包含 `backend/src/infra/db/repositories/iteration_repo.rs`、`frontend/src/components/ui/skeleton.tsx`）（`docs/implementation-artifacts/8-6-creative-task-diversity-detection-growth.md:495`）
- [x] [AI-Review][MEDIUM] 增强多样性分析异步回写的可靠性（延长重试或加入补偿任务），避免迭代产物未就绪时丢失分析结果（`backend/src/core/optimization_engine/common.rs:630`）
- [x] [AI-Review][LOW] 补充多样性 API 的 200/404 与基准线 upsert 路径测试（`backend/tests/diversity_api_test.rs:212`）

## Dev Notes

### Developer Context (Read This First)

- **现状基线（Story 8.1-8.5 已完成）**：
  - 评估器体系已建立（`Evaluator` trait + `DefaultEvaluator`）
  - 质量评估层（Layer 3）已实现
  - 前端元优化模块已建立
  - 任务配置 `OptimizationTaskConfig` 结构已稳定
  - TanStack Query 数据获取模式已统一
  - `task_mode` 存在于 **OptimizationTaskEntity / optimization_tasks 表**（不是 `OptimizationTaskConfig`）

- **业务价值（为什么做）**：创意任务优化时，仅关注通过率可能导致模型输出过于单一（如所有输出都相似）。多样性检测可以帮助用户识别这种问题，确保优化后的 Prompt 既满足质量要求，又保持输出多样性。

- **依赖关系**：
  - 依赖现有评估器体系（`Evaluator` trait）
  - 依赖任务配置扩展（`OptimizationTaskConfig`）
  - 依赖 `EvaluationResult` 结构扩展
  - 可选依赖：embedding 能力（语义多样性计算）
  - 复用 TanStack Query 数据获取模式

- **范围边界（必须遵守）**：
  - 本 Story 实现：多样性分析计算、配置、展示、告警
  - 不包含：自动调整优化策略、embedding 模型训练
  - Growth 功能：默认关闭，用户需手动启用

### 实现路径概述（10 行以内）

- 执行输出评估完成后，在 `core/optimization_engine/common.rs` 汇总 outputs → 调用 `diversity_analyzer.analyze(...)`
- 仅在 `task_mode == creative` 且 `diversity_config.enabled == true` 时执行
- 语义多样性默认关闭（`compute_semantic=false` 且 `embeddings=None`）
- 结果异步写入迭代产物（`IterationArtifacts.diversity_analysis`），由 `GET /diversity` 拉取
- 前端从评估结果拉取并渲染多样性卡片

### 与其他 Story 的关系

| 功能 | Epic 4 | Story 8.6（本 Story） |
| --- | --- | --- |
| 评估器体系 | ✅ 已实现 | 复用 + 扩展 |
| 质量评估层 | ✅ 已实现 | 集成多样性计算 |
| 任务配置 | ✅ 已实现 | 扩展多样性配置 |
| 多样性检测 | - | ✅ 新增 |

### Suggested Data Structures
**关键 DTO 字段（仅列字段，保持简洁）**：
- `DiversityMetrics`: lexical_diversity, structural_diversity, semantic_diversity, overall_score
- `BaselineComparison`: overall_diff, lexical_diff, structural_diff, semantic_diff, trend
- `DiversityWarning`: level, message, affected_metrics
- `DiversitySuggestion`: suggestion_type, content
- `DiversityAnalysisResult`: metrics, baseline_comparison?, warnings, suggestions, analyzed_at, sample_count
- `DiversityBaseline`: id, task_id, metrics, recorded_at, iteration
- `DiversityConfig`: enabled, warning_threshold, compute_lexical, compute_structural, compute_semantic  
  - 默认值：`enabled=false`、`warning_threshold=0.3`、`compute_semantic=false`

**数据库存储建议（SQLite 友好）**：
- `diversity_baselines`：
  - `id` TEXT PRIMARY KEY
  - `task_id` TEXT NOT NULL UNIQUE
  - `metrics_json` TEXT NOT NULL
  - `recorded_at` INTEGER NOT NULL
  - `iteration` INTEGER NOT NULL

**EvaluationResult 扩展位置与顺序**：
- 位置：`backend/src/domain/models/algorithm.rs::EvaluationResult`
- 新增字段：`diversity_analysis: Option<DiversityAnalysisResult>`
- 放置顺序：建议置于 `extra` 之前，并加 `#[serde(skip_serializing_if = "Option::is_none")]`

### Diversity Computation Logic

多样性计算逻辑（后端实现）：

```text
tokenize_for_diversity(text):
  - 若包含中文：去空白 → 2-gram（不足 2 字回退单字）
  - 否则：split_whitespace

compute_lexical_diversity(outputs):
  - 过滤空字符串，len < 2 返回 0
  - tokens -> TTR
  - pairwise 多样性：对 outputs 进行截断/采样（<=50），Jaccard 距离均值
  - 返回 (TTR + pairwise_avg) / 2

compute_structural_diversity(outputs):
  - len < 2 返回 0
  - 长度变异系数 + 格式特征计数 → 归一化后均值

compute_semantic_diversity(outputs, embeddings):
  - embeddings=None 或 len < 2 → 0
  - 否则取 embedding 余弦距离均值
  - embedding 由外部服务注入（本 Story 默认关闭）

compute_overall_score(metrics, semantic_available):
  - 仅对启用且**可用**的维度求平均
  - 若语义多样性未提供 embeddings，则不计入 overall_score
  - overall_score = average(enabled_metrics)

warning_logic(overall_score, warning_threshold):
  - overall_score < warning_threshold → 发出告警
  - Level:
    - < 0.15 → High
    - < 0.25 → Medium
    - 否则 → Low
```

### Suggested API Endpoints

```
# 获取多样性分析（新增）
GET /api/v1/tasks/{task_id}/diversity
Response: ApiResponse<DiversityAnalysisResult>
权限校验：需登录
说明：返回最近一次评估的多样性分析结果

# 记录基准线（新增）
POST /api/v1/tasks/{task_id}/diversity/baseline
Response: ApiResponse<DiversityBaseline>
权限校验：需登录
说明：将当前多样性指标记录为基准线（幂等 upsert：已存在则更新 metrics/recorded_at/iteration）
```

### Frontend Component Notes

**DiversityAnalysisCard.tsx 关键结构：**
- Props：`analysis?: DiversityAnalysisResult`, `error?: string`, `isLoading?: boolean`, `onRetry?: () => void`
- 结构：标题 + 综合评分 → 告警 Banner → 雷达图 → 基准线对比 → 优化建议列表
- Loading：`isLoading` 时显示 Skeleton/占位
- 错误态：`error` 存在时显示“多样性分析暂不可用” + 重试按钮（调用 `onRetry`）

**DiversityWarningBanner.tsx 关键结构：**
- Props：`warnings: DiversityWarning[]`
- 逻辑：计算最高级别（high/medium/low）并映射样式

**UX 对齐**：
- 多样性卡片仅在创意任务模式下显示
- 使用雷达图直观展示三维多样性
- 告警使用颜色区分严重程度
- 基准线对比使用箭头和颜色指示趋势
- 优化建议以列表形式展示

### Dev Agent Guardrails（避免常见踩坑）

- **仅创意任务启用**：检查 `task_mode == "creative"` 且 `diversity_config.enabled == true`
- **默认关闭**：Growth 功能，默认 `enabled: false`
- **样本数量检查**：至少 2 个输出才能计算多样性
- **空输出处理**：输出为空时返回 0.0 而非 panic
- **语义多样性可选**：需要 embedding 支持，默认关闭
- **基准线幂等**：重复记录基准线应更新而非新增
- **中文分词兜底**：`split_whitespace()` 对中文无效，需字符级 n-gram 或轻量分词策略
- **性能护栏**：pairwise 计算 O(n²)，输出样本>50 时采样或跳过 pairwise
- **采样可复现**：若使用随机采样，必须固定种子（保证结果可复现）
- **结果可用性**：多样性分析失败不阻塞主流程，前端提示“分析暂不可用”
- **日志安全**：日志不得包含完整输出内容，仅记录 task_id/metrics
- **性能考量**：多样性计算不应阻塞主评估流程，可异步执行

### Technical Requirements（必须满足）

- 时间戳使用 Unix 毫秒存储，API 返回 ISO 8601
- API 响应使用 `ApiResponse<T>` 统一结构
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`
- 多样性分析失败不得阻塞主评估流程，前端显示“多样性分析暂不可用”
- `GET /diversity` 返回 404 代表“暂无分析数据”，前端走空态而非错误态
- correlationId 缺失时由后端 middleware 生成（前端可不传）
- 多样性配置需持久化到 `config_json`
- 基准线需持久化到数据库

### Backward Compatibility / Non-Regressions（必须遵守）

- 扩展 `OptimizationTaskConfig`，新增 `diversity_config` 字段，默认关闭
- 扩展 `EvaluationResult`，新增 `diversity_analysis` 可选字段
- 新增数据库迁移，添加 `diversity_baselines` 表
- 不影响现有评估流程（多样性计算为可选附加）
- `schema_version` 保持为 1，`config_json` 未知字段保留（extra）
- 旧配置缺少 `diversity_config` 时自动填充默认值

### Previous Story Learnings (Story 8.5 复盘/模式/测试)

- **后端路由模式**：使用 `CurrentUser` 提取器进行权限校验
- **DTO 设计模式**：使用 `#[serde(rename_all = "camelCase")]` + `#[ts(export_to = "models/")]`
- **前端模块结构**：采用 `components/` + `hooks/` + `services/` + `index.ts`
- **测试实践**：使用 MSW + `QueryClientProvider`，通过 `useAuthStore` 注入登录态
- **可选功能模式**：使用配置字段控制功能开关

### Latest Technical Notes（基于当前项目版本）

**Breaking Changes / Best Practices**
- TanStack Query v5：query 使用 `useQuery` hook
- Axum 0.8：路由路径参数语法 `/{param}`
- recharts：雷达图绘制

**Performance / Deprecation Notes**
- 多样性计算可能耗时，建议异步执行或缓存结果
- 语义多样性需要 embedding 计算，资源消耗较大

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/domain/models/diversity_analysis.rs`：多样性 DTO（新增）
  - `backend/src/core/diversity_analyzer/mod.rs`：多样性计算服务（新增）
  - `backend/src/api/routes/diversity.rs`：多样性 API（新增）
  - `backend/src/api/routes/mod.rs`：注册新路由（扩展）
  - `backend/src/main.rs`：挂载 `/api/v1/tasks/{task_id}/diversity`（扩展）
  - `backend/src/domain/models/algorithm.rs`：扩展 `EvaluationResult`（新增字段）
  - `frontend/src/features/diversity/components/DiversityAnalysisCard.tsx`：分析卡片（新增）
  - `frontend/src/features/diversity/components/DiversityMetricsChart.tsx`：雷达图（新增）
  - `frontend/src/features/diversity/components/DiversityWarningBanner.tsx`：告警组件（新增）
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **错误处理**：后端 `thiserror` + `anyhow`
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types`

### Library / Framework Requirements (Version Snapshot)

- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`
- tokio：异步运行时
- chrono：时间戳处理
- React：`react@19.x`
- TanStack Query：服务端状态管理
- shadcn/ui：UI 组件库
- recharts：图表库（本 Story 统一选型，需新增依赖）
  - 引入时锁定版本：`^2.12.2`（与 React 19 兼容）

### Deployment / Environment Notes（部署/环境）

- 本 Story 新增数据库迁移（`diversity_baselines` 表）
- 前端需新增图表库依赖（recharts）
- 部署验证：建议执行 `cargo test`、`pnpm vitest run`、`pnpm vite build`

### File Structure Requirements（落点约束）

**后端**：
- 多样性 DTO：`backend/src/domain/models/diversity_analysis.rs`（新增）
- 多样性分析服务：`backend/src/core/diversity_analyzer/mod.rs`（新增）
- 多样性 API：`backend/src/api/routes/diversity.rs`（新增）
- 路由注册：`backend/src/api/routes/mod.rs`（扩展）
- 路由挂载：`backend/src/main.rs`（扩展，`/api/v1/tasks/{task_id}/diversity`）
- 任务配置扩展：`backend/src/domain/models/optimization_task_config.rs`（扩展）
- 评估结果扩展：`backend/src/domain/models/algorithm.rs`（扩展）
- OpenAPI：`backend/src/api/routes/docs.rs`（扩展）
- 类型生成：`backend/src/bin/gen-types.rs`（扩展）
- 数据库迁移：`backend/migrations/016_add_diversity_baselines.sql`（新增，按递增编号）

**前端**：
- 分析卡片：`frontend/src/features/diversity/components/DiversityAnalysisCard.tsx`（新增）
- 雷达图：`frontend/src/features/diversity/components/DiversityMetricsChart.tsx`（新增）
- 告警组件：`frontend/src/features/diversity/components/DiversityWarningBanner.tsx`（新增）
- 服务层：`frontend/src/features/diversity/services/diversityService.ts`（新增）
- Hook：`frontend/src/features/diversity/hooks/useDiversityAnalysis.ts`（新增）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

**测试**：
- 后端测试：`backend/tests/diversity_analysis_test.rs`（新增）
- 分析卡片测试：`frontend/src/features/diversity/components/DiversityAnalysisCard.test.tsx`（新增）
- 告警组件测试：`frontend/src/features/diversity/components/DiversityWarningBanner.test.tsx`（新增）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 词汇多样性 | 正确计算 TTR 和跨输出多样性 |
| 后端单测 | 结构多样性 | 正确计算长度变异和格式多样性 |
| 后端单测 | 告警生成 | 低于阈值时正确生成告警 |
| 后端单测 | 基准线 CRUD | 创建/读取/更新基准线 |
| 后端单测 | 空输入处理 | 空数组返回 0.0 |
| 后端单测 | 小样本处理 | 仅 1 条输出返回 0.0（不 panic） |
| 后端单测 | 中文分词兜底 | 中文输出不退化为单 token（字符级或 n-gram） |
| 后端单测 | 性能护栏 | 输出样本过多时采样/跳过 pairwise |
| 后端单测 | 幂等语义 | 重复 POST baseline 更新并返回最新 |
| 后端单测 | 语义多样性关闭 | compute_semantic=false 时 embeddings=None |
| 后端单测 | 非创意任务 | task_mode=fixed 不执行多样性计算 |
| 后端单测 | 权限校验 | 非自己的任务返回 403 |
| 前端测试 | 分析卡片 | 正确渲染多样性指标 |
| 前端测试 | 雷达图 | 正确绑定数据 |
| 前端测试 | 告警展示 | 告警级别颜色正确 |
| 前端测试 | 基准线对比 | 改进/退化趋势正确显示 |
| 前端测试 | 失败兜底 | 分析失败提示“多样性分析暂不可用” |
| 前端测试 | Loading 状态 | 加载时 Skeleton/占位正确显示 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

**自检命令（建议）**：
```
cargo test --test diversity_analysis_test
pnpm vitest run --grep diversity
cargo run --bin gen-types && git diff --exit-code frontend/src/types/generated/
```

### Project Structure Notes

- 参考 `frontend/src/features/meta-optimization/` 模块结构
- 参考 `backend/src/core/evaluator/` 评估器模块
- 参考 `backend/src/api/routes/meta_optimization.rs` API 路由模式
- 多样性模块可独立于现有评估器，作为可选附加功能

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 8 / Story 8.6）
- PRD 创意任务：`docs/project-planning-artifacts/prd.md#能力区域 4: 自动迭代优化`（FR34）
- 架构（评估器）：`docs/project-planning-artifacts/architecture.md`
- 评估器模块：`backend/src/core/evaluator/mod.rs`
- 任务配置模型：`backend/src/domain/models/optimization_task_config.rs`
- Story 8.5（前序）：`docs/implementation-artifacts/8-5-prompt-version-comparison-growth.md`

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

- `cargo run --bin gen-types`
- `cargo test`
- `npm test -- --run`
- `npm run lint`
- `npm run build`

### Completion Notes List

- 落地多样性 DTO/配置与分析器，扩展 EvaluationResult 与迭代产物，并在评估后按 creative + enabled 计算与写入结果。
- 新增多样性基准线表与仓储，自动首笔记录并提供基准线重置 API，补齐路由与 OpenAPI 描述。
- 前端新增多样性配置入口与展示卡片（雷达图/告警/建议/基准线对比），集成结果页并补齐服务层与 Hook。
- 更新生成类型与现有用例数据结构，补齐后端/前端测试并完成全量回归、lint 与 build。
- 修复 review 发现项：多样性分析后台计算回写、基准线读取可靠性、语义多样性可用性、前端校验与 404 空态。

### File List

- backend/migrations/016_add_diversity_baselines.sql
- backend/src/api/routes/diversity.rs
- backend/src/api/routes/docs.rs
- backend/src/api/routes/mod.rs
- backend/src/api/routes/optimization_tasks.rs
- backend/src/bin/gen-types.rs
- backend/src/core/diversity_analyzer/mod.rs
- backend/src/core/evaluator/default_impl.rs
- backend/src/core/evaluator/example_impl.rs
- backend/src/core/feedback_aggregator/default_impl.rs
- backend/src/core/iteration_engine/checkpoint.rs
- backend/src/core/iteration_engine/orchestrator.rs
- backend/src/core/iteration_engine/pause_state.rs
- backend/src/core/iteration_engine/recovery.rs
- backend/src/core/mod.rs
- backend/src/core/optimization_engine/common.rs
- backend/src/core/rule_engine/default_impl.rs
- backend/src/domain/models/algorithm.rs
- backend/src/domain/models/diversity_analysis.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/models/optimization_task_config.rs
- backend/src/domain/types/artifacts.rs
- backend/src/domain/types/extensions.rs
- backend/src/domain/types/mod.rs
- backend/src/infra/db/repositories/diversity_baseline_repo.rs
- backend/src/infra/db/repositories/iteration_repo.rs
- backend/src/infra/db/repositories/mod.rs
- backend/src/main.rs
- backend/tests/diversity_analysis_test.rs
- backend/tests/diversity_api_test.rs
- docs/implementation-artifacts/8-5-prompt-version-comparison-growth.md
- docs/implementation-artifacts/8-6-creative-task-diversity-detection-growth.md
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/validation-report-20260121-230921.md
- frontend/package-lock.json
- frontend/package.json
- frontend/src/App.routes.test.tsx
- frontend/src/components/ui/skeleton.tsx
- frontend/src/features/diversity/components/DiversityAnalysisCard.test.tsx
- frontend/src/features/diversity/components/DiversityAnalysisCard.tsx
- frontend/src/features/diversity/components/DiversityMetricsChart.test.tsx
- frontend/src/features/diversity/components/DiversityMetricsChart.tsx
- frontend/src/features/diversity/components/DiversityWarningBanner.test.tsx
- frontend/src/features/diversity/components/DiversityWarningBanner.tsx
- frontend/src/features/diversity/hooks/useDiversityAnalysis.ts
- frontend/src/features/diversity/index.ts
- frontend/src/features/diversity/services/diversityService.ts
- frontend/src/features/diversity/utils/diversityWarning.ts
- frontend/src/features/result-viewer/components/ResultView.tsx
- frontend/src/features/user-intervention/ArtifactEditor.test.tsx
- frontend/src/features/user-intervention/ArtifactEditor.tsx
- frontend/src/features/user-intervention/history/HistoryDetailView.test.tsx
- frontend/src/features/user-intervention/history/HistoryPanel.test.tsx
- frontend/src/features/user-intervention/history/IterationHistoryItem.test.tsx
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.test.tsx
- frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx
- frontend/src/pages/RunView/RunView.tsx
- frontend/src/types/generated/api/UpdateOptimizationTaskConfigRequest.ts
- frontend/src/types/generated/models/BaselineComparison.ts
- frontend/src/types/generated/models/DiversityAnalysisResult.ts
- frontend/src/types/generated/models/DiversityBaseline.ts
- frontend/src/types/generated/models/DiversityConfig.ts
- frontend/src/types/generated/models/DiversityMetrics.ts
- frontend/src/types/generated/models/DiversitySuggestion.ts
- frontend/src/types/generated/models/DiversityTrend.ts
- frontend/src/types/generated/models/DiversityWarning.ts
- frontend/src/types/generated/models/DiversityWarningLevel.ts
- frontend/src/types/generated/models/EvaluationResult.ts
- frontend/src/types/generated/models/IterationArtifacts.ts
- frontend/src/types/generated/models/OptimizationTaskConfig.ts

### Change Log

- 2026-01-22: 完成多样性分析后端/前端实现、基准线存储与 API、类型生成与全量回归（含 lint/build）。
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] 多样性分析在主评估路径同步执行，违背“计算不阻塞主流程”的关键决策（`backend/src/core/optimization_engine/common.rs`）。
- [x] [MEDIUM] 基准线读取超时/失败会跳过首笔基准线落库，导致后续对比缺失（`backend/src/core/optimization_engine/common.rs`）。
- [x] [MEDIUM] `compute_semantic` 可被开启但 embeddings 恒为 None → 语义多样性恒为 0，结果具有误导性（`backend/src/core/optimization_engine/common.rs` / `backend/src/core/diversity_analyzer/mod.rs`）。
- [x] [MEDIUM] Dev Agent Record 的 File List 缺少实际变更文件，影响审计与复盘（`docs/implementation-artifacts/8-6-creative-task-diversity-detection-growth.md:495`）。
- [x] [MEDIUM] 多样性分析写回仅重试 3 次且总等待 < 1s，迭代产物延迟落库时结果可能丢失（`backend/src/core/optimization_engine/common.rs:630`）。
- [x] [MEDIUM] 多样性检测关闭时仍校验告警阈值，异常配置会阻塞保存（`frontend/src/pages/OptimizationTaskConfigView/OptimizationTaskConfigView.tsx`）。
- [x] [LOW] 多样性分析 404 被当作错误处理，UI 进入错误态而非空态（`frontend/src/features/diversity/services/diversityService.ts`）。
- [x] [LOW] 多样性 API 测试仅覆盖 403，缺少 200/404 与基准线 upsert 的集成覆盖（`backend/tests/diversity_api_test.rs:212`）。
- [x] [HIGH] 明确 `EvaluationResult` 扩展位置与序列化策略（新增字段置于 `extra` 之前并 `skip_serializing_if`）。
- [x] [HIGH] 基准线自动记录的触发条件与时机明确（首次记录、评估完成后写入）。
- [x] [MEDIUM] 多样性计算逻辑由长代码块精简为伪代码，保留中文分词与采样护栏要点。
- [x] [MEDIUM] 前端卡片补充 Loading/Skeleton 与失败重试提示说明。
- [x] [MEDIUM] 前序反馈摘要（Story 8.5）：差异用例高亮、摘要统计一致性、per-case error 透出、超时边界处理、请求取消、速率限制等已在前序 Story 固化。
- [x] [LOW] 前序文件清单（Story 8.5，完整）：
```
backend/src/api/routes/docs.rs
backend/src/api/routes/meta_optimization.rs
backend/src/bin/gen-types.rs
backend/src/core/meta_optimization_service/mod.rs
backend/src/domain/models/mod.rs
backend/src/domain/models/teacher_prompt.rs
backend/src/shared/error_codes.rs
backend/tests/meta_optimization_test.rs
frontend/src/features/meta-optimization/components/CaseComparisonList.test.tsx
frontend/src/features/meta-optimization/components/CaseComparisonList.tsx
frontend/src/features/meta-optimization/components/CompareResultSummary.tsx
frontend/src/features/meta-optimization/components/PromptComparePanel.test.tsx
frontend/src/features/meta-optimization/components/PromptComparePanel.tsx
frontend/src/features/meta-optimization/components/PromptDiffViewer.test.tsx
frontend/src/features/meta-optimization/components/PromptDiffViewer.tsx
frontend/src/features/meta-optimization/components/PromptVersionList.test.tsx
frontend/src/features/meta-optimization/components/PromptVersionList.tsx
frontend/src/features/meta-optimization/hooks/usePromptCompare.ts
frontend/src/features/meta-optimization/index.ts
frontend/src/features/meta-optimization/services/metaOptimizationService.ts
frontend/src/pages/MetaOptimizationPage.tsx
frontend/src/lib/api.ts
frontend/src/types/generated/models/CaseComparisonResult.ts
frontend/src/types/generated/models/CompareSummary.ts
frontend/src/types/generated/models/PromptCompareRequest.ts
frontend/src/types/generated/models/PromptCompareResponse.ts
frontend/src/types/generated/models/VersionCompareResult.ts
docs/implementation-artifacts/8-4-advanced-user-edit-teacher-model-prompt.md
docs/implementation-artifacts/8-5-prompt-version-comparison-growth.md
docs/implementation-artifacts/validation-report-20260121-121630.md
docs/implementation-artifacts/sprint-status.yaml
```

### Decisions

- [x] 选择在文档层提供伪代码而非完整实现，降低 token 噪音但保留关键算法约束。
- [x] 将路由挂载方式明确到 `main.rs` 的 `nest("/api/v1/tasks/{task_id}/diversity", ...)`，避免实现分歧。
- [x] 吸收 Story 8.5 复盘：优先复用既有服务层/Hook/测试模式，避免重复造轮子。

### Risks / Tech Debt

- [x] 多样性分析同步执行可能拖慢评估路径，样本量上升后风险更高。
- [x] 基准线读取超时导致“无基准线可对比”长期存在。
- [ ] 若后续引入真实分词库或更复杂采样策略，需同步更新测试基准与性能阈值。
- [ ] 若前序 Story 的变更文件未同步检索，可能遗漏可复用实现（需在落地时对照 8.5 File List）。

### Follow-ups

- [x] 将本次更新同步到 `### Review Follow-ups (AI)`（如需额外行动项）
- [ ] 实施前快速回顾 Story 8.5 的 Review Notes，确保复用逻辑与错误处理模式一致
