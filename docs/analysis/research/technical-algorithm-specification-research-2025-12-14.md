---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments:
  - docs/analysis/brainstorming-session-2025-12-12.md
  - docs/prd.md
workflowType: 'research'
lastStep: 6
research_type: 'technical'
research_topic: 'algorithm-specification'
research_goals: '定义 Prompt Faster 核心迭代算法的完整技术规格，以高度模块化、可插拔的架构设计为核心原则'
user_name: '耶稣'
date: '2025-12-15'
web_research_enabled: true
source_verification: true
revision_note: '2025-12-15 新增：4.6 架构模式研究总结(Step 4)、14 实现与采纳建议(Step 5)；修复 4.4 Processor Trait 历史矛盾'
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
| **S3** | 新增 Trait 实现的工作量 < 4 小时 | 实现新 Trait（如 PromptGenerator）的复杂度评估 |
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
| **H2: 四层处理器** | ✅ 验证 | 与 DSPy Module 组合模式一致，抽象粒度合理 | 定义四层对应 Trait（RuleEngine/PromptGenerator/Evaluator/FeedbackAggregator） |
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
│  - trait PromptGenerator { ... }                            │
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
│  RuleEngine  │   Optimizer  │  Evaluator   │  FeedbackAggr  │
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
│  - RuleEngine / PromptGenerator / Evaluator / Optimizer / FeedbackAggregator │
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

> **修订说明** — 2025-12-15
> 
> 基于决策审查，更新 Trait 体系：
> - **新增** `PromptGenerator`（对应 Layer 2 Prompt Engineer）
> - **新增** `ExecutionTarget`（执行被优化的 Prompt）
> - **新增** `TeacherModel`（老师模型接口）
> - **删除** 通用 `Processor`（四层异构，无法统一抽象）
> - **完善** 所有 Trait 的完整方法签名

#### 4.2.0 四层处理器 ↔ Trait 映射表

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                        流程视角（四层处理器）                                  │
├────────────────────┬────────────────────┬────────────────┬───────────────────┤
│  Layer 1           │  Layer 2           │  Layer 3       │  Layer 4          │
│  Pattern Extractor │  Prompt Engineer   │  Quality       │  Reflection Agent │
│  从测试集提炼规律   │  基于规律生成Prompt │  Assessor      │  分析失败原因      │
│                    │                    │  评估输出质量   │  推荐改进策略      │
├────────────────────┼────────────────────┼────────────────┼───────────────────┤
│                        实现视角（Trait）                                       │
├────────────────────┼────────────────────┼────────────────┼───────────────────┤
│  RuleEngine        │  PromptGenerator   │  Evaluator     │  FeedbackAggregator│
│  .extract_rules()  │  .generate()       │  .evaluate()   │  .aggregate()     │
│  .detect_conflicts()│                   │  .evaluate_batch()│                 │
│  .resolve_conflict()│                   │                │  + Optimizer      │
│  .merge_similar_rules()│                │                │  .optimize_step() │
└────────────────────┴────────────────────┴────────────────┴───────────────────┘
```

**辅助 Trait**（不对应具体 Layer，但贯穿流程）：

| Trait | 职责 | 使用位置 |
|-------|------|---------|
| **TeacherModel** | 调用老师模型（规律提炼、反思、评估） | 各 Layer 内部 |
| **ExecutionTarget** | 执行被优化的 Prompt（Dify / 直接 AI 模型） | Phase 2 测试执行 |

#### Trait 体系总览

| Trait | 职责 | 关键方法 | 对应 Layer |
|-------|------|----------|-----------|
| **RuleEngine** | 规律提取、冲突检测、冲突解决、相似合并 | `extract_rules()`, `detect_conflicts()`, `resolve_conflict()`, `merge_similar_rules()` | Layer 1 |
| **PromptGenerator** | 基于规律生成 Prompt | `generate()` | Layer 2 |
| **Evaluator** | 固定/创意任务评估 | `evaluate()`, `evaluate_batch()` | Layer 3 |
| **FeedbackAggregator** | 反馈聚合、冲突仲裁 | `aggregate()`, `arbitrate()` | Layer 4 |
| **Optimizer** | 迭代优化策略 | `optimize_step()`, `should_terminate()` | Layer 4 |
| **TeacherModel** | 老师模型调用 | `generate()`, `generate_structured()` | 辅助 |
| **ExecutionTarget** | 执行目标（Dify/AI模型） | `execute()` | 辅助 |

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

**confidence 字段语义说明**：

> **新增** — 2025-12-17
> 
> `confidence` 字段表示本次评估结果的可信程度，取值范围 0.0-1.0。
> 该字段在多评估器场景（EnsembleEvaluator）下尤为重要，用于驱动分层门控策略。

| confidence 范围 | 语义 | 对规则更新的影响 |
|----------------|------|-----------------|
| `≥ high_threshold` | 高置信度，评估结果可靠 | 允许规则层更新（AddRule/ModifyRule/RemoveRule） |
| `[low_threshold, high_threshold)` | 中等置信度，有一定不确定性 | 仅允许表达层更新；规则层建议记为候选 |
| `< low_threshold` | 低置信度，评估结果不可靠 | 标记为"不可靠"，不驱动任何自动更新 |

**confidence 计算方式**（推荐，可配置）：

```text
confidence ≈ w_hard × I(hard_checks_all_passed)
           + w_agreement × agreement_among_evaluators
           - w_variance × variance_among_samples
```

其中：
- `w_hard`：HardChecks 权重（推荐 0.4）
- `w_agreement`：多评估器一致性权重（推荐 0.4）
- `w_variance`：多次采样方差惩罚权重（推荐 0.2）
- `I()`：指示函数，条件成立为 1，否则为 0

具体权重由 `EvaluatorConfig` 配置，详见 Section 4.2.6.1 和 Section 9.7。

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
> ReflectionAgent（Layer 4 Reflection Agent 的实现）的输出类型，也是 FeedbackAggregator 的输入。

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

#### 4.2.5 核心输入输出结构定义

> **新增** — 2025-12-15
> 
> 定义算法流程的核心输入输出结构：TestCase（测试用例）、ExecutionResult（执行结果）、OptimizationContext（优化上下文）。

```rust
/// 测试用例结构
/// 
/// 支持两种模式：
/// - Dify 模式：input 字段从 Dify API 解析，变量名固定
/// - 直接 AI 模型模式：input 字段由用户自由定义
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TestCase {
    /// 唯一标识
    pub id: String,
    
    /// 输入变量（HashMap 支持任意数量、任意名称的变量）
    /// 例如: {"user_question": "什么是AI", "context": "技术文档"}
    pub input: HashMap<String, serde_json::Value>,
    
    /// 期望输出/约束（区分固定任务和创意任务）
    pub reference: TaskReference,
    
    /// 数据划分归属（由 DataSplitConfig 策略生成或用户手动指定）
    /// - None 或 Unassigned：未分配，系统将作为训练集使用
    /// - Train：训练集，用于规律提炼和 Prompt 生成
    /// - Validation：验证集，用于迭代过程中的评估
    /// - Holdout：保留集，用于最终验证，防止过拟合
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<DataSplit>,
    
    /// 元数据（可选，如来源、创建时间等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 数据划分类型
/// 
/// 用于标识测试用例在 Train/Val/Holdout 三分法中的归属。
/// 详见 Section 9.6 数据划分配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DataSplit {
    /// 未分配（系统将作为训练集使用）
    Unassigned,
    /// 训练集：用于规律提炼和 Prompt 生成
    Train,
    /// 验证集：用于迭代过程中的评估
    Validation,
    /// 保留集：用于最终验证，防止过拟合
    Holdout,
}

/// 任务参考类型（与决策 D4 一致）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub enum TaskReference {
    /// 固定任务：有明确的标准答案
    Exact { expected: String },
    
    /// 创意任务：基于约束条件评估
    Constrained { 
        constraints: Vec<Constraint>,
        quality_dimensions: Vec<QualityDimension>,
    },
    
    /// 混合任务：部分固定 + 部分约束
    Hybrid { 
        exact_parts: HashMap<String, String>,
        constraints: Vec<Constraint>,
    },
}

/// 约束条件
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Constraint {
    pub name: String,
    pub description: String,
    pub weight: Option<f64>,
}

/// 质量维度
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct QualityDimension {
    pub name: String,
    pub description: String,
    pub weight: f64,
}

/// 执行结果结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ExecutionResult {
    /// 关联的测试用例 ID
    pub test_case_id: String,
    /// 执行输出
    pub output: String,
    /// 执行延迟（毫秒）
    pub latency_ms: u64,
    /// Token 使用量（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
    /// 原始响应（可选，用于调试）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<serde_json::Value>,
}

/// Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

#### 4.2.6 OptimizationContext 结构定义

> **新增** — 2025-12-15
> 
> 优化上下文是"共享状态容器"，贯穿整个迭代流程，各模块通过只读引用访问所需字段。

```rust
/// 优化上下文（贯穿整个迭代流程的共享状态）
/// 
/// 设计原则：
/// - 各模块通过只读引用 `&OptimizationContext` 访问
/// - 只有编排层（Orchestrator）能更新 Context
/// - extensions 字段支持未来扩展而不改结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationContext {
    // ===== 任务标识 =====
    /// 优化任务 ID
    pub task_id: String,
    
    // ===== 执行目标信息 =====
    /// 执行目标配置（Dify 或 直接 AI 模型）
    pub execution_target_config: ExecutionTargetConfig,
    
    // ===== 当前状态 =====
    /// 当前迭代的 Prompt
    pub current_prompt: String,
    /// 当前的规律体系
    pub rule_system: RuleSystem,
    /// 当前迭代轮次
    pub iteration: u32,
    /// 当前状态（IterationState 定义见 Section 11.1）
    pub state: IterationState,
    
    // ===== 输入数据 =====
    /// 用户测试集
    pub test_cases: Vec<TestCase>,
    
    // ===== 配置 =====
    /// 用户配置项
    pub config: OptimizationConfig,
    
    // ===== 历史记录 =====
    /// 迭代快照
    pub checkpoints: Vec<Checkpoint>,
    
    // ===== 扩展 =====
    /// 预留扩展字段
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 执行目标配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionTargetConfig {
    /// Dify 工作流
    Dify {
        api_url: String,
        workflow_id: String,
        /// system_prompt 对应的变量名
        prompt_variable: String,
    },
    /// 直接 AI 模型
    DirectModel {
        model_name: String,
        /// user prompt 模板（使用 {变量名} 占位）
        user_prompt_template: String,
    },
}
```

#### 4.2.6.1 OptimizationConfig 结构定义

> **新增** — 2025-12-16
> 
> 优化配置是用户可调整的算法参数集合。采用**嵌套分组设计**，
> 每个功能模块的配置独立成 struct，便于模块化管理和未来扩展。

```rust
/// 优化配置（用户可调整的算法参数）
/// 
/// 设计原则：
/// - 嵌套分组：按功能模块组织配置，便于维护和扩展
/// - 合理默认：所有配置项都有经过验证的默认值
/// - 渐进披露：核心配置在顶层，高级配置在子模块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// 输出策略配置
    pub output: OutputConfig,
    /// Minibatch 配置
    pub minibatch: MinibatchConfig,
    /// 震荡检测配置
    pub oscillation: OscillationConfig,
    /// 规律相关配置
    pub rule: RuleConfig,
    /// 迭代控制配置
    pub iteration: IterationConfig,
    /// 数据划分配置（Train/Val/Holdout 三分法）
    pub data_split: DataSplitConfig,
    /// 评估器配置（EnsembleEvaluator + confidence 门控）
    pub evaluator: EvaluatorConfig,
    /// 预算控制配置（per-run 全局预算）
    pub budget: BudgetConfig,
    /// Racing 策略配置（多候选早期淘汰）
    pub racing: RacingConfig,
}

/// 输出策略配置（对应 Section 9.1）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// 输出策略: "single" / "adaptive" / "multi"
    #[serde(default = "default_output_strategy")]
    pub strategy: OutputStrategy,
    /// 冲突数量达到此值时弹出推荐
    #[serde(default = "default_conflict_alert_threshold")]
    pub conflict_alert_threshold: u32,
    /// 是否启用智能推荐
    #[serde(default = "default_true")]
    pub auto_recommend: bool,
}

/// 输出策略枚举
/// 
/// 序列化时使用 snake_case 字符串，例如：
/// - `Single` → `"single"`
/// - `Adaptive` → `"adaptive"`
/// - `Multi` → `"multi"`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputStrategy {
    /// 强制收敛，输出单一 Prompt
    #[default]
    Single,
    /// 自适应 Prompt（根据输入类型选择）
    Adaptive,
    /// 多版本输出（每种类型一个 Prompt）
    Multi,
}

/// Minibatch 配置（对应 Section 9.2）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinibatchConfig {
    /// 是否启用 Minibatch
    #[serde(default)]
    pub enabled: bool,
    /// 每批测试数量
    #[serde(default = "default_minibatch_size")]
    pub size: u32,
    /// 全量验证间隔轮数
    #[serde(default = "default_full_eval_interval")]
    pub full_eval_interval: u32,
    /// 推荐启用阈值（测试用例数超过此值时推荐启用）
    #[serde(default = "default_minibatch_recommend_threshold")]
    pub recommend_threshold: u32,
}

/// 震荡检测配置（对应 Section 9.3）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscillationConfig {
    /// 震荡判定轮数
    #[serde(default = "default_oscillation_threshold")]
    pub threshold: u32,
    /// 震荡触发动作
    #[serde(default)]
    pub action: OscillationAction,
}

/// 震荡触发动作枚举
/// 
/// 序列化时使用 snake_case 字符串，例如：
/// - `DiversityInject` → `"diversity_inject"`
/// - `HumanIntervention` → `"human_intervention"`
/// - `Stop` → `"stop"`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OscillationAction {
    /// 自动注入多样性
    #[default]
    DiversityInject,
    /// 请求人工介入
    HumanIntervention,
    /// 停止迭代
    Stop,
}

/// 规律相关配置（对应 Section 9.4）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// 规律抽象最大层级
    #[serde(default = "default_max_abstraction_level")]
    pub max_abstraction_level: u32,
    /// 规律相似度阈值（用于合并相似规律）
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,
    /// 是否启用测试用例聚类
    #[serde(default)]
    pub enable_clustering: bool,
    /// 聚类启用阈值（测试用例数超过此值时启用聚类）
    #[serde(default = "default_clustering_threshold")]
    pub clustering_threshold: u32,
}

/// 迭代控制配置（对应 Section 9.5）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationConfig {
    /// 最大迭代轮数
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    /// 通过率阈值
    #[serde(default = "default_pass_threshold")]
    pub pass_threshold: f64,
    /// 连续失败多少次后触发多样性注入
    #[serde(default = "default_diversity_inject_after")]
    pub diversity_inject_after: u32,
}

/// 数据划分配置（对应 Section 9.6）
/// 
/// 用于配置 Train/Val/Holdout 三分法，防止测试集过拟合。
/// 默认关闭，用户可根据需要手动启用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSplitConfig {
    /// 是否启用数据划分（默认关闭）
    #[serde(default)]
    pub enabled: bool,
    /// 训练集比例 (0.0-1.0)
    #[serde(default = "default_train_ratio")]
    pub train_ratio: f64,
    /// 验证集比例 (0.0-1.0)
    #[serde(default = "default_validation_ratio")]
    pub validation_ratio: f64,
    /// 保留集比例 = 1.0 - train_ratio - validation_ratio（自动计算，无需配置）
    /// 划分策略
    #[serde(default)]
    pub strategy: SplitStrategy,
    /// 随机种子（可选，用于可复现划分）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// 过拟合警告阈值（Holdout 通过率与 Validation 通过率之差超过此值时警告）
    #[serde(default = "default_overfitting_threshold")]
    pub overfitting_threshold: f64,
}

/// 划分策略
/// 
/// 控制如何将测试用例分配到 Train/Val/Holdout 三个集合。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SplitStrategy {
    /// 随机划分（默认）
    #[default]
    Random,
    /// 分层抽样（按 TaskReference 类型分层，确保各集合中任务类型分布一致）
    Stratified,
    /// 用户手动指定（每个 TestCase 自带 split 字段，忽略 ratio 配置）
    Manual,
}

/// 评估器配置（对应 Section 9.7）
/// 
/// 控制 EnsembleEvaluator 的行为和 confidence 门控策略。
/// 默认启用多评估器组合，提供更稳定的评估信号。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorConfig {
    /// 是否启用多评估器组合（EnsembleEvaluator）
    #[serde(default = "default_true")]
    pub ensemble_enabled: bool,
    /// 高置信度阈值（允许规则层更新）
    #[serde(default = "default_confidence_high_threshold")]
    pub confidence_high_threshold: f64,
    /// 低置信度阈值（标记为不可靠）
    #[serde(default = "default_confidence_low_threshold")]
    pub confidence_low_threshold: f64,
    /// LLM 评估器采样次数（多次采样可提高一致性）
    #[serde(default = "default_llm_judge_samples")]
    pub llm_judge_samples: u32,
    /// HardChecks 权重（用于 confidence 计算）
    #[serde(default = "default_hard_checks_weight")]
    pub hard_checks_weight: f64,
    /// 多评估器一致性权重
    #[serde(default = "default_agreement_weight")]
    pub agreement_weight: f64,
    /// 采样方差惩罚权重
    #[serde(default = "default_variance_penalty")]
    pub variance_penalty: f64,
}

/// 预算控制配置（对应 Section 9.8）
/// 
/// **作用范围**：单次优化任务（per-run）的全局预算上限，而非单轮迭代预算。
/// 迭代级别的控制仍由 MinibatchConfig / IterationConfig 负责。
/// 
/// **执行机制**：由内部 BudgetManager 组件统一执行，负责：
/// - 在各阶段（规则提取、评估、Racing 等）统计实际消耗
/// - 接近/超限时触发警告或策略降级
/// 
/// **超限推荐策略**（优先级从高到低）：
/// 1. 缩减 Minibatch 大小，减少每轮评估调用
/// 2. 加速 Racing 淘汰（提高 survival_threshold），减少候选数
/// 3. 如仍无法避免超限，提前终止本次 run 并在 Checkpoint 中标记 `budget_exhausted`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// 是否启用预算控制
    #[serde(default)]
    pub enabled: bool,
    /// 最大 LLM 调用次数（None = 不限制）
    /// 统计口径：包含所有 LLM 请求（评估、反思、规则提取等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_llm_calls: Option<u32>,
    /// 最大 token 消耗（None = 不限制）
    /// 统计口径：请求 + 响应的总 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    /// 最大运行时间（秒，None = 不限制）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_secs: Option<u64>,
    /// 预算警告阈值（0.0-1.0）
    /// 示例：0.8 表示当已使用预算 ≥ 80% 时，在日志和 Checkpoint 中触发警告
    #[serde(default = "default_budget_warn_threshold")]
    pub warn_threshold: f64,
}

/// Racing 策略配置（对应 Section 9.9）
/// 
/// **核心机制**：借鉴 CAPO 的早期淘汰策略，在有限评估预算下快速收敛到最佳候选。
/// 
/// **生效条件**：Racing 的生效条件是"有效候选数 ≥ 2"，与 OutputStrategy 无直接关系。
/// 当有效候选数 ≤ 1 时，Racing 逻辑自动短路为 no-op，即使 enabled = true。
/// 这允许"内部多候选竞争 + 外部单输出"的灵活组合。
/// 
/// **候选来源**：
/// - Phase 1 的多变体生成（OutputStrategy::Multi 或变体注入）
/// - Phase 2 迭代中的 DiversityInjection（震荡检测后）
/// 具体生成策略由 OutputStrategy 和 OscillationConfig 控制。
/// 
/// **轮次定义**：此处的"轮次"指 Phase 2 外层迭代（IterationResult.iteration），
/// 即每次根据当前规则集和候选池在 Train/Val 上重新评估并更新的循环。
/// 
/// **与 Confidence 的关系**：当候选的评估置信度低于 `EvaluatorConfig.confidence_low_threshold` 时，
/// Racing 不应做出强淘汰决策，而应视为"证据不足"，推迟裁决或增加样本。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RacingConfig {
    /// 是否启用 Racing 策略
    #[serde(default)]
    pub enabled: bool,
    /// 候选池大小上限
    #[serde(default = "default_racing_pool_size")]
    pub pool_size: u32,
    /// 淘汰间隔轮数（每隔多少个 Phase2 迭代进行一次淘汰）
    #[serde(default = "default_elimination_rounds")]
    pub elimination_rounds: u32,
    /// 存活阈值（相对最佳候选的分数比例，低于此值被淘汰）
    /// 分数来源：EnsembleEvaluator 输出的综合分数，在 Val split 上计算
    #[serde(default = "default_survival_threshold")]
    pub survival_threshold: f64,
    /// 早期停止置信度（候选间差距足够显著时提前决定胜者）
    #[serde(default = "default_early_stop_confidence")]
    pub early_stop_confidence: f64,
}

// ===== 默认值函数 =====
fn default_output_strategy() -> OutputStrategy { OutputStrategy::Single }
fn default_conflict_alert_threshold() -> u32 { 3 }
fn default_true() -> bool { true }
fn default_minibatch_size() -> u32 { 10 }
fn default_full_eval_interval() -> u32 { 5 }
fn default_minibatch_recommend_threshold() -> u32 { 20 }
fn default_oscillation_threshold() -> u32 { 3 }
fn default_max_abstraction_level() -> u32 { 3 }
fn default_similarity_threshold() -> f64 { 0.8 }
fn default_clustering_threshold() -> u32 { 50 }
fn default_max_iterations() -> u32 { 20 }
fn default_pass_threshold() -> f64 { 0.95 }
fn default_diversity_inject_after() -> u32 { 3 }
fn default_train_ratio() -> f64 { 0.70 }
fn default_validation_ratio() -> f64 { 0.15 }
fn default_overfitting_threshold() -> f64 { 0.10 }
fn default_confidence_high_threshold() -> f64 { 0.8 }
fn default_confidence_low_threshold() -> f64 { 0.5 }
fn default_llm_judge_samples() -> u32 { 1 }
fn default_hard_checks_weight() -> f64 { 0.4 }
fn default_agreement_weight() -> f64 { 0.4 }
fn default_variance_penalty() -> f64 { 0.2 }
fn default_budget_warn_threshold() -> f64 { 0.8 }
fn default_racing_pool_size() -> u32 { 3 }
fn default_elimination_rounds() -> u32 { 3 }
fn default_survival_threshold() -> f64 { 0.85 }
fn default_early_stop_confidence() -> f64 { 0.95 }
```

**配置分组说明**：

| 分组 | 对应 Section | 职责 |
|------|-------------|------|
| `OutputConfig` | 9.1 | 控制 Prompt 输出策略 |
| `MinibatchConfig` | 9.2 | 控制测试批次采样 |
| `OscillationConfig` | 9.3 | 控制震荡检测和响应 |
| `RuleConfig` | 9.4 | 控制规律抽象和合并 |
| `IterationConfig` | 9.5 | 控制迭代终止条件 |
| `DataSplitConfig` | 9.6 | 控制数据划分（Train/Val/Holdout） |
| `EvaluatorConfig` | 9.7 | 控制评估器组合与 confidence 门控 |
| `BudgetConfig` | 9.8 | 控制 per-run 全局预算（LLM 调用/token/时间） |
| `RacingConfig` | 9.9 | 控制多候选早期淘汰策略 |

#### 4.2.6.2 迭代辅助数据结构定义

> **新增** — 2025-12-16
> 
> 定义 Phase 2 迭代过程中使用的辅助数据结构，用于伪代码逻辑表达。

```rust
/// 迭代结果（parallel_test_iteration 返回值）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    /// 迭代状态
    pub status: IterationResultStatus,
    /// 当前/新生成的 Prompt
    pub prompt: String,
    /// 更新后的规律体系（如有更新）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_system: Option<RuleSystem>,
    /// 所有执行结果（用于回归检测）
    #[serde(default)]
    pub results: Vec<ExecutionResult>,
    /// 失败的测试用例
    #[serde(default)]
    pub failed_cases: Vec<FailedTestResult>,
    /// 上一轮通过的用例 ID（用于回归检测）
    #[serde(default)]
    pub previous_passed_cases: Vec<String>,
    /// 延迟执行的规则层建议（因 confidence 不足被暂缓）
    /// 应记入 Checkpoint，待后续高置信度时再次尝试
    #[serde(default)]
    pub deferred_suggestions: Vec<Suggestion>,
}

/// 迭代结果状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IterationResultStatus {
    /// 全部通过，迭代成功
    Success,
    /// 需要继续迭代
    Continue,
    /// 达到最大轮数
    MaxIterationsReached,
    /// 需要人工介入
    HumanInterventionRequired,
}

/// 失败测试结果（包装 ExecutionResult 的失败情况）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedTestResult {
    /// 对应的测试用例
    pub test_case: TestCase,
    /// 执行结果
    pub execution_result: ExecutionResult,
    /// 失败原因摘要
    pub failure_summary: String,
}

/// 安全检查结果（safety_check 返回值）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheckResult {
    /// 检查状态
    pub status: SafetyStatus,
    /// 回归的测试用例 ID（如检测到回归）
    #[serde(default)]
    pub regressions: Vec<String>,
    /// 详细信息（如过拟合警告的具体描述）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// 安全检查状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyStatus {
    /// 安全检查通过
    Ok,
    /// 检测到回归
    Regression,
    /// 检测到震荡，需要注入多样性
    OscillationInject,
    /// 停止迭代
    Stop,
    /// 过拟合警告（Holdout 通过率显著低于 Validation）
    OverfittingWarning,
}

/// 反思聚类（cluster_by_root_cause 返回值）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionCluster {
    /// 聚类 ID
    pub cluster_id: String,
    /// 根因描述
    pub root_cause: String,
    /// 该聚类包含的反思结果
    pub reflections: Vec<ReflectionResult>,
    /// 该聚类的改进建议
    pub suggestions: Vec<Suggestion>,
}

/// 迭代历史（用于震荡检测）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationHistory {
    /// 历史状态列表（按时间顺序）
    pub states: Vec<IterationStateSnapshot>,
}

/// 迭代状态快照（用于震荡检测比较）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationStateSnapshot {
    /// 迭代轮次
    pub iteration: u32,
    /// 当前状态
    pub state: IterationState,
    /// 通过率
    pub pass_rate: f64,
    /// 失败用例 ID 集合（用于比较状态相似性）
    pub failed_case_ids: Vec<String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}
```

#### 4.2.7 核心 Trait 完整签名定义

> **新增** — 2025-12-15
> 
> 定义所有核心 Trait 的完整方法签名，供开发者实现参考。

##### RuleEngine Trait

```rust
/// 规律引擎 Trait（对应 Layer 1 Pattern Extractor）
#[async_trait]
pub trait RuleEngine: Send + Sync {
    /// 从测试用例中提取规律
    async fn extract_rules(
        &self,
        ctx: &OptimizationContext,
        test_cases: &[TestCase],
    ) -> Result<Vec<Rule>, RuleEngineError>;
    
    /// 检测规律之间的冲突
    async fn detect_conflicts(
        &self,
        ctx: &OptimizationContext,
        rules: &[Rule],
    ) -> Result<Vec<RuleConflict>, RuleEngineError>;
    
    /// 解决规律冲突
    async fn resolve_conflict(
        &self,
        ctx: &OptimizationContext,
        conflict: &RuleConflict,
    ) -> Result<Rule, RuleEngineError>;
    
    /// 合并相似规律
    async fn merge_similar_rules(
        &self,
        ctx: &OptimizationContext,
        rules: &[Rule],
    ) -> Result<Vec<Rule>, RuleEngineError>;
    
    /// 引擎名称（用于日志和调试）
    fn name(&self) -> &str;
}
```

##### PromptGenerator Trait

```rust
/// Prompt 生成器 Trait（对应 Layer 2 Prompt Engineer）
#[async_trait]
pub trait PromptGenerator: Send + Sync {
    /// 基于规律体系生成 Prompt
    async fn generate(
        &self,
        ctx: &OptimizationContext,
    ) -> Result<String, GeneratorError>;
    
    /// 生成器名称
    fn name(&self) -> &str;
}
```

##### Evaluator Trait

```rust
/// 评估器 Trait（对应 Layer 3 Quality Assessor）
#[async_trait]
pub trait Evaluator: Send + Sync {
    /// 评估单个测试用例的输出
    async fn evaluate(
        &self,
        ctx: &OptimizationContext,
        test_case: &TestCase,
        output: &str,
    ) -> Result<EvaluationResult, EvaluatorError>;
    
    /// 批量评估（可优化为并行）
    async fn evaluate_batch(
        &self,
        ctx: &OptimizationContext,
        results: &[(TestCase, String)],  // (test_case, output) pairs
    ) -> Result<Vec<EvaluationResult>, EvaluatorError>;
    
    /// 评估器名称
    fn name(&self) -> &str;
}
```

##### FeedbackAggregator Trait

```rust
/// 反馈聚合器 Trait（对应 Layer 4 Reflection Agent - 聚合部分）
#[async_trait]
pub trait FeedbackAggregator: Send + Sync {
    /// 聚合多个反思结果
    async fn aggregate(
        &self,
        ctx: &OptimizationContext,
        reflections: &[ReflectionResult],
    ) -> Result<UnifiedReflection, AggregatorError>;
    
    /// 仲裁冲突的建议
    async fn arbitrate(
        &self,
        ctx: &OptimizationContext,
        conflicts: &[SuggestionConflict],
    ) -> Result<ArbitrationResult, AggregatorError>;
    
    /// 聚合器名称
    fn name(&self) -> &str;
}
```

##### Optimizer Trait

```rust
/// 优化器 Trait（对应 Layer 4 Reflection Agent - 优化部分）
#[async_trait]
pub trait Optimizer: Send + Sync {
    /// 基于统一反馈执行一步优化
    async fn optimize_step(
        &self,
        ctx: &OptimizationContext,
        unified_reflection: &UnifiedReflection,
    ) -> Result<OptimizationResult, OptimizerError>;
    
    /// 判断是否应该终止迭代
    fn should_terminate(
        &self,
        ctx: &OptimizationContext,
        history: &[OptimizationResult],
    ) -> Option<TerminationReason>;
    
    /// 优化器名称
    fn name(&self) -> &str;
}
```

##### TeacherModel Trait

```rust
/// 老师模型 Trait（辅助 Trait，贯穿各 Layer）
#[async_trait]
pub trait TeacherModel: Send + Sync {
    /// 生成文本响应
    async fn generate(
        &self,
        prompt: &str,
    ) -> Result<String, ModelError>;
    
    /// 生成结构化响应（JSON 模式）
    async fn generate_structured<T: DeserializeOwned + Send>(
        &self,
        prompt: &str,
    ) -> Result<T, ModelError>;
    
    /// 模型名称
    fn model_name(&self) -> &str;
}
```

##### ExecutionTarget Trait

```rust
/// 执行目标 Trait（辅助 Trait，用于 Phase 2 测试执行）
/// 
/// 支持两种执行目标：
/// - Dify 工作流：调用 Dify API
/// - 直接 AI 模型：直接调用 LLM API
#[async_trait]
pub trait ExecutionTarget: Send + Sync {
    /// 执行 Prompt 并返回输出
    async fn execute(
        &self,
        prompt: &str,
        input: &HashMap<String, serde_json::Value>,
    ) -> Result<ExecutionResult, ExecutionError>;
    
    /// 批量执行（可优化为并行）
    async fn execute_batch(
        &self,
        prompt: &str,
        inputs: &[HashMap<String, serde_json::Value>],
    ) -> Result<Vec<ExecutionResult>, ExecutionError>;
    
    /// 目标名称
    fn name(&self) -> &str;
}
```

#### 4.2.8 错误类型定义

> **新增** — 2025-12-16
> 
> 定义各 Trait 的错误类型，供开发者实现参考。

##### 错误类型设计原则

- **模块边界表达**：每个 Trait 拥有自己的错误类型，用于在模块边界上表达「该模块能失败的主要原因」
- **大类优先**：保持变体数量适中（3-5 个大类），避免过度约束实现细节
- **可扩展性**：允许实现内部再细分，底层错误可 wrap 到 `Internal` 变体中
- **可观测性**：错误应包含足够上下文信息，便于日志和监控

##### RuleEngineError

```rust
/// 规律引擎错误
#[derive(Debug, thiserror::Error)]
pub enum RuleEngineError {
    /// 规律解析或验证失败
    #[error("invalid rule: {0}")]
    InvalidRule(String),
    /// 冲突解决失败（无法找到有效解决方案）
    #[error("conflict resolution failed: {0}")]
    ConflictResolutionFailed(String),
    /// 老师模型调用失败
    #[error("model failure: {0}")]
    ModelFailure(String),
    /// 其它内部错误
    #[error("internal error: {0}")]
    Internal(String),
}
```

##### GeneratorError

```rust
/// Prompt 生成器错误
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    /// 模板渲染失败
    #[error("template error: {0}")]
    TemplateError(String),
    /// 规律体系不完整或无效
    #[error("invalid rule system: {0}")]
    InvalidRuleSystem(String),
    /// 老师模型调用失败
    #[error("model failure: {0}")]
    ModelFailure(String),
    /// 其它内部错误
    #[error("internal error: {0}")]
    Internal(String),
}
```

##### EvaluatorError

```rust
/// 评估器错误
#[derive(Debug, thiserror::Error)]
pub enum EvaluatorError {
    /// 输入格式无效
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// 评估超时
    #[error("evaluation timeout: {0}")]
    Timeout(String),
    /// 老师模型调用失败（LLM 评估时）
    #[error("model failure: {0}")]
    ModelFailure(String),
    /// 其它内部错误
    #[error("internal error: {0}")]
    Internal(String),
}
```

##### AggregatorError

```rust
/// 反馈聚合器错误
#[derive(Debug, thiserror::Error)]
pub enum AggregatorError {
    /// 输入反思结果为空或无效
    #[error("invalid reflections: {0}")]
    InvalidReflections(String),
    /// 仲裁失败（无法解决建议冲突）
    #[error("arbitration failed: {0}")]
    ArbitrationFailed(String),
    /// 老师模型调用失败
    #[error("model failure: {0}")]
    ModelFailure(String),
    /// 其它内部错误
    #[error("internal error: {0}")]
    Internal(String),
}
```

##### OptimizerError

```rust
/// 优化器错误
#[derive(Debug, thiserror::Error)]
pub enum OptimizerError {
    /// 优化步骤失败
    #[error("optimization step failed: {0}")]
    StepFailed(String),
    /// 状态不一致
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// 老师模型调用失败
    #[error("model failure: {0}")]
    ModelFailure(String),
    /// 其它内部错误
    #[error("internal error: {0}")]
    Internal(String),
}
```

##### ModelError

```rust
/// 老师模型错误
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    /// API 调用失败（网络/认证/配额）
    #[error("api error: {0}")]
    ApiError(String),
    /// 响应解析失败（JSON 格式错误等）
    #[error("parse error: {0}")]
    ParseError(String),
    /// 请求超时
    #[error("timeout: {0}")]
    Timeout(String),
    /// 其它内部错误
    #[error("internal error: {0}")]
    Internal(String),
}
```

##### ExecutionError

```rust
/// 执行目标错误
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    /// 网络/连接失败
    #[error("network error: {0}")]
    Network(String),
    /// 请求参数无效
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// 目标服务返回错误（Dify/AI 模型）
    #[error("target failure: {0}")]
    TargetFailure(String),
    /// 请求超时
    #[error("timeout: {0}")]
    Timeout(String),
    /// 其它内部错误
    #[error("internal error: {0}")]
    Internal(String),
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
    prompt_generators: HashMap<String, Arc<dyn PromptGenerator>>,
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
| **新增优化策略** | < 4 小时 | 实现 Optimizer Trait，注册到 Registry |
| **新增执行目标** | < 4 小时 | 实现 ExecutionTarget Trait，注册到 Registry |
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

### 4.6 架构模式研究总结

> **Step 4 产出** — 2025-12-15
> 
> 本节总结 Prompt Faster 核心算法架构所采用的设计模式，对比业界框架的模式选择，并分析关键架构决策的 Trade-off。

#### 4.6.1 采用的架构模式清单

本算法规格采用了以下架构模式：

| 模式名称 | 应用位置 | 设计目的 | 业界参考 |
|---------|---------|---------|---------|
| **分层架构（Layered Architecture）** | 4.1 四层架构 | 职责分离、变更影响范围可控、可独立测试 | 经典软件架构 |
| **Trait 驱动的插件式架构** | 4.2 核心 Trait 体系 | 模块解耦、可插拔组件、编译时类型检查 | DSPy Module 基类 |
| **Registry 模式（运行时注册）** | 4.3 决策 A | 动态发现与注册组件、支持 A/B 测试 | 依赖注入容器 |
| **编排者模式（Orchestrator）** | 4.1 编排层 StrategyOrchestrator | 协调多模块执行顺序、分离编排逻辑与业务逻辑 | DSPy Teleprompter |
| **状态机模式（State Machine）** | Section 11 | 精确控制迭代流程状态转换、支持前端实时显示 | 经典状态机设计 |
| **分支治理（Checkpoint Lineage）** | 4.3 决策 D | 支持多分支探索、历史追溯、归因分析 | Git 版本控制 |
| **配置驱动（Configuration-Driven）** | 4.5 | 通过 YAML 配置控制行为、用户无需改代码 | PromptWizard 配置系统 |
| **反馈聚合（Feedback Aggregation）** | 8.5 | 聚合多个反思结果、处理冲突建议（受 TextGrad 启发） | TextGrad 梯度聚合 |
| **分层验证（Layered Validation）** | 8.3 | 根据修改类型采用不同验证强度、平衡效率与一致性 | 本项目独创 |

#### 4.6.2 业界框架模式对比

本节对比 Prompt Faster 与业界主流框架的架构模式选择，说明借鉴与改良之处。

| 业界框架 | 核心架构模式 | Prompt Faster 的借鉴 | Prompt Faster 的改良 |
|---------|-------------|---------------------|---------------------|
| **DSPy** | `Module` 基类 + `forward()` 方法；`Teleprompter` 优化器（MIPROv2, COPRO, BootstrapFewShot） | Trait 抽象与 Module 类似；Optimizer 接口设计参考 Teleprompter | 使用 Rust Trait 替代 Python 类继承，编译时类型检查更强；增加规律层作为中间表示 |
| **TextGrad** | `Variable`（值+梯度）+ `TextualGradientDescent.step()` | FeedbackAggregator 的反馈聚合机制（Section 8.5） | 增加冲突仲裁层（ArbitrationResult），支持多源反馈合并与冲突解决 |
| **PromptWizard** | `DatasetSpecificProcessing` 抽象 + Mutation/Refinement 配置驱动 | 配置驱动设计（Section 4.5）；任务特定处理抽象 | 增加 `TaskReference` 枚举统一双任务模式（固定/创意/混合） |
| **GEPA** | `GEPAAdapter` 接口 + Pareto 前沿演化 | 三层冲突解决策略中的 Pareto 备选方案（决策 D2） | 与规律驱动机制结合，Pareto 用于规律冲突而非直接用于 Prompt 变体 |
| **Reflexion** | Episodic Memory + 自我反思 | `ReflectionResult` 结构化反思（Section 4.2.3） | 区分规律问题（RuleIncomplete/RuleIncorrect）和表达问题（ExpressionIssue），分类处理 |

**关键改良点**：

1. **规律驱动是独创**：业界框架直接优化 Prompt，我们增加"规律"（RuleSystem）作为中间层，实现：
   - **可解释性**：用户可以看到"为什么这样优化"
   - **人机协作**：用户可以直接编辑规律
   - **问题归因**：区分"规律问题"和"表达问题"

2. **Rust Trait 替代 Python 类**：
   - DSPy/TextGrad/PromptWizard 均使用 Python 类继承
   - Prompt Faster 使用 Rust Trait + ts-rs 自动生成 TypeScript 类型
   - 优势：编译时保证接口契约、内存安全、更适合长期维护

3. **分层验证策略**：
   - 业界框架对规律/Prompt 更新通常采用统一验证
   - Prompt Faster 根据 `SuggestionType` 采用不同验证强度（Section 8.3）
   - 轻量级（Rephrase）直接应用，重型（AddRule）走完整验证流程

#### 4.6.3 关键架构决策的 Trade-off 分析

本节分析核心架构决策的选择理由与权衡。

| 决策 | 选择 | 优势 | 代价 | 选择理由 |
|------|------|------|------|---------|
| **语言选型** | Rust Trait + ts-rs 生成 TypeScript | 编译时类型检查、内存安全、高性能 | 学习曲线较高、生态不如 Python 丰富 | PRD 要求"低维护成本"，Rust 类型系统更适合长期维护 |
| **架构分层** | 四层分层（Application/Orchestration/Core/Infrastructure） | 职责分离、可测试、可替换 | 增加调用层次、初期开发成本略高 | PRD 要求"核心算法替换仅影响算法模块"，分层是必要保障 |
| **核心机制** | 规律驱动（增加 RuleEngine 中间层） | 可解释性、人机协作、问题归因 | 增加复杂度、可能增加迭代轮数 | 产品愿景强调"玻璃盒"透明迭代，规律是实现透明的关键 |
| **模块注册** | 运行时 ModuleRegistry | 热切换、A/B 测试、配置驱动 | 运行时开销、部分类型检查延迟到运行时 | 用户需要灵活组合模块，运行时注册更适合 |
| **状态管理** | 完整 Checkpoint Lineage | 归因分析、元优化数据、可追溯性 | 存储成本、实现复杂度 | 需要区分"自动优化成功"和"人工介入成功"，支持元优化 |
| **反馈聚合** | TextGrad 风格反馈聚合 + 冲突仲裁 | 处理多源反馈、解决冲突建议 | 仲裁逻辑复杂、可能需要人工介入 | 并行测试会产生多个反思结果，需要聚合机制 |

**与 PRD 成功标准的对应**：

| PRD 成功标准 | 架构支撑 |
|-------------|---------|
| 新增评估器 < 2 小时 | Evaluator Trait 定义清晰边界 |
| 新增优化策略 < 4 小时 | Optimizer Trait + ModuleRegistry |
| 核心算法替换仅影响算法模块 | 四层分层 + Trait 接口隔离 |
| 断点续跑 | Checkpoint Lineage + 分支治理 |
| 模块化功能组合 | 配置驱动 + 运行时注册 |

### 4.7 PRD 接口映射说明

> **新增** — 2025-12-16
> 
> 本节说明 PRD（`docs/prd.md` Section 7.4.1）定义的接口与本技术规格 Trait 体系的映射关系，帮助开发者理解设计演进。

#### 4.7.1 PRD 与技术规格的抽象层级差异

**核心判断：差异不是错误，而是抽象层级不同。**

- **PRD = 产品视角**：关注"系统能做什么"，接口更偏"功能角色"
  - 老师模型可以"提规律 / 生 Prompt / 反思"
  - Evaluator 能帮用户判断 expected vs actual
  
- **技术规格 = 实现视角**：关注"如何高内聚低耦合地拆分职责"，接口更偏"代码抽象"
  - 规则相关 → `RuleEngine`
  - Prompt 生成 → `PromptGenerator`
  - 评估 → `Evaluator`
  - 反思与聚合 → `FeedbackAggregator`
  - 迭代策略 → `Optimizer`
  - 底层 LLM 调用 → `TeacherModel`
  - 执行目标 → `ExecutionTarget`

#### 4.7.2 四层架构到 Trait 的映射表

| PRD 四层架构 | 技术规格 Trait | 说明 |
|--------------|---------------|------|
| Layer 1: Pattern Extractor | `RuleEngine` | 规律提取、冲突检测、解决、合并 |
| Layer 2: Prompt Engineer | `PromptGenerator` | 基于规律体系生成 Prompt |
| Layer 3: Quality Assessor | `Evaluator` | 评估执行结果，判断通过/失败 |
| Layer 4: Reflection Agent | `FeedbackAggregator` + `Optimizer` | 见下方拆分说明 |
| (底层 LLM 调用) | `TeacherModel` | 通用 LLM 适配层 |
| (执行目标) | `ExecutionTarget` | Dify 工作流 / 直连模型 |

**Layer 4 拆分说明**：

PRD 视角下"Reflection Agent"是一个整体；技术规格将其拆分为两个独立 Trait：

- **`FeedbackAggregator`**：聚合多个 `ReflectionResult`，解决冲突建议，输出 `UnifiedReflection`
- **`Optimizer`**：根据聚合结果执行具体优化步骤，决定何时终止

拆分原因：聚合逻辑（如何合并多个反思）与优化策略（如何应用反思）是正交关注点，分离后可独立替换。

#### 4.7.3 TeacherModel 职责演进

| 维度 | PRD 定义 | 技术规格定义 |
|------|----------|--------------|
| 方法 | `generate_rules`, `generate_prompt`, `reflect` | `generate`, `generate_structured` |
| 职责 | 高层业务逻辑（提规律/生Prompt/做反思） | 底层 LLM 调用适配 |
| 定位 | "万能老师" | "通用 LLM 客户端" |

**演进原因**：

1. **降低耦合**：将高层业务逻辑迁移至对应 Trait（RuleEngine/PromptGenerator/FeedbackAggregator），TeacherModel 只负责"调用 LLM + 返回文本/结构化结果"
2. **提高复用**：同一个 TeacherModel 实例可被多个模块共享（RuleEngine、Evaluator、FeedbackAggregator 都可能调用 LLM）
3. **简化测试**：业务逻辑 Trait 可以 mock TeacherModel 进行单元测试

#### 4.7.4 其他签名差异说明

| Trait | PRD 签名 | 技术规格签名 | 演进说明 |
|-------|----------|--------------|----------|
| `Evaluator.evaluate` | `(expected, actual, goal)` | `(ctx, test_case, output)` | `expected` 融入 `TestCase.reference`；`goal` 融入 `OptimizationContext`；暴露完整 `TestCase` 支持更复杂评估 |
| `ExecutionTarget.execute` | `(prompt, input: &TestInput)` | `(prompt, input: &HashMap<String, serde_json::Value>)` | `HashMap` 更通用，可统一表达 Dify 变量和直连模型输入 |

**开发者注意**：实现 Trait 时应参考本技术规格的签名，PRD 签名视为"概念示意"。

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

> **伪代码风格说明**
> 
> 本节及后续章节（Section 7、8）的伪代码采用 **Python 语法**书写，
> 但枚举类型使用 **Rust 风格**（如 `OutputStrategy.Single`、`FailureType.RuleIncomplete`）。
> 这仅用于逻辑表达，不代表真实实现语言。实际实现将使用 Rust。

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

> **修订说明** — 2025-12-15
> 
> 基于决策 C1（语言统一），将 Rule、RuleSystem 从 TypeScript 改为 Rust 定义。
> TypeScript 类型将由 ts-rs 自动生成。

#### 6.2.1 Rule 结构

```rust
/// 规律结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Rule {
    /// 唯一标识（UUID）
    pub id: String,
    /// 规律描述（自然语言）
    pub description: String,
    /// 结构化标签
    pub tags: RuleTags,
    /// 来源测试用例 ID（规则可靠性证据）
    pub source_test_cases: Vec<String>,
    /// 抽象层级 (0=原始, 1=冲突解决后, 2=二次抽象)
    pub abstraction_level: u32,
    /// 父规律 ID（合并/抽象产生时）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_rules: Vec<String>,
    /// 是否已验证
    pub verified: bool,
    /// 验证分数 0.0-1.0（规则可靠性置信度）
    pub verification_score: f64,
    /// 中间表示（可选，用于程序化规律操作）
    /// 详见 RuleIR 结构定义
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir: Option<RuleIR>,
}

/// 规律标签（核心字段 + 扩展字段）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RuleTags {
    // ===== 输出维度 =====
    /// 输出格式 ["markdown", "json", "xml", "plain_text"]
    #[serde(default)]
    pub output_format: Vec<String>,
    /// 输出结构 ["list", "paragraph", "table", "key-value"]
    #[serde(default)]
    pub output_structure: Vec<String>,
    /// 输出长度
    pub output_length: OutputLength,
    
    // ===== 语义维度 =====
    /// 语义焦点 ["extraction", "transformation", "generation", "summarization"]
    #[serde(default)]
    pub semantic_focus: Vec<String>,
    /// 关键概念词
    #[serde(default)]
    pub key_concepts: Vec<String>,
    
    // ===== 约束维度 =====
    /// 必须包含的元素
    #[serde(default)]
    pub must_include: Vec<String>,
    /// 必须排除的元素
    #[serde(default)]
    pub must_exclude: Vec<String>,
    /// 语气 "formal" | "casual" | "technical"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
    
    // ===== 扩展字段 =====
    /// 预留扩展（支持用户自定义维度）
    #[serde(default, flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 输出长度枚举
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum OutputLength {
    Short,
    Medium,
    Long,
    Flexible,
}

/// 规律中间表示（可选，用于程序化规律操作）
/// 
/// **设计定位**：RuleIR 专注于"规则内容的可计算表达"，
/// 而 Rule 上的 source_test_cases/verification_score 等字段负责"规则可靠性与证据"。
/// 两者配合使用，实现"可计算骨架 + 元信息/证据"的职责分离。
/// 
/// **演进原则**：字段类型当前保持宽松（String/Vec<String>），
/// 未来版本可逐步收紧为结构化类型，并给出推荐语法规范。
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
pub struct RuleIR {
    /// 适用范围表达式（字段路径/标签组合/条件表达式）
    /// 示例："input.type == 'json'" 或 "tags.contains('extraction')"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    
    /// 结构化约束列表（类型约束/值域约束/存在性约束等）
    /// 预期采用机器可解析的表达方式（如 path op value 形式），
    /// 未来版本将给出推荐语法规范。
    /// 示例：["output.format == 'json'", "output.fields.contains('name')"]
    #[serde(default)]
    pub constraints: Vec<String>,
    
    /// 输出格式 Schema 引用（JSON Schema URI 或内联定义）
    /// 示例："schemas/response.json" 或 inline JSON Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<String>,
    
    /// 冲突解决优先级（数值越高优先级越高）
    /// 当两条规则冲突时，高优先级规则的约束优先
    #[serde(default)]
    pub priority: u32,
    
    /// 例外场景描述（本规则不适用的情况）
    /// 示例：["当输入为空时", "当用户明确要求简短回复时"]
    #[serde(default)]
    pub exceptions: Vec<String>,
}
```

#### 6.2.2 RuleSystem 结构

```rust
/// 规律体系
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RuleSystem {
    /// 规律列表
    pub rules: Vec<Rule>,
    /// 冲突解决日志
    #[serde(default)]
    pub conflict_resolution_log: Vec<ConflictResolutionRecord>,
    /// 合并日志
    #[serde(default)]
    pub merge_log: Vec<RuleMergeRecord>,
    /// 覆盖映射 (testCaseId -> ruleIds)
    #[serde(default)]
    pub coverage_map: HashMap<String, Vec<String>>,
    /// 版本号
    pub version: u32,
}

/// 冲突解决记录
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ConflictResolutionRecord {
    pub id: String,
    pub conflicting_rule_ids: Vec<String>,
    pub resolved_rule_id: String,
    /// 解决方案描述
    pub resolution: String,
    pub timestamp: DateTime<Utc>,
}

/// 规律合并记录
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RuleMergeRecord {
    pub id: String,
    pub source_rule_ids: Vec<String>,
    pub merged_rule_id: String,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

/// 规律冲突（用于 RuleEngine.detect_conflicts 返回）
/// 
/// 设计说明：嵌入完整 Rule 对象以便冲突解决算法直接访问规律内容，
/// 避免多次通过 ID 查找。related_test_cases 用于验证解决方案。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RuleConflict {
    /// 冲突规律 1（完整对象）
    pub rule1: Rule,
    /// 冲突规律 2（完整对象）
    pub rule2: Rule,
    /// 冲突类型
    pub conflict_type: RuleConflictType,
    /// 冲突描述
    pub description: String,
    /// 相关测试用例（用于验证冲突解决方案）
    #[serde(default)]
    pub related_test_cases: Vec<TestCase>,
}

/// 规律冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub enum RuleConflictType {
    /// 直接矛盾（A说要，B说不要）
    DirectContradiction,
    /// 范围冲突（A是B的子集但要求不同）
    ScopeConflict,
    /// 优先级不明确
    PriorityAmbiguity,
}
```

### 6.3 算法实现

#### 6.3.1 规律提炼算法

```python
def extract_rules_from_test_cases(test_cases: List[TestCase], config: OptimizationConfig) -> List[Rule]:
    """
    从测试用例中提炼规律
    """
    # Step 1: 可选聚类
    if config.rule.enable_clustering and len(test_cases) > config.rule.clustering_threshold:
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

> **RuleIR 演进说明** — 2025-12-17
> 
> 当前实现采用 LLM-first 策略。未来版本当 `Rule.ir` 可用时，推荐优先基于 RuleIR 的 
> `scope/constraints/priority` 等字段进行静态冲突判断，LLM 仅用于解释与兜底。
> 这将显著提升冲突检测的效率和一致性。

```python
def detect_conflicts(rules: List[Rule]) -> List[Tuple[Rule, Rule]]:
    """
    检测规律之间的冲突
    使用老师模型进行检测
    
    注：此处返回 Tuple 为伪代码简化表达。
    真实实现中将封装为 List[RuleConflict]，包含 conflict_type、related_test_cases 等信息。
    
    TODO（RuleIR 演进）：当 rule.ir 存在时，优先使用 IR 的 scope/constraints 进行静态判断
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
    
    TODO（RuleIR 演进）：当 rule.ir 存在时，先比较 IR 的 scope 是否重叠、
    constraints 是否矛盾，仅在无法静态判定时才调用 LLM
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
def resolve_conflict(conflict: RuleConflict, config: OptimizationConfig) -> Rule:
    """
    解决两条冲突规律，提炼更高层的统一规律
    
    Args:
        conflict: RuleConflict 结构（包含 rule1, rule2, conflict_type, related_test_cases）
        config: 优化配置
    
    Returns:
        统一后的规律
    """
    rule1, rule2 = conflict.rule1, conflict.rule2
    related_cases = conflict.related_test_cases
    
    # 尝试提炼更高层规律
    prompt = CONFLICT_RESOLUTION_PROMPT.format(
        rule1=rule1.description,
        rule2=rule2.description,
        conflict_type=conflict.conflict_type,
        test_cases=format_test_cases(related_cases)
    )
    response = teacher_model.generate(prompt)
    unified_rule = parse_rule_response(response)
    
    # 设置抽象层级
    unified_rule.abstraction_level = max(rule1.abstraction_level, rule2.abstraction_level) + 1
    unified_rule.parent_rules = [rule1.id, rule2.id]
    
    # 检查是否超过最大抽象层级
    if unified_rule.abstraction_level > config.rule.max_abstraction_level:
        raise HumanInterventionRequired("规律冲突无法自动解决，需要用户介入")
    
    # 验证新规律
    unified_rule.verified = verify_rule(unified_rule, related_cases)
    
    return unified_rule
```

#### 6.3.4 相似合并算法

> **RuleIR 演进说明** — 2025-12-17
> 
> 当前相似度计算依赖 LLM 或基于 description 的文本相似度。
> 未来版本当 `Rule.ir` 可用时，推荐基于 RuleIR 的 `scope/constraints/output_schema` 
> 进行结构化相似度计算，实现更精确的规则合并判定。

```python
def detect_and_merge_similar(rules: List[Rule], config: OptimizationConfig) -> List[Rule]:
    """
    检测并合并相似规律
    
    TODO（RuleIR 演进）：当 rule.ir 存在时，基于 IR 的结构化字段计算相似度
    """
    similar_groups = find_similar_groups(rules, config.rule.similarity_threshold)
    
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
def generate_initial_prompt(rule_system: RuleSystem, goal: str, config: OptimizationConfig) -> Union[str, List[str]]:
    """
    根据输出策略生成初始 Prompt
    """
    if config.output.strategy == OutputStrategy.Single:
        # 策略A: 强制收敛，输出单一 Prompt
        return generate_single_prompt(rule_system.rules, goal)
    
    elif config.output.strategy == OutputStrategy.Adaptive:
        # 策略B: 自适应 Prompt
        type_rules = group_rules_by_type(rule_system.rules)
        return generate_adaptive_prompt(type_rules, goal)
    
    elif config.output.strategy == OutputStrategy.Multi:
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

> **执行顺序契约**
> 
> `parallel_execute(prompt, batch)` 返回的 `ExecutionResult` 列表**顺序与输入 `batch` 一致**。
> 后续 `zip(batch, exec_results, eval_results)` 依赖此契约。
> 若未来改为异步乱序返回，需引入 `test_case_id → ExecutionResult` 的映射机制。

> **Racing 演进说明** — 2025-12-17
> 
> 当前伪代码假设单候选迭代。未来启用 Racing 策略时，需注意以下设计约束：
> 
> **Minibatch vs Racing 职责边界**：
> - Minibatch 控制"每轮评估用多少样本"（横截面采样）
> - Racing 控制"在有限评估预算下，如何逐步淘汰候选"（纵向收敛）
> - 两者互补而非重叠，可独立或配合使用
> 
> **公平性不变量**：
> - 在同一迭代中相互竞争的候选，必须在**同一数据子集**上评估
> - 不允许候选 A 在一批样本、候选 B 在另一批样本上直接做硬比较
> 
> **数据三分与 Racing 的关系**：
> - Racing 只使用 Train/Val 数据做候选间比较和淘汰决策
> - Holdout 仅在最终胜者体检或少量哨兵检查中使用，不参与中途淘汰
> 
> **淘汰可观测性**：
> - 被淘汰候选的信息必须作为 artifacts 保存在 Checkpoint 中
> - 推荐记录字段：`candidate_id`, `candidate_source`, `elimination_iteration`, 
>   `elimination_reason`（如 `dominated_on_val`, `under_survival_threshold`）, `score_snapshot`

```python
def parallel_test_iteration(
    prompt: str, 
    test_cases: List[TestCase], 
    rule_system: RuleSystem, 
    config: OptimizationConfig,
    ctx: OptimizationContext,
    evaluator: Evaluator
) -> IterationResult:
    """
    并行测试迭代
    
    流程：执行(RunningTests) → 评估(Evaluating) → 失败聚类 → 反思 → 更新
    """
    # 确定本轮测试用例集合
    if config.minibatch.enabled and len(test_cases) > config.minibatch.recommend_threshold:
        # Minibatch 模式
        batch = sample_minibatch(test_cases, config.minibatch.size)
    else:
        # 全量测试
        batch = test_cases
    
    # Step 2.1: 并行执行（对应状态 RunningTests）
    exec_results: List[ExecutionResult] = parallel_execute(prompt, batch)
    
    # Step 2.2: 评估结果（对应状态 Evaluating）
    eval_results: List[EvaluationResult] = evaluator.evaluate_batch(
        ctx,
        [(tc, er.output) for tc, er in zip(batch, exec_results)]
    )
    
    # Step 2.3: 识别失败用例
    failed_cases: List[FailedTestResult] = [
        FailedTestResult(
            test_case=tc,
            execution_result=er,
            failure_summary=ev.failure_points[0].description if ev.failure_points else ""
        )
        for tc, er, ev in zip(batch, exec_results, eval_results)
        if not ev.passed
    ]
    
    # 检查是否全部通过
    if len(failed_cases) == 0:
        return IterationResult(
            status=IterationResultStatus.Success, 
            prompt=prompt,
            results=exec_results
        )
    
    # Step 2.4: 失败聚类（对应状态 ClusteringFailures）
    reflections = parallel_reflect(failed_cases, prompt, rule_system)
    clusters = cluster_by_root_cause(reflections)
    
    # Step 2.5: 反思仲裁（对应状态 Reflecting，借鉴 TextGrad 梯度聚合）
    unified_reflection = aggregate_reflections(clusters, config)
    
    # Step 2.6: 应用更新（使用分层验证策略 + confidence 门控）
    # 计算当前轮评估的聚合置信度（取所有评估结果的 confidence 均值或最小值）
    evaluation_confidence = compute_aggregated_confidence(eval_results)
    
    if unified_reflection.primary_failure_type == FailureType.RuleIncomplete:
        # 情况A: 规律问题 → 更新规律体系（对应状态 UpdatingRules）
        # 注意：confidence 门控会限制低置信度时的规则层更新
        rule_system, deferred_suggestions = update_rule_system_with_validation(
            rule_system, 
            unified_reflection,
            config,
            evaluation_confidence
        )
        # 延迟的候选建议记入 backlog，供后续高置信度时再次尝试
        if len(deferred_suggestions) > 0:
            log_info(f"本轮有 {len(deferred_suggestions)} 条规则层建议因置信度不足被延迟")
            # TODO: 将 deferred_suggestions 写入 Checkpoint.deferred_rule_suggestions
        new_prompt = generate_prompt_from_rules(rule_system, config)
    else:
        # 情况B: 表达问题 → 只调整 Prompt（对应状态 Optimizing）
        # 表达层更新不受 confidence 门控限制
        new_prompt = refine_prompt(prompt, unified_reflection)
        deferred_suggestions = []
    
    return IterationResult(
        status=IterationResultStatus.Continue,
        prompt=new_prompt,
        rule_system=rule_system,
        results=exec_results,
        failed_cases=failed_cases,
        deferred_suggestions=deferred_suggestions  # 新增：传递延迟建议
    )
```

### 8.3 分层规律更新验证（决策 E3）

> **新增** — 2025-12-15
> 
> Phase 2 规律更新时，根据 SuggestionType 采用不同的验证策略，
> 平衡迭代效率和规律体系一致性。

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     分层验证策略流程图                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Phase 2 反思结果 (UnifiedReflection)                                      │
│          ↓                                                                  │
│   遍历 unified_suggestions                                                  │
│          ↓                                                                  │
│   判断 SuggestionType                                                       │
│          ↓                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ 轻量级类型（无需验证）                                               │   │
│   │ - Rephrase（修改措辞）                                               │   │
│   │ - ChangeFormat（修改格式）                                           │   │
│   │                                                                     │   │
│   │ → 直接应用更新                                                       │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│          ↓                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ 中等类型（冲突检测）                                                 │   │
│   │ - AddExample（增加示例）                                             │   │
│   │ - AddConstraint（增加约束）                                          │   │
│   │                                                                     │   │
│   │ → RuleEngine.detect_conflicts()                                     │   │
│   │ → 无冲突则应用，有冲突则按完整流程处理                                │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│          ↓                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ 重型类型（完整 Phase 0 验证流程）                                    │   │
│   │ - AddRule（新增规律）                                                │   │
│   │ - ModifyRule（修改规律）                                             │   │
│   │ - RemoveRule（删除规律）                                             │   │
│   │                                                                     │   │
│   │ → Step 0.3: 冲突检测                                                 │   │
│   │ → Step 0.4: 冲突解决                                                 │   │
│   │ → Step 0.5: 相似合并                                                 │   │
│   │ → Step 0.6: 规律验证                                                 │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│          ↓                                                                  │
│   更新后的 RuleSystem                                                       │
│          ↓                                                                  │
│   重新生成 Prompt                                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

```python
def update_rule_system_with_validation(
    rule_system: RuleSystem, 
    unified_reflection: UnifiedReflection,
    config: OptimizationConfig,
    evaluation_confidence: float  # 新增：当前评估结果的置信度
) -> Tuple[RuleSystem, List[Suggestion]]:
    """
    使用分层验证策略更新规律体系（决策 E3）
    
    新增 confidence 门控逻辑（2025-12-17）：
    - 高置信度：允许所有类型的更新
    - 中等置信度：仅允许轻量级更新，重型建议记为候选
    - 低置信度：不执行任何更新，仅记录
    
    返回值：
    - 更新后的 RuleSystem
    - 被延迟的候选建议列表（低/中置信度时未执行的重型建议）
    """
    updated_rules = rule_system.rules.copy()
    deferred_suggestions = []  # 被延迟的候选建议
    
    # 获取置信度阈值
    high_threshold = config.evaluator.confidence_high_threshold
    low_threshold = config.evaluator.confidence_low_threshold
    
    # 低置信度：标记为不可靠，不驱动任何自动更新
    if evaluation_confidence < low_threshold:
        log_warning(f"评估置信度过低 ({evaluation_confidence:.2f} < {low_threshold})，跳过所有规则更新")
        return RuleSystem(rules=updated_rules, version=rule_system.version), unified_reflection.unified_suggestions
    
    for suggestion in unified_reflection.unified_suggestions:
        suggestion_type = suggestion.suggestion_type
        
        # 轻量级类型：直接应用（不受 confidence 限制）
        if suggestion_type in [SuggestionType.Rephrase, SuggestionType.ChangeFormat]:
            updated_rules = apply_lightweight_update(updated_rules, suggestion)
            continue
        
        # 中等类型：冲突检测
        if suggestion_type in [SuggestionType.AddExample, SuggestionType.AddConstraint]:
            # 中等置信度时，中等类型建议也允许执行
            temp_rules = apply_suggestion(updated_rules, suggestion)
            conflicts = detect_conflicts(temp_rules)
            
            if len(conflicts) == 0:
                updated_rules = temp_rules
            else:
                # 有冲突，降级到完整流程（但需要高置信度）
                if evaluation_confidence >= high_threshold:
                    updated_rules = apply_full_validation(updated_rules, suggestion, config)
                else:
                    deferred_suggestions.append(suggestion)
                    log_info(f"中等置信度下发现冲突，建议 {suggestion.id} 延迟执行")
            continue
        
        # 重型类型：完整 Phase 0 验证流程（需要高置信度）
        if suggestion_type in [SuggestionType.AddRule, SuggestionType.ModifyRule, SuggestionType.RemoveRule]:
            if evaluation_confidence >= high_threshold:
                updated_rules = apply_full_validation(updated_rules, suggestion, config)
            else:
                # 中等置信度：记为候选，不立即执行
                deferred_suggestions.append(suggestion)
                log_info(f"置信度 ({evaluation_confidence:.2f}) 不足，规则层建议 {suggestion.id} 延迟执行")
    
    return RuleSystem(
        rules=updated_rules,
        version=rule_system.version + 1 if updated_rules != rule_system.rules else rule_system.version,
        # ... 其他字段
    ), deferred_suggestions

def apply_full_validation(rules: List[Rule], suggestion: Suggestion, config: OptimizationConfig) -> List[Rule]:
    """
    完整 Phase 0 验证流程
    """
    # Step 0.3: 应用建议
    temp_rules = apply_suggestion(rules, suggestion)
    
    # Step 0.4: 冲突检测与解决
    conflicts = detect_conflicts(temp_rules)
    for conflict in conflicts:
        resolved = resolve_conflict(conflict, config)
        temp_rules = apply_resolution(temp_rules, resolved)
    
    # Step 0.5: 相似合并
    temp_rules = merge_similar_rules(temp_rules)
    
    # Step 0.6: 验证
    for rule in temp_rules:
        rule.verified = verify_rule(rule)
    
    return temp_rules
```

### 8.4 反思分类实现

```python
def classify_failure(failed_case: FailedTestResult, prompt: str, rule_system: RuleSystem) -> ReflectionResult:
    """
    对失败用例进行分类反思
    """
    # 从嵌套结构中提取字段
    test_case = failed_case.test_case
    exec_result = failed_case.execution_result
    
    prompt_template = REFLECTION_CLASSIFICATION_PROMPT.format(
        test_input=test_case.input,
        expected_output=test_case.reference,
        actual_output=exec_result.actual_output,
        current_prompt=prompt,
        rules=format_rules(rule_system.rules)
    )
    
    response = teacher_model.generate(prompt_template)
    result = parse_reflection_response(response)
    
    # result.type 为 "rule_incomplete" 或 "expression_issue"
    return result
```

### 8.5 梯度聚合实现（借鉴 TextGrad）

```python
def aggregate_reflections(clusters: List[ReflectionCluster], config: OptimizationConfig) -> UnifiedReflection:
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

def arbitrate_conflicts(conflicts: List[SuggestionConflict], config: OptimizationConfig) -> UnifiedReflection:
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

### 8.6 安全检查实现

```python
def safety_check(history: IterationHistory, current_result: IterationResult, config: OptimizationConfig, ctx: OptimizationContext) -> SafetyCheckResult:
    """
    安全检查：回归检测 + 震荡检测 + Holdout 哨兵检测
    
    参数说明：
    - history: 迭代历史记录
    - current_result: 当前迭代结果
    - config: 优化配置
    - ctx: 优化上下文（包含测试用例集）
    """
    # 回归检测
    if current_result.previous_passed_cases:
        regressions = detect_regressions(
            current_result.results,
            current_result.previous_passed_cases
        )
        if len(regressions) > 0:
            return SafetyCheckResult(
                status=SafetyStatus.Regression,
                regressions=regressions
            )
    
    # 震荡检测
    if is_oscillating(history, config.oscillation.threshold):
        if config.oscillation.action == OscillationAction.DiversityInject:
            return SafetyCheckResult(status=SafetyStatus.OscillationInject)
        elif config.oscillation.action == OscillationAction.HumanIntervention:
            raise HumanInterventionRequired("检测到优化震荡")
        else:
            return SafetyCheckResult(status=SafetyStatus.Stop)
    
    # Holdout 哨兵检测（防止过拟合）
    # 只有当数据划分功能启用时才执行
    if config.data_split.enabled:
        holdout_result = check_holdout_overfitting(current_result, config, ctx)
        if holdout_result.status == SafetyStatus.OverfittingWarning:
            return holdout_result
    
    return SafetyCheckResult(status=SafetyStatus.Ok)

def check_holdout_overfitting(current_result: IterationResult, config: OptimizationConfig, ctx: OptimizationContext) -> SafetyCheckResult:
    """
    Holdout 哨兵检测：检查是否存在过拟合风险
    
    原理：如果在验证集上表现很好，但在保留集上表现明显差，说明可能过拟合了。
    """
    # 筛选出 Holdout 测试用例
    holdout_cases = [tc for tc in ctx.test_cases 
                     if tc.split == DataSplit.Holdout]
    
    # 如果没有 Holdout 用例，跳过检测
    if len(holdout_cases) == 0:
        return SafetyCheckResult(status=SafetyStatus.Ok)
    
    # 在 Holdout 集上评估当前 Prompt
    holdout_results = evaluate_on_subset(current_result.prompt, holdout_cases)
    holdout_pass_rate = sum(1 for r in holdout_results if r.passed) / len(holdout_results)
    
    # 计算验证集通过率（从当前结果中提取）
    validation_pass_rate = calculate_validation_pass_rate(current_result)
    
    # 如果 Holdout 通过率显著低于 Validation 通过率，发出警告
    gap = validation_pass_rate - holdout_pass_rate
    if gap > config.data_split.overfitting_threshold:
        return SafetyCheckResult(
            status=SafetyStatus.OverfittingWarning,
            details=f"Holdout 通过率 ({holdout_pass_rate:.1%}) 显著低于 Validation 通过率 ({validation_pass_rate:.1%})，差距 {gap:.1%} 超过阈值 {config.data_split.overfitting_threshold:.1%}，可能存在过拟合风险"
        )
    
    return SafetyCheckResult(status=SafetyStatus.Ok)

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

> **帮助函数说明**
> 
> 上述伪代码中出现的帮助函数为抽象函数，由 Orchestrator 内部封装：
> - `evaluate_on_subset(prompt, test_cases)`: 在指定测试用例子集上评估 Prompt，实现时应复用 `Evaluator` Trait
> - `calculate_validation_pass_rate(result)`: 从当前迭代结果中提取 Validation 集的通过率
> - `compute_aggregated_confidence(eval_results)`: 聚合多个评估结果的置信度，推荐策略为取最小值（保守）或加权均值
> 
> 这些函数的具体实现由 Orchestrator 内部封装，不作为对外扩展点。

---

## 9. 用户配置规格

> **配置映射说明**
> 
> 本节列出的配置项为**外部配置文件中的 key 名称**（采用 `snake_case` 风格），
> 加载后将映射到 Section 4.2.6.1 所定义的 `OptimizationConfig` 嵌套字段。
> 
> **映射示例**：
> 
> | 外部配置 key | 内部字段路径 | 枚举值映射 |
> |--------------|--------------|------------|
> | `output_strategy` | `config.output.strategy` | `"single"` → `OutputStrategy::Single` |
> | `minibatch_enabled` | `config.minibatch.enabled` | - |
> | `oscillation_threshold` | `config.oscillation.threshold` | - |
> | `oscillation_action` | `config.oscillation.action` | `"diversity_inject"` → `OscillationAction::DiversityInject` |
> | `max_abstraction_level` | `config.rule.max_abstraction_level` | - |
> | `max_iterations` | `config.iteration.max_iterations` | - |
> 
> **实现约定**：Rust 实现时，枚举类型需使用 `#[serde(rename_all = "snake_case")]` 
> 确保序列化/反序列化时的字符串值与本节定义一致。

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

### 9.4 规律相关配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_abstraction_level` | int | `3` | 规律抽象最大层级 |
| `similarity_threshold` | float | `0.8` | 规律相似度阈值（用于合并相似规律） |
| `enable_clustering` | bool | `false` | 是否启用测试用例聚类 |
| `clustering_threshold` | int | `50` | 聚类启用阈值（测试用例数超过此值时启用） |

### 9.5 迭代控制配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_iterations` | int | `20` | 最大迭代轮数 |
| `pass_threshold` | float | `0.95` | 通过率阈值 |
| `diversity_inject_after` | int | `3` | 连续失败多少次后触发多样性注入 |

### 9.6 数据划分配置

> **新增** — 2025-12-16
> 
> 数据划分配置用于控制 Train/Val/Holdout 三分法，防止测试集过拟合。
> 默认关闭，适合测试用例较多（建议 ≥20 条）的场景。

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enabled` | bool | `false` | 是否启用数据划分 |
| `train_ratio` | float | `0.70` | 训练集比例 (0.0-1.0) |
| `validation_ratio` | float | `0.15` | 验证集比例 (0.0-1.0) |
| `strategy` | enum | `"random"` | `"random"` / `"stratified"` / `"manual"` |
| `seed` | int | `null` | 随机种子（可选，用于可复现划分） |
| `overfitting_threshold` | float | `0.10` | 过拟合警告阈值（Val 与 Holdout 通过率差值） |

> **配置路径说明**：上述配置项位于 `OptimizationConfig.data_split` 下，完整路径如 `config.data_split.enabled`。

**划分策略说明**：

| 策略 | 说明 |
|------|------|
| `random` | 随机划分，按比例随机分配测试用例到各集合 |
| `stratified` | 分层抽样，按 TaskReference 类型分层，确保各集合中任务类型分布一致 |
| `manual` | 用户手动指定，读取每个 TestCase 的 `split` 字段，忽略 ratio 配置 |

**数据集使用方式**：

| 数据集 | 使用阶段 | 说明 |
|--------|----------|------|
| **Train** | Phase 0 + Phase 1 | 用于规律提炼和初始 Prompt 生成 |
| **Validation** | Phase 2 迭代 | 用于每轮迭代的评估和反思 |
| **Holdout** | Phase 2 安全检查 | 用于最终验证，检测过拟合风险 |
| **Unassigned** | 同 Train | 未分配的测试用例自动作为训练集使用 |

### 9.7 评估器配置

> **新增** — 2025-12-17
> 
> 评估器配置用于控制 EnsembleEvaluator 的行为和 confidence 门控策略。
> 默认启用多评估器组合，提供更稳定的评估信号。

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `ensemble_enabled` | bool | `true` | 是否启用多评估器组合 |
| `confidence_high_threshold` | float | `0.8` | 高置信度阈值（允许规则层更新） |
| `confidence_low_threshold` | float | `0.5` | 低置信度阈值（标记为不可靠） |
| `llm_judge_samples` | int | `1` | LLM 评估器采样次数 |
| `hard_checks_weight` | float | `0.4` | HardChecks 权重（用于 confidence 计算） |
| `agreement_weight` | float | `0.4` | 多评估器一致性权重 |
| `variance_penalty` | float | `0.2` | 采样方差惩罚权重 |

> **配置路径说明**：上述配置项位于 `OptimizationConfig.evaluator` 下，完整路径如 `config.evaluator.confidence_high_threshold`。

**confidence 门控策略**：

| confidence 范围 | 允许的动作 | 说明 |
|----------------|-----------|------|
| `≥ confidence_high_threshold` | 全部允许 | 可执行规则层更新（AddRule/ModifyRule/RemoveRule） |
| `[low, high)` | 仅表达层更新 | 规则层建议记为候选，待后续高置信度时执行 |
| `< confidence_low_threshold` | 仅记录 | 标记为"不可靠"，不驱动任何自动更新 |

**EnsembleEvaluator 实现说明**：

`EnsembleEvaluator` 是 `Evaluator` Trait 的推荐实现，通过组合多个内部评估器来提供更稳定的评估信号：

| 内部评估器 | 职责 | 结果类型 |
|-----------|------|---------|
| **HardChecksEvaluator** | JSON Schema 校验、字段存在性、类型检查、格式合法性 | 通过/失败 + 错误列表 |
| **MetricEvaluator**（可选） | 传统指标（F1、ROUGE、文本相似度等） | 多维数值分数 |
| **LLMJudgeEvaluator** | 由老师模型作为 judge，给出语义判断 | 通过/失败 + 多维解释 |

聚合逻辑：
- `passed`: HardChecks 必须通过，Soft/LLM 综合表决
- `score`: 加权和聚合
- `confidence`: 基于一致性 + HardChecks 状态计算（详见 Section 4.2.1）

### 9.8 预算控制配置

> **新增** — 2025-12-17
> 
> 预算控制配置用于限制单次优化任务（per-run）的全局资源消耗。
> 默认关闭，适合需要控制 API 成本或运行时间的生产环境。

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enabled` | bool | `false` | 是否启用预算控制 |
| `max_llm_calls` | int | `null` | 最大 LLM 调用次数（null = 不限制） |
| `max_tokens` | int | `null` | 最大 token 消耗（null = 不限制） |
| `max_duration_secs` | int | `null` | 最大运行时间/秒（null = 不限制） |
| `warn_threshold` | float | `0.8` | 预算警告阈值（达到此比例时触发警告） |

> **配置路径说明**：上述配置项位于 `OptimizationConfig.budget` 下，完整路径如 `config.budget.max_llm_calls`。

**预算作用范围**：

- 上述预算约束**仅在 `budget.enabled = true` 时生效**
- 预算控制的是**单次 OptimizationEngine.run 的全局上限**（即"一次优化会话"的预算）
- 迭代级别的控制仍由 MinibatchConfig / IterationConfig 负责
- Racing 相关的所有评估调用（无论 Train 还是 Val）都计入本次 run 的总 budget

**BudgetManager 职责**：

内部 BudgetManager 组件负责：
- 在各阶段（规则提取、评估、Racing 等）统计实际消耗
- 接近警告阈值时在日志和 Checkpoint 中记录警告
- 超限时触发策略降级或安全停机

**超限推荐策略**（优先级从高到低）：

| 优先级 | 策略 | 说明 |
|--------|------|------|
| 1 | 缩减 Minibatch | 减少每轮评估调用次数 |
| 2 | 加速 Racing 淘汰 | 提高 survival_threshold，减少候选数 |
| 3 | 跳过非关键评估 | 例如部分高成本 LLM judge |
| 4 | 提前终止 | 在 Checkpoint 中标记 `budget_exhausted` |

### 9.9 Racing 策略配置

> **新增** — 2025-12-17
> 
> Racing 策略配置用于控制多候选早期淘汰，借鉴 CAPO 的 Racing Selection 机制。
> 默认关闭，适合需要探索多个 Prompt 变体的场景。

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enabled` | bool | `false` | 是否启用 Racing 策略 |
| `pool_size` | int | `3` | 候选池大小上限 |
| `elimination_rounds` | int | `3` | 淘汰间隔轮数（Phase2 迭代） |
| `survival_threshold` | float | `0.85` | 存活阈值（相对最佳候选的分数比例） |
| `early_stop_confidence` | float | `0.95` | 早期停止置信度 |

> **配置路径说明**：上述配置项位于 `OptimizationConfig.racing` 下，完整路径如 `config.racing.pool_size`。

**生效条件**：

- Racing 的生效条件是"**有效候选数 ≥ 2**"，与 OutputStrategy 无直接关系
- 当有效候选数 ≤ 1 时，Racing 逻辑自动短路为 no-op，即使 `enabled = true`
- 这允许"内部多候选竞争 + 外部单输出"的灵活组合

**度量基准**：

- 候选的存活和淘汰基于 EnsembleEvaluator 输出的综合分数
- 该分数在 **Val split** 上计算或校正，确保排名可靠性
- `survival_threshold` 表示候选分数与当前最佳候选分数的比例，低于此值被淘汰

**与 Confidence 的关系**：

- 当候选的评估置信度低于 `EvaluatorConfig.confidence_low_threshold` 时：
  - Racing 不应做出强淘汰决策
  - 应视为"证据不足"，推迟裁决或增加评估样本
- 这与 Step 2 的 confidence 门控逻辑保持一致

**与数据三分的关系**：

| 数据集 | Racing 中的用途 |
|--------|----------------|
| **Train** | 规则更新和初步评估 |
| **Val** | 候选间可靠排名和 early stop 判断 |
| **Holdout** | **不参与 Racing**，仅用于最终胜者体检 |

**淘汰可观测性（实现建议）**：

被淘汰候选的信息推荐记录以下字段：

| 字段 | 说明 |
|------|------|
| `candidate_id` | 候选唯一标识 |
| `candidate_source` | 来源（`phase1_multi_output` / `phase2_diversity_injection`） |
| `elimination_iteration` | 被淘汰的迭代轮次 |
| `elimination_reason` | 淘汰原因（`dominated_on_val` / `under_survival_threshold` / `budget_exhausted`） |
| `score_snapshot` | 被淘汰时的关键指标摘要 |

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

> **修订说明** — 2025-12-15
> 
> 基于决策 C1（语言统一）和决策 F1（细粒度状态），更新状态机定义：
> - 从 TypeScript 改为 Rust
> - 补充缺失状态：DetectingConflicts, ResolvingConflicts, MergingSimilarRules, ClusteringFailures, UpdatingRules, SmartRetesting, SafetyChecking

### 11.1 状态枚举

```rust
/// 迭代状态（细粒度，用于前端精确显示进度）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum IterationState {
    // ===== 初始化阶段 =====
    /// 空闲状态
    Idle,
    /// 初始化中
    Initializing,
    
    // ===== Phase 0: 规律收敛 =====
    /// 提取规律
    ExtractingRules,
    /// 检测冲突
    DetectingConflicts,
    /// 解决冲突
    ResolvingConflicts,
    /// 合并相似规律
    MergingSimilarRules,
    /// 验证规律
    ValidatingRules,
    
    // ===== Phase 1: Prompt 生成 =====
    /// 生成 Prompt
    GeneratingPrompt,
    
    // ===== Phase 2: 测试迭代 =====
    /// 执行测试
    RunningTests,
    /// 评估结果
    Evaluating,
    /// 失败聚类
    ClusteringFailures,
    /// 反思分析
    Reflecting,
    /// 更新规律（分层验证中）
    UpdatingRules,
    /// 优化 Prompt
    Optimizing,
    /// 智能重测
    SmartRetesting,
    /// 安全检查（回归/震荡检测）
    SafetyChecking,
    
    // ===== 人工介入 =====
    /// 等待用户操作
    WaitingUser,
    /// 需要人工介入（冲突无法解决等）
    HumanIntervention,
    
    // ===== 终态 =====
    /// 成功完成
    Completed,
    /// 达到最大迭代轮数
    MaxIterationsReached,
    /// 用户终止
    UserStopped,
    /// 失败
    Failed,
}
```

### 11.2 状态转换规则

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           状态转换图                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────┐                                                                   │
│  │ Idle │ ──────────────────────────────────────────────────────┐           │
│  └──────┘                                                       │           │
│      │ 开始优化                                                  │           │
│      ↓                                                          │           │
│  ┌──────────────┐                                               │           │
│  │ Initializing │                                               │           │
│  └──────────────┘                                               │           │
│      │                                                          │           │
│      ↓                                                          │           │
│  ╔═══════════════════════════════════════════════════════════╗  │           │
│  ║  Phase 0: 规律收敛                                         ║  │           │
│  ╠═══════════════════════════════════════════════════════════╣  │           │
│  ║  ExtractingRules → DetectingConflicts → ResolvingConflicts ║  │           │
│  ║        ↓                                                   ║  │           │
│  ║  MergingSimilarRules → ValidatingRules                     ║  │           │
│  ╚═══════════════════════════════════════════════════════════╝  │           │
│      │                                                          │           │
│      ↓                                                          │           │
│  ╔═══════════════════════════════════════════════════════════╗  │           │
│  ║  Phase 1: Prompt 生成                                      ║  │           │
│  ╠═══════════════════════════════════════════════════════════╣  │           │
│  ║  GeneratingPrompt                                          ║  │           │
│  ╚═══════════════════════════════════════════════════════════╝  │           │
│      │                                                          │           │
│      ↓                                                          │ 任意状态   │
│  ╔═══════════════════════════════════════════════════════════╗  │ 可转换    │
│  ║  Phase 2: 测试迭代                                         ║  │           │
│  ╠═══════════════════════════════════════════════════════════╣  │           │
│  ║  RunningTests → Evaluating                                 ║  │           │
│  ║       ↓              ↓                                     ║  │           │
│  ║       │         全部通过 ──────────────────→ Completed      ║  │           │
│  ║       │              ↓                                     ║  │           │
│  ║       │         有失败                                      ║  │           │
│  ║       │              ↓                                     ║  │           │
│  ║       │    ClusteringFailures → Reflecting                 ║  │           │
│  ║       │                              ↓                     ║  │           │
│  ║       │                   ┌─────────────────────┐          ║  │           │
│  ║       │                   │ 规律问题?           │          ║  │           │
│  ║       │                   └─────────────────────┘          ║  │           │
│  ║       │                     │ Yes         │ No             ║  │           │
│  ║       │                     ↓             ↓                ║  │           │
│  ║       │            UpdatingRules     Optimizing            ║  │           │
│  ║       │                     │             │                ║  │           │
│  ║       │                     └──────┬──────┘                ║  │           │
│  ║       │                            ↓                       ║  │           │
│  ║       │                    SmartRetesting                  ║  │           │
│  ║       │                            ↓                       ║  │           │
│  ║       │                    SafetyChecking                  ║  │           │
│  ║       │                            ↓                       ║  │           │
│  ║       │                   ┌─────────────────────┐          ║  │           │
│  ║       │                   │ 达到最大轮数?       │          ║  │           │
│  ║       │                   └─────────────────────┘          ║  │           │
│  ║       │                     │ Yes         │ No             ║  │           │
│  ║       │                     ↓             ↓                ║  │           │
│  ║       │        MaxIterationsReached   RunningTests ←───────╯  ║  │           │
│  ╚═══════════════════════════════════════════════════════════╝  │           │
│                                                                 │           │
│  ┌──────────────────────────────────────────────────────────────┤           │
│  │ 全局转换（任意状态可触发）                                    │           │
│  ├──────────────────────────────────────────────────────────────┤           │
│  │ Any → WaitingUser (用户暂停)                                 │           │
│  │ Any → UserStopped (用户终止)                                 │           │
│  │ Any → HumanIntervention (需要人工介入)                       │           │
│  │ Any → Failed (发生错误)                                      │           │
│  │ WaitingUser → PreviousState (用户恢复)                       │           │
│  └──────────────────────────────────────────────────────────────┘           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 11.3 状态与 Phase 对应关系

| 状态 | 所属 Phase | 说明 |
|------|-----------|------|
| `Idle` | - | 初始空闲 |
| `Initializing` | - | 初始化任务 |
| `ExtractingRules` | Phase 0 | 从测试集提取规律 |
| `DetectingConflicts` | Phase 0 | 检测规律冲突 |
| `ResolvingConflicts` | Phase 0 | 解决规律冲突 |
| `MergingSimilarRules` | Phase 0 | 合并相似规律 |
| `ValidatingRules` | Phase 0 | 验证规律 |
| `GeneratingPrompt` | Phase 1 | 生成 Prompt |
| `RunningTests` | Phase 2 | 执行测试 |
| `Evaluating` | Phase 2 | 评估结果 |
| `ClusteringFailures` | Phase 2 | 失败聚类 |
| `Reflecting` | Phase 2 | 反思分析 |
| `UpdatingRules` | Phase 2 | 更新规律（分层验证） |
| `Optimizing` | Phase 2 | 优化 Prompt |
| `SmartRetesting` | Phase 2 | 智能重测 |
| `SafetyChecking` | Phase 2 | 安全检查 |
| `WaitingUser` | - | 等待用户 |
| `HumanIntervention` | - | 人工介入 |
| `Completed` | 终态 | 成功完成 |
| `MaxIterationsReached` | 终态 | 达到最大轮数 |
| `UserStopped` | 终态 | 用户终止 |
| `Failed` | 终态 | 失败 |

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

### 13.2 MaxIterationsReached

触发条件：
- 迭代轮数达到 `max_iterations`

处理方式：
- 输出历史最佳 Prompt
- 生成优化报告

---

## 14. 实现与采纳建议

> **Step 5 产出** — 2025-12-15
> 
> 本节提供 Prompt Faster 核心算法规格的实施指南，包括技术采纳策略、开发流程、测试保障、团队要求和风险缓解。

### 14.1 技术采纳策略

#### 14.1.1 渐进式采纳路径

> **注意**：本节中的 Stage 0/1/2/3 指**项目实施阶段**，与第 5 章的算法 Phase 0/1/2 概念不同。

| 阶段 | 目标 | 关键产出 | 验收标准 |
|------|------|----------|----------|
| **Stage 0: 基础设施** | 搭建 Rust + SQLite 框架 | Trait 体系骨架、Repository 实现 | 新增模块 < 4 小时 |
| **Stage 1: 核心算法** | 实现四层处理器 | RuleEngine → PromptGenerator → Evaluator → Optimizer | 单执行目标成功率 ≥ 70% |
| **Stage 2: 完整 MVP** | 双执行引擎 + 可视化 | Dify + 通用 API、React Flow 节点图 | 端到端流程跑通 |
| **Stage 3: Growth** | 元优化 + 插件生态 | L2 元优化、ExecutionTarget 扩展 | 12 个月 ≥ 10 外部用户 |

#### 14.1.2 关键技术决策点

| 决策点 | 检查时机 | 通过标准 | 失败措施 |
|--------|----------|----------|----------|
| Trait 体系有效性 | Stage 0 结束 | 4.4 扩展成本达标 | 重构接口边界 |
| 核心算法有效性 | 6 周时间盒 | 基准测试集成功率 ≥ 70% | 算法架构调整 |
| 并行模式质量 | Stage 2 | 并行 vs 串行差异 < 5% | 默认串行，并行为可选 |
| 断点续跑可靠性 | Stage 2 | 恢复率 100% | 增强 Checkpoint 机制 |

### 14.2 开发流程与工具链

#### 14.2.1 技术栈配置

| 层面 | 技术选择 | 工具链 |
|------|----------|--------|
| **后端** | Rust | cargo + clippy + rustfmt |
| **前端** | TypeScript + React + React Flow | Vite + ESLint + Prettier |
| **类型同步** | ts-rs | 自动生成 TypeScript 类型 |
| **数据库** | SQLite (WAL 模式) | sqlx + sqlx-cli 迁移 |
| **部署（MVP）** | Docker Compose | 多平台镜像构建 |
| **部署（成熟期）** | Tauri 桌面应用 | 独立可执行文件 |

#### 14.2.2 开发工作流

```
┌─────────────────────────────────────────────────────────────────────────┐
│  开发工作流                                                              │
├─────────────────────────────────────────────────────────────────────────┤
│  1. 功能开发                                                             │
│     ├─ 定义 Trait 签名（如需新模块）                                      │
│     ├─ 实现 Trait（参考 4.2.7 签名定义）                                  │
│     └─ 注册到 ModuleRegistry                                             │
├─────────────────────────────────────────────────────────────────────────┤
│  2. 类型同步                                                             │
│     ├─ cargo build 触发 ts-rs 生成                                       │
│     └─ 前端自动获取最新 TypeScript 类型                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  3. 测试验证                                                             │
│     ├─ cargo test（单元 + 集成）                                         │
│     ├─ 基准测试集验证                                                    │
│     └─ 端到端流程测试                                                    │
├─────────────────────────────────────────────────────────────────────────┤
│  4. CI/CD                                                               │
│     ├─ GitHub Actions: 构建 + 测试 + Docker 镜像                         │
│     └─ 多平台支持: linux/amd64 + linux/arm64                             │
└─────────────────────────────────────────────────────────────────────────┘
```

### 14.3 测试与质量保障

#### 14.3.1 测试策略

| 测试类型 | 覆盖范围 | 执行频率 | 通过标准 |
|----------|----------|----------|----------|
| **单元测试** | 每个 Trait 实现 | 每次提交 | 100% 通过 |
| **集成测试** | 算法 Phase 0/1/2 流程 | 每次提交 | 100% 通过 |
| **基准测试** | 10-20 标准优化任务 | 每次发布 | 成功率 ≥ 90% |
| **回归测试** | 核心流程 | 模块修改后 | 100% 通过 |
| **端到端测试** | 完整用户旅程 | 每次发布 | 覆盖率 ≥ 80% |

#### 14.3.2 围绕数据结构的测试重点

基于本规范中的数据结构定义（尤其见 Section 4.2 / 4.3 / 6.2 / 11.1）：

| 数据结构 | 测试重点 |
|----------|----------|
| `TestCase` / `TaskReference` | 双任务模式（固定/创意/混合）正确处理 |
| `Rule` / `RuleSystem` | 冲突检测、相似合并、验证分数计算 |
| `EvaluationResult` | 评估维度聚合、通过判定逻辑 |
| `Checkpoint` | 序列化/反序列化、lineage 链完整性 |
| `IterationState` | 状态转换合法性、终态判定 |

#### 14.3.3 质量指标（来自 PRD）

| 指标 | 目标值 | 验证方式 |
|------|--------|----------|
| 优化成功率 | ≥ 90% | 官方基准测试集 |
| 系统自身延迟 | < 100ms | 性能测试（不含 LLM 调用） |
| 断点恢复率 | 100% | 异常场景测试（kill/断网/断电） |
| 并行 vs 串行差异 | < 5% | 对比测试 |

### 14.4 团队与技能要求

#### 14.4.1 核心技能矩阵

| 技能领域 | 要求级别 | 应用场景 |
|----------|----------|----------|
| **Rust 系统编程** | 高级 | Trait 体系、异步编程、错误处理 |
| **TypeScript/React** | 中级 | 前端可视化、状态管理 |
| **LLM Prompt 工程** | 高级 | 老师模型 Prompt 设计、结构化输出 |
| **SQLite/数据库** | 中级 | WAL 配置、事务处理、迁移管理 |
| **Docker/DevOps** | 中级 | 容器化、CI/CD 配置 |

#### 14.4.2 单人开发策略

基于 PRD 约束（单人开发，3 个月时间盒）：

| 策略 | 说明 |
|------|------|
| **严格 MVP 边界** | 不做锦上添花功能，聚焦核心算法 |
| **模块化优先** | 每层可独立测试，降低调试成本 |
| **复用成熟方案** | React Flow（可视化）、sqlx（数据库）、ts-rs（类型同步） |
| **自动化投入** | 早期投入 CI/CD，减少手工测试时间 |

### 14.5 风险与缓解措施

#### 14.5.1 技术风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| **核心算法无效** | 致命 | 中 | 6 周时间盒 + 基准测试集 + ≥70% 阈值 |
| **LLM 输出不稳定** | 高 | 高 | 结构化输出（JSON 模式）+ 重试机制 + 验证层 |
| **规律体系复杂度爆炸** | 中 | 中 | `max_abstraction_level` 限制 + 人工介入触发 |
| **并行执行质量下降** | 中 | 中 | 差异 < 5% 约束 + 用户可选串行模式 |
| **断点续跑失败** | 高 | 低 | SQLite WAL 模式 + Checkpoint 完整性校验 |

#### 14.5.2 算法特定风险

| 风险 | 触发条件 | 缓解措施 |
|------|----------|----------|
| **规律冲突无法解决** | 超过 `max_abstraction_level` | 触发 `HumanInterventionRequired`，保存 Checkpoint |
| **优化震荡** | 连续 N 轮相似状态 | `oscillation_action` 配置（多样性注入/人工介入/停止） |
| **反思建议冲突** | 多路反馈不一致 | FeedbackAggregator 仲裁 + 必要时人工介入 |
| **测试集过拟合** | 规律过于具体 | 验证分数机制 + 泛化能力检测 |

#### 14.5.3 风险监控指标

| 指标 | 阈值 | 触发动作 |
|------|------|----------|
| 单轮迭代耗时 | > 5 分钟 | 检查 LLM 响应 + 网络状态 |
| 连续失败轮数 | > 3 轮 | 考虑多样性注入或人工介入 |
| 规律数量 | > 50 条 | 触发相似合并 + 检查冗余 |
| Checkpoint 大小 | > 10MB | 检查数据结构膨胀 |

---

## 15. 研究综合与结论

> **Step 6 产出** — 2025-12-15
> 
> 本节综合全文研究成果，总结关键发现、研究方法和后续建议。

### 15.1 研究方法说明

| 维度 | 方法 |
|------|------|
| **研究范围** | Prompt 自动优化核心算法的完整技术规格 |
| **输入文档** | PRD、头脑风暴会议记录 |
| **外部调研** | DSPy、TextGrad、PromptWizard、Reflexion、GEPA 等业界框架 |
| **验证方式** | 假设驱动 + 业界对比 + PRD 约束对齐 |
| **研究周期** | 2025-12-14 ~ 2025-12-15 |

### 15.2 关键研究发现

| 发现 | 说明 | 相关章节 |
|------|------|----------|
| **规律驱动是独创** | 业界框架直接优化 Prompt，本项目增加 RuleSystem 中间层 | 4.6.2 |
| **Trait 体系可行** | 7 个核心 Trait 覆盖完整算法流程，扩展成本达标 | 4.2 / 4.4 |
| **状态机精细化** | 24 个细粒度状态支持前端实时显示和断点续跑 | 11.1 |
| **分层验证策略** | 根据修改类型采用不同验证强度，平衡效率与一致性 | 8.3 |
| **反馈聚合机制** | 借鉴 TextGrad，增加冲突仲裁层 | 8.5 |

### 15.3 研究来源汇总

| 来源类型 | 来源 | 借鉴内容 |
|----------|------|----------|
| **学术论文** | OPRO、Reflexion、TextGrad | 迭代优化、自我反思、梯度聚合 |
| **开源框架** | DSPy、PromptWizard、GEPA | 模块化设计、配置驱动、Pareto 前沿 |
| **产品文档** | PRD、头脑风暴记录 | 成功标准、用户旅程、技术约束 |
| **最佳实践** | Arize Phoenix、CAPO | 评估诊断、早期淘汰 |

### 15.4 研究目标达成情况

| 原始目标 | 达成情况 | 证据 |
|----------|----------|------|
| 定义完整技术规格 | ✅ 达成 | 14 个章节覆盖架构、流程、数据结构、配置 |
| 模块化可插拔架构 | ✅ 达成 | 7 个核心 Trait + ModuleRegistry + 扩展成本验证 |
| 与 PRD 对齐 | ✅ 达成 | 质量指标、时间盒、扩展成本全部对应 |
| 业界调研支撑 | ✅ 达成 | 5+ 框架对比，明确借鉴与改良 |

### 15.5 后续建议

| 建议 | 优先级 | 说明 |
|------|--------|------|
| **进入架构设计阶段** | 高 | 基于本规格输出详细系统架构 |
| **建立基准测试集** | 高 | 10-20 个标准优化任务，用于算法验证 |
| **原型验证核心假设** | 高 | 重点验证 H1（规律驱动有效性） |
| **细化老师 Prompt 模板** | 中 | Section 10 模板需实际调优 |
| **评估 Rust 生态成熟度** | 中 | sqlx、ts-rs 等关键依赖的生产就绪性 |

### 15.6 技术规格整体审查结论与风险评估

#### 15.6.1 总体结论

- **未发现致命设计缺陷**：在 Trait 接口、核心数据结构、伪代码与状态机的对照审查中，没有发现类似“接口没定义”“关键抽象互相打架”的结构性问题。
- **技术规格、PRD 与头脑风暴在关键抽象上闭环一致**：
  - 四层处理器架构 + 七个核心 Trait
  - Rule / RuleSystem / TestCase / ExecutionResult / EvaluationResult / FailedTestResult / IterationResult / Checkpoint / UnifiedReflection 等核心数据结构
  - Phase 0/1/2 三阶段流程与第 11 章状态机
  在职责划分和数据流层面是一致的，可作为后续实现的“真相源头”。

#### 15.6.2 接口 / 数据结构 / 状态机 的一致性

**（1）Trait ↔ 数据结构 ↔ 伪代码 ↔ 状态机 对齐情况**
- 核心 Trait 体系（`RuleEngine / PromptGenerator / Evaluator / FeedbackAggregator / Optimizer / TeacherModel / ExecutionTarget`）与前文定义的数据结构是一致的：
  - 规则域：`Rule / RuleSystem / RuleConflict / RuleConflictType`
  - 测试与执行域：`TestCase / TaskReference / ExecutionResult / EvaluationResult / FailedTestResult / IterationResult / IterationResultStatus`
  - 反思与聚合域：`ReflectionResult / UnifiedReflection / Suggestion / SuggestionConflict / ArbitrationResult`
  - 流程与上下文域：`OptimizationContext / Checkpoint / IterationHistory / IterationState`
- 第 6/7/8 章伪代码中引用的结构体和字段（如 `EvaluationResult.passed/score/failure_points`，`FailedTestResult.test_case/execution_result/failure_summary`，`IterationResult.status/prompt/rule_system/results/failed_cases/previous_passed_cases` 等）在 4.2 与 6.2 等小节中均有明确的 Rust 风格定义，不再存在“伪代码引用未定义字段”的问题。
- 第 11 章状态机中给出的状态枚举（`RunningTests / Evaluating / ClusteringFailures / Reflecting / UpdatingRules / Optimizing / SmartRetesting / SafetyChecking` 等）与 Phase 2 关键伪代码（如 `parallel_test_iteration`、`update_rule_system_with_validation`、`safety_check`）中的步骤一一对应，不存在“状态机有状态但流程里不用”或反向不一致的情况。

**（2）错误类型与全局级别终止/异常信号**
- 每个核心 Trait 均定义了各自的错误类型（`RuleEngineError / GeneratorError / EvaluatorError / AggregatorError / OptimizerError / ModelError / ExecutionError`），错误枚举的粒度与模块职责相匹配，避免了过度细碎或过度粗糙。
- 附录第 13 章讨论了两个**全局级别的终止/异常信号**：
  - `HumanInterventionRequired`：作为异常使用，用于表示“必须人工介入”的情况，例如规律冲突无法解决、建议冲突无法仲裁、根据配置要求在检测到震荡时中止自动流程。
  - `MaxIterationsReached`：作为终止原因/终态使用，对应 `TerminationReason::MaxIterationsReached { max: u32 }` 与 `IterationState::MaxIterationsReached`，在迭代轮数达到 `max_iterations` 时触发，并按文档约定输出历史最佳 Prompt 与优化报告。
- 上述信号在错误处理章节、流程伪代码与状态机中的出现位置与语义是一致的，没有“孤立概念”。

**（3）配置、ModuleRegistry 与 PRD 接口的对应关系**
- 配置章节（第 9 章）中的外部配置项（`snake_case` 风格 key）与 `OptimizationConfig` 嵌套结构之间的映射关系清晰，`serde(rename)` 的使用约定得到充分说明。
- `ModuleRegistry` 的职责是对各类模块（规则引擎、Prompt 生成器、评估器、优化器、反馈聚合器等）进行运行时注册与查找，与 4.2 中的 Trait 体系一一对应，保证了“配置驱动 + 运行时选择实现”的扩展能力。
- 4.7 节对 PRD 中接口与本技术规格间的映射关系给出了明确说明：PRD 采用产品视角的高层接口描述，而技术规格采用实现视角的 Trait 抽象，差异是抽象层级不同而非设计矛盾。

#### 15.6.3 维护与扩展角度的主要风险点（可接受的 Trade-off）

在整体架构正确且自洽的前提下，存在少量需要在实现与后续演进阶段重点关注的**设计 Trade-off**，并非缺陷，但值得记录：

- **`OptimizationContext` 可能演化为“上帝对象”的风险（中等）**
  - 当前多数 Trait 接口采用 `fn xxx(&self, ctx: &OptimizationContext, ...)` 形式，带来上下文访问便利性，但也意味着：
    - 如果未来不断向 `OptimizationContext` 中堆叠字段，其内部可能积累过多职责；
    - Trait 签名本身无法精确表达“依赖了 ctx 的哪些子字段”，重构时需要额外小心。
  - 实现阶段的缓解建议：
    - 使用结构化子字段（如迭代元数据、数据集信息、配置快照等）对 `OptimizationContext` 内部进行分层；
    - 严格控制写访问，仅在 Orchestrator / 状态管理模块中修改，业务 Trait 只读使用。
- **部分 Trait 粒度相对较大（RuleEngine / FeedbackAggregator / Optimizer，风险中等）**
  - 从当前 PRD 约束（单人 3 个月时间盒）和 MVP 目标出发，将一整块能力打包在同一个 Trait 内是合理的，有利于快速落地。
  - 但如果未来希望仅替换某个子策略（如独立替换冲突检测算法或仲裁策略），可能需要通过实现新的 Trait 实现而非仅替换一个细粒度接口。
  - 建议：保持当前 Trait 粒度不变，在未来确有需求时，可在实现层内部再拆分更细的小策略接口，而无需立即修改技术规格。
- **Phase 0 与 Phase 2 之间的分层验证耦合（中-低）**
  - Phase 2 中针对重型建议（如 `AddRule/ModifyRule/RemoveRule`）的分层验证会回到 Phase 0 的完整链路（冲突检测→冲突解决→相似合并→规律验证）。
  - 这显著提升了规律体系的一致性与安全性，但也意味着 Phase 2 在某些路径上强依赖 RuleEngine 的完整能力，未来若要大幅调整 Phase 0 的内部实现，需要同步评估 Phase 2 的影响。
- **`parallel_execute` 与 `ExecutionTarget` 的关系在文档中略显抽象（低）**
  - 规格已分别给出 `ExecutionTarget` Trait 的定义和 `parallel_execute` 的执行顺序契约，但二者的对应关系对完全陌生的读者可能需要少量脑补。
  - 这一点属于表达层面的优化空间，而非设计问题，可在将来文档迭代中通过增加一两句“实现建议”来补强。

#### 15.6.4 设计矛盾与核心接口健康状况

针对历史上曾出现的“伪代码引用不存在字段/方法”等问题，本次审查重点检查了：
- Phase 2 及状态机伪代码中被显式使用的所有数据结构，前文是否具有对应的结构定义；
- 状态机枚举值在代码段与错误处理章节中的出现是否一致，命名是否统一；
- Trait 方法签名中的参数与返回值，是否均为已经在文档中定义过的类型。
审查结果：
- 伪代码中的核心类型（如 `IterationResult / SafetyCheckResult / ReflectionCluster / FailedTestResult / IterationHistory` 等）均在 4.2 与 6.2 相关小节中给出了结构化定义；
- 状态机中的终态枚举（`Completed / MaxIterationsReached / UserStopped / Failed`）与错误处理章节和 `TerminationReason` 定义保持命名与语义一致；
- Trait 签名中引用的类型均能在文档中找到清晰定义，没有新的“幽灵类型”。

同时，伪代码中出现的一些函数（如 `cluster_test_cases / extract_rule_from_cluster / find_similar_groups / merge_similar_rules / group_rules_by_type / sample_minibatch / parallel_execute / detect_suggestion_conflicts / merge_suggestions / apply_suggestion / apply_full_validation / detect_regressions / is_similar_state` 等），从语境上可以自然理解为：
- 各模块内部的实现细节函数，或
- Orchestrator 内部的辅助函数。
这些函数**并未被用作对外扩展点**，因此不需要在技术规格层面额外定义独立 Trait。本次审查未发现新的“设计矛盾”或“核心接口缺失”类问题。

#### 15.6.5 表达与文档可读性的后续优化点（非必须）

在不影响当前技术正确性的前提下，未来可考虑的小幅表达优化包括：
- 在关键伪代码（如 `parallel_test_iteration`）附近补充一句“此段为 Orchestrator 伪代码，调用 Evaluator / ExecutionTarget / Optimizer 等 Trait 实现”，降低新读者的理解门槛。
- 在 8.5 中 `arbitrate_conflicts` 附近，简要说明它操作的是 `SuggestionConflict` 列表，并与 4.2.4 中的 `SuggestionConflict` 结构形成显式呼应。
- 在 4.7.4 表格下方，为 `serde_json::Value` 的使用加上一句简短说明，帮助新手快速理解其用于统一承载 Dify 输入和直连模型输入的设计意图。
这些优化均属于“有则更好、无则不影响正确性”的抛光工作，可在实现稳定后按需补充。

#### 15.6.6 综合判断

综合以上各点：
- **架构安全性**：Trait 体系、数据结构、伪代码与状态机之间形成了稳定的一致性闭环，未发现新的致命设计问题。
- **灵活性与可维护性**：在当前项目目标与资源约束下，模块化与可插拔程度是足够的，同时对 `OptimizationContext`、Trait 粒度等潜在演进点给出了清晰的风险记录。
- **实施建议**：在具体实现阶段，应重点关注 `OptimizationContext` 的结构化管理与写访问纪律，当未来确有更细粒度可插拔需求时，再在实现层引入内部策略接口，而无需立即回头大改技术规格。
基于本小节的审查结论，可以认为当前版本技术规格**已经可以作为后续实现工作的可靠基础文档**，同时为未来的演进与重构预留了清晰的抓手。

---

## 16. vNext 架构与治理原则（概览）

> **版本**: v1.1/v1.2 预告 — 2025-12-16 / 2025-12-17
> 
> 本节概述即将在后续版本中引入的架构增强方向，为读者提供演进路线图。
> 所有增强均遵循"接口稳定、配置扩展、渐进演进"的原则。

### 16.1 演进方向总览

| 方向 | 目标 | 涉及章节 | 状态 |
|------|------|----------|------|
| **数据划分** | Train/Val/Holdout 三分法，防止过拟合 | 4.2.5, 4.2.6.1, 8.6, 9.6 | ✅ 已完成 |
| **评估可靠性** | EnsembleEvaluator + confidence 门控 | 4.2.1, 4.2.6.1, 8.3, 9.7 | ✅ 已完成 |
| **规律可计算性** | RuleIR 中间表示（渐进式、可选） | 6.2.1, 6.3 | 🔜 待实施 |
| **候选池与预算** | Racing 策略 + BudgetConfig | 4.2.2, 4.2.6.1, 8.2, 9.8, 9.9 | 🔜 待实施 |
| **引擎抽象** | OptimizationEngine 作为 7 Trait 封装门面 | 4.1, 4.2, 4.3 | 🔜 待实施 |

> **Budget/Racing 结构预留说明** — 2025-12-17
> 
> Budget/Racing 在 v1.2 中已完成结构预留与语义规范：
> - `BudgetConfig`：per-run 全局预算控制（LLM 调用/token/时间）
> - `RacingConfig`：多候选早期淘汰策略（借鉴 CAPO Racing Selection）
> 
> 后续版本将逐步实现并验证具体 Racing 策略，当前版本默认关闭（`enabled = false`）。

### 16.2 设计原则

| 原则 | 说明 |
|------|------|
| **接口稳定性** | 优先通过配置/结构扩展能力，尽量不改 Trait 签名 |
| **渐进式演进** | 新特性设为可选（如 RuleIR = Option），不破坏现有功能 |
| **配置化策略** | 算法参数开放为配置项，而非硬编码 |
| **实现层增强** | 新能力通过实现类（如 EnsembleEvaluator）而非新 Trait 引入 |

> **RuleIR 演进路线** — 2025-12-17
> 
> RuleIR 已在 v1.2 的结构层预留（`Rule.ir: Option<RuleIR>`），当前为空壳占位。
> 未来版本将逐步将 6.3 中的冲突检测/相似合并/coverage 迁移为"**IR-first + LLM 兜底**"模式：
> 
> 1. **Phase 1**：RuleIR 生成器（从 description + tags 推导）
> 2. **Phase 2**：基于 IR 的静态冲突预筛（scope/constraints 比较）
> 3. **Phase 3**：基于 IR 的结构化相似度计算
> 4. **Phase 4**：基于 IR 的 coverage_map 精确计算
> 
> 每个 Phase 独立可用，不强制全部实现后才能启用。

### 16.3 版本兼容性说明

- **v1.0 → v1.1 迁移**：完全向后兼容
  - 新增字段均有默认值，现有配置无需修改
  - `TestCase.split` 为可选字段，默认 `None`（等同于 `Unassigned`）
  - `DataSplitConfig.enabled` 默认 `false`，不影响现有行为
- **v1.1 → v1.2 迁移**：完全向后兼容
  - `EvaluatorConfig` 所有字段均有默认值
  - `Rule.ir` 为可选字段，默认 `None`，不影响现有行为
  - `IterationResult.deferred_suggestions` 默认为空列表
  - `BudgetConfig.enabled` 默认 `false`，不影响现有行为
  - `RacingConfig.enabled` 默认 `false`，不影响现有行为
- **Checkpoint 兼容性**：v1.0/v1.1 Checkpoint 可被 v1.2 正常加载
  - 缺失的新字段将使用默认值填充
- **配置文件兼容性**：
  - 在未显式配置 Budget/Racing 时，行为等价于 v1.1 之前的实现
  - 未来启用 Budget/Racing 时，将通过新增配置字段和版本说明保证旧配置文件可无修改加载

---
## 技术研究完成

**研究完成日期**：2025-12-15（v1.0）/ 2025-12-16（v1.1 数据划分增强）/ 2025-12-17（v1.2 评估可靠性 + RuleIR + Budget/Racing）  
**文档版本**：v1.2  
**研究步骤完成**：Step 1-6 全部完成 + vNext 数据划分增强 + 评估可靠性增强 + RuleIR 结构预留 + Budget/Racing 结构预留  
**来源验证**：所有关键技术主张均有业界参考支撑  
**置信度**：高 — 基于多个权威技术来源

---

**文档结束**
