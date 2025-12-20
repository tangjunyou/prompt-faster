---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
status: 'complete'
completedAt: '2025-12-20'
inputDocuments:
  - docs/prd.md
  - docs/ux-design-specification.md
  - docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md
workflowType: 'architecture'
lastStep: 8
project_name: 'Prompt Faster'
user_name: '耶稣'
date: '2025-12-20'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
- 共 66 个功能需求，覆盖 10 个能力区域
- 核心能力：自动迭代优化（11 FR）、测试集管理（10 FR）、用户介入（7 FR）
- 架构关键：四层处理器算法、WebSocket 实时通信、Checkpoint 持久化

**Non-Functional Requirements:**
- 共 28 个非功能需求，覆盖 8 个质量属性
- 性能：系统延迟 < 100ms，流式首字节 < 500ms
- 可靠性：断点恢复率 100%，WAL 模式
- 可扩展性：新增执行引擎 < 4h，Trait 接口体系
- 安全性：API Key 加密存储、本地用户认证、用户数据隔离

**Scale & Complexity:**
- Primary domain: Full-Stack (Rust + React)
- Complexity level: Medium-High
- Complexity drivers: 实时 WebSocket 流式输出 + 前端可视化节点图 + 四层处理器算法
- Estimated architectural components: 15-20 核心模块

### Technical Constraints & Dependencies

| 层级 | 技术约束 |
|------|----------|
| 后端 | Axum 0.8.x + Tokio + SQLite/SQLx |
| 前端 | React 19.x + React Flow 12.x + shadcn/ui |
| 通信 | WebSocket (实时) + HTTP API (配置) |
| 部署 | Docker Compose V2 |
| 算法 | 四层处理器 + 7 Trait 体系（详见技术规格 Section 4） |

### Cross-Cutting Concerns Identified

1. **状态持久化** — Checkpoint 机制贯穿所有模块
2. **流式输出** — 前后端 WebSocket + 节点内动画
3. **错误处理与恢复** — 断点续跑、优雅降级
4. **模块化 Trait 体系** — 核心算法可替换
5. **配置管理** — 用户配置 + 智能默认值
6. **日志与可观测性** — 调试与用户透明度

## Starter Template Evaluation

### Primary Technology Domain

Full-Stack Desktop/Web 应用（Rust 后端 + React 前端 + SQLite 数据库）

### Starter Options Considered

| 选项 | 评估结果 |
|------|----------|
| create-tauri-app | 未来路线，MVP 阶段使用 Docker Compose |
| 社区 Axum+Vite 模板 | 版本可能与 PRD 规格冲突 |
| **自定义项目结构** | ✅ 选用 — 最大程度匹配 PRD 和技术规格 |

### Selected Approach: Custom Project Structure

**Rationale for Selection:**
1. PRD 已锁定技术栈版本，社区模板可能引入版本冲突
2. 四层处理器 + 7 Trait 体系需要自定义模块结构
3. 最大控制力，确保与技术规格完全一致

**Initialization Commands:**

```bash
# 后端（在 backend/ 目录）
cargo init --bin --name prompt_faster

# 前端（在 frontend/ 目录）
npm create vite@latest . -- --template react-ts
```

### Architectural Decisions Established

**Language & Runtime:**
- 后端：Rust (edition 2024)，异步运行时 Tokio
- 前端：TypeScript 5.x + React 19.x

**Styling Solution:**
- Tailwind CSS（采用官方最新稳定版本，需确认浏览器兼容性）
- shadcn/ui 组件库 + Framer Motion 动画库

**Build Tooling:**
- 后端：Cargo + cargo-watch (开发)
- 前端：Vite（采用项目初始化时的最新稳定版本）

**Testing Framework（初步选型，可在实现阶段根据需求微调）:**
- 后端：Rust 内置测试 + tokio-test
- 前端：Vitest + React Testing Library

**Code Organization:**
- 后端：模块化 Trait 架构（见技术规格 Section 4）
- 前端：Feature-based 目录结构

**Development Experience:**
- Docker Compose 统一开发环境
- Hot reload（前后端）
- TypeScript 类型共享：优先考虑 ts-rs 等 Rust→TS 类型生成方案

**Note:** 项目初始化应作为第一个实现故事。

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- 数据库迁移策略：SQLx Migrations
- API Key 加密方案：AES-GCM + 用户密码派生密钥
- 用户认证方案：Argon2 密码哈希
- 错误处理标准：thiserror + anyhow

**Important Decisions (Shape Architecture):**
- 路由策略：React Router 7.x
- 组件边界：Pages / Features / Components 三层
- 日志框架：tracing + tracing-subscriber
- API 文档：utoipa + Swagger UI

**Deferred Decisions (Post-MVP):**
- 缓存层：未来在 StateManager 后面挂缓存
- TanStack Router：作为 React Router 的未来备选

### Data Architecture

| 决策项 | 选型 | 理由 |
|--------|------|------|
| **数据库** | SQLite + SQLx 0.8.x (WAL) | PRD 7.5 明确指定 |
| **迁移策略** | SQLx Migrations | 与 SQLx 深度集成，类型安全，版本控制 |
| **缓存策略** | 无额外缓存（MVP）+ SQLite 连接池 | SQLite 本地访问足够快，NFR1 < 100ms 可达成 |
| **未来缓存** | StateManager 后挂缓存层 | 预留扩展点，不阻塞 MVP |

### Authentication & Security

| 决策项 | 选型 | 理由 |
|--------|------|------|
| **API Key 加密** | AES-GCM + 用户密码派生密钥 | 平衡安全性与复杂度，密钥来源明确 |
| **密钥派生** | Argon2（同时用于密码哈希） | 单一算法，简化依赖 |
| **用户认证** | Argon2 密码哈希 | 单用户起步，多用户就绪，符合 NFR11a |
| **数据隔离** | 按用户隔离 | 符合 NFR11b |

**密钥来源说明：**
- 用户首次登录时输入密码
- 使用 Argon2 派生加密密钥
- 密钥仅存于内存，不持久化
- API Key 使用该密钥进行 AES-GCM 加密后存储

### API & Communication Patterns

| 决策项 | 选型 | 理由 |
|--------|------|------|
| **实时通信** | WebSocket | 流式输出、节点状态同步 |
| **配置 API** | HTTP REST | 非实时操作 |
| **API 文档** | utoipa + Swagger UI | Axum 生态标准方案 |
| **错误处理** | thiserror（库）+ anyhow（应用） | Rust 社区最佳实践 |

**错误处理约定：**
- 库层（core/）：使用 `thiserror` 定义类型安全错误
- 应用层（api/）：使用 `anyhow` 包装错误，统一返回格式
- 前端：统一错误响应结构 `{ code, message, details? }`

### Frontend Architecture

| 决策项 | 选型 | 理由 |
|--------|------|------|
| **路由** | React Router 7.x | 生态默认选择，成熟稳定 |
| **状态管理** | Zustand（全局）+ Jotai（原子） | PRD 7.3 明确指定 |
| **组件边界** | Pages / Features / Components 三层 | 清晰职责分离 |

**组件层级定义：**

| 层级 | 职责 | 示例 |
|------|------|------|
| **Pages** | 视图容器，路由入口 | RunView, FocusView, WorkspaceView |
| **Features** | 业务功能模块 | TaskConfig, TestCaseManager, IterationMonitor |
| **Components** | 可复用 UI 组件 | Node, Button, Modal, Toast |

### Infrastructure & Deployment

| 决策项 | 选型 | 理由 |
|--------|------|------|
| **容器化** | Docker Compose V2 | PRD 7.7 明确指定 |
| **CI/CD** | GitHub Actions（MVP 启用） | 最小流水线：lint + test + build |
| **日志框架** | tracing + tracing-subscriber | Axum/Tokio 生态标准 |
| **未来部署** | Tauri 桌面应用 | 成熟期分发方式 |

**MVP CI/CD 流水线：**
```yaml
# .github/workflows/ci.yml
- lint: cargo clippy + eslint
- test: cargo test + vitest
- build: cargo build --release + vite build
```

### Decision Impact Analysis

**Implementation Sequence:**
1. 项目结构初始化（backend/ + frontend/）
2. 数据库 Schema + SQLx Migrations
3. 用户认证 + API Key 加密
4. 核心 Trait 接口定义
5. WebSocket + HTTP API 骨架
6. 前端路由 + 组件骨架
7. CI/CD 流水线

**Cross-Component Dependencies:**

```
用户认证 (Argon2)
    ↓
API Key 加密 (AES-GCM + 派生密钥)
    ↓
ExecutionTarget Trait (使用解密后的 API Key)
    ↓
OptimizationContext (贯穿整个迭代流程)
```

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:** 10 areas where AI agents could make different choices

### Naming Patterns

**Database Naming Conventions:**
- 表名：snake_case, 复数（`optimization_tasks`, `test_cases`）
- 列名：snake_case（`created_at`, `user_id`）
- 外键：`{表名单数}_id`（`task_id`）
- 索引：`idx_{表名}_{列名}`（`idx_test_cases_task_id`）

**API Naming Conventions:**
- 端点：snake_case, 复数（`/api/v1/optimization_tasks`）
- 路由参数：`:param`（Axum 风格）
- 查询参数：snake_case
- **API 版本策略：** `/api/v1` 为稳定公共接口前缀，版本升级时保持向后兼容

**Code Naming Conventions:**
- Rust：结构体 PascalCase, 函数/变量 snake_case
- TypeScript：组件 PascalCase, 函数/变量 camelCase
- 跨语言边界：`#[serde(rename_all = "camelCase")]`

### Structure Patterns

**Backend Structure:**
```
src/
├── api/       # HTTP/WS 路由
├── core/      # Trait 实现（7 个核心模块）
├── domain/    # 领域模型
├── infra/     # 数据库、外部服务
└── shared/    # 工具、配置、错误
```

**Frontend Structure:**
```
src/
├── pages/      # 路由入口（RunView, FocusView, WorkspaceView）
├── features/   # 业务模块
├── components/ # 可复用组件
├── stores/     # 状态管理
├── services/   # API 调用
└── types/      # ts-rs 生成类型
```

**Test Location:**

| 类型 | 位置 | 命名 |
|------|------|------|
| Rust 单元测试 | 同文件 `#[cfg(test)]` | — |
| Rust 集成测试 | `backend/tests/` | `test_*.rs` |
| 前端单元测试 | 同目录 | `*.test.ts(x)` |
| 前端 E2E 测试 | `frontend/tests/e2e/` | `*.spec.ts` |

### Format Patterns

**API Response Format:**

```typescript
interface ApiSuccess<T> {
  data: T;
  meta?: { page?: number; pageSize?: number; total?: number };
}

interface ApiError {
  error: {
    code: string;      // 格式：DOMAIN_ACTION_REASON
    message: string;   // 用户可见消息
    details?: Record<string, unknown>; // 仅开发环境
  };
}

type ApiResponse<T> = ApiSuccess<T> | ApiError;
```

**MUST:** `data` 和 `error` **互斥**（类型系统已强制约束）：
- 成功响应：`data` 非空，`error` 缺失
- 失败响应：`error` 非空，`data` 缺失

**DateTime Format:**
- **数据库存储：INTEGER (Unix 毫秒时间戳)**
- Rust 层：使用 `time`/`chrono` 封装为 `DateTime<Utc>`
- API 传输：ISO 8601 字符串（`2025-12-20T18:45:00Z`）
- 前端显示：本地化（date-fns）

### Communication Patterns

**WebSocket Events:**
- 服务端推送：`{domain}:{action}`（`iteration:started`）
- 客户端命令：`{domain}:{command}`（`task:pause`）

**Event Payload Structure:**
```typescript
interface WsMessage<T> {
  type: string;
  payload: T;
  timestamp: string;      // ISO 8601
  correlationId?: string; // 追踪 ID
}
```

**correlationId 规则：**
- **默认由后端在请求入口生成**
- 如前端已传入，则复用前端值
- **MUST:** 在 HTTP 请求 → 后端处理 → WebSocket 推送 → tracing 日志 中保持一致

**State Management（架构级决策）：**

| 状态类型 | 工具 | 命名约定 |
|----------|------|----------|
| 全局共享状态 | Zustand | `use{Domain}Store` |
| 原子/派生状态 | Jotai | `{domain}{State}Atom` |
| **服务端状态** | **TanStack Query** | 标准 hooks |

### Process Patterns

**Loading State:**
```typescript
type LoadingState = 'idle' | 'loading' | 'success' | 'error';
```

**Error Handling Layers:**

| 层级 | 职责 |
|------|------|
| API 层 | 返回统一错误格式 |
| Service 层 | 转换为前端友好错误 |
| UI 层 | Toast + Error Boundary |

**MUST:** 前端**不得直接展示** `error.details` 中的内容给用户，以避免泄露调试信息（尤其是 API Key 相关错误）。

### Enforcement Guidelines

**All AI Agents MUST:**
1. 遵循命名约定：Rust snake_case，TypeScript camelCase
2. 跨语言边界使用 `#[serde(rename_all = "camelCase")]`
3. API 响应使用 `ApiResponse<T>` 类型（data/error 互斥）
4. WebSocket 事件使用 `{domain}:{action}` 格式
5. 数据库时间字段统一使用 INTEGER (Unix ms)
6. correlationId 贯穿请求全链路（后端入口生成）
7. 测试文件放置在约定位置

**Anti-Patterns to Avoid:**
- ❌ 混用 snake_case 和 camelCase 于同一语言
- ❌ API 直接返回裸数据（无包装）
- ❌ 使用自定义日期格式或混用 INTEGER/TEXT
- ❌ 测试文件散落在非约定位置
- ❌ 前端直接展示 `error.details` 给用户
- ❌ 未更新规范/文档的情况下，引入新的 API 响应格式或错误结构

## Project Structure & Boundaries

### PRD 能力区域 → 架构组件映射（完整版）

| PRD 能力区域 | 后端模块组合 | 前端模块 |
|--------------|--------------|----------|
| **1. API 配置与连接** (FR1-5) | `api/routes/auth.rs` + `infra/external/api_key_manager.rs` | `features/api-config/` |

> **注**：`auth.rs` 负责 API Key & 连接配置；若未来扩展更复杂配置（代理、Endpoint 切换等），可拆分为 `api_config.rs` 等独立路由。
| **2. 测试集管理** (FR6-15) | `api/routes/test_cases.rs` + `domain/models/test_case.rs` + `infra/db/repositories/test_case_repo.rs` | `features/test-case-manager/` |
| **3. 优化任务配置** (FR16-23c) | `api/routes/tasks.rs` + `domain/models/optimization_task.rs` + `domain/types/config.rs` | `features/task-config/` |
| **4. 自动迭代优化** (FR24-34) | `core/` (7 Trait) + `core/iteration_engine/` | `features/iteration-monitor/` |
| **5. 可视化** (FR35-39) | `api/ws/events.rs`（推送状态） | `pages/RunView/` + `components/nodes/` |
| **6. 用户介入** (FR40-46) | `api/ws/connection.rs` + `api/ws/events.rs` + `core/iteration_engine/checkpoint.rs` | `features/user-intervention/` |
| **7. 工作区管理** (FR47-51) | `api/routes/workspaces.rs` + `domain/models/workspace.rs` + `infra/db/repositories/workspace_repo.rs` | `features/workspace-manager/` |
| **8. 可靠性与恢复** (FR52-55) | `core/iteration_engine/checkpoint.rs` + `infra/db/repositories/checkpoint_repo.rs` | `features/checkpoint-recovery/` |
| **9. 元优化** (FR56-59) | `core/teacher_model/` + `domain/models/teacher_prompt.rs` + `infra/db/repositories/teacher_prompt_repo.rs` | `features/meta-optimization/` |
| **10. 结果输出与分析** (FR60-63) | `core/evaluator/` + `core/feedback_aggregator/` + `api/routes/results.rs` + `domain/models/evaluation_result.rs` | `features/result-viewer/` |

> **注**：`features/user-intervention/`、`features/workspace-manager/`、`features/checkpoint-recovery/`、`features/meta-optimization/` 为 MVP 后期或 Phase 2 功能模块，目录结构中以占位形式预留。

### 技术规格 7 Trait → 后端 core/ 子模块

| Trait | 模块位置 | 说明 |
|-------|----------|------|
| RuleEngine | `core/rule_engine/` | 从测试用例提取规律 |
| PromptGenerator | `core/prompt_generator/` | 基于规律生成 Prompt |
| Evaluator | `core/evaluator/` | 评估执行结果 |
| FeedbackAggregator | `core/feedback_aggregator/` | 聚合反思结果 |
| Optimizer | `core/optimizer/` | 执行优化步骤 |
| TeacherModel | `core/teacher_model/` | LLM 调用适配 |
| ExecutionTarget | `core/execution_target/` | Dify/直连 API 执行 |

### 完整项目目录结构

```
prompt-faster/
├── README.md
├── docker-compose.yml
├── .gitignore
├── .env.example
├── .github/
│   └── workflows/
│       └── ci.yml
│
├── backend/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── .env
│   ├── .env.example
│   │
│   ├── migrations/                    # SQLx 迁移
│   │   └── 001_initial_schema.sql
│   │
│   ├── src/
│   │   ├── main.rs                    # 入口点
│   │   ├── lib.rs                     # 库导出
│   │   │
│   │   ├── api/                       # HTTP/WS 路由层
│   │   │   ├── mod.rs
│   │   │   ├── routes/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tasks.rs           # /api/v1/optimization_tasks
│   │   │   │   ├── test_cases.rs      # /api/v1/test_cases
│   │   │   │   ├── iterations.rs      # /api/v1/iterations
│   │   │   │   ├── rules.rs           # /api/v1/rules
│   │   │   │   ├── workspaces.rs      # /api/v1/workspaces
│   │   │   │   ├── results.rs         # /api/v1/results
│   │   │   │   ├── auth.rs            # /api/v1/auth
│   │   │   │   └── health.rs          # /api/v1/health
│   │   │   ├── handlers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── task_handler.rs
│   │   │   │   ├── test_case_handler.rs
│   │   │   │   ├── iteration_handler.rs
│   │   │   │   ├── rule_handler.rs
│   │   │   │   ├── workspace_handler.rs
│   │   │   │   ├── result_handler.rs
│   │   │   │   └── auth_handler.rs
│   │   │   ├── ws/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── connection.rs      # WS 连接管理
│   │   │   │   └── events.rs          # {domain}:{action} 事件
│   │   │   ├── middleware/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── auth.rs
│   │   │   │   ├── tracing.rs
│   │   │   │   └── correlation_id.rs
│   │   │   └── response.rs            # ApiResponse<T> 实现（Step 5 规范落地）
│   │   │
│   │   ├── core/                      # 核心业务逻辑（7 Trait + IterationEngine）
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs              # 所有 Trait 定义
│   │   │   ├── rule_engine/
│   │   │   │   ├── mod.rs
│   │   │   │   └── default_impl.rs
│   │   │   ├── prompt_generator/
│   │   │   │   ├── mod.rs
│   │   │   │   └── default_impl.rs
│   │   │   ├── evaluator/
│   │   │   │   ├── mod.rs
│   │   │   │   └── default_impl.rs
│   │   │   ├── feedback_aggregator/
│   │   │   │   ├── mod.rs
│   │   │   │   └── default_impl.rs
│   │   │   ├── optimizer/
│   │   │   │   ├── mod.rs
│   │   │   │   └── default_impl.rs
│   │   │   ├── teacher_model/
│   │   │   │   ├── mod.rs
│   │   │   │   └── default_impl.rs
│   │   │   ├── execution_target/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── dify_impl.rs       # Dify 工作流实现
│   │   │   │   └── direct_api_impl.rs # 直连 LLM API 实现
│   │   │   └── iteration_engine/
│   │   │       ├── mod.rs
│   │   │       ├── orchestrator.rs    # 四层处理器编排
│   │   │       ├── checkpoint.rs      # 断点续跑
│   │   │       └── state_manager.rs   # OptimizationContext
│   │   │
│   │   ├── domain/                    # 领域模型
│   │   │   ├── mod.rs
│   │   │   ├── models/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── optimization_task.rs
│   │   │   │   ├── test_case.rs
│   │   │   │   ├── iteration.rs
│   │   │   │   ├── evaluation_result.rs
│   │   │   │   ├── checkpoint.rs
│   │   │   │   ├── rule.rs
│   │   │   │   ├── workspace.rs
│   │   │   │   ├── teacher_prompt.rs  # 元优化版本管理
│   │   │   │   └── user.rs
│   │   │   └── types/
│   │   │       ├── mod.rs
│   │   │       ├── optimization_context.rs
│   │   │       └── config.rs
│   │   │
│   │   ├── infra/                     # 基础设施
│   │   │   ├── mod.rs
│   │   │   ├── db/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── pool.rs            # SQLx 连接池
│   │   │   │   └── repositories/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── task_repo.rs
│   │   │   │       ├── test_case_repo.rs
│   │   │   │       ├── iteration_repo.rs
│   │   │   │       ├── checkpoint_repo.rs
│   │   │   │       ├── rule_repo.rs
│   │   │   │       ├── workspace_repo.rs
│   │   │   │       ├── teacher_prompt_repo.rs
│   │   │   │       └── user_repo.rs
│   │   │   └── external/
│   │   │       ├── mod.rs
│   │   │       ├── llm_client.rs      # LLM API 调用
│   │   │       ├── api_key_manager.rs # AES-GCM 加密
│   │   │       └── observability.rs   # 预留：Prometheus/OTel/tracing 指标上报（MVP 仅 trace 日志）
│   │   │
│   │   └── shared/                    # 共享工具
│   │       ├── mod.rs
│   │       ├── error.rs               # thiserror 定义
│   │       ├── config.rs              # 后端配置唯一入口（所有模块从此获取配置）
│   │       └── tracing_setup.rs       # tracing 初始化
│   │
│   └── tests/                         # 集成测试
│       ├── common/
│       │   └── mod.rs
│       ├── test_task_api.rs
│       ├── test_iteration_flow.rs
│       └── test_checkpoint.rs
│
├── frontend/
│   ├── package.json
│   ├── package-lock.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── tailwind.config.js
│   ├── postcss.config.js
│   ├── .env
│   ├── .env.example
│   ├── index.html
│   │
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── index.css                  # Tailwind 入口
│   │   │
│   │   ├── pages/                     # 路由入口（三视图模式）
│   │   │   ├── index.ts
│   │   │   ├── RunView/
│   │   │   │   ├── index.tsx
│   │   │   │   └── RunView.tsx
│   │   │   ├── FocusView/
│   │   │   │   ├── index.tsx
│   │   │   │   └── FocusView.tsx
│   │   │   └── WorkspaceView/
│   │   │       ├── index.tsx
│   │   │       └── WorkspaceView.tsx
│   │   │
│   │   ├── features/                  # 业务功能模块
│   │   │   ├── api-config/            # 能力区域 1
│   │   │   │   ├── index.ts
│   │   │   │   ├── ApiConfigPanel.tsx
│   │   │   │   └── hooks/
│   │   │   │       └── useApiConfig.ts
│   │   │   ├── test-case-manager/     # 能力区域 2
│   │   │   │   ├── index.ts
│   │   │   │   ├── TestCaseList.tsx
│   │   │   │   ├── TestCaseEditor.tsx
│   │   │   │   ├── TestCaseManager.test.tsx
│   │   │   │   └── hooks/
│   │   │   │       └── useTestCases.ts
│   │   │   ├── task-config/           # 能力区域 3
│   │   │   │   ├── index.ts
│   │   │   │   ├── TaskConfigPanel.tsx
│   │   │   │   ├── TaskConfigPanel.test.tsx
│   │   │   │   └── hooks/
│   │   │   │       └── useTaskConfig.ts
│   │   │   ├── iteration-monitor/     # 能力区域 4
│   │   │   │   ├── index.ts
│   │   │   │   ├── IterationTimeline.tsx
│   │   │   │   ├── IterationNode.tsx
│   │   │   │   └── hooks/
│   │   │   │       └── useIterationStream.ts
│   │   │   ├── user-intervention/     # 能力区域 6（预留）
│   │   │   │   ├── index.ts
│   │   │   │   └── # TODO: Phase 2 — 用户暂停/恢复/回滚等交互入口
│   │   │   ├── workspace-manager/     # 能力区域 7（预留）
│   │   │   │   ├── index.ts
│   │   │   │   └── # TODO: Phase 2 — 多工作区创建/切换/删除
│   │   │   ├── checkpoint-recovery/   # 能力区域 8（预留）
│   │   │   │   ├── index.ts
│   │   │   │   └── # TODO: Phase 2 — 历史回滚/断点恢复界面
│   │   │   ├── meta-optimization/     # 能力区域 9（预留）
│   │   │   │   ├── index.ts
│   │   │   │   └── # TODO: Phase 2 — 老师模型 Prompt 版本管理/元优化
│   │   │   ├── result-viewer/         # 能力区域 10
│   │   │   │   ├── index.ts
│   │   │   │   ├── ResultDashboard.tsx
│   │   │   │   ├── ComparisonView.tsx
│   │   │   │   └── hooks/
│   │   │   │       └── useResults.ts
│   │   │   └── rule-editor/
│   │   │       ├── index.ts
│   │   │       ├── RuleEditor.tsx
│   │   │       └── RulePreview.tsx
│   │   │
│   │   ├── components/                # 可复用组件
│   │   │   ├── ui/                    # shadcn/ui 组件
│   │   │   │   ├── button.tsx
│   │   │   │   ├── input.tsx
│   │   │   │   ├── dialog.tsx
│   │   │   │   ├── toast.tsx
│   │   │   │   └── ...
│   │   │   ├── nodes/                 # React Flow 节点
│   │   │   │   ├── BaseNode.tsx
│   │   │   │   ├── IterationNode.tsx
│   │   │   │   ├── EvaluationNode.tsx
│   │   │   │   └── ConnectionEdge.tsx
│   │   │   └── common/
│   │   │       ├── LoadingSpinner.tsx
│   │   │       ├── ErrorBoundary.tsx
│   │   │       └── AppLayout.tsx
│   │   │
│   │   ├── stores/                    # 状态管理
│   │   │   ├── index.ts
│   │   │   ├── useTaskStore.ts        # Zustand（全局）
│   │   │   ├── useAuthStore.ts        # Zustand（全局）
│   │   │   └── atoms/                 # Jotai（原子）
│   │   │       ├── selectedNodeAtom.ts
│   │   │       └── filterAtom.ts
│   │   │
│   │   ├── services/                  # API 调用层（只导出纯函数）
│   │   │   ├── index.ts
│   │   │   ├── apiClient.ts           # axios/fetch 封装
│   │   │   ├── taskService.ts
│   │   │   ├── testCaseService.ts
│   │   │   ├── iterationService.ts
│   │   │   ├── ruleService.ts
│   │   │   ├── authService.ts
│   │   │   └── wsClient.ts            # WebSocket 客户端
│   │   │
│   │   ├── hooks/                     # 全局自定义 Hooks
│   │   │   ├── index.ts
│   │   │   ├── useWebSocket.ts
│   │   │   ├── useCorrelationId.ts
│   │   │   └── useToast.ts
│   │   │
│   │   ├── types/                     # TypeScript 类型
│   │   │   ├── index.ts
│   │   │   ├── api.ts                 # ApiResponse<T>
│   │   │   ├── generated/             # ts-rs 生成
│   │   │   │   └── ...
│   │   │   └── models.ts
│   │   │
│   │   └── utils/                     # 工具函数
│   │       ├── index.ts
│   │       ├── dateUtils.ts           # date-fns 封装
│   │       └── errorUtils.ts
│   │
│   └── tests/
│       └── e2e/
│           ├── task-flow.spec.ts
│           └── iteration-flow.spec.ts
│
└── docs/
    ├── architecture.md
    ├── prd.md
    ├── ux-design-specification.md
    └── analysis/
        └── research/
```

### 架构边界定义

**API 边界：**

| 边界 | 端点前缀 | 职责 |
|------|----------|------|
| 公共 API | `/api/v1/*` | 所有客户端可访问 |
| WebSocket | `/ws` | 实时通信 |
| 健康检查 | `/api/v1/health` | 负载均衡探针 |

**组件边界：**

| 层级 | 边界规则 |
|------|----------|
| `api/` → `core/` | 只通过 Trait 接口调用 |
| `core/` → `domain/` | 直接使用领域模型 |
| `core/` → `infra/` | 通过 Repository Trait 抽象 |
| `infra/external/` | 封装所有外部 HTTP 调用 |

**数据边界：**

| 边界 | 规则 |
|------|------|
| Repository | 唯一的数据库访问点 |
| LLM Client | 唯一的外部 LLM 调用点 |
| API Key Manager | 唯一的敏感数据加解密点 |
| config.rs | 唯一的后端配置入口（所有模块从此获取配置，不得直接读 env） |

### 数据流

```
前端 (React)
    ↓ HTTP/WS (携带 correlationId)
API 层 (middleware/correlation_id.rs 生成或复用)
    ↓ Trait 调用 (透传 correlationId)
Core 层 (7 Trait + IterationEngine)
    ↓ Repository Trait
Infra 层 (SQLx + LLM Client)
    ↓
SQLite / 外部 LLM API
```

> **correlationId 全链路透传**：从 HTTP 请求 → handler → core → infra → WebSocket 事件（详见 Step 5 Communication Patterns）

### TanStack Query 使用约束

| 层级 | 职责 | 约束 |
|------|------|------|
| `services/*Service.ts` | 只导出纯函数（fetch 调用） | ❌ 禁止使用 React hooks |
| `features/*/hooks/` | 封装 TanStack Query hooks | ✅ 推荐位置 |
| `hooks/` | 全局共享的 Query hooks | ✅ 允许 |
| 组件 | 只通过 hooks 访问服务端状态 | ❌ 禁止直接 fetch/axios |

### 配置文件组织

| 文件 | 位置 | 说明 |
|------|------|------|
| `docker-compose.yml` | 根目录 | 开发环境容器编排 |
| `.github/workflows/ci.yml` | 根目录 | CI/CD 流水线 |
| `backend/.env` | 后端 | 环境变量（不提交） |
| `backend/.env.example` | 后端 | 环境变量模板 |
| `frontend/.env` | 前端 | 环境变量（不提交） |
| `frontend/vite.config.ts` | 前端 | Vite 构建配置 |
| `frontend/tailwind.config.js` | 前端 | Tailwind 配置 |
| `shared/config.rs` | `backend/src/shared/` | **后端配置唯一入口**（所有模块从此获取配置） |
| `observability.rs` | `backend/src/infra/external/` | 预留：Prometheus/OTel/tracing 指标上报（MVP 仅 trace 日志） |

## Architecture Validation Results

### 一致性验证 ✅

**决策兼容性：**

| 验证项 | 状态 | 说明 |
|--------|------|------|
| Axum 0.8 + Tokio 1.x + SQLx 0.8 | ✅ | Tokio 运行时是 Axum 和 SQLx 官方文档共同支持的主流选择，三者在版本与运行时层面兼容 |
| React 19 + React Flow 12 + shadcn/ui | ✅ | React Flow 12 面向现代 React 18+，在 React 19 上无已知不兼容点；通过依赖锁定与集成测试验证 |
| Zustand 5 + Jotai 2 + TanStack Query | ✅ | 状态管理分层清晰，无冲突 |
| WebSocket (Axum tungstenite) + HTTP API | ✅ | Axum 原生支持，协议并行无冲突 |
| SQLite WAL + Docker Compose | ✅ | 适配本地/单机部署场景，WAL 模式显著提升并发读写能力 |

**模式一致性：**

| 验证项 | 状态 | 说明 |
|--------|------|------|
| 命名约定跨语言一致 | ✅ | Rust snake_case ↔ TS camelCase + serde rename |
| API 响应格式统一 | ✅ | `ApiResponse<T>` 唯一出口 + TypeScript 接口定义 |
| WebSocket 事件格式 | ✅ | `{domain}:{action}` + `WsMessage<T>` 结构 |
| 数据库时间格式 | ✅ | INTEGER (Unix ms) 统一 |
| correlationId 全链路 | ✅ | middleware → core → WS 透传 |

**结构对齐：**

| 验证项 | 状态 | 说明 |
|--------|------|------|
| 后端分层 | ✅ | api → core → domain → infra → shared |
| 前端分层 | ✅ | pages → features → components → stores → services |
| 边界定义 | ✅ | Repository/LLM Client/API Key Manager 唯一访问点 |
| 配置入口 | ✅ | `shared/config.rs` 唯一入口 |

### 需求覆盖验证 ✅

**10 个能力区域 FR 覆盖：**

| 能力区域 | FR 范围 | 架构支持 | 验证 |
|----------|---------|----------|------|
| 1. API 配置与连接 | FR1–FR5 | `auth.rs` + `api_key_manager.rs` | ✅ |
| 2. 测试集管理 | FR6–FR15 | `test_cases.rs` + `test_case.rs` + `test_case_repo.rs` | ✅ |
| 3. 优化任务配置 | FR16–FR23c | `tasks.rs` + `optimization_task.rs` + `config.rs` | ✅ |
| 4. 自动迭代优化 | FR24–FR34 | 7 Trait + `iteration_engine/` | ✅ |
| 5. 可视化 | FR35–FR39 | `ws/events.rs` + `components/nodes/` | ✅ |
| 6. 用户介入 | FR40–FR46 | `ws/connection.rs` + `ws/events.rs` + `checkpoint.rs` | ✅ |
| 7. 工作区管理 | FR47–FR51 | `workspaces.rs` + `workspace.rs` | ✅ |
| 8. 可靠性与恢复 | FR52–FR55 | `checkpoint.rs` + `checkpoint_repo.rs` | ✅ |
| 9. 元优化 | FR56–FR59 | `teacher_model/` + `teacher_prompt.rs` | ✅ |
| 10. 结果输出与分析 | FR60–FR63 | `evaluator/` + `feedback_aggregator/` + `results.rs` | ✅ |

> **注**：FR 范围按 PRD 当前版本（2025-12-20）标注，PRD 更新时需同步维护本表。

**NFR 覆盖：**

| NFR 类别 | 架构支持 | 验证 |
|----------|----------|------|
| 性能 (NFR1-4) | Tokio 异步 + WebSocket 流式 + React Flow 高性能交互（在合理节点规模下可达成 60fps 目标） | ✅ |
| 可靠性 (NFR5-8) | SQLite WAL + Checkpoint 设计，并在 core/infra 层预留自动重试策略实现空间 | ✅ |
| 安全性 (NFR9-11b) | AES-GCM 加密 + 本地存储 + 用户隔离 | ✅ |
| 可扩展性 (NFR12-15) | 7 Trait 体系 + 模块化设计 | ✅ |
| 可用性 (NFR16-18) | Docker Compose 一键启动 | ✅ |
| 测试覆盖 (NFR19-22) | 单元/集成/E2E 测试位置已定义 | ✅ |
| 错误处理 (NFR23-24) | 统一错误格式 + thiserror | ✅ |
| 资源与离线 (NFR25-26) | SQLite 本地存储 | ✅ |

### 实现就绪验证 ✅

**决策完整性：**

| 验证项 | 状态 | 说明 |
|--------|------|------|
| 技术版本明确 | ✅ | Axum 0.8, React 19, SQLx 0.8 等 |
| 实现模式完整 | ✅ | 命名/结构/通信/流程模式均定义 |
| 一致性规则清晰 | ✅ | Enforcement Guidelines + Anti-Patterns |
| 示例代码/类型约定 | ✅ | TypeScript 接口示例已提供，Rust 侧类型设计有明确约定 |

**结构完整性：**

| 验证项 | 状态 | 说明 |
|--------|------|------|
| 目录结构完整 | ✅ | 后端/前端所有模块已定义 |
| 文件位置明确 | ✅ | 每个模块有具体文件列表 |
| 集成点清晰 | ✅ | API 边界 + 组件边界 + 数据边界 |
| 预留模块标注 | ✅ | Phase 2 模块以 TODO + 职责描述占位 |

**模式完整性：**

| 验证项 | 状态 | 说明 |
|--------|------|------|
| 冲突点处理 | ✅ | 跨语言命名 + 时间格式 + 响应格式 |
| 命名约定全面 | ✅ | 后端/前端/数据库/API 均覆盖 |
| 通信模式明确 | ✅ | HTTP + WebSocket + 事件格式 |
| 错误处理模式 | ✅ | 分层处理 + 统一格式 |

### 缺口分析

**Critical Gaps:** 无

**Important Gaps:** 无

**Nice-to-Have Gaps:**

| 缺口 | 优先级 | 说明 |
|------|--------|------|
| 数据库 Schema DDL | 低 | 可在开发阶段通过 SQLx 迁移生成 |
| CI/CD 流水线详细配置 | 低 | ci.yml 已占位，具体内容可后续补充 |
| 性能基准测试规范 | 低 | MVP 阶段可暂缓 |

### 架构完整性检查清单

**✅ 需求分析**
- [x] 项目上下文全面分析
- [x] 规模与复杂度评估
- [x] 技术约束识别
- [x] 横切关注点映射

**✅ 架构决策**
- [x] 关键决策含版本号记录
- [x] 技术栈完整指定
- [x] 集成模式定义
- [x] 性能考量已处理

**✅ 实现模式**
- [x] 命名约定建立
- [x] 结构模式定义
- [x] 通信模式指定
- [x] 流程模式记录

**✅ 项目结构**
- [x] 完整目录结构定义
- [x] 组件边界建立
- [x] 集成点映射
- [x] 需求到结构映射完成

### 架构就绪评估

**整体状态：READY FOR IMPLEMENTATION**

**置信度：高** — 基于完整的 Step 1-7 验证结果

**核心优势：**
- 7 Trait 体系提供高度可扩展性
- 前后端分层清晰，边界明确
- 实现模式全面，消除 AI Agent 歧义
- PRD 10 个能力区域完整映射

**未来增强点：**
- 数据库 Schema DDL 可在开发阶段生成
- CI/CD 详细配置可后续补充
- 性能基准测试可在 MVP 后期引入

### 实现交接

**AI Agent 指南：**
1. 严格遵循本文档所有架构决策
2. 在所有组件中一致使用实现模式
3. 尊重项目结构与边界
4. 所有架构问题参考本文档

**首要实现优先级：**
1. 初始化项目结构（后端 Cargo + 前端 Vite）
2. 配置 Docker Compose 开发环境
3. 实现核心 Trait 接口定义
4. 搭建 API 路由骨架

## Architecture Completion Summary

### 工作流完成状态

**架构决策工作流：** COMPLETED ✅  
**完成步骤总数：** 8  
**完成日期：** 2025-12-20  
**文档位置：** `docs/architecture.md`

### 最终架构交付物

**📋 完整架构文档**
- 所有架构决策含具体版本号
- 实现模式确保 AI Agent 一致性
- 完整项目结构含所有文件与目录
- 需求到架构的完整映射
- 验证确认一致性与完整性

**🏗️ 实现就绪基础**
- 20+ 架构决策已制定
- 15+ 实现模式已定义
- 15-20 架构组件已指定
- 66 FR + 28 NFR 完整支持

**📚 AI Agent 实现指南**
- 技术栈含验证版本号
- 一致性规则防止实现冲突
- 项目结构含清晰边界
- 集成模式与通信标准

### 质量保证检查清单

**✅ 架构一致性**
- [x] 所有决策协同工作无冲突
- [x] 技术选型相互兼容
- [x] 模式支持架构决策
- [x] 结构与所有选型对齐

**✅ 需求覆盖**
- [x] 所有功能需求获得支持
- [x] 所有非功能需求已处理
- [x] 横切关注点已覆盖
- [x] 集成点已定义

**✅ 实现就绪**
- [x] 决策具体且可执行
- [x] 模式防止 Agent 冲突
- [x] 结构完整且无歧义
- [x] 示例已提供便于理解

### 项目成功要素

**🎯 清晰决策框架**  
每个技术选型都经过协作讨论并有明确理由，确保所有利益相关者理解架构方向。

**🔧 一致性保证**  
实现模式和规则确保多个 AI Agent 产出兼容、一致的代码，无缝协作。

**📋 完整覆盖**  
所有项目需求都有架构支持，业务需求到技术实现有清晰映射。

**🏗️ 坚实基础**  
选定的技术栈和架构模式提供遵循当前最佳实践的生产就绪基础。

---

**架构状态：** READY FOR IMPLEMENTATION ✅

**下一阶段：** 使用本文档中记录的架构决策和模式开始实现。

**文档维护：** 实现过程中如有重大技术决策变更，请同步更新本架构文档。

