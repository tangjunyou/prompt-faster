---
stepsCompleted: [1, 2, 3]
inputDocuments:
  - docs/analysis/brainstorming-session-2025-12-12.md
  - docs/prd.md
workflowType: 'research'
lastStep: 3
research_type: 'technical'
research_topic: 'algorithm-specification'
research_goals: '定义 Prompt Faster 核心迭代算法的完整技术规格，以高度模块化、可插拔的架构设计为核心原则'
user_name: '耶稣'
date: '2025-12-15'
web_research_enabled: true
source_verification: true
revision_note: '2025-12-15 Step 3 增量补丁：4.2.1 EvaluationResult、4.2.2 OptimizationResult、4.2.3 ReflectionResult、4.2.4 UnifiedReflection、决策D 分支治理策略'
---

# Research Report: Algorithm Specification

**Date:** 2025-12-14  
**Author:** 耶稣  
**Research Type:** Technical Specification

---

## Executive Summary

本文档定义 Prompt Faster 核心迭代算法的完整技术规格，**以高度模块化、可插拔的架构设计为核心原则**，供架构师和开发者实现参考。

**Key Technical Findings:**
- 三阶段迭代流程：规律收敛 → Prompt生成 → 测试迭代
- 四层处理器架构：Pattern Extractor → Prompt Engineer → Quality Assessor → Reflection Agent
- 完整数据结构定义：Rule、RuleSystem、Checkpoint、IterationState
- **模块化接口设计**：核心组件均可独立替换

**Technical Recommendations:**
- 按本规格实现核心算法
- 开放用户配置项以支持灵活定制
- 实现完整的安全检查机制（回归检测、震荡检测）
- **遵循模块化原则，每个核心组件定义清晰的 Trait 接口**

---

## Table of Contents

1. 文档概述
2. **研究目标与核心假设** *(Step 1)*
3. **业界调研与设计决策** *(Step 2)*
4. **核心算法架构设计** *(Step 3 新增)*
5. 算法总体架构
6. Phase 0: 规律收敛阶段
7. Phase 1: 首次 Prompt 生成
8. Phase 2: 测试与反思迭代
9. 用户配置规格
10. 老师 Prompt 模板规格
11. 状态机定义
12. 最佳实践来源
13. 附录：错误处理

---

## 1. 文档概述

### 1.1 目的

本文档定义 Prompt Faster 核心迭代算法的完整技术规格，供架构师和开发者实现参考。

### 1.2 设计来源

- 业界最佳实践：DSPy MIPROv2、Reflexion、PromptWizard、GEPA、TextGrad
- 产品负责人设计输入：规律驱动迭代机制
- 深度讨论确认：2025-12-14

### 1.3 核心设计原则

| 原则 | 说明 |
|------|------|
| **规律是上层建筑** | 规律贯穿整个迭代流程，随迭代同步进化 |
| **反思分类处理** | 区分规律问题和表达问题，采用不同处理方式 |
| **多样性保持** | 通过 Pareto 前沿思想避免过早收敛到局部最优 |
| **用户可控** | 关键参数开放给用户配置 |
| **高度模块化** | 每个核心组件可独立替换，通过 Trait 接口抽象 |
| **可插拔架构** | 评估器、处理器、老师模型均可灵活切换 |

---

## 2. 研究目标与核心假设

> **Step 1 产出** — 2025-12-15
> 
> 本章节定义研究目标、核心假设、研究边界与成功标准，为后续步骤提供明确方向。

### 2.1 研究目标声明

**主目标**：
> 定义 Prompt Faster 核心迭代算法的完整技术规格，以**高度模块化、可插拔的架构设计**为核心原则，确保：
> 1. **算法层模块化** — 四层处理器均可独立替换
> 2. **评估器可插拔** — 支持多种评估策略切换
> 3. **策略编排层** — 用户可通过配置组合不同模块
> 4. **双任务模式** — 固定任务与创意任务统一抽象

**子目标**：
- 为架构师提供清晰的模块边界定义
- 为开发者提供可直接实现的接口规格
- 为后续迭代提供可扩展的技术基础

### 2.2 核心假设列表

以下假设将在 Step 2（深度调研）中验证。如果某个假设被推翻，只需修改对应模块，而不需要推倒重来。

| 假设编号 | 假设内容 | 验证方式 | 如果推翻的影响范围 |
|----------|----------|----------|--------------------|
| **H1** | "规律驱动"是有效的核心机制，优于直接 Prompt 修改 | 原型测试 + 案例分析 + 业界对比 | 需替换 Phase 0 设计 |
| **H2** | 四层处理器（Pattern Extractor → Prompt Engineer → Quality Assessor → Reflection Agent）是合理的抽象粒度 | 对比 DSPy、TextGrad、PromptWizard 架构 | 调整处理器边界 |
| **H3** | 反思分类（规律问题 vs 表达问题）能有效区分问题根因 | 案例分析 + 原型测试 | 修改反思逻辑 |
| **H4** | 规律冲突可通过抽象层级解决（max_abstraction_level 机制） | 原型测试 + 边界案例分析 | 增加冲突处理策略 |

**假设验证原则**：
- 每个假设独立验证
- 验证失败时，明确影响范围
- 局部修正优于全局重构

### 2.3 关键研究问题

以下问题将在 Step 2（深度调研）和 Step 3（架构设计）中回答：

| 问题编号 | 问题内容 | 回答步骤 |
|----------|----------|----------|
| **Q1** | "规律驱动"机制是否是最优选择？有哪些替代方案？ | Step 2 |
| **Q2** | 四层处理器的边界如何定义才能实现真正的模块化？ | Step 3 |
| **Q3** | 评估器应该支持哪些评估策略？如何抽象为统一接口？ | Step 2 + Step 3 |
| **Q4** | 策略编排层如何设计才能让用户灵活组合模块？ | Step 3 |
| **Q5** | 双任务模式（固定/创意）如何统一抽象？ | Step 3 |
| **Q6** | 业界框架（DSPy、TextGrad、PromptWizard）的模块化设计有哪些可借鉴之处？ | Step 2 |

### 2.4 研究边界

**IN SCOPE（研究范围内）**：
- 核心算法架构设计
- 模块化接口定义（Trait）
- 策略编排层设计
- 双任务模式统一抽象
- 数据结构与状态机定义
- 老师模型 Prompt 模板规格

**OUT OF SCOPE（研究范围外）**：
- 具体实现代码（由开发阶段完成）
- UI/UX 设计（由 UX 设计文档覆盖）
- 部署方案（由架构设计文档覆盖）
- 前端状态管理细节

### 2.5 成功标准

| 标准编号 | 成功标准 | 验证方式 |
|----------|----------|----------|
| **S1** | 每个核心模块可独立测试 | 架构评审 |
| **S2** | 新增评估器的工作量 < 2 小时 | 实现 Evaluator trait 的复杂度评估 |
| **S3** | 新增处理器的工作量 < 4 小时 | 实现 Processor trait 的复杂度评估 |
| **S4** | 策略可通过配置组合而非代码修改 | 配置文件设计评审 |
| **S5** | 核心算法替换仅影响算法模块 | 模块依赖分析 |

### 2.6 业界参考来源

以下来源将在 Step 2 深度调研：

| 来源 | 核心机制 | 调研重点 | 参考链接 |
|------|----------|----------|----------|
| **DSPy MIPROv2** | Teleprompter 基类 + Bayesian 搜索 | Optimizer 接口设计、模块化架构 | [GitHub: stanfordnlp/dspy](https://github.com/stanfordnlp/dspy) |
| **TextGrad** | 文本梯度 + 反向传播 | 梯度聚合策略、Variable 抽象 | [arXiv:2406.07496](https://arxiv.org/abs/2406.07496) |
| **PromptWizard** | Mutation + Refinement | DatasetSpecificProcessing 抽象 | [GitHub: microsoft/promptwizard](https://github.com/microsoft/promptwizard) |
| **Reflexion** | 失败记忆 + 自我反思 | 反思策略设计 | [arXiv:2303.11366](https://arxiv.org/abs/2303.11366) |
| **GEPA** | Pareto 前沿 | 多样性保持策略 | [arXiv:2402.00399](https://arxiv.org/abs/2402.00399) |

---

## 3. 业界调研与设计决策

> **Step 2 产出** — 2025-12-15
> 
> 本章节记录业界框架调研结果、假设验证结论、关键设计决策。

### 3.1 业界框架调研结果

| 框架 | 核心抽象 | 优化机制 | 模块化设计 | 借鉴点 |
|------|----------|----------|------------|--------|
| **DSPy MIPROv2** | `Module` 基类 + `forward()` | `Teleprompter.compile()` | ✅ 每个 Module 可独立替换 | Optimizer 接口设计 |
| **TextGrad** | `Variable` (值+梯度) | `TextualGradientDescent.step()` | ✅ 梯度函数可自定义 | 反馈聚合机制 |
| **PromptWizard** | `DatasetSpecificProcessing` | Mutation + Refinement | ✅ 配置驱动 | 任务特定处理抽象 |
| **Reflexion** | Episodic Memory | Verbal Reinforcement | ✅ 反馈类型可扩展 | 反思策略设计 |
| **GEPA** | Pareto 前沿 | 遗传算法 + 反思 | ✅ 多目标优化 | 多样性保持策略 |

### 3.2 假设验证结论

| 假设 | 验证状态 | 结论 | 后续行动 |
|------|----------|------|----------|
| **H1: 规律驱动** | ✅ 保留 | 业界无直接对应，是我们的**创新点**；通过 RuleEngine Trait 模块化降低风险 | 作为核心机制，支持可替换 |
| **H2: 四层处理器** | ✅ 验证 | 与 DSPy Module 组合模式一致，抽象粒度合理 | 定义 Processor Trait |
| **H3: 反思分类** | ✅ 验证 | Reflexion + TextGrad 验证了 verbal feedback，我们的分类是细化 | 保留反思分类机制 |
| **H4: 冲突解决** | ⚠️ 增强 | 单纯抽象层级不够，需增加 Pareto 前沿作为备选 | 三层策略设计 |

### 3.3 关键设计决策

#### 决策 D1: 规律驱动定位

**决策**：规律驱动作为**核心机制**保留，通过 `RuleEngine Trait` 实现模块化。

**理由**：
- 符合产品愿景（可解释性、人机协作）
- 独特价值（显式知识表示）
- 问题归因能力强
- 通过 Trait 降低风险，支持未来算法升级

#### 决策 D2: 三层冲突解决策略

**决策**：采用三层策略处理规律冲突。

| 层级 | 策略 | 适用场景 |
|------|------|----------|
| **主策略** | 抽象层级提升 | 大多数可调和的冲突 |
| **备选策略** | Pareto 前沿 | 难以调和时保留多候选 |
| **兜底策略** | 人工介入 | 极端情况 |

**配置项**：`conflict_resolution_strategy: "abstract" | "pareto" | "hybrid"`

#### 决策 D3: Rust Trait 风格

**决策**：使用 Rust Trait 定义核心接口，自动生成 TypeScript 类型。

**架构**：
```
┌─────────────────────────────────────────────────────────────┐
│                     TypeScript (前端)                        │
│  - UI 组件 + 状态管理                                        │
│  - 类型定义 (从 Rust 自动生成，使用 ts-rs 或 specta)         │
├─────────────────────────────────────────────────────────────┤
│                     Tauri Bridge                             │
├─────────────────────────────────────────────────────────────┤
│                     Rust (后端)                              │
│  - trait RuleEngine { ... }                                 │
│  - trait Processor { ... }                                  │
│  - trait Evaluator { ... }                                  │
│  - trait Optimizer { ... }                                  │
└─────────────────────────────────────────────────────────────┘
```

#### 决策 D4: 双任务模式统一

**决策**：使用 `TaskReference` 枚举统一固定任务和创意任务。

```rust
enum TaskReference {
    Exact { expected: String },                    // 固定任务
    Constrained { constraints: Vec<Constraint>, quality_dimensions: Vec<QualityDimension> },  // 创意任务
    Hybrid { exact_parts: HashMap<String, String>, constraints: Vec<Constraint> },  // 混合任务
}
```

### 3.4 模块化架构保障

为确保**高度模块化**和**可重构性**，设计遵循以下原则：

| 保障措施 | 说明 |
|----------|------|
| **Trait 体系** | 每个核心模块定义清晰的 Trait 接口 |
| **配置驱动** | 用户可通过配置选择不同策略组合 |
| **依赖倒置** | 高层模块依赖抽象，不依赖具体实现 |
| **编译时检查** | Rust Trait + 类型系统保证接口契约 |
| **独立测试** | 每个模块可独立测试 |

**模块化架构总览**：

```
┌─────────────────────────────────────────────────────────────┐
│              Strategy Orchestrator (编排层)                  │
│  - 读取配置，组装模块                                        │
│  - 不包含具体算法逻辑                                        │
├──────────────┬──────────────┬──────────────┬────────────────┤
│  RuleEngine  │   Optimizer  │  Evaluator   │  Aggregator    │
│    Trait     │     Trait    │    Trait     │     Trait      │
│  ┌────────┐  │  ┌────────┐  │  ┌────────┐  │  ┌──────────┐  │
│  │Default │  │  │Default │  │  │Semantic│  │  │TextGrad  │  │
│  │RuleEng │  │  │Optim   │  │  │  F1    │  │  │  Style   │  │
│  └────────┘  │  └────────┘  │  └────────┘  │  └──────────┘  │
│  ┌────────┐  │  ┌────────┐  │  ┌────────┐  │  ┌──────────┐  │
│  │Future  │  │  │Genetic │  │  │Exact   │  │  │Voting    │  │
│  │V2 Eng  │  │  │Style   │  │  │ Match  │  │  │  Style   │  │
│  └────────┘  │  └────────┘  │  └────────┘  │  └──────────┘  │
└──────────────┴──────────────┴──────────────┴────────────────┘
```

**重构保障**：
- 重构具体算法不影响编排逻辑
- 新增算法只需实现对应 Trait
- 模块间低耦合，修改影响范围可控

---

## 4. 核心算法架构设计

> **Step 3 产出** — 2025-12-15
> 
> 本章节定义四层分层架构、5 个核心 Trait 体系、策略编排层设计、扩展点规格。

### 4.1 四层分层架构

```
┌─────────────────────────────────────────────────────────────┐
│                    应用层 (Application)                      │
│  - CLI / Tauri API / UI 接口                                │
├─────────────────────────────────────────────────────────────┤
│                    编排层 (Orchestration)                    │
│  - StrategyOrchestrator / TaskManager / ConfigManager       │
├─────────────────────────────────────────────────────────────┤
│                    核心层 (Core)                             │
│  - RuleEngine / Processor / Evaluator / Optimizer / Aggregator │
├─────────────────────────────────────────────────────────────┤
│                    基础层 (Infrastructure)                   │
│  - TeacherModel / StateManager / EventBus                   │
└─────────────────────────────────────────────────────────────┘
```

| 层级 | 职责 | 变更影响范围 |
|------|------|--------------|
| **应用层** | 用户交互、API 暴露 | 仅影响交互方式 |
| **编排层** | 模块组装、流程控制 | 仅影响流程逻辑 |
| **核心层** | 算法实现、业务逻辑 | 仅影响具体算法 |
| **基础层** | 基础设施、外部依赖 | 仅影响底层实现 |

### 4.2 核心 Trait 体系

| Trait | 职责 | 关键方法 |
|-------|------|----------|
| **RuleEngine** | 规律提取、冲突检测、冲突解决 | `extract_rules()`, `detect_conflicts()`, `resolve_conflict()` |
| **Processor** | 四层处理器统一抽象 | `process()`, `processor_type()` |
| **Evaluator** | 固定/创意任务评估 | `evaluate()`, `evaluate_batch()` |
| **Optimizer** | 迭代优化策略 | `optimize_step()`, `should_terminate()` |
| **FeedbackAggregator** | 反馈聚合、冲突仲裁 | `aggregate()`, `arbitrate()` |

#### 4.2.1 EvaluationResult 结构定义

> **增量补丁** — 2025-12-15
> 
> 为确保评估信号的结构化与可扩展性，定义 Evaluator Trait 的返回类型。
> 设计原则：**最小必要 + 预留扩展**，使用开放结构（HashMap）支持后期维度增删。

```rust
/// 评估结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    // === 核心判定（必须）===
    /// 是否通过评估
    pub passed: bool,
    /// 综合评分 0.0-1.0
    pub score: f64,
    
    // === 多维评估（开放结构，后期增删维度零成本）===
    /// 各维度评分，key 为维度名称
    pub dimensions: HashMap<String, DimensionScore>,
    
    // === 失败诊断（供 Reflection Agent 使用）===
    /// 失败点列表，仅在 passed=false 时填充
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub failure_points: Vec<FailurePoint>,
    
    // === 元数据（预留扩展）===
    /// 产生此结果的评估器类型
    pub evaluator_type: String,
    /// 评估置信度（LLM 评估时尤为重要）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// 评估推理过程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// 任意扩展字段
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 单维度评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    /// 该维度评分 0.0-1.0
    pub score: f64,
    /// 该维度是否通过
    pub passed: bool,
    /// 该维度权重（用于计算综合分数）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// 细节说明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// 失败点（供 Reflection Agent 分析）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePoint {
    /// 失败的维度名称
    pub dimension: String,
    /// 失败描述
    pub description: String,
    /// 严重程度
    pub severity: Severity,
    /// 期望值（固定任务时填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// 实际值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
}

/// 失败严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    /// 致命错误，必须修复
    Critical,
    /// 主要问题，应该修复
    Major,
    /// 次要问题，可选修复
    Minor,
}
```

**常见评估维度参考**（不同 Evaluator 可选择性使用）：

| 维度名称 | 适用任务类型 | 说明 |
|---------|-------------|------|
| `format_compliance` | 固定/创意 | 输出格式是否符合要求 |
| `structure_match` | 固定 | 结构是否匹配（JSON/XML 等） |
| `field_completeness` | 固定 | 必要字段是否齐全 |
| `type_correctness` | 固定 | 字段类型是否正确 |
| `information_coverage` | 固定/创意 | 关键信息是否覆盖 |
| `factual_consistency` | 固定/创意 | 事实是否一致 |
| `constraint_satisfaction` | 创意 | 约束条件是否满足 |
| `tone_match` | 创意 | 语气/风格是否匹配 |
| `length_compliance` | 创意 | 长度是否符合要求 |

#### 4.2.2 OptimizationResult 结构定义

> **增量补丁** — 2025-12-15
> 
> Optimizer Trait 的输出类型。设计支持单候选（MVP）和候选池（未来）两种模式。

```rust
/// 优化结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    // === 主输出 ===
    /// 主候选 Prompt（MVP 只用这个）
    pub primary: PromptCandidate,
    
    // === 候选池（预留给 Racing/Pareto 策略）===
    /// 备选候选列表
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub alternatives: Vec<PromptCandidate>,
    
    // === 终止信号 ===
    /// 是否建议终止迭代
    pub should_terminate: bool,
    /// 终止原因（should_terminate=true 时填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_reason: Option<TerminationReason>,
    
    // === 迭代元数据 ===
    /// 当前迭代轮次
    pub iteration: u32,
    /// 本轮改进摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub improvement_summary: Option<String>,
    
    // === 预留扩展 ===
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Prompt 候选
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCandidate {
    /// 候选 ID
    pub id: String,
    /// Prompt 内容
    pub content: String,
    /// 综合评分 0.0-1.0
    pub score: f64,
    /// 来源（首次生成 / 规律更新 / 表达优化）
    pub source: CandidateSource,
    /// 失败指纹（用于去重）
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub failure_fingerprints: Vec<String>,
}

/// 候选来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CandidateSource {
    /// 首次从规律生成
    InitialGeneration,
    /// 规律体系更新后重新生成
    RuleSystemUpdate,
    /// 仅表达层优化
    ExpressionRefinement,
    /// 多样性注入
    DiversityInjection,
    /// 用户手动编辑
    ManualEdit,
}

/// 终止原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminationReason {
    /// 全部测试通过
    AllTestsPassed,
    /// 达到通过率阈值
    PassThresholdReached { threshold: f64, actual: f64 },
    /// 达到最大迭代轮数
    MaxIterationsReached { max: u32 },
    /// 检测到震荡
    OscillationDetected,
    /// 用户手动终止
    UserStopped,
    /// 需要人工介入
    HumanInterventionRequired { reason: String },
}
```

#### 4.2.3 ReflectionResult 结构定义

> **增量补丁** — 2025-12-15
> 
> ReflectionAgent（Processor 的一种实现）的输出类型，也是 FeedbackAggregator 的输入。

```rust
/// 反思结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    // === 失败分类 ===
    /// 失败类型
    pub failure_type: FailureType,
    
    // === 分析内容 ===
    /// 详细分析
    pub analysis: String,
    /// 根因判断
    pub root_cause: String,
    
    // === 改进建议 ===
    /// 建议列表
    pub suggestions: Vec<Suggestion>,
    
    // === 关联信息 ===
    /// 关联的失败测试用例 ID
    pub failed_test_case_ids: Vec<String>,
    /// 关联的规律 ID（如果是规律问题）
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related_rule_ids: Vec<String>,
    /// 关联的 EvaluationResult（用于追溯）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation_ref: Option<String>,
    
    // === 预留扩展 ===
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 失败类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    /// 规律不完备（缺少某种模式的规律）
    RuleIncomplete,
    /// 规律错误（现有规律有问题）
    RuleIncorrect,
    /// 表达问题（规律正确但 Prompt 表达不当）
    ExpressionIssue,
    /// 边界情况（测试用例是特殊边界）
    EdgeCase,
    /// 无法判断（需要人工介入）
    Undetermined,
}

/// 改进建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// 建议类型
    pub suggestion_type: SuggestionType,
    /// 建议内容
    pub content: String,
    /// 置信度 0.0-1.0
    pub confidence: f64,
    /// 预期影响范围（受影响的测试用例数）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_impact: Option<u32>,
}

/// 建议类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    /// 新增规律
    AddRule,
    /// 修改规律
    ModifyRule,
    /// 删除规律
    RemoveRule,
    /// 修改 Prompt 格式
    ChangeFormat,
    /// 修改 Prompt 措辞
    Rephrase,
    /// 增加示例
    AddExample,
    /// 增加约束说明
    AddConstraint,
}
```

#### 4.2.4 UnifiedReflection 结构定义

> **增量补丁** — 2025-12-15
> 
> FeedbackAggregator 的输出类型，聚合多个 ReflectionResult 后的统一反馈，作为 Optimizer 的输入。

```rust
/// 统一反思结构（聚合后）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedReflection {
    // === 聚合结果 ===
    /// 主要失败类型（投票或权重决定）
    pub primary_failure_type: FailureType,
    /// 聚合后的改进建议（已去重、合并、排序）
    pub unified_suggestions: Vec<UnifiedSuggestion>,
    
    // === 冲突处理 ===
    /// 是否存在建议冲突
    pub has_conflicts: bool,
    /// 冲突详情（如有）
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub conflicts: Vec<SuggestionConflict>,
    /// 仲裁结果（如有冲突）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arbitration_result: Option<ArbitrationResult>,
    
    // === 统计信息 ===
    /// 聚合的原始 ReflectionResult 数量
    pub source_count: u32,
    /// 失败类型分布
    pub failure_type_distribution: HashMap<String, u32>,
    
    // === 行动指令 ===
    /// 推荐的下一步行动
    pub recommended_action: RecommendedAction,
    
    // === 预留扩展 ===
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 统一建议（聚合后）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSuggestion {
    /// 建议类型
    pub suggestion_type: SuggestionType,
    /// 聚合后的建议内容
    pub content: String,
    /// 聚合置信度
    pub confidence: f64,
    /// 支持此建议的原始 ReflectionResult 数量
    pub support_count: u32,
    /// 优先级（1 最高）
    pub priority: u32,
}

/// 建议冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionConflict {
    /// 冲突的建议 A
    pub suggestion_a: Suggestion,
    /// 冲突的建议 B
    pub suggestion_b: Suggestion,
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 冲突描述
    pub description: String,
}

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// 直接矛盾（A 说加，B 说删）
    DirectContradiction,
    /// 资源竞争（都要修改同一规律）
    ResourceCompetition,
    /// 优先级冲突（都重要但只能选一个）
    PriorityConflict,
}

/// 仲裁结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationResult {
    /// 选择的建议
    pub chosen_suggestions: Vec<UnifiedSuggestion>,
    /// 仲裁推理
    pub reasoning: String,
    /// 仲裁方式
    pub method: ArbitrationMethod,
}

/// 仲裁方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArbitrationMethod {
    /// 投票（多数决）
    Voting,
    /// LLM 仲裁
    LLMArbitration,
    /// 人工仲裁
    HumanArbitration,
    /// 全部保留（Pareto）
    KeepAll,
}

/// 推荐行动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    /// 更新规律体系后重新生成 Prompt
    UpdateRulesAndRegenerate,
    /// 仅优化 Prompt 表达
    RefineExpression,
    /// 需要人工介入
    RequestHumanIntervention { reason: String },
    /// 注入多样性
    InjectDiversity,
    /// 终止迭代
    Terminate { reason: TerminationReason },
}
```

### 4.3 关键架构决策

#### 决策 A: 运行时模块注册（动态）

**决策**：采用运行时注册机制，模块在启动时注册到 Registry。

**优势**：
- 运行时切换模块，无需重新编译
- 支持 A/B 测试，对比不同策略
- 配置驱动，用户可自定义策略组合

```rust
pub struct ModuleRegistry {
    rule_engines: HashMap<String, Arc<dyn RuleEngine>>,
    evaluators: HashMap<String, Arc<dyn Evaluator>>,
    optimizers: HashMap<String, Arc<dyn Optimizer>>,
    aggregators: HashMap<String, Arc<dyn FeedbackAggregator>>,
}
```

#### 决策 B: 分层 TeacherModel 配置

**决策**：为不同模块提供不同的模型配置，支持成本优化。

```yaml
teacher_models:
  rule_extraction: "gpt-4o"      # 高能力模型
  conflict_detection: "gpt-4o"   # 高能力模型
  evaluation: "gpt-4o-mini"      # 轻量模型
  reflection: "gpt-4o"           # 高能力模型
```

#### 决策 C: 混合状态持久化策略

**决策**：采用混合策略，关键 Checkpoint 自动保存，用户可配置保存频率。

| 保存点 | 触发条件 | 配置项 |
|--------|----------|--------|
| Phase 完成 | 自动 | 不可配置 |
| 迭代轮次 | 每 N 轮 | `checkpoint_interval: 5` |
| 用户暂停 | 手动 | 不可配置 |
| 人工介入 | 自动 | 不可配置 |

#### 决策 D: 分支治理策略

> **增量补丁** — 2025-12-15
> 
> 为支持人工介入后的版本追溯与元优化数据统计，定义分支治理机制。
> 核心原则：**人工介入 = 新分支起点**，确保自动优化路径与人工修改路径可区分。

**决策**：采用完整分支治理，每个 Checkpoint 携带 lineage 信息。

```rust
/// Checkpoint 结构（扩展 lineage 字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// 唯一标识
    pub id: String,
    /// 所属优化任务 ID
    pub task_id: String,
    /// 迭代轮次
    pub iteration: u32,
    /// 当前状态
    pub state: IterationState,
    /// 当前 Prompt
    pub prompt: String,
    /// 当前规律体系
    pub rule_system: RuleSystem,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    // === 分支治理字段 ===
    /// 分支 ID（首次创建时生成，人工介入时生成新分支）
    pub branch_id: String,
    /// 父 Checkpoint ID（首个 Checkpoint 为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// 分支类型
    pub lineage_type: LineageType,
    /// 分支描述（人工介入时记录介入原因）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_description: Option<String>,
}

/// 分支类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineageType {
    /// 自动优化产生
    Automatic,
    /// 用户手动编辑 Prompt
    ManualPromptEdit,
    /// 用户手动编辑规律
    ManualRuleEdit,
    /// 用户通过对话引导修改
    DialogueGuided,
    /// 从历史 Checkpoint 恢复
    Restored,
}
```

**分支治理规则**：

| 操作 | branch_id | parent_id | lineage_type |
|------|-----------|-----------|--------------|
| 首次创建任务 | 新生成 | None | Automatic |
| 自动迭代 | 继承 | 上一轮 Checkpoint ID | Automatic |
| 用户编辑 Prompt | **新生成** | 当前 Checkpoint ID | ManualPromptEdit |
| 用户编辑规律 | **新生成** | 当前 Checkpoint ID | ManualRuleEdit |
| 对话引导修改 | **新生成** | 当前 Checkpoint ID | DialogueGuided |
| 从历史恢复 | **新生成** | 被恢复的 Checkpoint ID | Restored |

**分支治理的价值**：

1. **可追溯性**：任何 Prompt 都可以回溯其完整演化路径
2. **归因分析**：区分"自动优化成功"和"人工介入成功"
3. **元优化数据**：只统计 `Automatic` 分支的成功率，避免人工介入污染
4. **A/B 对比**：同一起点的不同分支可以对比效果

### 4.4 扩展点设计

| 扩展类型 | 扩展成本 | 扩展方式 |
|----------|----------|----------|
| **新增评估器** | < 2 小时 | 实现 Evaluator Trait，注册到 Registry |
| **新增处理器** | < 4 小时 | 实现 Processor Trait，注册到 Registry |
| **新增优化策略** | < 4 小时 | 实现 Optimizer Trait，注册到 Registry |
| **新增规律引擎** | < 8 小时 | 实现 RuleEngine Trait（较复杂） |

### 4.5 配置驱动示例

```yaml
optimization:
  rule_engine: "default"
  evaluator: "semantic_f1"
  optimizer: "iterative"
  feedback_aggregator: "textgrad"
  conflict_resolution: "hybrid"
  output_mode: "single"
  max_iterations: 20
  pass_threshold: 0.95
```

---

## 5. 算法总体架构

### 5.1 三阶段流程

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Phase 0: 规律收敛阶段                                                  │
│  输入：测试集（输入变量 + 标准输出/约束）                                │
│  输出：规律体系 RuleSystem                                              │
├─────────────────────────────────────────────────────────────────────────┤
│  Phase 1: 首次 Prompt 生成                                              │
│  输入：规律体系 + 核心目标 + 用户配置                                   │
│  输出：Prompt v1                                                        │
├─────────────────────────────────────────────────────────────────────────┤
│  Phase 2: 测试与反思迭代                                                │
│  输入：当前 Prompt + 测试集 + 规律体系                                  │
│  输出：最终 Prompt（或多版本 Prompt）                                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 四层处理器

| 层级 | 名称 | 职责 |
|------|------|------|
| Layer 1 | Pattern Extractor | 从测试集提炼规律 |
| Layer 2 | Prompt Engineer | 基于规律生成 Prompt |
| Layer 3 | Quality Assessor | 评估 Prompt 输出质量 |
| Layer 4 | Reflection Agent | 分析失败原因，推荐策略 |

---

## 6. Phase 0: 规律收敛阶段

### 6.1 流程定义

```
Step 0.1: 测试用例聚类（可选）
    ↓
Step 0.2: 逐类/逐用例规律提炼
    ↓
Step 0.3: 规律汇总与检测（冲突/相似/覆盖）
    ↓
Step 0.4: 规律冲突解决
    ↓
Step 0.5: 规律相似合并
    ↓
Step 0.6: 规律体系验证
    ↓
输出: RuleSystem
```

### 6.2 数据结构定义

#### 6.2.1 Rule 结构

```typescript
interface Rule {
  id: string;                    // UUID
  description: string;           // 规律描述（自然语言）
  tags: RuleTags;               // 结构化标签
  sourceTestCases: string[];    // 来源测试用例 ID
  abstractionLevel: number;     // 抽象层级 (0=原始, 1=冲突解决后, 2=二次抽象)
  parentRules: string[];        // 父规律 ID（合并/抽象产生时）
  verified: boolean;            // 是否已验证
  verificationScore: number;    // 验证分数 0-1
}

interface RuleTags {
  // 输出维度
  outputFormat: string[];       // ["markdown", "json", "xml", "plain_text"]
  outputStructure: string[];    // ["list", "paragraph", "table", "key-value"]
  outputLength: "short" | "medium" | "long" | "flexible";
  
  // 语义维度
  semanticFocus: string[];      // ["extraction", "transformation", "generation", "summarization"]
  keyConcepts: string[];        // 关键概念词
  
  // 约束维度
  mustInclude: string[];        // 必须包含的元素
  mustExclude: string[];        // 必须排除的元素
  tone: string;                 // "formal" | "casual" | "technical"
}
```

#### 6.2.2 RuleSystem 结构

```typescript
interface RuleSystem {
  rules: Rule[];
  conflictResolutionLog: ConflictResolution[];
  mergeLog: RuleMerge[];
  coverageMap: Map<string, string[]>;  // testCaseId -> ruleIds
  version: number;
}

interface ConflictResolution {
  id: string;
  conflictingRuleIds: string[];
  resolvedRuleId: string;
  resolution: string;           // 解决方案描述
  timestamp: Date;
}

interface RuleMerge {
  id: string;
  sourceRuleIds: string[];
  mergedRuleId: string;
  reason: string;
  timestamp: Date;
}
```

### 6.3 算法实现

#### 6.3.1 规律提炼算法

```python
def extract_rules_from_test_cases(test_cases: List[TestCase], config: Config) -> List[Rule]:
    """
    从测试用例中提炼规律
    """
    # Step 1: 可选聚类
    if config.enable_clustering and len(test_cases) > config.clustering_threshold:
        clusters = cluster_test_cases(test_cases)
    else:
        clusters = [[tc] for tc in test_cases]
    
    # Step 2: 逐类提炼
    rules = []
    for cluster in clusters:
        rule = extract_rule_from_cluster(cluster)
        rule.verified = verify_rule(rule, cluster)
        rules.append(rule)
    
    return rules

def extract_rule_from_cluster(cluster: List[TestCase]) -> Rule:
    """
    使用老师模型从测试用例聚类中提炼规律
    """
    prompt = RULE_EXTRACTION_PROMPT.format(
        test_cases=format_test_cases(cluster)
    )
    response = teacher_model.generate(prompt)
    return parse_rule_response(response)
```

#### 6.3.2 冲突检测算法

```python
def detect_conflicts(rules: List[Rule]) -> List[Tuple[Rule, Rule]]:
    """
    检测规律之间的冲突
    使用老师模型进行检测
    """
    conflicts = []
    for i, rule1 in enumerate(rules):
        for rule2 in rules[i+1:]:
            if is_conflicting(rule1, rule2):
                conflicts.append((rule1, rule2))
    return conflicts

def is_conflicting(rule1: Rule, rule2: Rule) -> bool:
    """
    使用老师模型判断两条规律是否冲突
    """
    prompt = CONFLICT_DETECTION_PROMPT.format(
        rule1=rule1.description,
        rule2=rule2.description
    )
    response = teacher_model.generate(prompt)
    return parse_conflict_response(response)
```

#### 6.3.3 冲突解决算法

```python
def resolve_conflict(rule1: Rule, rule2: Rule, test_cases: List[TestCase], config: Config) -> Rule:
    """
    解决两条冲突规律，提炼更高层的统一规律
    """
    # 获取相关测试用例
    related_cases = get_related_test_cases(rule1, rule2, test_cases)
    
    # 尝试提炼更高层规律
    prompt = CONFLICT_RESOLUTION_PROMPT.format(
        rule1=rule1.description,
        rule2=rule2.description,
        test_cases=format_test_cases(related_cases)
    )
    response = teacher_model.generate(prompt)
    unified_rule = parse_rule_response(response)
    
    # 设置抽象层级
    unified_rule.abstraction_level = max(rule1.abstraction_level, rule2.abstraction_level) + 1
    unified_rule.parent_rules = [rule1.id, rule2.id]
    
    # 检查是否超过最大抽象层级
    if unified_rule.abstraction_level > config.max_abstraction_level:
        raise HumanInterventionRequired("规律冲突无法自动解决，需要用户介入")
    
    # 验证新规律
    unified_rule.verified = verify_rule(unified_rule, related_cases)
    
    return unified_rule
```

#### 6.3.4 相似合并算法

```python
def detect_and_merge_similar(rules: List[Rule], config: Config) -> List[Rule]:
    """
    检测并合并相似规律
    """
    similar_groups = find_similar_groups(rules, config.similarity_threshold)
    
    merged_rules = []
    merged_ids = set()
    
    for group in similar_groups:
        if len(group) > 1:
            merged = merge_similar_rules(group)
            merged_rules.append(merged)
            merged_ids.update(r.id for r in group)
    
    # 保留未合并的规律
    for rule in rules:
        if rule.id not in merged_ids:
            merged_rules.append(rule)
    
    return merged_rules
```

---

## 7. Phase 1: 首次 Prompt 生成

### 7.1 流程定义

```
输入: RuleSystem + 核心目标 + 用户配置
    ↓
Step 1.1: 根据输出策略选择规律组合
    ↓
Step 1.2: 生成 Prompt（可能多个变体）
    ↓
输出: Prompt v1（或多版本）
```

### 7.2 输出策略处理

```python
def generate_initial_prompt(rule_system: RuleSystem, goal: str, config: Config) -> Union[str, List[str]]:
    """
    根据输出策略生成初始 Prompt
    """
    if config.output_strategy == "single":
        # 策略A: 强制收敛，输出单一 Prompt
        return generate_single_prompt(rule_system.rules, goal)
    
    elif config.output_strategy == "adaptive":
        # 策略B: 自适应 Prompt
        type_rules = group_rules_by_type(rule_system.rules)
        return generate_adaptive_prompt(type_rules, goal)
    
    elif config.output_strategy == "multi":
        # 策略C: 多版本输出
        type_rules = group_rules_by_type(rule_system.rules)
        prompts = {}
        for type_name, rules in type_rules.items():
            prompts[type_name] = generate_single_prompt(rules, goal)
        prompts["general"] = generate_single_prompt(rule_system.rules, goal)
        return prompts
```

---

## 8. Phase 2: 测试与反思迭代

### 8.1 流程定义

```
Step 2.1: 执行测试（串行/并行）
    ↓
全部通过? → YES → 成功结束
    ↓ NO
Step 2.2: 失败聚类
    ↓
Step 2.3: 反思仲裁（梯度聚合）
    ↓
Step 2.4: 规律/Prompt 更新
    ↓
Step 2.5: 智能重测
    ↓
Step 2.6: 安全检查（回归/震荡）
    ↓
达到最大轮数? → YES → 输出最佳结果
    ↓ NO
回到 Step 2.1
```

### 8.2 并行测试实现

```python
def parallel_test_iteration(prompt: str, test_cases: List[TestCase], rule_system: RuleSystem, config: Config) -> IterationResult:
    """
    并行测试迭代
    """
    # Step 2.1: 并行执行
    if config.minibatch_enabled and len(test_cases) > config.minibatch_recommend_threshold:
        # Minibatch 模式
        batch = sample_minibatch(test_cases, config.minibatch_size)
        results = parallel_execute(prompt, batch)
    else:
        # 全量测试
        results = parallel_execute(prompt, test_cases)
    
    # 检查是否全部通过
    failed_cases = [r for r in results if not r.passed]
    if len(failed_cases) == 0:
        return IterationResult(status="success", prompt=prompt)
    
    # Step 2.2: 失败聚类
    reflections = parallel_reflect(failed_cases, prompt, rule_system)
    clusters = cluster_by_root_cause(reflections)
    
    # Step 2.3: 反思仲裁（借鉴 TextGrad 梯度聚合）
    unified_reflection = aggregate_reflections(clusters, config)
    
    # Step 2.4: 应用更新
    if unified_reflection.type == "rule_incomplete":
        # 情况A: 规律问题 → 更新规律体系
        rule_system = update_rule_system(rule_system, unified_reflection)
        new_prompt = generate_prompt_from_rules(rule_system, config)
    else:
        # 情况B: 表达问题 → 只调整 Prompt
        new_prompt = refine_prompt(prompt, unified_reflection)
    
    return IterationResult(
        status="continue",
        prompt=new_prompt,
        rule_system=rule_system,
        failed_cases=failed_cases
    )
```

### 8.3 反思分类实现

```python
def classify_failure(failed_case: FailedTestResult, prompt: str, rule_system: RuleSystem) -> ReflectionResult:
    """
    对失败用例进行分类反思
    """
    prompt_template = REFLECTION_CLASSIFICATION_PROMPT.format(
        test_input=failed_case.input,
        expected_output=failed_case.expected,
        actual_output=failed_case.actual,
        current_prompt=prompt,
        rules=format_rules(rule_system.rules)
    )
    
    response = teacher_model.generate(prompt_template)
    result = parse_reflection_response(response)
    
    # result.type 为 "rule_incomplete" 或 "expression_issue"
    return result
```

### 8.4 梯度聚合实现（借鉴 TextGrad）

```python
def aggregate_reflections(clusters: List[ReflectionCluster], config: Config) -> UnifiedReflection:
    """
    聚合多个反思结果，处理冲突
    """
    # 收集所有改进建议（"梯度"）
    gradients = []
    for cluster in clusters:
        gradients.extend(cluster.suggestions)
    
    # 检测冲突
    conflicts = detect_suggestion_conflicts(gradients)
    
    if len(conflicts) > 0:
        # 有冲突，需要仲裁
        unified = arbitrate_conflicts(conflicts, config)
    else:
        # 无冲突，直接合并
        unified = merge_suggestions(gradients)
    
    return unified

def arbitrate_conflicts(conflicts: List[Conflict], config: Config) -> UnifiedReflection:
    """
    使用老师模型仲裁冲突的建议
    """
    prompt = ARBITRATION_PROMPT.format(
        conflicts=format_conflicts(conflicts)
    )
    response = teacher_model.generate(prompt)
    
    # 如果无法仲裁，触发人工介入
    if response.cannot_resolve:
        raise HumanInterventionRequired("建议冲突无法自动解决")
    
    return parse_arbitration_response(response)
```

### 8.5 安全检查实现

```python
def safety_check(history: IterationHistory, current_result: IterationResult, config: Config) -> SafetyCheckResult:
    """
    安全检查：回归检测 + 震荡检测
    """
    # 回归检测
    if current_result.previous_passed_cases:
        regressions = detect_regressions(
            current_result.results,
            current_result.previous_passed_cases
        )
        if len(regressions) > 0:
            return SafetyCheckResult(
                status="regression",
                regressions=regressions
            )
    
    # 震荡检测
    if is_oscillating(history, config.oscillation_threshold):
        if config.oscillation_action == "diversity_inject":
            return SafetyCheckResult(status="oscillation_inject")
        elif config.oscillation_action == "human_intervention":
            raise HumanInterventionRequired("检测到优化震荡")
        else:
            return SafetyCheckResult(status="stop")
    
    return SafetyCheckResult(status="ok")

def is_oscillating(history: IterationHistory, threshold: int) -> bool:
    """
    检测是否发生震荡（连续 N 轮出现相同/相似状态）
    """
    if len(history.states) < threshold:
        return False
    
    recent_states = history.states[-threshold:]
    
    # 检查是否有重复状态
    for i, state1 in enumerate(recent_states):
        for state2 in recent_states[i+1:]:
            if is_similar_state(state1, state2):
                return True
    
    return False
```

---

## 9. 用户配置规格

### 9.1 输出策略配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `output_strategy` | enum | `"single"` | `"single"` / `"adaptive"` / `"multi"` |
| `conflict_alert_threshold` | int | `3` | 冲突数量达到此值时弹出推荐 |
| `auto_recommend` | bool | `true` | 是否启用智能推荐 |

### 9.2 Minibatch 配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `minibatch_enabled` | bool | `false` | 是否启用 Minibatch |
| `minibatch_size` | int | `10` | 每批测试数量 |
| `full_eval_interval` | int | `5` | 全量验证间隔轮数 |
| `minibatch_recommend_threshold` | int | `20` | 推荐启用阈值 |

### 9.3 震荡检测配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `oscillation_threshold` | int | `3` | 震荡判定轮数 |
| `oscillation_action` | enum | `"diversity_inject"` | `"diversity_inject"` / `"human_intervention"` / `"stop"` |

### 9.4 规律抽象配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_abstraction_level` | int | `3` | 规律抽象最大层级 |

### 9.5 迭代控制配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_iterations` | int | `20` | 最大迭代轮数 |
| `pass_threshold` | float | `0.95` | 通过率阈值 |
| `diversity_inject_after` | int | `3` | 连续失败多少次后触发多样性注入 |

---

## 10. 老师 Prompt 模板规格

### 10.1 规律提炼 Prompt

```markdown
# Role: Pattern Extraction Expert

## Input
- Goal: {goal}
- Test Cases:
{test_cases}

## Task
分析测试用例，提取可泛化的规律。

## Required Steps
[Step 1] 逐用例分析
- 输入特征
- 输出特征
- 转换模式

[Step 2] 跨用例模式识别
- 格式模式
- 语义模式
- 逻辑模式

[Step 3] 抽象为规律
- 必须适用于所有测试用例
- 足够具体以指导 Prompt 生成
- 足够抽象以避免过拟合

## Output Format (JSON)
{
  "analysis_trace": "分析推理过程",
  "rule": {
    "description": "规律描述",
    "tags": {
      "outputFormat": [],
      "outputStructure": [],
      "outputLength": "",
      "semanticFocus": [],
      "keyConcepts": [],
      "mustInclude": [],
      "mustExclude": [],
      "tone": ""
    },
    "validationPoints": ["检验点1", "检验点2"]
  }
}
```

### 10.2 冲突检测 Prompt

```markdown
# Role: Rule Conflict Detector

## Input
- Rule 1: {rule1}
- Rule 2: {rule2}

## Task
判断两条规律是否存在冲突。

## Conflict Types
1. 直接矛盾：一条规律的要求与另一条直接冲突
2. 格式冲突：输出格式要求不兼容
3. 语义冲突：语义焦点或内容要求冲突

## Output Format (JSON)
{
  "is_conflicting": true/false,
  "conflict_type": "直接矛盾" / "格式冲突" / "语义冲突" / null,
  "conflict_description": "冲突描述（如有）",
  "reasoning": "判断推理过程"
}
```

### 10.3 反思分类 Prompt

```markdown
# Role: Failure Analysis Expert

## Input
- Test Input: {test_input}
- Expected Output: {expected_output}
- Actual Output: {actual_output}
- Current Prompt: {current_prompt}
- Current Rules: {rules}

## Task
分析失败原因，判断是规律问题还是表达问题。

## Classification
- 情况A: 规律之外的情况（规律体系不完备）
  现象：失败用例的模式不在任何规律中
  
- 情况B: 规律满足但表达不当（Prompt 表达问题）
  现象：规律已覆盖，但 Prompt 表达导致 LLM 理解错误

## Output Format (JSON)
{
  "failure_type": "rule_incomplete" / "expression_issue",
  "analysis": "详细分析",
  "suggestion": {
    "type": "add_rule" / "modify_rule" / "change_format" / "rephrase",
    "details": "具体建议"
  }
}
```

---

## 11. 状态机定义

### 11.1 状态枚举

```typescript
enum IterationState {
  INIT = "init",                    // 初始化
  RULE_EXTRACT = "rule_extract",    // 规律提炼
  RULE_CONVERGE = "rule_converge",  // 规律收敛
  PROMPT_GEN = "prompt_gen",        // Prompt 生成
  EXECUTING = "executing",          // 执行测试
  EVALUATING = "evaluating",        // 评估结果
  REFLECTING = "reflecting",        // 反思分析
  UPDATING = "updating",            // 更新状态
  PAUSED = "paused",                // 用户暂停
  SUCCESS = "success",              // 成功
  MAX_ITER = "max_iter",            // 达到最大轮数
  USER_STOPPED = "user_stopped",    // 用户终止
  HUMAN_INTERVENTION = "human_intervention"  // 需要人工介入
}
```

### 11.2 状态转换规则

```
INIT → RULE_EXTRACT
RULE_EXTRACT → RULE_CONVERGE
RULE_CONVERGE → PROMPT_GEN
PROMPT_GEN → EXECUTING
EXECUTING → EVALUATING
EVALUATING → SUCCESS (全部通过)
EVALUATING → REFLECTING (有失败)
REFLECTING → UPDATING
UPDATING → EXECUTING (继续)
UPDATING → MAX_ITER (达到最大轮数)
UPDATING → RULE_EXTRACT (需要更新规律)
Any → PAUSED (用户暂停)
PAUSED → Previous State (用户恢复)
Any → USER_STOPPED (用户终止)
Any → HUMAN_INTERVENTION (需要人工介入)
```

---

## 12. 最佳实践来源

| 来源 | 核心机制 | 应用位置 | 参考链接 |
|------|----------|----------|----------|
| DSPy MIPROv2 | Grounded Instruction Proposal | 规律提炼 | [arXiv:2406.11695](https://arxiv.org/abs/2406.11695) |
| DSPy MIPROv2 | Minibatch + Full Eval | 大规模测试 | [GitHub: stanfordnlp/dspy](https://github.com/stanfordnlp/dspy) |
| Reflexion | 失败记忆 | 失败档案 | [arXiv:2303.11366](https://arxiv.org/abs/2303.11366) |
| TextGrad | 文本梯度聚合 | 反思仲裁 | [arXiv:2406.07496](https://arxiv.org/abs/2406.07496) |
| GEPA | Pareto 前沿 | 规律多样性 | [arXiv:2402.00399](https://arxiv.org/abs/2402.00399) |
| PromptWizard | Mutation + Refinement | Prompt 进化 | [arXiv:2405.18369](https://arxiv.org/abs/2405.18369) |
| CAPO | Racing Selection | 早期淘汰 | [arXiv:2311.06562](https://arxiv.org/abs/2311.06562) |
| Arize | 多维度反馈 | 评估诊断 | [Arize Phoenix Docs](https://docs.arize.com/phoenix)

---

## 13. 附录：错误处理

### 13.1 HumanInterventionRequired 异常

触发条件：
- 规律冲突无法自动解决（超过最大抽象层级）
- 建议冲突无法仲裁
- 检测到优化震荡（且配置为人工介入）

处理方式：
- 保存当前状态到 Checkpoint
- 向用户展示问题详情
- 等待用户指导后继续

### 13.2 MaxIterationReached

触发条件：
- 迭代轮数达到 `max_iterations`

处理方式：
- 输出历史最佳 Prompt
- 生成优化报告

---

**文档结束**
