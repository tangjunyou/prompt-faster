# Story 8.3: 元优化基础（老师模型 Prompt 优化）

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 8-3-meta-optimization-basics

## Epic 8 概述

> **Epic 8: 结果输出与元优化** - 用户成果：用户可以查看、导出优化结果，查看诊断报告，并使用元优化功能优化老师模型 Prompt。

**Epic 8 Story 列表**：
- 8.1 结果查看与导出（FR60, FR61，NFR18）- ✅ done
- 8.2 诊断报告（FR63）- ✅ done
- **8.3 元优化基础（本 Story，FR56, FR57, FR58）** - MVP
- 8.4 高级用户直接编辑老师模型 Prompt（FR59）- MVP
- 8.5 Prompt 版本对比（FR62）- Growth
- 8.6 创意任务多样性检测（FR34）- Growth

## Key Decisions (MVP)

- **元优化范围**：本 Story 聚焦老师模型 Prompt 版本管理与成功率统计，不包含高级编辑功能（8.4）。
- **版本管理策略**：每次老师模型 Prompt 变更自动创建新版本，保留完整历史。
- **成功率统计**：仅基于显式关联到版本的任务统计（`optimization_tasks.teacher_prompt_version_id`），不做时间推断；无关联任务显示 `success_rate = null`。
- **数据模型**：新增 `teacher_prompts` 表存储版本，与现有 `teacher_model_settings` 表分离。
- **版本关联策略**：任务与版本使用外键字段显式绑定（`teacher_prompt_version_id`），避免回滚/切换导致统计错乱。
- **元优化入口**：老师模型 Prompt 可作为优化目标，使用用户历史任务作为测试集。
- **权限校验**：仅用户自己的 Prompt 版本可见/可操作。
- **AR2 遵循**：所有操作记录 correlationId，支持全链路追踪。

## Story

As a 高级用户,
I want 将老师模型 Prompt 作为优化目标并追踪版本效果,
so that 我可以优化老师模型本身，提升整体优化质量。

## Acceptance Criteria

1. **Given** 用户进入元优化模式
   **When** 选择"优化老师模型 Prompt"
   **Then** 系统将老师模型 Prompt 作为优化目标（FR56）
   **And** 展示用户历史任务作为测试集（MVP：仅数据展示/选择入口，不执行真实元优化流程）

2. **Given** 老师模型 Prompt 发生变更
   **When** 保存变更
   **Then** 系统持久化该版本（FR57）
   **And** 记录版本号、时间戳、变更说明

3. **Given** 用户查看老师模型 Prompt 版本列表
   **When** 打开版本管理页面
   **Then** 显示所有历史版本
   **And** 每个版本显示成功率统计（FR58）

## Tasks / Subtasks

- 文件落点以 **File Structure Requirements** 为准；本节只描述职责，避免重复写路径。

- [x] 后端：数据库迁移（AC: 2,3）
  - [x] 创建新迁移文件 `014_teacher_prompts.sql`
    - `teacher_prompts` 表：id, user_id, version, content, description, is_active, created_at, updated_at
    - 约束：UNIQUE(user_id, version)
    - 索引：idx_teacher_prompts_user_id, idx_teacher_prompts_user_active
  - [x] 创建新迁移文件 `015_add_teacher_prompt_version_id_to_optimization_tasks.sql`
    - `optimization_tasks` 新增字段：teacher_prompt_version_id (TEXT, nullable)
    - 外键：teacher_prompt_version_id → teacher_prompts(id)
    - 索引：idx_optimization_tasks_teacher_prompt_version_id

- [x] 后端：任务版本关联写入（AC: 1,3）
  - [x] 元优化入口创建任务时，读取当前 active prompt，写入 `optimization_tasks.teacher_prompt_version_id`
  - [x] 非元优化入口创建任务保持 NULL（不影响既有任务）

- [x] 后端：老师模型 Prompt 数据模型定义（AC: 2,3）
  - [x] 创建 `teacher_prompt.rs`（路径见 File Structure Requirements）
    - **以本 Story 的 “Suggested Data Structures” 为唯一权威定义（避免重复/冲突）**
    - 包含：`TeacherPrompt` / `TeacherPromptVersion` / `TeacherPromptStats` / `CreateTeacherPromptInput` / `MetaOptimizationOverview`
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增类型

- [x] 后端：老师模型 Prompt 仓储（AC: 2,3）
  - [x] 创建 `teacher_prompt_repo.rs`（路径见 File Structure Requirements）
    - `create(user_id, input) -> TeacherPrompt`
    - `list_by_user(user_id) -> Vec<TeacherPromptVersion>`
    - `find_by_id(id, user_id) -> TeacherPrompt`
    - `find_active(user_id) -> Option<TeacherPrompt>`
    - `set_active(id, user_id) -> TeacherPrompt`
    - `calculate_stats(version_id, user_id) -> TeacherPromptStats`
    - 统计口径基于 `optimization_tasks.teacher_prompt_version_id`（不做时间推断）

- [x] 后端：元优化服务（AC: 1,2,3）
  - [x] 创建 `meta_optimization_service/mod.rs`（路径见 File Structure Requirements）
    - `create_prompt_version(user_id, input) -> TeacherPromptVersion`
    - `list_prompt_versions(user_id) -> Vec<TeacherPromptVersion>`
    - `get_active_prompt(user_id) -> Option<TeacherPrompt>`
    - `set_active_prompt(user_id, version_id) -> TeacherPrompt`
    - `get_overview(user_id) -> MetaOptimizationOverview`
    - `get_historical_tasks_for_meta_optimization(user_id) -> Vec<OptimizationTaskSummary>`（仅用于展示历史任务列表）

- [x] 后端：元优化 API 实现（AC: 1,2,3）
  - [x] 创建 `meta_optimization.rs`（路径见 File Structure Requirements）
    - `POST /api/v1/meta-optimization/prompts` 创建新版本
    - `GET /api/v1/meta-optimization/prompts` 获取版本列表（含成功率）
  - `GET /api/v1/meta-optimization/prompts/{id}` 获取版本详情
  - `PUT /api/v1/meta-optimization/prompts/{id}/activate` 设为活跃版本
  - `GET /api/v1/meta-optimization/stats` 获取统计概览
  - `GET /api/v1/meta-optimization/tasks` 获取历史任务列表（元优化选择入口）
  - [x] 权限校验：仅当前用户可操作自己的 Prompt 版本
  - [x] correlationId：从 headers 提取并写入 tracing 日志（参考 `diagnostic.rs` 模式）
  - [x] 添加 OpenAPI 文档描述
  - [x] 在 `mod.rs` 注册新路由
  - [x] 在 `backend/src/main.rs` 挂载路由到 `/api/v1/meta-optimization`
  - [x] 在 `backend/src/api/routes/docs.rs` 注册 OpenAPI path/schema

- [x] 前端：meta-optimization feature 目录结构（AC: 1-3）
  - [x] 创建 `meta-optimization/` 目录（路径见 File Structure Requirements）
  - [x] 创建 `index.ts` 导出模块
  - [x] 创建 `components/`、`hooks/`、`services/` 子目录

- [x] 前端：版本列表组件（AC: 3）
  - [x] 创建 `PromptVersionList.tsx`（路径见 File Structure Requirements）
    - 列表展示所有版本
    - 每行显示版本号、描述、成功率、创建时间
    - 高亮当前活跃版本
    - 支持点击设为活跃版本

- [x] 前端：版本详情组件（AC: 2,3）
  - [x] 创建 `PromptVersionDetail.tsx`（路径见 File Structure Requirements）
    - 显示 Prompt 完整内容（只读，编辑功能在 8.4）
    - 显示版本元信息
    - 显示成功率统计图表

- [x] 前端：统计概览组件（AC: 3）
  - [x] 创建 `MetaOptimizationStats.tsx`（路径见 File Structure Requirements）
    - 显示各版本成功率对比
    - 简单柱状图或趋势线

- [x] 前端：服务层封装（AC: 1-3）
  - [x] 创建 `metaOptimizationService.ts`（路径见 File Structure Requirements）
    - `getPromptVersions(): Promise<TeacherPromptVersion[]>`
    - `getPromptVersion(id: string): Promise<TeacherPrompt>`
    - `createPromptVersion(input: CreateTeacherPromptInput): Promise<TeacherPromptVersion>`
    - `activatePromptVersion(id: string): Promise<TeacherPrompt>`
    - `getOverview(): Promise<MetaOptimizationOverview>`
- [x] 创建 `hooks/usePromptVersions.ts` TanStack Query hook
- [x] 创建 `hooks/useMetaOptimizationOverview.ts` query hook
- [x] 创建 `hooks/useMetaOptimizationTasks.ts` query hook（历史任务入口）

- [x] 前端：页面入口与路由（AC: 1-3）
  - [x] 在 `frontend/src/pages/` 添加 MetaOptimizationPage 或集成到 WorkspaceView
  - [x] 添加导航入口（侧边栏或顶部菜单）
  - [x] 配置路由 `/meta-optimization`

- [x] 测试与回归（AC: 1-3）
  - [x] 按 **Testing Requirements** 表执行
  - [x] 新增/覆盖测试文件
    - `backend/tests/meta_optimization_test.rs`（集成测试）
    - `backend/src/core/meta_optimization_service/mod.rs`（单元测试）
    - `frontend/src/features/meta-optimization/components/PromptVersionList.test.tsx`

### Hard Gate Checklist

> 必填：跨 Story 硬门禁清单（若不适用请标注 N/A 并说明原因）。

- [x] correlationId 全链路透传（HTTP/WS/日志）
- [x] A2 日志字段齐全（元优化场景：correlation_id/user_id/version_id/action/prev_state/new_state/timestamp；task_id/iteration_state 标注 N/A）
- [x] 新增/变更类型已运行 gen-types 并提交生成产物
- [x] 状态一致性与幂等性已校验（如 is_active 切换幂等）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review][CRITICAL] 补齐 Testing Requirements：无任务 success_rate=null、并发版本号、分页 limit/offset、前端空态测试；并完成回归（`cargo test meta_optimization` / `vitest`）。
- [x] [AI-Review][HIGH] 修复 CreateOptimizationTaskRequest 新增字段导致的前端构建问题：创建任务 payload 必须包含 `meta_optimization`（至少为 `null`）。
- [x] [AI-Review][HIGH] 增加“优化老师模型 Prompt”入口并创建元优化任务时传 `meta_optimization.use_active_teacher_prompt = true`，确保写入 `teacher_prompt_version_id`（入口提供、未启用执行）。
- [x] [AI-Review][HIGH] 增加“创建 Prompt 版本/变更说明”UI 并调用 `createPromptVersion`，落实 AC2。
- [x] [AI-Review][MEDIUM] 历史任务测试集按用户展示/选择；补充 API（`get_historical_tasks_for_meta_optimization`）并在前端接入。
- [x] [AI-Review][MEDIUM] 优化统计查询性能：避免 `list_prompt_versions` + `calculate_stats` 的 N+1 查询放大。
- [x] [AI-Review][MEDIUM] Story File List 与 git 不一致：补充实际变更文件（如 `docs/implementation-artifacts/8-2-diagnostic-report.md`、`docs/project-planning-artifacts/architecture.md`、`docs/project-planning-artifacts/epics.md`、`docs/implementation-artifacts/validation-report-2026-01-20-184521.md`）。

## Dev Notes

### Developer Context (Read This First)

- **现状基线（Epic 1-7 + Story 8.1-8.2 已完成）**：
  - 老师模型参数配置已实现（`backend/src/infra/db/repositories/teacher_settings_repo.rs`）
  - `TeacherModel` trait 已定义（`backend/src/core/traits.rs`）
  - 诊断报告已实现（`backend/src/core/diagnostic_service/`）
  - TanStack Query 模式已建立
  - 用户认证与数据隔离机制已就绪

- **业务价值（为什么做）**：元优化允许用户优化老师模型本身的 Prompt，形成"优化系统优化自己"的闭环，持续提升整体优化质量。版本管理和成功率统计帮助用户理解哪个版本效果最好。

- **依赖关系**：
  - 依赖 `optimization_tasks` + `iterations` 表获取任务 pass_rate
  - 依赖 `optimization_tasks.teacher_prompt_version_id` 进行版本关联
  - 依赖现有用户认证机制
  - 复用 TanStack Query 数据获取模式
  - 复用 Story 8.2 的模块结构模式

- **范围边界（必须遵守）**：
  - 本 Story 实现：版本创建、版本列表、成功率统计、活跃版本切换
  - 不包含：高级编辑功能（8.4）、版本对比（8.5）
  - 元优化执行流程：本 Story 仅提供入口与数据模型；历史任务仅用于展示/选择，不执行真实元优化流程（执行与 TeacherModel 绑定在 8.4）

### 与其他 Story 的关系

| 功能 | Story 8.2 | Story 8.3（本 Story） | Story 8.4 |
| --- | --- | --- | --- |
| 诊断报告 | ✅ 已实现 | - | - |
| Prompt 版本管理 | - | ✅ 新增 | 复用 |
| 版本成功率统计 | - | ✅ 新增 | 复用 |
| 高级编辑 | - | - | 新增 |

### Database Schema (新增)

```sql
-- 老师模型 Prompt 版本表
-- Story 8.3: 元优化基础
CREATE TABLE IF NOT EXISTS teacher_prompts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    version INTEGER NOT NULL,           -- 版本号，用户维度自增
    content TEXT NOT NULL,              -- Prompt 完整内容
    description TEXT,                   -- 版本变更说明
    is_active INTEGER NOT NULL DEFAULT 0, -- 是否为当前活跃版本（0/1）
    created_at INTEGER NOT NULL,        -- Unix 毫秒时间戳 (AR3)
    updated_at INTEGER NOT NULL,        -- Unix 毫秒时间戳 (AR3)
    UNIQUE(user_id, version)            -- 每个用户的版本号唯一
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_teacher_prompts_user_id ON teacher_prompts(user_id);
CREATE INDEX IF NOT EXISTS idx_teacher_prompts_user_active ON teacher_prompts(user_id, is_active);

-- 优化任务与 Prompt 版本关联（显式绑定）
ALTER TABLE optimization_tasks ADD COLUMN teacher_prompt_version_id TEXT;
CREATE INDEX IF NOT EXISTS idx_optimization_tasks_teacher_prompt_version_id
    ON optimization_tasks(teacher_prompt_version_id);
```

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/models/teacher_prompt.rs（新增）

// 本节为唯一权威定义，Tasks/其他段落不再重复字段清单

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 老师模型 Prompt（数据库完整记录）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TeacherPrompt {
    pub id: String,
    pub user_id: String,
    pub version: i32,
    pub content: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,   // ISO 8601
    pub updated_at: String,   // ISO 8601
}

/// 老师模型 Prompt 版本摘要（列表展示用）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TeacherPromptVersion {
    pub id: String,
    pub version: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub success_rate: Option<f64>,  // 该版本的平均成功率
    pub task_count: i32,            // 使用该版本完成的任务数
    pub created_at: String,         // ISO 8601
}

/// 版本统计信息
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TeacherPromptStats {
    pub version_id: String,
    pub version: i32,
    pub total_tasks: i32,
    pub successful_tasks: i32,       // pass_rate >= 1.0 的任务数
    pub success_rate: Option<f64>,   // successful_tasks / total_tasks（无任务为 None）
    pub average_pass_rate: Option<f64>, // 所有任务的平均通过率（无任务为 None）
}

/// 创建新版本的输入
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CreateTeacherPromptInput {
    pub content: String,
    pub description: Option<String>,
    /// 是否立即设为活跃版本（默认 true）
    #[serde(default = "default_activate")]
    pub activate: bool,
}

fn default_activate() -> bool {
    true
}

/// 元优化概览
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct MetaOptimizationOverview {
    pub total_versions: i32,
    pub active_version: Option<TeacherPromptVersion>,
    pub best_version: Option<TeacherPromptVersion>,  // 成功率最高的版本
    pub stats: Vec<TeacherPromptStats>,
}

/// 元优化历史任务摘要（选择入口展示）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct MetaOptimizationTaskSummary {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub status: String,
    pub pass_rate: Option<f64>,
    pub created_at: String, // ISO 8601
}
```

### Success Rate Calculation Logic (必须遵守)

**成功率统计规则：**
- 统计范围：`optimization_tasks.teacher_prompt_version_id = {version_id}` 的任务
- 任务 pass_rate 口径：优先取 `optimization_tasks.selected_iteration_id` 对应的 `iterations.pass_rate`；否则取该任务最新 completed 迭代的 `iterations.pass_rate`
- 成功任务：`pass_rate >= 1.0` 的任务
- 成功率：`successful_tasks / total_tasks`
- 平均通过率：所有任务的 `pass_rate` 平均值
- 边界处理：无任务时 success_rate = null，task_count = 0

**版本关联规则（MVP 简化方案）：**
- 任务与版本通过 `optimization_tasks.teacher_prompt_version_id` 显式关联（不做时间推断）
- 未关联任务不参与统计；版本无任务时显示 `success_rate = null`
- 元优化入口创建的任务需写入当前 active 版本 id（其余入口保持 NULL）

**版本号规则：**
- 用户维度自增：每个用户独立的版本号序列
- 新建版本时：`MAX(version) + 1 WHERE user_id = ?`
- 第一个版本：version = 1

### Suggested API Endpoints

```
# 创建新版本
POST /api/v1/meta-optimization/prompts
Request: CreateTeacherPromptInput
Response: ApiResponse<TeacherPromptVersion>
权限校验：需登录

# 获取版本列表（含成功率）
GET /api/v1/meta-optimization/prompts
Response: ApiResponse<Vec<TeacherPromptVersion>>
权限校验：仅返回当前用户的版本
Query Params:
  - limit: i32 (default: 50, max: 100)
  - offset: i32 (default: 0)

# 获取版本详情
GET /api/v1/meta-optimization/prompts/{id}
Response: ApiResponse<TeacherPrompt>
权限校验：仅版本所有者可访问

# 设为活跃版本
PUT /api/v1/meta-optimization/prompts/{id}/activate
Response: ApiResponse<TeacherPrompt>
权限校验：仅版本所有者可操作

# 获取统计概览
GET /api/v1/meta-optimization/stats
Response: ApiResponse<MetaOptimizationOverview>
权限校验：仅返回当前用户数据

# 获取历史任务列表（元优化选择入口）
GET /api/v1/meta-optimization/tasks
Response: ApiResponse<Vec<MetaOptimizationTaskSummary>>
权限校验：仅返回当前用户任务
Query Params:
  - limit: i32 (default: 50, max: 100)
  - offset: i32 (default: 0)
```

### Frontend Component Notes

**PromptVersionList.tsx 结构：**
```tsx
// 版本列表
<PromptVersionListContainer>
  {versions.map(version => (
    <VersionCard 
      key={version.id}
      version={version}
      isActive={version.isActive}
      onActivate={handleActivate}
      onClick={handleViewDetail}
    />
  ))}
</PromptVersionListContainer>

// VersionCard 内容
<VersionCard>
  <VersionNumber>v{version.version}</VersionNumber>
  <Description>{version.description || '无描述'}</Description>
  <SuccessRate rate={version.successRate} count={version.taskCount} />
  <Timestamp>{version.createdAt}</Timestamp>
  {version.isActive && <ActiveBadge>当前使用</ActiveBadge>}
</VersionCard>
```

**MetaOptimizationStats.tsx 使用图表：**
```tsx
import { BarChart, Bar, XAxis, YAxis, Tooltip } from 'recharts';

// 或使用更轻量的方案
<SimpleBarChart 
  data={stats.map(s => ({ 
    name: `v${s.version}`, 
    successRate: s.successRate * 100 
  }))}
/>
```

**UX 对齐**：
- 元优化入口位于侧边栏或设置页面
- 版本列表按创建时间倒序排列
- 活跃版本高亮显示
- 成功率使用百分比展示
- 历史任务列表支持选择入口（checkbox/selection state）

### Dev Agent Guardrails（避免常见踩坑）

- **版本号生成**：使用数据库原子操作获取下一个版本号，避免并发问题
- **活跃版本切换**：使用事务确保只有一个活跃版本（先清除所有 is_active，再设置目标）
- **成功率计算**：考虑无任务时返回 null 而非 0
- **版本关联**：只使用 `teacher_prompt_version_id` 显式绑定，不做时间推断
- **分页保护**：版本列表必须有上限（默认 50，最大 100）
- **空态处理**：无版本时显示引导创建界面
- **日志安全**：日志不得包含 Prompt 完整内容，仅记录 id/version

**SQL Patterns（可选，减少歧义）**

```sql
-- 原子生成版本号并插入（事务内）
BEGIN TRANSACTION;
INSERT INTO teacher_prompts (id, user_id, version, content, description, is_active, created_at, updated_at)
SELECT ?1, ?2, COALESCE(MAX(version), 0) + 1, ?3, ?4, ?5, ?6, ?6
FROM teacher_prompts WHERE user_id = ?2;
COMMIT;

-- 切换活跃版本（事务内）
BEGIN TRANSACTION;
UPDATE teacher_prompts SET is_active = 0 WHERE user_id = ?1;
UPDATE teacher_prompts SET is_active = 1 WHERE id = ?2 AND user_id = ?1;
COMMIT;
```

### Technical Requirements（必须满足）

- 时间戳使用 Unix 毫秒存储，API 返回 ISO 8601
- API 响应使用 `ApiResponse<T>` 统一结构
- 版本列表默认按 version desc 排序
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`
- is_active 字段使用 INTEGER (0/1) 存储，API 返回 boolean
- 成功率统计仅使用 `optimization_tasks.teacher_prompt_version_id` + `iterations.pass_rate`，不做时间推断

### Backward Compatibility / Non-Regressions（必须遵守）

- 新增 `teacher_prompts` 表，不修改现有 `teacher_model_settings` 表
- `optimization_tasks` 仅新增可空字段 `teacher_prompt_version_id`（向后兼容）
- 新增 `/api/v1/meta-optimization/*` 端点，不修改现有 API
- 与 `TeacherModel` trait 解耦，本 Story 仅管理 Prompt 版本数据
- 后续 Story 8.4 负责将版本与实际 TeacherModel 执行关联

### Previous Story Learnings (Story 8.2 复盘/模式/测试)

- **后端路由模式**：使用 `CurrentUser` 提取器进行权限校验
- **DTO 设计模式**：使用 `#[serde(rename_all = "camelCase")]` + `#[ts(export_to = "models/")]`
- **前端模块结构**：采用 `components/` + `hooks/` + `services/` + `index.ts`
- **测试实践**：使用 MSW + `QueryClientProvider`，通过 `useAuthStore` 注入登录态
- **Review 结论承接**：8.2 强调"分页/上限"，8.3 同样必须遵循

### Latest Technical Notes（基于当前项目版本）

**Breaking Changes / Best Practices**
- React 19：渲染错误不再自动重新抛出（改用 root 的 error 回调）
- TanStack Query v5：仅支持对象签名；`cacheTime` 改名 `gcTime`
- Axum 0.8：路由路径参数语法改为 `/{param}`
- SQLite：`is_active` 使用 INTEGER 而非 BOOLEAN

**Performance / Deprecation Notes**
- 版本列表默认分页，避免一次性加载过多数据
- 成功率统计可考虑缓存（MVP 阶段实时计算即可）

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/domain/models/teacher_prompt.rs`：Prompt 版本 DTO（新增）
  - `backend/src/core/meta_optimization_service/mod.rs`：元优化服务（新增）
  - `backend/src/api/routes/meta_optimization.rs`：元优化 API 路由（新增）
  - `backend/src/infra/db/repositories/teacher_prompt_repo.rs`：Prompt 版本仓储（新增）
  - `frontend/src/features/meta-optimization/`：元优化功能模块（新增）
- **路由挂载**：`meta_optimization.rs` 挂在 `/api/v1/meta-optimization`
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
- recharts：图表库（可选，用于成功率可视化）

### Deployment / Environment Notes（部署/环境）

- 本 Story 新增 `teacher_prompts` 表，需运行数据库迁移
- 不新增环境变量
- 部署验证：建议执行 `cargo test`、`pnpm vitest run`、`pnpm vite build`

### File Structure Requirements（落点约束）

**后端**：
- 数据库迁移：`backend/migrations/014_teacher_prompts.sql`（新增）
- 数据库迁移：`backend/migrations/015_add_teacher_prompt_version_id_to_optimization_tasks.sql`（新增）
- Prompt DTO：`backend/src/domain/models/teacher_prompt.rs`（新增）
- 模型导出：`backend/src/domain/models/mod.rs`（更新）
- 元优化服务：`backend/src/core/meta_optimization_service/mod.rs`（新增）
- 服务导出：`backend/src/core/mod.rs`（更新）
- Prompt 仓储：`backend/src/infra/db/repositories/teacher_prompt_repo.rs`（新增）
- 仓储导出：`backend/src/infra/db/repositories/mod.rs`（更新）
- 元优化 API：`backend/src/api/routes/meta_optimization.rs`（新增）
- 路由注册：`backend/src/api/routes/mod.rs`（更新）
- 路由挂载：`backend/src/main.rs`（更新，挂载到 `/api/v1/meta-optimization`）
- OpenAPI 注册：`backend/src/api/routes/docs.rs`（更新，新增 meta_optimization 路由）
- 类型生成：`backend/src/bin/gen-types.rs`（更新）

**前端**：
- 版本列表组件：`frontend/src/features/meta-optimization/components/PromptVersionList.tsx`（新增）
- 版本详情组件：`frontend/src/features/meta-optimization/components/PromptVersionDetail.tsx`（新增）
- 统计概览组件：`frontend/src/features/meta-optimization/components/MetaOptimizationStats.tsx`（新增）
- 服务层：`frontend/src/features/meta-optimization/services/metaOptimizationService.ts`（新增）
- Versions Hook：`frontend/src/features/meta-optimization/hooks/usePromptVersions.ts`（新增）
- Overview Hook：`frontend/src/features/meta-optimization/hooks/useMetaOptimizationOverview.ts`（新增）
- 历史任务 Hook：`frontend/src/features/meta-optimization/hooks/useMetaOptimizationTasks.ts`（新增）
- 模块入口：`frontend/src/features/meta-optimization/index.ts`（新增）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 版本创建 | 正确创建并分配版本号 |
| 后端单测 | 版本列表 | 按用户隔离返回列表 |
| 后端单测 | 活跃版本切换 | 正确切换活跃状态，幂等性验证 |
| 后端单测 | 成功率计算 | 正确统计任务通过率 |
| 后端单测 | 无任务时 | 返回 success_rate=null, task_count=0 |
| 后端单测 | 权限校验 | 非版本所有者返回 403 |
| 后端单测 | 并发版本号 | 使用事务确保版本号唯一 |
| 后端单测 | 版本列表分页 | limit/offset 正确生效 |
| 前端测试 | 版本列表展示 | 正确渲染版本列表 |
| 前端测试 | 活跃版本高亮 | 正确显示活跃标识 |
| 前端测试 | 空态处理 | 无版本显示引导界面 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 参考 `backend/src/api/routes/diagnostic.rs` 的路由实现模式
- 参考 `frontend/src/features/diagnostic-report/` 的模块结构
- 复用 `backend/src/infra/db/repositories/optimization_task_repo.rs` 获取任务数据
- 遵循 Repository 模式访问数据库
- 与 `teacher_settings_repo.rs` 独立，职责不同

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 8 / Story 8.3）
- PRD 元优化：`docs/project-planning-artifacts/prd.md#能力区域 9: 元优化`
- 架构（元优化）：`docs/project-planning-artifacts/architecture.md#9. 元优化`
- Story 8.2（前序）：`docs/implementation-artifacts/8-2-diagnostic-report.md`
- 老师模型设置仓储：`backend/src/infra/db/repositories/teacher_settings_repo.rs`
- TeacherModel trait：`backend/src/core/traits.rs`

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

- `cargo run --bin gen-types`
- `cargo test meta_optimization`
- `pnpm vitest run src/features/meta-optimization/components/PromptVersionList.test.tsx`
- `cargo test`
- `pnpm vitest run`
- `pnpm run build`

### Completion Notes List

- ✅ 新增 teacher_prompts 表与任务版本关联字段，补齐版本仓储/服务/API 与成功率统计逻辑。
- ✅ 元优化前端模块与页面完成（版本列表/详情/统计概览/历史任务入口），并加入导航入口。
- ✅ 新增 Prompt 版本创建入口与历史任务选择入口，修复 meta_optimization 绑定并补测核心用例。
- ✅ 全量回归通过：`cargo test` / `pnpm vitest run` / `pnpm run build`。
- ✅ 独立 code review 已完成并记录在 `## Review Notes`。

### File List

- backend/migrations/014_teacher_prompts.sql
- backend/migrations/015_add_teacher_prompt_version_id_to_optimization_tasks.sql
- backend/src/domain/models/teacher_prompt.rs
- backend/src/domain/models/mod.rs
- backend/src/infra/db/repositories/teacher_prompt_repo.rs
- backend/src/infra/db/repositories/mod.rs
- backend/src/infra/db/repositories/optimization_task_repo.rs
- backend/src/core/meta_optimization_service/mod.rs
- backend/src/core/mod.rs
- backend/src/api/routes/meta_optimization.rs
- backend/src/api/routes/mod.rs
- backend/src/api/routes/docs.rs
- backend/src/api/routes/optimization_tasks.rs
- backend/src/api/routes/diagnostic.rs
- backend/src/api/routes/results.rs
- backend/src/api/routes/iteration_control.rs
- backend/src/core/iteration_engine/recovery.rs
- backend/src/main.rs
- backend/src/bin/gen-types.rs
- backend/tests/meta_optimization_test.rs
- backend/tests/recovery_api_test.rs
- frontend/src/features/meta-optimization/components/MetaOptimizationStats.tsx
- frontend/src/features/meta-optimization/components/PromptVersionDetail.tsx
- frontend/src/features/meta-optimization/components/PromptVersionList.tsx
- frontend/src/features/meta-optimization/components/PromptVersionList.test.tsx
- frontend/src/features/meta-optimization/hooks/useMetaOptimizationTasks.ts
- frontend/src/features/meta-optimization/hooks/useMetaOptimizationOverview.ts
- frontend/src/features/meta-optimization/hooks/usePromptVersions.ts
- frontend/src/features/meta-optimization/services/metaOptimizationService.ts
- frontend/src/features/meta-optimization/index.ts
- frontend/src/pages/MetaOptimizationPage.tsx
- frontend/src/pages/index.ts
- frontend/src/App.tsx
- frontend/src/components/common/ViewSwitcher.tsx
- frontend/src/types/generated/api/CreateOptimizationTaskRequest.ts
- frontend/src/types/generated/api/MetaOptimizationTaskHint.ts
- frontend/src/types/generated/models/MetaOptimizationTaskSummary.ts
- frontend/src/types/generated/models/CreateTeacherPromptInput.ts
- frontend/src/types/generated/models/MetaOptimizationOverview.ts
- frontend/src/types/generated/models/TeacherPrompt.ts
- frontend/src/types/generated/models/TeacherPromptStats.ts
- frontend/src/types/generated/models/TeacherPromptVersion.ts
- docs/implementation-artifacts/8-3-meta-optimization-basics.md
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/8-2-diagnostic-report.md
- docs/implementation-artifacts/validation-report-2026-01-20-184521.md
- docs/project-planning-artifacts/architecture.md
- docs/project-planning-artifacts/epics.md

### Change Log

- 2026-01-20: 完成元优化 Prompt 版本管理（DB/后端/前端/测试），并接入版本统计与导航入口。
- 2026-01-20: 补齐元优化版本创建/历史任务选择入口，修复 meta_optimization 绑定并补测分页/空态。
- 2026-01-20: 全量回归通过（cargo test / pnpm vitest run / pnpm run build），Story 标记完成。
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [CRITICAL] Testing Requirements 已补齐：无任务 success_rate/null、并发版本号、分页 limit/offset、前端空态测试已实现并通过。
- [x] [HIGH] 创建任务 payload 已补充 `meta_optimization` 字段，避免 TS 构建失败并支持版本绑定。
- [x] [HIGH] 元优化入口已补齐：新增历史任务选择入口 + 目标展示（MVP 不执行真实元优化）。
- [x] [HIGH] Prompt 版本创建 UI 已补齐，支持保存变更说明并创建新版本。
- [x] [MEDIUM] 版本统计查询已消除 N+1（使用聚合 SQL）。
- [x] [MEDIUM] Story File List 已补齐实际变更文件。
- [x] [INFO] 独立 code review 完成：未发现新增阻塞问题（残余风险仍为“元优化执行流程为 MVP 占位”）。

### Decisions

- [x] 直接修复并补测，确保 AC 与性能问题闭环。

### Risks / Tech Debt

- [ ] 元优化执行流程仍为 MVP 占位（不执行真实优化），需在后续 Story 落地完整流程。

### Follow-ups

- [x] 见上方 `### Review Follow-ups (AI)`（已完成）。
