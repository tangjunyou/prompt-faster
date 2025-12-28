---
stepsCompleted: [1, 2, 3, 4]
inputDocuments:
  - docs/project-planning-artifacts/prd.md
  - docs/project-planning-artifacts/architecture.md
  - docs/project-planning-artifacts/ux-design-specification.md
workflowType: 'epics'
lastStep: 4
project_name: 'Prompt Faster'
user_name: '耶稣'
date: '2025-12-21'
validation_status: 'PASSED'
total_epics: 8
total_stories: 50
total_frs_covered: 66
total_nfrs_addressed: 28
total_ars_addressed: 3
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

**NFRs addressed:** NFR9（API Key 加密存储）, NFR10（凭证仅本地传输）, NFR11（日志脱敏）, NFR11a（本地用户认证）, NFR11b（用户数据隔离）, NFR16（首次配置 ≤ 5 分钟）, NFR19（核心流程端到端测试 ≥ 80%）, NFR20（模块回归测试 100% 通过）

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
**NFRs addressed:** NFR1（系统延迟 < 100ms）, NFR4（并行测试集线性加速）, NFR12（新增执行引擎 < 4 小时）, NFR13（新增评估器 < 2 小时）, NFR14（新增老师模型 < 2 小时）, NFR15（核心算法替换仅影响算法模块）, NFR21（优化成功率基准 ≥ 90%）, NFR22（并行 vs 串行差异 < 5%）

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
**And** 数据库迁移使用 `SQLx migrations` 管理（对应 Architecture 要求）

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

### Story 1.6: 本地用户认证与登录流

**Related NFRs:** NFR11a（本地用户认证）

**As a** 本地使用 Prompt Faster 的用户,
**I want** 通过本地账户登录应用并确保密码安全存储,
**So that** 我的工作区和优化任务不会被未授权用户访问。

**Acceptance Criteria:**

**Given** 应用第一次启动
**When** 用户创建本地账户并设置密码
**Then** 系统使用 Argon2 对密码进行哈希存储
**And** 数据库中不存在明文密码或可逆加密密码字段

**Given** 本地账户已创建
**When** 用户输入正确的用户名和密码并尝试登录
**Then** 系统校验通过并建立登录会话
**And** UI 显示清晰的"已登录用户"状态

**Given** 用户输入错误的用户名或密码
**When** 连续尝试登录
**Then** 系统始终返回通用的"用户名或密码错误"提示
**And** 不泄露"用户名是否存在"等敏感信息

**Given** 用户已登录
**When** 用户点击"退出登录"或会话过期
**Then** 清理本地会话状态
**And** 后续访问工作区/任务配置/历史记录页面时需要重新登录

**Given** 前后端之间需要在多个请求中识别当前登录用户
**When** 设计和实现认证相关的 HTTP API 调用
**Then** 统一通过单一机制在请求中携带会话标识（例如 HTTP-only Cookie 或统一的 Authorization 头）
**And** 所有需要鉴权的接口都只依赖该机制, 不混用 query 参数、本地存储拼接等多种身份来源

**Given** 测试人员检查本地数据库和日志
**When** 查看与用户认证相关的记录
**Then** 可以验证密码仅以 Argon2 哈希形式存在
**And** 日志中不包含明文密码或完整凭证

---

### Story 1.7: 用户数据隔离与访问控制

**Related NFRs:** NFR11b（用户数据隔离）

**As a** 在同一台机器上有多个本地账户的团队成员,
**I want** 不同本地用户的工作区、优化任务、测试集和历史记录在数据层严格隔离,
**So that** 我无法看到或修改其他用户的私有数据。

**Acceptance Criteria:**

**Given** 已设计好本地数据库 schema
**When** 查看涉及工作区、任务配置、测试集、执行历史、检查点等业务表结构
**Then** 数据模型满足以下其一：
  - 模式 A：所有核心业务表都直接包含 `user_id` 字段, 用于区分不同用户的数据行
  - 模式 B：workspace 等顶层实体表包含 `user_id` 字段, 其他业务表通过 `workspace_id` 外键与之关联, 通过 join 实现用户数据隔离
**And** 无论采用哪种模式, 数据访问层在查询和写入时都必须约束当前登录用户, 确保无法跨不同 `user_id` 读写数据

**Given** 当前有用户 A 已登录
**When** 用户 A 打开工作区列表、测试集列表或历史记录视图
**Then** 返回结果仅包含 `user_id = A` 的记录
**And** 其他用户的数据不会出现在列表或详情中

**Given** 存在多个本地账户（用户 A 与用户 B）
**When** 用户 B 登录后尝试通过直接访问某个 URL/ID 加载用户 A 的工作区或执行记录
**Then** API 层基于当前登录用户进行鉴权
**And** 返回"无权限访问"或等价错误, 而不是加载成功

**Given** 系统支持导出配置或查看调试日志
**When** 登录用户执行导出或查看针对某任务的调试日志
**Then** 导出内容和日志仅包含当前用户自己的数据
**And** 不包含其他用户的任务配置或 Prompt 内容

---

### Story 1.8: 统一错误响应结构与 OpenAPI 文档

**Related ARs:** AR1（ApiResponse 响应结构）

**As a** 使用 Prompt Faster 的前端开发者/调试者,
**I want** 所有 HTTP API 有统一的错误响应结构并提供可浏览的 OpenAPI 文档,
**So that** 我可以稳定解析错误并快速理解接口。

**Acceptance Criteria:**

**Given** 服务端已实现 HTTP API
**When** 使用 HTTP 客户端调用任意业务接口（成功或失败）
**Then** 外层响应使用 `ApiResponse<T>` 结构, `data` 与 `error` 字段互斥 (AR1)
**And** 当发生错误时, `error` 字段的内容符合 `{ code: string, message: string, details?: object }` 结构
**And** `code` 字段遵循统一编码规范

**Given** Rust 服务端代码已经存在
**When** 检查业务逻辑层与 API 层的错误处理
**Then** 业务错误使用 `thiserror` 定义
**And** 应用入口/HTTP 层使用 `anyhow` 或等价机制将内部错误映射为统一响应结构 (AR1 对应实现)

**Given** HTTP 服务已启动
**When** 检查路由配置
**Then** 所有对外公开的 REST API 均挂载在 `/api/v1/...` 路径下
**And** 不存在无版本前缀的对外 API

**Given** 应用在本地开发模式启动
**When** 访问 `http://localhost:PORT/swagger`
**Then** 可以看到通过 utoipa 生成的 OpenAPI 文档
**And** 至少包含核心业务 API 的路径及请求/响应 schema

**Given** 测试人员构造多个典型错误场景（参数缺失、权限不足、资源不存在、服务内部错误等）
**When** 观察返回的 JSON 错误体
**Then** 均符合统一结构
**And** `message` 字段可读、明确, 便于前端展示与用户理解

---

### Story 1.9: 前端应用架构与类型安全 API 客户端

**As a** 参与 Prompt Faster 的前端开发者,
**I want** 前端项目使用约定的路由、数据获取与类型生成方案（React Router 7、TanStack Query、ts-rs）, 
**So that** 页面路由清晰、数据获取模式统一, 并与后端类型保持一致。

**Acceptance Criteria:**

**Given** 前端项目已创建
**When** 查看路由配置
**Then** 采用 React Router 7.x 的官方推荐写法, 并为主视图（Run View/Focus View/Workspace View）预留清晰的路由层级

**Given** 某个需要从后端获取数据的前端模块（如工作区列表、测试集列表）
**When** 检查数据请求逻辑
**Then** 使用 TanStack Query 管理请求、缓存与 loading/error 状态
**And** 避免在业务组件中散落裸露的 `fetch`/`axios` 调用

**Given** 后端已使用 Rust 定义核心 DTO（请求/响应结构）
**When** 运行 ts-rs 类型生成流程
**Then** 在前端代码中可以直接 import 对应的 TypeScript 类型
**And** 不需要手写重复的请求/响应类型定义

**Given** 有新前端开发者加入项目
**When** 查阅项目内文档或示例代码
**Then** 可以看到一个"标准页面"示例, 展示路由、TanStack Query 和 ts-rs 类型结合使用的推荐模式

---

### Story 1.10: CI 流水线与测试门禁

**Related NFRs:** NFR19（核心流程端到端测试 ≥ 80% 覆盖率）, NFR20（模块回归测试 100% 通过）

**As a** 维护 Prompt Faster 代码质量的工程负责人,
**I want** 使用 GitHub Actions + Docker Compose 建立最小可用 CI 流水线并设置测试门槛,
**So that** 每次提交都能自动验证核心流程和模块回归。

**Acceptance Criteria:**

**Given** 代码仓库托管在 GitHub
**When** 查看 `.github/workflows` 目录
**Then** 至少存在一个 CI workflow 文件, 覆盖 lint、单元测试和构建三个基本步骤

**Given** 开发者本地装有 Docker
**When** 在仓库根目录执行 `docker compose up`（或文档中约定的命令）
**Then** 可以启动包含后端、前端和数据库的开发环境
**And** 为端到端测试提供基础运行环境

**Given** 已配置针对核心用户旅程的端到端测试用例
**When** CI 在主分支或 Pull Request 上运行 E2E 测试
**Then** 报告中显示的"核心用户旅程集合被自动化 E2E 用例覆盖的比例"不低于 80%（以旅程数量计）
**And** 当该比例低于阈值时 CI 标记为失败（NFR19）

**Given** 后端和前端模块均有相应的单元/集成测试
**When** CI 在每次 Pull Request 上运行回归测试任务
**Then** 所有被标记为"必需通过"的测试全部通过, 否则 CI 状态为失败且不允许合并（NFR20）

**Given** 一次完整的 PR 流程已经执行
**When** 查看 GitHub Actions 的执行记录
**Then** 可以看到构建成功、测试通过/失败状态以及必要时的覆盖率报告链接

---

### Epic 1 Story 总结

| Story | FR 覆盖 | NFR/AR 覆盖 |
|-------|---------|-------------|
| 1.1 | - | AR1, AR3 |
| 1.2 | FR1 | - |
| 1.3 | FR2 | - |
| 1.4 | FR3 | NFR11 |
| 1.5 | FR4, FR5 | NFR9, NFR10, NFR16 |
| 1.6 | - | NFR11a |
| 1.7 | - | NFR11b |
| 1.8 | - | AR1 |
| 1.9 | - | 前端架构（React Router 7, TanStack Query, ts-rs） |
| 1.10 | - | NFR19, NFR20 |

**共 10 个 Stories，覆盖 Epic 1 全部 5 个 FRs + 8 个 NFRs + 2 个 ARs。**

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

---

## Epic 4: 自动迭代优化执行

### Story 4.1: 规律抽取层（Layer 1: Pattern Extractor）

**Related FRs:** FR24（执行规律抽取）

**As a** 系统,
**I want** 从测试集结果中抽取成功/失败规律,
**So that** 后续层可以基于规律生成更优的候选 Prompt。

**Acceptance Criteria:**

**Given** 优化任务已启动且测试集已执行
**When** Layer 1 接收到测试结果
**Then** 分析成功用例的共性特征
**And** 分析失败用例的共性特征
**And** 输出结构化的规律描述

**Given** 测试结果中既有成功也有失败
**When** 执行规律抽取
**Then** 规律描述包含"什么情况下成功"和"什么情况下失败"
**And** 规律以自然语言形式输出，供 Layer 2 使用

**Given** 所有测试用例都成功
**When** 执行规律抽取
**Then** 输出"当前 Prompt 已满足所有测试用例"的信号

---

### Story 4.2: Prompt 生成层（Layer 2: Prompt Engineer）

**Related FRs:** FR25（生成候选 Prompt）

**As a** 系统,
**I want** 基于规律抽取结果生成候选 Prompt,
**So that** 可以尝试改进当前 Prompt 的效果。

**Acceptance Criteria:**

**Given** Layer 1 输出了规律描述
**When** Layer 2 执行候选生成
**Then** 根据任务配置生成指定数量的候选 Prompt（参考 FR22 配置）
**And** 每个候选 Prompt 尝试解决识别出的失败规律

**Given** 规律描述指出特定失败模式
**When** 生成候选 Prompt
**Then** 候选 Prompt 针对该失败模式进行优化
**And** 保留原 Prompt 的成功特性

**Given** 用户配置的初始 Prompt 为空
**When** 首次执行 Layer 2
**Then** 系统从优化目标和测试集信息中生成初始候选 Prompt

---

### Story 4.3: 质量评估层（Layer 3: Quality Assessor）

**Related FRs:** FR26（评估 Prompt 效果）
**Related NFRs:** NFR21（优化成功率基准 ≥ 90%）

**As a** 系统,
**I want** 评估候选 Prompt 在测试集上的效果,
**So that** 可以判断候选 Prompt 是否优于当前 Prompt。

**Acceptance Criteria:**

**Given** Layer 2 生成了候选 Prompt
**When** Layer 3 执行质量评估
**Then** 使用配置的评估器（精确匹配/语义相似度/约束检查/老师模型）评估
**And** 计算每个候选 Prompt 的通过率

**Given** 评估结果已计算
**When** 比较候选 Prompt
**Then** 选择通过率最高的候选作为本轮最优
**And** 记录评估详情（每个测试用例的结果）

**Given** 候选 Prompt 的通过率达到配置阈值（如 95%）
**When** 评估完成
**Then** 标记优化成功
**And** 系统提供优化任务的成功率统计与报表能力，支持验证 NFR21（整体优化成功率 ≥ 90%）

---

### Story 4.4: 反思迭代层（Layer 4: Reflection Agent）

**Related FRs:** FR27（执行反思迭代）

**As a** 系统,
**I want** 对本轮迭代进行反思并决定下一步行动,
**So that** 可以持续改进 Prompt 直到达成目标或触发终止条件。

**Acceptance Criteria:**

**Given** Layer 3 完成质量评估
**When** Layer 4 执行反思
**Then** 分析本轮迭代的改进效果
**And** 判断是否继续迭代、是否需要调整策略

**Given** 本轮最优候选优于当前 Prompt
**When** 反思完成
**Then** 将最优候选设为新的当前 Prompt
**And** 继续下一轮迭代

**Given** 达到终止条件（通过率阈值 / 最大迭代轮数）
**When** 反思完成
**Then** 终止迭代循环
**And** 输出最终优化结果

---

### Story 4.5: 执行模式与并行调度

**Related FRs:** FR28（串行/并行模式选择）, FR29（并行执行测试用例）, FR30（汇总并行结果）
**Related NFRs:** NFR1（系统延迟 < 100ms）, NFR4（并行测试集线性加速）, NFR22（并行 vs 串行差异 < 5%）

**As a** Prompt 优化用户,
**I want** 选择串行或并行模式执行测试集,
**So that** 我可以根据需求在速度和资源消耗之间权衡。

**Acceptance Criteria:**

**Given** 用户在任务配置中选择"串行模式"
**When** 执行测试集
**Then** 按顺序逐条执行测试用例
**And** 每条用例执行完成后再执行下一条

**Given** 用户在任务配置中选择"并行模式"
**When** 执行测试集
**Then** 同时执行多条测试用例（并发数可配置）
**And** 提供性能监控能力，支持验证调度开销 < 100ms（NFR1，不含模型调用时间）
**And** 提供并行压测报告，支持验证并行加速比接近线性（NFR4）

**Given** 并行执行完成
**When** 汇总结果
**Then** 将所有测试用例结果合并为统一格式
**And** 提供结果对比工具，支持验证并行/串行在相同输入下结果差异 < 5%（NFR22）

---

### Story 4.6: 失败档案与多样性注入

**Related FRs:** FR31（失败档案）, FR32（多样性注入）, FR33（自动判断优化成功）

**As a** 系统,
**I want** 记录失败档案并在连续失败时触发多样性注入,
**So that** 可以避免重蹈覆辙并跳出局部最优。

**Acceptance Criteria:**

**Given** 某个候选 Prompt 在测试用例上失败
**When** 记录失败档案
**Then** 存储失败的 Prompt 片段、测试用例、失败原因
**And** 后续生成候选时参考失败档案避免重复

**Given** 连续 N 次迭代未能提升通过率（N = 用户配置的多样性注入阈值，FR23）
**When** 检测到连续失败
**Then** 触发多样性注入策略
**And** 下一轮候选生成时增加随机性/扰动

**Given** 所有测试用例都通过
**When** 系统判断优化状态
**Then** 自动标记"优化成功"(FR33)
**And** 终止迭代循环

---

### Story 4.7: 扩展模板与文档（ExecutionTarget / Evaluator / TeacherModel）

**Related NFRs:** NFR12（新增执行引擎 < 4 小时）, NFR13（新增评估器 < 2 小时）, NFR14（新增老师模型 < 2 小时）

**As a** 开发者,
**I want** 通过统一的扩展模板和文档快速新增执行目标、评估器和老师模型实现,
**So that** 可以在不修改核心框架的前提下扩展支持新的执行/评估逻辑。

**Acceptance Criteria:**

**Given** 项目已经实现基础的 ExecutionTarget / Evaluator / TeacherModel trait 定义
**When** 开发者查阅项目文档
**Then** 能找到一节专门介绍如何新增执行引擎、评估器和老师模型的扩展点
**And** 文档中包含从复制模板、实现必要方法到在配置中启用新实现的完整步骤

**Given** 开发者按文档为 ExecutionTarget 新增一个示例实现（例如 `ExampleExecutionTarget`）
**When** 仅在示例模块中实现必要接口并在工厂/注册表中注册该实现
**Then** 不需要修改现有调用 ExecutionTarget 的业务代码即可在任务配置中选择该执行目标
**And** 使用该执行目标可以完成一轮端到端优化任务
**And** 实际人力投入时间不超过 4 小时（NFR12）

**Given** 开发者按文档为 Evaluator 新增一个示例实现（例如 `ExampleEvaluator`）
**When** 仅在示例模块中实现必要接口并在配置中声明可用
**Then** 用户可以在任务配置中选择该评估器并成功运行一轮评估
**And** 实际人力投入时间不超过 2 小时（NFR13）

**Given** 开发者按文档为 TeacherModel 新增一个示例实现（例如 `ExampleTeacherModel`）
**When** 仅在示例模块中实现必要接口并在配置中声明可用
**Then** 可以将该老师模型用于一轮完整的优化任务执行
**And** 实际人力投入时间不超过 2 小时（NFR14）

**Given** 团队需要验证上述扩展耗时指标
**When** 以具备 Prompt Faster 项目上下文的开发者为基准, 从复制官方扩展模板开始到跑通文档中的示例用例结束计时
**Then** 计时范围仅包含编码与本地验证时间, 不包含依赖下载、CI 排队等等待时间

---

### Story 4.8: 核心算法模块解耦与替换演练

**Related NFRs:** NFR15（核心算法替换仅影响算法模块）

**As a** 开发者,
**I want** 将核心算法实现与调用方解耦并完成一次替换演练,
**So that** 在需要更换优化算法时，只修改算法模块自身。

**Acceptance Criteria:**

**Given** 项目已经实现完整的四层架构和自动迭代优化流程
**When** 查看代码结构和模块依赖
**Then** 可以清晰识别出"核心算法模块"所在的 crate/模块/目录
**And** 该模块对外仅通过少量受控接口（如 trait 或 facade）暴露能力
**And** 调用方（任务配置、执行调度、可视化、可靠性等）只依赖这些接口而不直接依赖内部实现细节

**Given** 为验证模块可替换性，开发者实现一个替代算法实现（例如不同的搜索/优化策略或 mock 实现）
**When** 在依赖注入/工厂或编译配置中切换到该替代实现
**Then** 代码变更局限在算法模块及其测试文件, 以及用于注册/选择算法实现的单一入口点（如工厂或 DI 配置）
**And** 其他模块无需修改即可完成编译

**Given** 使用替代算法模块重新运行一套核心端到端测试
**When** 执行测试流水线
**Then** 所有与算法无关的行为（配置、工作区管理、执行调度、可视化、可靠性、断点续跑等）测试全部通过
**And** 在切换回原始算法模块后，测试同样全部通过，证明算法模块可以在不影响其他模块的前提下被替换（NFR15）

---

### Epic 4 Story 总结

| Story | FR 覆盖 | NFR 覆盖 |
|-------|---------|----------|
| 4.1 | FR24 | - |
| 4.2 | FR25 | - |
| 4.3 | FR26 | NFR21 |
| 4.4 | FR27 | - |
| 4.5 | FR28, FR29, FR30 | NFR1, NFR4, NFR22 |
| 4.6 | FR31, FR32, FR33 | - |
| 4.7 | - | NFR12, NFR13, NFR14 |
| 4.8 | - | NFR15 |

**共 8 个 Stories，覆盖 Epic 4 全部 10 个 FRs + 8 个 NFRs。**

---

## Epic 5: 可视化与实时反馈

### Story 5.1: 节点图基础渲染

**Related FRs:** FR35（节点图形式迭代流程）, FR36（节点状态颜色变化）
**Related NFRs:** NFR3（节点图 60fps）

**As a** Prompt 优化用户,
**I want** 通过节点图形式查看迭代流程，并看到节点状态颜色变化,
**So that** 我可以直观了解优化进度和各层的执行状态。

**Acceptance Criteria:**

**Given** 优化任务正在运行
**When** 用户查看可视化面板
**Then** 显示节点图，包含四层架构节点（Pattern Extractor / Prompt Engineer / Quality Assessor / Reflection Agent）
**And** 节点之间有连接线表示数据流向

**Given** 某个节点开始执行
**When** 节点状态变化
**Then** 节点颜色更新：
  - 灰色：未开始
  - 蓝色：执行中
  - 绿色：成功完成
  - 红色：失败
  - 黄色：需要用户介入

**Given** 节点图正在渲染
**When** 用户观察动画流畅度
**Then** 提供性能监控能力，支持验证渲染帧率达到 60fps（NFR3）

---

### Story 5.2: 边动画与数据流可视化

**Related FRs:** FR37（边动画展示数据流动）

**As a** Prompt 优化用户,
**I want** 看到边动画展示数据在节点间的流动,
**So that** 我可以理解优化过程中数据是如何传递的。

**Acceptance Criteria:**

**Given** 数据从一个节点传递到下一个节点
**When** 传递发生
**Then** 对应的边显示动画效果（如流动的粒子或高亮脉冲）
**And** 动画方向与数据流向一致

**Given** 多个数据同时在不同边上流动
**When** 并行传递发生
**Then** 各条边的动画独立运行
**And** 不会造成视觉混乱

**Given** 数据传递完成
**When** 目标节点开始处理
**Then** 边动画平滑过渡到静止状态

---

### Story 5.3: 流式思考过程展示

**Related FRs:** FR38（实时查看老师模型思考过程）
**Related NFRs:** NFR2（流式首字节 < 500ms）
**Related ARs:** AR2（WebSocket correlationId 透传）

**As a** Prompt 优化用户,
**I want** 实时查看老师模型的思考过程（流式输出）,
**So that** 我可以理解模型的推理逻辑并在需要时介入。

**Acceptance Criteria:**

**Given** 老师模型正在生成响应
**When** 用户查看思考面板
**Then** 以流式方式逐字显示模型输出
**And** 提供性能监控能力，支持验证首字节延迟 < 500ms（NFR2）

**Given** 流式输出正在进行
**When** 有新内容到达
**Then** 平滑追加到已有内容后
**And** 自动滚动到最新位置

**Given** 后端使用 WebSocket 推送
**When** 建立连接
**Then** 使用 correlationId 关联请求与响应（AR2）
**And** 确保多任务场景下消息不会错乱

---

### Story 5.4: 思考面板环节标识

**Related FRs:** FR39（思考面板中当前环节标识）

**As a** Prompt 优化用户,
**I want** 在思考面板中看到当前执行环节的标识,
**So that** 我可以清楚知道模型正在进行哪个阶段的思考。

**Acceptance Criteria:**

**Given** 用户正在查看思考面板
**When** 老师模型执行不同环节
**Then** 面板顶部或侧边显示当前环节标识（如"规律抽取中"/"候选生成中"/"质量评估中"/"反思迭代中"）

**Given** 环节切换
**When** 从一个环节进入下一个
**Then** 环节标识平滑更新
**And** 之前环节的输出保留在历史区域

**Given** 环节切换后
**When** 用户查看历史区域
**Then** 历史区域以折叠/摘要形式保留各环节的输出片段（只读展示）
**And** 完整历史查看交互由 Epic 6 FR43 承接

---

### Epic 5 Story 总结

| Story | FR 覆盖 | NFR/AR 覆盖 |
|-------|---------|-------------|
| 5.1 | FR35, FR36 | NFR3 |
| 5.2 | FR37 | - |
| 5.3 | FR38 | NFR2, AR2 |
| 5.4 | FR39 | - |

**共 4 个 Stories，覆盖 Epic 5 全部 5 个 FRs + 2 个 NFRs + 1 个 AR。**

---

## Epic 6: 用户介入与控制

### Story 6.1: 暂停与继续迭代

**Related FRs:** FR40（任意节点暂停）, FR44（从暂停点继续）

**As a** Prompt 优化用户,
**I want** 在任意节点暂停迭代，并从暂停点继续,
**So that** 我可以在需要时审视当前状态或进行干预。

**Acceptance Criteria:**

**Given** 优化任务正在运行
**When** 用户点击"暂停"按钮
**Then** 系统在当前节点完成后暂停迭代
**And** 暂停按钮点击区域 ≥ 44px × 44px（UX 无障碍规范）

**Given** 迭代已暂停
**When** 用户查看界面
**Then** 节点图显示暂停状态（黄色节点）
**And** 显示"继续"按钮

**Given** 迭代已暂停
**When** 用户点击"继续"按钮
**Then** 系统从暂停点恢复迭代
**And** 保持所有中间状态不变

---

### Story 6.2: 编辑中间产物

**Related FRs:** FR41（编辑中间产物）

**As a** Prompt 优化用户,
**I want** 直接编辑当前迭代的中间产物（规律假设、Prompt）,
**So that** 我可以基于自己的理解修正或引导优化方向。

**Acceptance Criteria:**

**Given** 迭代已暂停
**When** 用户查看中间产物
**Then** 显示当前的规律假设和候选 Prompt
**And** 提供"编辑"按钮

**Given** 用户点击"编辑"按钮
**When** 进入编辑模式
**Then** 规律假设和 Prompt 变为可编辑状态
**And** 提供"保存"和"取消"按钮

**Given** 用户完成编辑
**When** 点击"保存"
**Then** 保存用户修改
**And** 后续迭代使用用户修改后的产物

---

### Story 6.3: 对话引导老师模型

**Related FRs:** FR42（对话引导老师模型）
**Related NFRs:** NFR24（错误信息可读性 100% 覆盖）

**As a** Prompt 优化用户,
**I want** 通过对话引导老师模型,
**So that** 我可以告诉模型我的想法，让它朝特定方向优化。

**Acceptance Criteria:**

**Given** 迭代已暂停
**When** 用户想要引导模型
**Then** 显示对话输入框
**And** 输入框提示"告诉老师模型你的想法..."

**Given** 用户输入引导信息
**When** 点击"发送"
**Then** 系统将用户引导信息传递给老师模型
**And** 下一轮迭代时模型参考用户引导

**Given** 用户输入无效或发送失败
**When** 系统检测到错误
**Then** 显示清晰的错误信息，说明问题原因和解决方法
**And** 本功能所有错误信息遵循系统统一错误文案规范，支撑全局 NFR24（错误信息可读性 100% 覆盖）

---

### Story 6.4: 历史迭代产物查看

**Related FRs:** FR43（查看历史迭代产物）

**As a** Prompt 优化用户,
**I want** 查看历史迭代产物,
**So that** 我可以回顾优化过程，理解模型的演进路径。

**Acceptance Criteria:**

**Given** 优化任务已执行多轮迭代
**When** 用户点击"历史"按钮
**Then** 显示历史迭代列表（按轮次排序）
**And** 每轮显示：轮次编号、时间戳、通过率

**Given** 用户选择某一轮历史
**When** 点击查看详情
**Then** 展开显示该轮的完整产物：
  - 规律假设
  - 候选 Prompt
  - 评估结果
  - 反思总结

**Given** 用户查看历史产物
**When** 尝试编辑
**Then** 历史产物为只读状态，不可编辑
**And** 显示提示"历史记录仅供查看"

---

### Story 6.5: 迭代控制（增加轮数/手动终止）

**Related FRs:** FR45（增加迭代轮数）, FR46（手动终止并选择 Prompt）

**As a** Prompt 优化用户,
**I want** 随时增加迭代轮数或手动终止并选择满意的 Prompt,
**So that** 我可以灵活控制优化进程。

**Acceptance Criteria:**

**Given** 优化任务正在运行或已暂停
**When** 用户想增加迭代轮数
**Then** 显示"增加轮数"输入框
**And** 用户可以输入要增加的轮数

**Given** 用户输入增加轮数
**When** 点击"确认"
**Then** 系统将最大迭代轮数增加指定数量
**And** 继续执行直到达到新的上限或成功

**Given** 优化任务正在运行或已暂停
**When** 用户点击"终止"按钮
**Then** 显示候选 Prompt 列表（按通过率排序）
**And** 用户可以选择满意的 Prompt

**Given** 用户选择 Prompt
**When** 点击"确认终止"
**Then** 终止迭代并保存用户选择的 Prompt 作为最终结果
**And** 标记任务为"用户终止"状态

---

### Epic 6 Story 总结

| Story | FR 覆盖 | NFR 覆盖 |
|-------|---------|----------|
| 6.1 | FR40, FR44 | - |
| 6.2 | FR41 | - |
| 6.3 | FR42 | NFR24 |
| 6.4 | FR43 | - |
| 6.5 | FR45, FR46 | - |

**共 5 个 Stories，覆盖 Epic 6 全部 7 个 FRs + 1 个 NFR。**

---

## Epic 7: 可靠性与断点续跑

### Story 7.1: Checkpoint 自动保存

**Related FRs:** FR52（每个迭代步骤保存 Checkpoint）
**Related NFRs:** NFR6（WAL + FULL synchronous）, NFR7（Checkpoint 完整性 100%）, NFR25（内存限制）

**As a** 系统,
**I want** 在每个迭代步骤自动保存 Checkpoint,
**So that** 可以在任何时刻恢复到已保存的状态。

**Acceptance Criteria:**

**Given** 优化任务正在执行
**When** 完成一个迭代步骤（如 Layer 1/2/3/4 执行完毕）
**Then** 自动保存当前状态为 Checkpoint
**And** 使用 WAL + FULL synchronous 模式确保数据持久化（NFR6）

**Given** 保存 Checkpoint
**When** 写入完成
**Then** 提供完整性校验能力，支撑 NFR7（Checkpoint 完整性 100%）
**And** Checkpoint 包含：当前迭代轮次、各层输出、测试结果、时间戳

**Given** 系统运行中
**When** 内存使用接近限制
**Then** 遵循内存管理策略（NFR25）
**And** 必要时将旧 Checkpoint 持久化到磁盘以释放内存
**And** 提供内存使用监控与告警阈值配置，用于验证运行时内存符合 NFR25 要求

**Given** 无运行任务
**When** 5 分钟计时器触发
**Then** 自动保存当前状态（可靠性补充说明）

---

### Story 7.2: 断点恢复与异常处理

**Related FRs:** FR53（异常中断后恢复到断点状态）
**Related NFRs:** NFR5（断点恢复率 100%）, NFR8（API 自动重试 3 次）, NFR23（API 调用超时阈值 ≤ 60 秒）, NFR26（离线功能 100%）

**As a** Prompt 优化用户,
**I want** 在异常中断后恢复到断点状态,
**So that** 不会因为意外中断而丢失优化进度。

**Acceptance Criteria:**

**Given** 系统异常中断（如崩溃、断电、网络中断）
**When** 用户重新启动应用
**Then** 自动检测未完成的任务
**And** 提示用户"检测到上次未完成的任务，是否恢复？"

**Given** 用户选择恢复
**When** 执行恢复操作
**Then** 从最近的 Checkpoint 恢复状态
**And** 提供恢复率统计能力，支撑 NFR5（断点恢复率 100%）

**Given** API 调用失败
**When** 检测到网络错误或超时
**Then** 自动重试最多 3 次（NFR8）
**And** 单次调用超时阈值 ≤ 60 秒（NFR23）

**Given** 网络完全离线
**When** 用户尝试操作
**Then** 离线功能正常可用（NFR26）
**And** 显示提示"当前离线，部分功能受限"

---

### Story 7.3: 历史 Checkpoint 回滚

**Related FRs:** FR54（从任意历史 Checkpoint 回滚）

**As a** Prompt 优化用户,
**I want** 从任意历史 Checkpoint 回滚,
**So that** 我可以撤销错误的操作或尝试不同的优化路径。

**Acceptance Criteria:**

**Given** 优化任务已执行多轮迭代
**When** 用户点击"回滚"按钮
**Then** 显示所有可用的 Checkpoint 列表
**And** 每个 Checkpoint 显示：时间戳、迭代轮次、通过率摘要

**Given** 用户选择某个 Checkpoint
**When** 点击"确认回滚"
**Then** 系统恢复到该 Checkpoint 的状态
**And** 该 Checkpoint 之后的状态被归档（不删除，可追溯）

**Given** 回滚完成
**When** 用户继续迭代
**Then** 从回滚点开始新的迭代分支
**And** 历史记录保留完整的分支信息

---

### Story 7.4: 完整迭代历史记录

**Related FRs:** FR55（保存完整的迭代历史记录）

**As a** Prompt 优化用户,
**I want** 系统保存完整的迭代历史记录,
**So that** 我可以追溯整个优化过程的演进路径。

**Acceptance Criteria:**

**Given** 优化任务执行过程中
**When** 发生任何状态变化
**Then** 记录该变化到历史日志
**And** 包含：时间戳、操作类型、操作者（系统/用户）、变化详情

**Given** 用户查看历史记录
**When** 打开历史面板
**Then** 显示完整的时间线视图
**And** 支持按时间/操作类型/轮次筛选

**Given** 用户导出历史记录
**When** 点击"导出"
**Then** 支持导出为 JSON 格式
**And** 包含完整的元数据和状态快照

---

### Epic 7 Story 总结

| Story | FR 覆盖 | NFR 覆盖 |
|-------|---------|----------|
| 7.1 | FR52 | NFR6, NFR7, NFR25 |
| 7.2 | FR53 | NFR5, NFR8, NFR23, NFR26 |
| 7.3 | FR54 | - |
| 7.4 | FR55 | - |

**共 4 个 Stories，覆盖 Epic 7 全部 4 个 FRs + 7 个 NFRs。**

---

## Epic 8: 结果输出与元优化

### Story 8.1: 结果查看与导出

**Related FRs:** FR60（查看最终优化结果）, FR61（复制或导出 Prompt）
**Related NFRs:** NFR18（界面中文）

**As a** Prompt 优化用户,
**I want** 查看最终优化结果 Prompt 并导出,
**So that** 我可以使用优化后的 Prompt 并分享给他人。

**Acceptance Criteria:**

**Given** 优化任务完成
**When** 用户查看结果页面
**Then** 显示最终优化结果 Prompt
**And** 界面文案使用中文（NFR18）

**Given** 用户想要复制 Prompt
**When** 点击"复制"按钮
**Then** Prompt 内容复制到剪贴板
**And** 显示"已复制"提示

**Given** 用户想要导出 Prompt
**When** 点击"导出"按钮
**Then** 显示格式选择：Markdown / JSON / XML
**And** 用户选择后下载对应格式的文件

---

### Story 8.2: 诊断报告

**Related FRs:** FR63（查看诊断报告了解"为什么之前不行"）

**As a** Prompt 优化用户,
**I want** 查看诊断报告了解"为什么之前不行",
**So that** 我可以理解优化过程中的问题并学习如何写更好的 Prompt。

**Acceptance Criteria:**

**Given** 优化任务完成或中途失败
**When** 用户点击"诊断报告"
**Then** 显示优化过程的诊断分析
**And** 包含：失败原因摘要、关键转折点、改进建议

**Given** 诊断报告显示
**When** 用户查看失败原因
**Then** 以自然语言解释"为什么之前的 Prompt 不行"
**And** 提供具体的失败用例示例

**Given** 用户想要深入分析
**When** 点击某个失败用例
**Then** 展开显示详细的输入/输出/期望对比
**And** 高亮差异部分

---

### Story 8.3: 元优化基础（老师模型 Prompt 优化）

**Related FRs:** FR56（老师模型 Prompt 作为优化目标）, FR57（持久化老师模型 Prompt 版本）, FR58（统计每个版本成功率）

**As a** 高级用户,
**I want** 将老师模型 Prompt 作为优化目标并追踪版本效果,
**So that** 我可以优化老师模型本身，提升整体优化质量。

**Acceptance Criteria:**

**Given** 用户进入元优化模式
**When** 选择"优化老师模型 Prompt"
**Then** 系统将老师模型 Prompt 作为优化目标（FR56）
**And** 使用用户历史任务作为测试集

**Given** 老师模型 Prompt 发生变更
**When** 保存变更
**Then** 系统持久化该版本（FR57）
**And** 记录版本号、时间戳、变更说明

**Given** 用户查看老师模型 Prompt 版本列表
**When** 打开版本管理页面
**Then** 显示所有历史版本
**And** 每个版本显示成功率统计（FR58）

---

### Story 8.4: 高级用户直接编辑老师模型 Prompt

**Related FRs:** FR59（高级用户直接编辑老师模型 Prompt）

**As a** 高级用户,
**I want** 直接编辑老师模型 Prompt,
**So that** 我可以根据经验快速调整老师模型行为。

**Acceptance Criteria:**

**Given** 用户进入老师模型管理页面
**When** 点击"编辑"按钮
**Then** 显示老师模型 Prompt 编辑器
**And** 提供语法高亮和格式化

**Given** 用户编辑完成
**When** 点击"保存"
**Then** 验证 Prompt 格式有效性
**And** 保存为新版本

**Given** 用户担心改坏
**When** 编辑过程中
**Then** 显示"回滚到上一版本"选项
**And** 提供"预览效果"功能（在少量测试用例上试运行）

---

### Story 8.5: Prompt 版本对比（Growth）

**Related FRs:** FR62（对比任意两个 Prompt 版本效果差异）
**Phase:** Growth

**As a** Prompt 优化用户,
**I want** 对比任意两个 Prompt 版本在同一测试集上的效果差异,
**So that** 我可以量化评估不同版本的优劣。

**Acceptance Criteria:**

**Given** 用户有多个 Prompt 版本
**When** 选择"版本对比"功能
**Then** 显示版本选择器（可选择任意两个版本）

**Given** 用户选择两个版本
**When** 点击"开始对比"
**Then** 使用同一测试集分别评估两个版本
**And** 显示对比结果：通过率差异、具体用例差异

**Given** 对比结果显示
**When** 用户查看详情
**Then** 高亮显示两个版本表现不同的用例
**And** 对表现不同的用例提供简要差异说明（如失败原因或输出差异），帮助用户理解为什么 A 更好/更差

---

### Story 8.6: 创意任务多样性检测（Growth）

**Related FRs:** FR34（检测创意任务输出的多样性分数）
**Phase:** Growth

**As a** 创意任务用户,
**I want** 系统检测创意任务输出的多样性分数,
**So that** 我可以确保优化后的 Prompt 不会导致输出过于单一。

**Acceptance Criteria:**

**Given** 用户创建的优化任务标记为"创意任务"
**When** 执行质量评估
**Then** 除通过率外，额外计算多样性分数

**Given** 多样性分数计算完成
**When** 显示评估结果
**Then** 展示多样性指标（如：词汇多样性、结构多样性、语义多样性）
**And** 与基准线对比显示改进/退化

**Given** 多样性分数过低
**When** 系统检测到问题
**Then** 发出警告"优化可能导致输出过于单一"
**And** 建议用户考虑调整优化目标

---

### Epic 8 Story 总结

| Story | FR 覆盖 | NFR 覆盖 | Phase |
|-------|---------|----------|-------|
| 8.1 | FR60, FR61 | NFR18 | MVP |
| 8.2 | FR63 | - | MVP |
| 8.3 | FR56, FR57, FR58 | - | MVP |
| 8.4 | FR59 | - | MVP |
| 8.5 | FR62 | - | Growth |
| 8.6 | FR34 | - | Growth |

**共 6 个 Stories，覆盖 Epic 8 全部 9 个 FRs + 1 个 NFR（其中 2 个 Growth Stories）。**
