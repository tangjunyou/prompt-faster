---
stepsCompleted: [1]
inputDocuments:
  - docs/prd.md
  - docs/architecture.md
  - docs/ux-design-specification.md
workflowType: 'epics'
lastStep: 1
project_name: 'Prompt Faster'
user_name: '耶稣'
date: '2025-12-20'
---

# Prompt Faster - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for Prompt Faster, decomposing the requirements from the PRD, UX Design, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

**共 66 个功能需求，覆盖 10 个能力区域。**

#### 能力区域 1: API 配置与连接 (5 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR1 | 用户可以配置 Dify 工作流 API 凭证（地址 + API Key） | MVP |
| FR2 | 用户可以配置通用大模型 API 凭证（硅基流动、魔搭社区） | MVP |
| FR3 | 用户可以测试 API 连接是否成功 | MVP |
| FR4 | 系统可以持久化保存 API 凭证供后续使用 | MVP |
| FR5 | 用户可以配置老师模型参数（温度、top_p 等） | MVP |

#### 能力区域 2: 测试集管理 (10 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR6 | 用户可以手动创建测试集 | MVP |
| FR7 | 用户可以批量导入测试集（txt 格式） | MVP |
| FR8 | 用户可以将测试集配置保存为模板复用 | MVP |
| FR9 | 系统可以从 Dify API 自动解析输入变量结构 | MVP |
| FR10 | 用户可以指定哪个 Dify 变量是待优化的 system prompt | MVP |
| FR11 | 用户可以在通用 API 场景下自定义输入变量 | MVP |
| FR12 | 用户可以为固定任务提供标准答案 | MVP |
| FR13 | 用户可以为创意任务仅提供核心诉求（无标准答案） | MVP |
| FR14 | 用户可以为创意任务设置结构化约束（长度限制、必含关键词、禁止内容、格式要求） | MVP |
| FR15 | 用户可以编辑和删除已有测试集 | MVP |

#### 能力区域 3: 优化任务配置 (8 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR16 | 用户可以创建新的优化任务 | MVP |
| FR17 | 用户可以选择执行目标类型（Dify / 通用 API） | MVP |
| FR18 | 用户可以选择任务模式（固定任务 / 创意任务） | MVP |
| FR19 | 用户可以输入优化目标（自然语言描述） | MVP |
| FR20 | 用户可以选择填写初始 Prompt 或留空 | MVP |
| FR21 | 用户可以关联测试集到优化任务 | MVP |
| FR22 | 用户可以配置规律假设/候选 Prompt 的生成数量 | MVP |
| FR23 | 用户可以配置连续失败触发多样性注入的阈值 | MVP |
| FR23a | 用户可以配置核心算法参数（IterationConfig/DataSplitConfig/OutputConfig 等） | MVP |
| FR23b | 系统提供经过验证的默认配置，用户可随时一键重置为默认值 | MVP |
| FR23c | 用户可以配置最大迭代轮数、通过率阈值、数据划分策略等核心参数 | MVP |

#### 能力区域 4: 自动迭代优化 (11 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR24 | 系统可以执行规律抽取（Layer 1: Pattern Extractor） | MVP |
| FR25 | 系统可以生成候选 Prompt（Layer 2: Prompt Engineer） | MVP |
| FR26 | 系统可以评估 Prompt 效果（Layer 3: Quality Assessor） | MVP |
| FR27 | 系统可以执行反思迭代（Layer 4: Reflection Agent） | MVP |
| FR28 | 用户可以选择串行模式或并行模式执行测试集 | MVP |
| FR29 | 系统可以并行执行多条测试用例 | MVP |
| FR30 | 系统可以汇总并行执行结果进行综合分析 | MVP |
| FR31 | 系统可以记录失败档案并避免重蹈覆辙 | MVP |
| FR32 | 系统可以在连续失败时触发多样性注入 | MVP |
| FR33 | 系统可以自动判断优化是否成功（所有测试通过） | MVP |
| FR34 | 系统可以检测创意任务输出的多样性分数（可选功能） | Growth |

#### 能力区域 5: 可视化 (5 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR35 | 用户可以查看节点图形式的迭代流程 | MVP |
| FR36 | 用户可以看到节点状态颜色变化（灰/蓝/绿/红/黄） | MVP |
| FR37 | 用户可以看到边动画展示数据流动 | MVP |
| FR38 | 用户可以实时查看老师模型思考过程（流式输出） | MVP |
| FR39 | 用户可以在思考面板中看到当前环节标识 | MVP |

#### 能力区域 6: 用户介入 (7 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR40 | 用户可以在任意节点暂停迭代 | MVP |
| FR41 | 用户可以直接编辑当前迭代的中间产物（规律假设、Prompt） | MVP |
| FR42 | 用户可以通过对话引导老师模型（告诉它你的想法） | MVP |
| FR43 | 用户可以查看历史迭代产物（只读） | MVP |
| FR44 | 用户可以从暂停点继续迭代 | MVP |
| FR45 | 用户可以随时增加迭代轮数 | MVP |
| FR46 | 用户可以随时手动终止并选择满意的 Prompt | MVP |

#### 能力区域 7: 工作区管理 (5 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR47 | 用户可以创建多个工作区（优化任务） | MVP |
| FR48 | 用户可以切换查看不同工作区 | MVP |
| FR49 | 用户可以删除工作区 | MVP |
| FR50 | 系统保证工作区之间数据完全隔离 | MVP |
| FR51 | 用户可以配置多个老师模型并切换使用 | MVP |

#### 能力区域 8: 可靠性与恢复 (4 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR52 | 系统可以在每个迭代步骤保存 Checkpoint | MVP |
| FR53 | 系统可以在异常中断后恢复到断点状态 | MVP |
| FR54 | 用户可以从任意历史 Checkpoint 回滚 | MVP |
| FR55 | 系统可以保存完整的迭代历史记录 | MVP |

#### 能力区域 9: 元优化 (4 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR56 | 用户可以将老师模型 Prompt 作为优化目标 | MVP |
| FR57 | 系统可以持久化所有老师模型 Prompt 版本 | MVP |
| FR58 | 系统可以统计每个版本的成功率 | MVP |
| FR59 | 高级用户可以直接编辑老师模型 Prompt | MVP |

#### 能力区域 10: 结果输出与分析 (4 FR)

| FR# | 功能需求 | Phase |
|-----|----------|-------|
| FR60 | 用户可以查看最终优化结果 Prompt | MVP |
| FR61 | 用户可以复制或导出 Prompt（支持 Markdown/JSON/XML） | MVP |
| FR62 | 用户可以对比任意两个 Prompt 版本在同一测试集上的效果差异 | Growth |
| FR63 | 用户可以查看诊断报告了解"为什么之前不行" | MVP |

### NonFunctional Requirements

**共 28 个非功能需求，覆盖 8 个质量属性类别。**

#### 性能 (Performance)

| NFR# | 需求 | 目标值 |
|------|------|--------|
| NFR1 | 系统自身延迟 | < 100ms（不含大模型 API 调用时间） |
| NFR2 | 流式输出首字节延迟 | < 500ms |
| NFR3 | 节点图渲染性能 | 60fps |
| NFR4 | 并行测试集执行效率 | 接近线性加速 |

#### 可靠性 (Reliability)

| NFR# | 需求 | 目标值 |
|------|------|--------|
| NFR5 | 断点恢复率 | 100%（覆盖 kill/断网/断电/跨版本） |
| NFR6 | 数据持久化保障 | WAL 模式 + FULL synchronous |
| NFR7 | Checkpoint 完整性 | 100% |
| NFR8 | API 调用重试 | 自动重试 3 次 |

#### 安全性 (Security)

| NFR# | 需求 | 目标值 |
|------|------|--------|
| NFR9 | API Key 存储 | 加密存储（AES-GCM） |
| NFR10 | 凭证传输 | 仅本地 |
| NFR11 | 日志脱敏 | 自动脱敏 |
| NFR11a | 本地用户认证 | 支持本地登录（Argon2 密码哈希） |
| NFR11b | 用户数据隔离 | 按用户隔离 |

#### 可扩展性 (Extensibility)

| NFR# | 需求 | 目标值 |
|------|------|--------|
| NFR12 | 新增执行引擎 | < 4 小时（实现 ExecutionTarget trait） |
| NFR13 | 新增评估器 | < 2 小时（实现 Evaluator trait） |
| NFR14 | 新增老师模型 | < 2 小时（实现 TeacherModel trait） |
| NFR15 | 核心算法替换 | 仅影响算法模块 |

#### 可用性 (Usability)

| NFR# | 需求 | 目标值 |
|------|------|--------|
| NFR16 | 安装时间 | < 5 分钟 |
| NFR17 | 首次完成优化 | < 30 分钟 |
| NFR18 | 界面语言 | 中文为主，英文为辅 |

#### 测试覆盖 (Quality Assurance)

| NFR# | 需求 | 目标值 |
|------|------|--------|
| NFR19 | 核心流程端到端测试 | ≥ 80% 覆盖率 |
| NFR20 | 模块回归测试 | 100% 通过 |
| NFR21 | 优化成功率基准 | ≥ 90% |
| NFR22 | 并行 vs 串行差异 | < 5% |

#### 错误处理 (Error Handling)

| NFR# | 需求 | 目标值 |
|------|------|--------|
| NFR23 | API 调用超时阈值 | ≤ 60 秒 |
| NFR24 | 错误信息可读性 | 100% 覆盖 |

#### 资源与离线 (Resource & Offline)

| NFR# | 需求 | 目标值 |
|------|------|--------|
| NFR25 | 内存占用限制 | 空闲 ≤ 500MB，运行 ≤ 2GB |
| NFR26 | 离线功能可用性 | 100% 本地功能可用 |

### Additional Requirements

#### 架构规范型需求（可测试验收标准）

> 以下规范来自 Architecture，需在相关 Story 验收时强制检查。

| AR# | 规范需求 | 验收标准 |
|-----|----------|----------|
| AR1 | API 响应结构必须使用 `ApiResponse<T>` | `data` 与 `error` 字段互斥，不可同时存在非空值 |
| AR2 | WebSocket payload 必须携带 `correlationId` | 全链路透传，请求-响应可追溯，日志包含该字段 |
| AR3 | 数据库时间字段统一使用 Unix 毫秒时间戳 | 字段命名规范：`*_at` 后缀，值为 `i64` 类型毫秒时间戳 |

#### 来自 Architecture 的技术需求

**项目初始化与结构：**
- 自定义项目结构（backend/ + frontend/），不使用现有模板
- 后端：`cargo init --bin --name prompt_faster`
- 前端：`npm create vite@latest . -- --template react-ts`
- Rust edition 2024 + TypeScript 5.x

**数据库与迁移：**
- SQLx Migrations 数据库迁移策略
- SQLite WAL 模式 + FULL synchronous
- 初始 Schema：workspaces, checkpoints, test_sets, api_credentials, teacher_prompts 等表

**安全与认证：**
- AES-GCM + 用户密码派生密钥（Argon2）用于 API Key 加密
- Argon2 密码哈希用于用户认证
- 密钥仅存于内存，不持久化

**错误处理：**
- thiserror（库层）+ anyhow（应用层）
- 统一错误响应结构 `{ code, message, details? }`

**API 设计：**
- utoipa + Swagger UI API 文档
- API 版本策略：`/api/v1` 前缀
- WebSocket 事件格式：`{domain}:{action}`
- correlationId 全链路透传

**前端架构：**
- React Router 7.x 路由
- Pages / Features / Components 三层组件边界
- TanStack Query 服务端状态管理
- ts-rs 类型生成

**日志与可观测性：**
- tracing + tracing-subscriber
- 预留 Prometheus/OTel 指标上报

**7 Trait 体系实现：**
- RuleEngine（规律引擎）
- PromptGenerator（Prompt 生成器）
- Evaluator（评估器）
- FeedbackAggregator（反馈聚合器）
- Optimizer（优化器）
- TeacherModel（老师模型）
- ExecutionTarget（执行目标）

**CI/CD：**
- GitHub Actions 最小流水线：lint + test + build
- Docker Compose V2 开发环境

#### 来自 UX Design 的体验需求

**三视图模式架构：**
- **Run View**（默认）：左画布 + 右详情面板，核心体验主舞台
- **Focus View**：全画布中心型，长任务监控/演示
- **Workspace View**：三栏工作台型，多工作区管理/历史回看
- 视图切换：顶栏切换器 + 快捷键 `Cmd/Ctrl + 1/2/3`

**自定义组件需求（8 个）：**
- IterationNode（迭代节点）：状态、得分、耗时、动画
- StreamingText（流式文本）：逐字渲染、光标指示、虚拟滚动
- PromptDiff（Prompt 对比）：MVP 使用 Monaco DiffEditor
- DualProgress（双层进度）：整体 + 当前轮次
- StatusHUD（状态浮层）：Focus View 轻量状态展示
- InsightCard（规律卡片）：规律提炼展示
- InterventionForm（介入表单）：暂停/编辑/引导操作入口
- TaskCard（任务卡片）：Workspace View 任务列表项

**动画要求：**
- 动画按 V2+ 规格完整实现，用户可选择隐藏但不可缺失
- 节点状态过渡、LLM 流式输出、评估进度环、代码执行光带
- 成功庆祝粒子效果

**键盘快捷键：**
- `Cmd/Ctrl + 1/2/3` 视图切换
- `Cmd/Ctrl + B` 面板展开/收起
- `Space` 暂停/继续
- `Cmd/Ctrl + K` 命令面板

**无障碍与响应式：**
- WCAG 2.1 AA 合规
- 桌面优先设计（≥1280px 主战场）
- 最小支持 768px（平板竖屏，只读）
- 不支持 <768px

**情感设计：**
- 三感并重：掌控感、理解感、成就感
- 失败积极化：展示"学到了什么"
- 等待不焦虑：实时进度反馈

### FR Coverage Map

| FR# | Epic | 描述 |
|-----|------|------|
| FR1 | Epic 1 | 配置 Dify 工作流 API 凭证 |
| FR2 | Epic 1 | 配置通用大模型 API 凭证 |
| FR3 | Epic 1 | 测试 API 连接 |
| FR4 | Epic 1 | 持久化保存 API 凭证 |
| FR5 | Epic 1 | 配置老师模型参数 |
| FR6 | Epic 2 | 手动创建测试集 |
| FR7 | Epic 2 | 批量导入测试集 |
| FR8 | Epic 2 | 测试集模板复用 |
| FR9 | Epic 2 | Dify API 变量解析 |
| FR10 | Epic 2 | 指定待优化 system prompt 变量 |
| FR11 | Epic 2 | 通用 API 自定义输入变量 |
| FR12 | Epic 2 | 固定任务标准答案 |
| FR13 | Epic 2 | 创意任务核心诉求 |
| FR14 | Epic 2 | 创意任务结构化约束 |
| FR15 | Epic 2 | 编辑删除测试集 |
| FR16 | Epic 3 | 创建优化任务 |
| FR17 | Epic 3 | 选择执行目标类型 |
| FR18 | Epic 3 | 选择任务模式 |
| FR19 | Epic 3 | 输入优化目标 |
| FR20 | Epic 3 | 填写初始 Prompt |
| FR21 | Epic 3 | 关联测试集 |
| FR22 | Epic 3 | 配置生成数量 |
| FR23 | Epic 3 | 配置多样性注入阈值 |
| FR23a | Epic 3 | 配置核心算法参数 |
| FR23b | Epic 3 | 默认配置与重置 |
| FR23c | Epic 3 | 配置迭代轮数等参数 |
| FR24 | Epic 4 | 执行规律抽取 (Layer 1) |
| FR25 | Epic 4 | 生成候选 Prompt (Layer 2) |
| FR26 | Epic 4 | 评估 Prompt 效果 (Layer 3) |
| FR27 | Epic 4 | 执行反思迭代 (Layer 4) |
| FR28 | Epic 4 | 串行/并行模式选择 |
| FR29 | Epic 4 | 并行执行测试用例 |
| FR30 | Epic 4 | 汇总并行结果 |
| FR31 | Epic 4 | 记录失败档案 |
| FR32 | Epic 4 | 多样性注入 |
| FR33 | Epic 4 | 自动判断优化成功 |
| FR34 | Epic 8 | 创意任务多样性分数（Growth） |
| FR35 | Epic 5 | 节点图迭代流程 |
| FR36 | Epic 5 | 节点状态颜色变化 |
| FR37 | Epic 5 | 边动画数据流动 |
| FR38 | Epic 5 | 流式输出思考过程 |
| FR39 | Epic 5 | 思考面板环节标识 |
| FR40 | Epic 6 | 任意节点暂停 |
| FR41 | Epic 6 | 编辑中间产物 |
| FR42 | Epic 6 | 对话引导老师模型 |
| FR43 | Epic 6 | 查看历史迭代产物 |
| FR44 | Epic 6 | 从暂停点继续 |
| FR45 | Epic 6 | 增加迭代轮数 |
| FR46 | Epic 6 | 手动终止选择 Prompt |
| FR47 | Epic 3 | 创建多个工作区 |
| FR48 | Epic 3 | 切换工作区 |
| FR49 | Epic 3 | 删除工作区 |
| FR50 | Epic 3 | 工作区数据隔离 |
| FR51 | Epic 3 | 多老师模型切换 |
| FR52 | Epic 7 | 迭代步骤 Checkpoint |
| FR53 | Epic 7 | 异常中断恢复 |
| FR54 | Epic 7 | 历史 Checkpoint 回滚 |
| FR55 | Epic 7 | 完整迭代历史记录 |
| FR56 | Epic 8 | 老师模型 Prompt 作为优化目标 |
| FR57 | Epic 8 | 持久化老师模型 Prompt 版本 |
| FR58 | Epic 8 | 统计版本成功率 |
| FR59 | Epic 8 | 编辑老师模型 Prompt |
| FR60 | Epic 8 | 查看最终优化结果 |
| FR61 | Epic 8 | 复制导出 Prompt |
| FR62 | Epic 8 | Prompt 版本对比（Growth） |
| FR63 | Epic 8 | 查看诊断报告 |

## Epic List

### Epic 1: 系统基础与 API 连接

**用户成果：** 用户可以安装系统、配置各平台 API 凭证，并验证连接成功。

**FRs covered:** FR1, FR2, FR3, FR4, FR5

**NFRs addressed:** NFR9（API Key 加密存储）, NFR10（凭证仅本地传输）, NFR11（日志脱敏）, NFR16（首次配置 ≤ 5 分钟）

**ARs addressed:** AR1（ApiResponse 响应结构）, AR3（时间字段规范）

**独立性：** ✅ 完全独立，系统入口点

---

### Epic 2: 测试集管理

**用户成果：** 用户可以创建、导入、编辑测试集，支持固定任务和创意任务两种模式。

**FRs covered:** FR6, FR7, FR8, FR9, FR10, FR11, FR12, FR13, FR14, FR15

**NFRs addressed:** NFR6（WAL + FULL synchronous 数据持久化）

**依赖：** Epic 1（需要 API 连接来解析 Dify 变量）

**独立性：** ✅ 可独立使用测试集管理功能

---

### Epic 3: 优化任务配置与工作区

**用户成果：** 用户可以创建优化任务、配置算法参数、管理多个工作区。

**FRs covered:** FR16, FR17, FR18, FR19, FR20, FR21, FR22, FR23, FR23a, FR23b, FR23c, FR47, FR48, FR49, FR50, FR51

**NFRs addressed:** NFR17（首次完成优化 < 30 分钟）

**依赖：** Epic 1（API 配置）、Epic 2（测试集）

**独立性：** ✅ 可独立创建和管理任务配置

---

### Epic 4: 自动迭代优化执行

**用户成果：** 用户可以启动自动迭代优化，系统执行四层架构完成 Prompt 优化。

**FRs covered:** FR24, FR25, FR26, FR27, FR28, FR29, FR30, FR31, FR32, FR33

**NFRs addressed:** NFR1（系统延迟 < 100ms）, NFR4（并行测试集线性加速）, NFR21（优化成功率基准 ≥ 90%）, NFR22（并行 vs 串行差异 < 5%）

**依赖：** Epic 1-3（API + 测试集 + 任务配置）

**独立性：** ✅ 核心算法完整实现，可端到端运行优化

---

### Epic 5: 可视化与实时反馈

**用户成果：** 用户可以通过节点图可视化观察迭代过程，实时查看老师模型思考过程。

**FRs covered:** FR35, FR36, FR37, FR38, FR39

**NFRs addressed:** NFR2（流式首字节 < 500ms）, NFR3（节点图 60fps）

**ARs addressed:** AR2（WebSocket correlationId 透传）

**依赖：** Epic 4（优化引擎产生可视化数据）

**独立性：** ✅ 可视化层完整，无需后续 Epic

---

### Epic 6: 用户介入与控制

**用户成果：** 用户可以暂停迭代、编辑中间产物、对话引导老师模型、控制迭代进程。

**FRs covered:** FR40, FR41, FR42, FR43, FR44, FR45, FR46

**NFRs addressed:** NFR24（错误信息可读性 100% 覆盖）

**UX 补充说明：** 关键操作按钮点击区域 ≥ 44px × 44px（来自 UX 无障碍规范）

**依赖：** Epic 4-5（优化引擎 + 可视化）

**独立性：** ✅ 介入功能完整，无需后续 Epic

---

### Epic 7: 可靠性与断点续跑

**用户成果：** 用户可以在异常中断后恢复迭代，从任意 Checkpoint 回滚，完整历史记录可追溯。

**FRs covered:** FR52, FR53, FR54, FR55

**NFRs addressed:** NFR5（断点恢复率 100%）, NFR6（WAL + FULL synchronous）, NFR7（Checkpoint 完整性 100%）, NFR8（API 自动重试 3 次）, NFR23（API 调用超时阈值 ≤ 60 秒）, NFR25（内存限制）, NFR26（离线功能 100%）

**可靠性补充说明：** 无运行任务时 5 分钟自动保存当前状态

**依赖：** Epic 4（优化引擎产生 Checkpoint 数据）

**独立性：** ✅ 可靠性机制完整，无需后续 Epic

---

### Epic 8: 结果输出与元优化

**用户成果：** 用户可以查看、导出优化结果，查看诊断报告，并使用元优化功能优化老师模型 Prompt。

**FRs covered:** FR34, FR56, FR57, FR58, FR59, FR60, FR61, FR62, FR63
（其中 Growth FR: FR34, FR62）

**NFRs addressed:** NFR18（界面中文）

**依赖：** Epic 4-7（完整优化流程）

**独立性：** ✅ 结果输出和元优化功能完整

---

## Epic 1: 系统基础与 API 连接

### Story 1.1: 项目初始化与基础架构

**Related ARs:** AR1（ApiResponse 响应结构）, AR3（时间字段规范）

**As a** 开发者,
**I want** 初始化 Prompt Faster 项目的前后端基础架构,
**So that** 后续功能开发有统一的代码组织和配置基础。

**Acceptance Criteria:**

**Given** 空的项目目录
**When** 执行项目初始化
**Then** 创建以下结构：
  - `backend/`: Rust Cargo 项目 (edition 2024)
  - `frontend/`: Vite + React + TypeScript 项目
  - 配置 SQLite 连接与 WAL 模式（仅为后续表创建做准备，不一次性创建全部业务表）
**And** 后端可以 `cargo run` 启动 HTTP 服务器
**And** 前端可以 `npm run dev` 启动开发服务器
**And** 前后端可以通过 API 通信
**And** 实现 `ApiResponse<T>` 统一响应结构，`data` 与 `error` 字段互斥 (AR1)
**And** 时间字段使用 Unix 毫秒时间戳，字段命名 `*_at` 后缀 (AR3)

---

### Story 1.2: Dify API 凭证配置

**Related FRs:** FR1（配置 Dify 工作流 API 凭证）

**As a** Prompt 优化用户,
**I want** 配置 Dify 工作流的 API 凭证（地址 + API Key）,
**So that** 系统可以调用我的 Dify 工作流进行优化测试。

**Acceptance Criteria:**

**Given** 用户首次进入系统
**When** 用户填写 Dify API 地址和 API Key
**Then** 表单验证格式正确性（URL 格式、Key 非空）
**And** 凭证暂存于前端状态（尚未持久化）
**And** 界面提示"已填写，待测试连接"

**Given** 用户输入无效的 API 地址格式
**When** 提交表单
**Then** 显示友好的错误提示

---

### Story 1.3: 通用大模型 API 凭证配置

**Related FRs:** FR2（配置通用大模型 API 凭证）

**As a** Prompt 优化用户,
**I want** 配置通用大模型 API 凭证（硅基流动、魔搭社区）,
**So that** 系统可以使用这些平台的模型作为老师模型。

**Acceptance Criteria:**

**Given** 用户在 API 配置页面
**When** 用户选择 Provider 类型（硅基流动 / 魔搭社区）
**Then** 显示对应的配置表单（API Key、Base URL）
**And** 表单验证格式正确性
**And** 凭证暂存于前端状态

**Given** 用户切换 Provider 类型
**When** 选择不同的 Provider
**Then** 表单自动切换并清空之前输入

---

### Story 1.4: API 连接测试

**Related FRs:** FR3（测试 API 连接是否成功）
**Related NFRs:** NFR11（日志脱敏）

**As a** Prompt 优化用户,
**I want** 测试已配置的 API 连接是否成功,
**So that** 我能确认配置正确后再开始优化任务。

**Acceptance Criteria:**

**Given** 用户已填写 Dify API 凭证
**When** 用户点击"测试连接"
**Then** 系统调用 Dify API 的健康检查端点
**And** 成功时显示绿色"连接成功"提示
**And** 失败时显示红色错误信息（含具体原因）

**Given** 用户已填写通用大模型 API 凭证
**When** 用户点击"测试连接"
**Then** 系统发送简单请求验证 API Key 有效性
**And** 成功时显示模型列表或"连接成功"
**And** 失败时显示具体错误（401/网络错误等）

**Given** API 调用失败
**When** 显示错误信息
**Then** 日志中 API Key 被脱敏处理 (NFR11)

---

### Story 1.5: 凭证持久化与老师模型参数配置

**Related FRs:** FR4（持久化保存 API 凭证）, FR5（配置老师模型参数）
**Related NFRs:** NFR9（API Key 加密存储）, NFR10（凭证仅本地）, NFR16（首次配置 ≤ 5 分钟）

**As a** Prompt 优化用户,
**I want** 保存 API 凭证供后续使用，并配置老师模型参数,
**So that** 下次启动系统时无需重新配置，且可以调整模型行为。

**Acceptance Criteria:**

**Given** 用户已配置并测试通过 API 凭证
**When** 用户点击"保存配置"
**Then** 凭证使用 AES-GCM 加密后存入 SQLite (NFR9)
**And** 仅在本地存储，不传输到外部 (NFR10)
**And** 下次启动时自动加载已保存的凭证

**Given** 用户在老师模型参数配置区域
**When** 用户调整 temperature、top_p、max_tokens 等参数
**Then** 参数值显示合理范围约束
**And** 参数与凭证一起持久化保存

**Given** 用户首次配置完成
**When** 计算配置耗时
**Then** 总耗时应 ≤ 5 分钟 (NFR16)

---

### Epic 1 Story 总结

| Story | FR 覆盖 | NFR/AR 覆盖 |
|-------|---------|-------------|
| 1.1 | - | AR1, AR3 |
| 1.2 | FR1 | - |
| 1.3 | FR2 | - |
| 1.4 | FR3 | NFR11 |
| 1.5 | FR4, FR5 | NFR9, NFR10, NFR16 |

**共 5 个 Stories，覆盖 Epic 1 全部 5 个 FRs + 4 个 NFRs + 2 个 ARs。**

---

## Epic 2: 测试集管理

### Story 2.1: 测试集数据模型与基础 CRUD

**Related FRs:** FR6（手动创建测试集）, FR15（编辑删除测试集）
**Related NFRs:** NFR6（WAL + FULL synchronous 数据持久化）

**As a** Prompt 优化用户,
**I want** 手动创建、编辑和删除测试集,
**So that** 我可以管理用于优化任务的测试数据。

**Acceptance Criteria:**

**Given** 用户在测试集管理页面
**When** 用户点击"新建测试集"
**Then** 显示测试集创建表单（名称、描述）
**And** 创建成功后显示在测试集列表中
**And** 数据持久化到 SQLite，配置为 WAL 模式 + FULL synchronous（NFR6）

**Given** 用户选择一个已有测试集
**When** 用户点击"编辑"
**Then** 可以修改测试集名称、描述和测试用例
**And** 保存后变更立即生效

**Given** 用户选择一个测试集
**When** 用户点击"删除"并确认
**Then** 测试集从列表中移除
**And** 关联数据同步清理

---

### Story 2.2: 测试集批量导入

**Related FRs:** FR7（批量导入测试集）

**As a** Prompt 优化用户,
**I want** 批量导入测试集（txt 格式）,
**So that** 我可以快速导入大量测试用例而无需逐条手动输入。

**Acceptance Criteria:**

**Given** 用户在测试集页面
**When** 用户上传符合格式的 txt 文件
**Then** 系统解析文件内容并显示预览
**And** 用户确认后批量创建测试用例

**Given** 用户上传格式不正确的文件
**When** 解析失败
**Then** 显示友好的错误提示，说明正确格式

**Given** 文件包含 100+ 条测试用例
**When** 批量导入
**Then** 导入过程显示进度
**And** 导入完成后显示成功/失败统计

---

### Story 2.3: 测试集模板保存与复用

**Related FRs:** FR8（测试集配置保存为模板）

**As a** Prompt 优化用户,
**I want** 将测试集配置保存为模板复用,
**So that** 我可以快速创建结构相似的测试集。

**Acceptance Criteria:**

**Given** 用户已配置好一个测试集
**When** 用户点击"保存为模板"
**Then** 弹出模板命名对话框
**And** 保存后模板出现在模板列表中

**Given** 用户在创建新测试集时
**When** 用户选择"从模板创建"
**Then** 显示可用模板列表
**And** 选择后自动填充模板配置
**And** 用户可以在此基础上修改

---

### Story 2.4: Dify 变量解析与 Prompt 变量指定

**Related FRs:** FR9（Dify API 自动解析输入变量）, FR10（指定待优化 system prompt 变量）

**As a** Prompt 优化用户,
**I want** 系统自动解析 Dify 工作流的输入变量，并让我指定哪个变量是待优化的 system prompt,
**So that** 系统知道如何调用我的工作流以及优化哪个部分。

**Acceptance Criteria:**

**Given** 用户已配置 Dify API 凭证且连接成功
**When** 用户关联 Dify 工作流到测试集
**Then** 系统自动调用 Dify API 获取工作流输入变量结构
**And** 显示变量列表供用户配置

**Given** 变量列表已加载
**When** 用户选择一个变量作为"待优化 system prompt"
**Then** 该变量被标记为优化目标
**And** 其他变量可以设置默认值或关联测试用例字段

**Given** Dify API 调用失败
**When** 无法获取变量结构
**Then** 显示错误信息并提供重试选项

---

### Story 2.5: 通用 API 自定义变量与固定任务标准答案

**Related FRs:** FR11（通用 API 自定义输入变量）, FR12（固定任务标准答案）

**As a** Prompt 优化用户,
**I want** 在通用 API 场景下自定义输入变量，并为固定任务提供标准答案,
**So that** 系统可以正确调用通用大模型并评估输出是否正确。

**Acceptance Criteria:**

**Given** 用户选择"通用 API"作为执行目标
**When** 用户配置测试集
**Then** 显示自定义变量编辑器
**And** 用户可以添加/编辑/删除输入变量
**And** 每个变量可设置名称、类型、默认值

**Given** 用户选择"固定任务"模式
**When** 用户编辑测试用例
**Then** 显示"标准答案"输入字段
**And** 标准答案与测试用例一起保存
**And** 系统后续可用于精确匹配评估

---

### Story 2.6: 创意任务配置（核心诉求与结构化约束）

**Related FRs:** FR13（创意任务核心诉求）, FR14（创意任务结构化约束）

**As a** Prompt 优化用户,
**I want** 为创意任务仅提供核心诉求（无标准答案），并设置结构化约束,
**So that** 系统可以评估创意任务输出是否符合我的期望。

**Acceptance Criteria:**

**Given** 用户选择"创意任务"模式
**When** 用户编辑测试用例
**Then** 显示"核心诉求"输入字段（替代标准答案）
**And** 核心诉求用自然语言描述期望

**Given** 用户在创意任务配置页面
**When** 用户点击"添加结构化约束"
**Then** 可以配置以下约束类型：
  - 长度限制（最小/最大字符数）
  - 必含关键词列表
  - 禁止内容列表
  - 格式要求（如 JSON/Markdown）

**Given** 用户保存创意任务配置
**When** 配置包含约束
**Then** 约束与测试用例一起持久化
**And** 约束以结构化方式存储，供后续评估引擎读取和检查

---

### Epic 2 Story 总结

| Story | FR 覆盖 | NFR 覆盖 |
|-------|---------|----------|
| 2.1 | FR6, FR15 | NFR6 |
| 2.2 | FR7 | - |
| 2.3 | FR8 | - |
| 2.4 | FR9, FR10 | - |
| 2.5 | FR11, FR12 | - |
| 2.6 | FR13, FR14 | - |

**共 6 个 Stories，覆盖 Epic 2 全部 10 个 FRs + 1 个 NFR。**

---

## Epic 3: 优化任务配置与工作区

### Story 3.1: 优化任务创建与基本配置

**Related FRs:** FR16（创建优化任务）, FR17（选择执行目标类型）, FR18（选择任务模式）, FR19（输入优化目标）, FR21（关联测试集）
**Related NFRs:** NFR17（首次完成优化 < 30 分钟）

**As a** Prompt 优化用户,
**I want** 创建优化任务，配置执行目标、任务模式、优化目标，并关联测试集,
**So that** 我可以开始一个新的 Prompt 优化流程。

**Acceptance Criteria:**

**Given** 用户在任务管理页面
**When** 用户点击"新建优化任务"
**Then** 显示任务创建向导
**And** 可以输入任务名称和描述 (FR16)

**Given** 用户在创建任务时
**When** 用户选择"执行目标"
**Then** 可以在 Dify 工作流 / 通用 API 之间选择 (FR17)
**And** 根据选择显示对应的配置项

**Given** 用户在创建任务时
**When** 用户选择"任务模式"
**Then** 可以在 固定任务 / 创意任务 之间选择 (FR18)

**Given** 用户在创建任务时
**When** 用户填写"优化目标"
**Then** 以自然语言描述期望的优化效果 (FR19)

**Given** 用户在创建任务时
**When** 用户选择"关联测试集"
**Then** 显示可用测试集列表（来自 Epic 2）(FR21)
**And** 可以选择一个或多个测试集

---

### Story 3.2: 初始 Prompt 与迭代终止条件

**Related FRs:** FR20（初始 Prompt 填写/留空）, FR23c（最大迭代轮数、通过率阈值、数据划分策略）

**As a** Prompt 优化用户,
**I want** 配置初始 Prompt 和迭代终止条件,
**So that** 我可以控制优化的起点和终止条件。

**Acceptance Criteria:**

**Given** 用户在任务配置页面
**When** 用户配置"初始 Prompt"
**Then** 可以选择留空或填写初始 Prompt (FR20)
**And** 当用户选择留空时，UI 显示提示文案：『留空时，系统将在后续迭代中自动生成候选 Prompt』

**Given** 用户在任务配置页面
**When** 用户设置"最大迭代轮数"
**Then** 显示数值输入框，带合理范围约束（如 1-100）(FR23c)
**And** 默认值为推荐值（如 10）

**Given** 用户在任务配置页面
**When** 用户设置"通过率阈值"
**Then** 显示百分比输入框 (FR23c)
**And** 提供说明文字解释通过率阈值的含义
**And** 默认值为推荐值（如 95%）

**Given** 用户在任务配置页面
**When** 用户配置"数据划分策略"
**Then** 可以选择 训练集/验证集 的划分比例 (FR23c)

---

### Story 3.3: 候选生成与多样性注入配置

**Related FRs:** FR22（候选 Prompt 生成数量）, FR23（连续失败触发多样性注入阈值）

**As a** Prompt 优化用户,
**I want** 配置候选 Prompt 生成数量和多样性注入触发条件,
**So that** 我可以控制优化算法的探索强度。

**Acceptance Criteria:**

**Given** 用户在任务配置页面
**When** 用户配置"候选 Prompt 生成数量"
**Then** 显示数值输入框 (FR22)
**And** 默认值为推荐值（如 3-5 个）
**And** 提供说明文字解释生成数量对优化效果的影响

**Given** 用户在任务配置页面
**When** 用户配置"多样性注入阈值"
**Then** 可以设置连续失败多少次后触发多样性注入 (FR23)
**And** 默认值为推荐值（如 3 次）
**And** 提供说明文字解释多样性注入的作用

---

### Story 3.4: 核心算法参数与默认配置

**Related FRs:** FR23a（核心算法参数配置）, FR23b（默认配置与一键重置）

**As a** 高级用户,
**I want** 配置核心算法参数（评估器、迭代策略等），并可一键重置为默认值,
**So that** 我可以精细调整优化算法的行为。

**Acceptance Criteria:**

**Given** 用户在任务配置页面
**When** 用户展开"高级配置"区域
**Then** 显示核心算法参数配置项 (FR23a)：
  - IterationConfig（迭代策略）
  - DataSplitConfig（高级数据划分：交叉验证折数、采样策略）
  - OutputConfig（输出配置）
  - EvaluatorConfig（评估器选择与配置）

**Given** 用户配置评估器
**When** 用户选择评估器类型
**Then** 可以选择：精确匹配 / 语义相似度 / 约束检查 / 老师模型评估
**And** 每种评估器有对应的配置项

**Given** 用户修改了高级配置
**When** 用户点击"重置为默认值"
**Then** 所有高级配置恢复为系统推荐的默认值 (FR23b)
**And** 显示确认提示

**Given** 用户不修改高级配置
**When** 使用默认值
**Then** 系统使用经过验证的默认参数
**And** 默认配置足以应对常见场景

---

### Story 3.5: 工作区创建与切换

**Related FRs:** FR47（创建多个工作区）, FR48（切换工作区）

**As a** Prompt 优化用户,
**I want** 创建多个工作区并在它们之间切换,
**So that** 我可以隔离不同项目或实验的数据。

**Acceptance Criteria:**

**Given** 用户在系统中
**When** 用户点击工作区选择器
**Then** 显示当前工作区列表
**And** 显示"新建工作区"选项

**Given** 用户点击"新建工作区"
**When** 用户输入工作区名称
**Then** 创建新工作区
**And** 自动切换到新工作区

**Given** 用户在工作区列表中
**When** 用户点击另一个工作区
**Then** 切换到该工作区
**And** 界面显示该工作区的数据（任务、测试集等）
**And** 切换过程平滑，无明显延迟

---

### Story 3.6: 工作区删除与数据隔离

**Related FRs:** FR49（删除工作区）, FR50（工作区数据隔离）

**As a** Prompt 优化用户,
**I want** 删除不需要的工作区，并确保工作区之间数据完全隔离,
**So that** 我可以保持系统整洁，且不同项目互不干扰。

**Acceptance Criteria:**

**Given** 用户在工作区管理页面
**When** 用户选择删除一个工作区
**Then** 显示确认对话框，警告数据将被删除
**And** 确认后工作区及其所有数据被删除

**Given** 系统有多个工作区
**When** 用户在工作区 A 创建任务/测试集
**Then** 这些数据不会出现在工作区 B 中
**And** 数据库层面通过 workspace_id 隔离

**Given** 用户删除当前工作区
**When** 删除完成
**Then** 自动切换到默认工作区或其他工作区

---

### Story 3.7: 多老师模型切换

**Related FRs:** FR51（不同任务使用不同老师模型）

**As a** Prompt 优化用户,
**I want** 为不同的优化任务配置不同的老师模型,
**So that** 我可以根据任务特点选择最合适的模型。

**Acceptance Criteria:**

**Given** 用户在任务配置页面
**When** 用户进入"老师模型"配置区域
**Then** 显示已配置的老师模型列表（来自 Epic 1）
**And** 可以选择一个模型用于当前任务

**Given** 用户选择不同的老师模型
**When** 保存任务配置
**Then** 该任务使用选定的老师模型
**And** 不影响其他任务的老师模型配置

**Given** 用户有多个任务
**When** 查看任务列表
**Then** 每个任务显示其配置的老师模型名称
**And** 方便用户区分不同配置

---

### Epic 3 Story 总结

| Story | FR 覆盖 | NFR 覆盖 |
|-------|---------|----------|
| 3.1 | FR16, FR17, FR18, FR19, FR21 | NFR17 |
| 3.2 | FR20, FR23c | - |
| 3.3 | FR22, FR23 | - |
| 3.4 | FR23a, FR23b | - |
| 3.5 | FR47, FR48 | - |
| 3.6 | FR49, FR50 | - |
| 3.7 | FR51 | - |

**共 7 个 Stories，覆盖 Epic 3 全部 16 个 FRs + 1 个 NFR。**
