---
stepsCompleted: [1]
inputDocuments:
  - docs/analysis/brainstorming-session-2025-12-12.md
workflowType: 'research'
lastStep: 1
research_type: 'technical'
research_topic: 'algorithm-specification'
research_goals: '定义 Prompt Faster 核心迭代算法的完整技术规格，供架构师和开发者实现参考'
user_name: '耶稣'
date: '2025-12-14'
web_research_enabled: true
source_verification: true
---

# Research Report: Algorithm Specification

**Date:** 2025-12-14  
**Author:** 耶稣  
**Research Type:** Technical Specification

---

## Executive Summary

本文档定义 Prompt Faster 核心迭代算法的完整技术规格，供架构师和开发者实现参考。

**Key Technical Findings:**
- 三阶段迭代流程：规律收敛 → Prompt生成 → 测试迭代
- 四层处理器架构：Pattern Extractor → Prompt Engineer → Quality Assessor → Reflection Agent
- 完整数据结构定义：Rule、RuleSystem、Checkpoint、IterationState

**Technical Recommendations:**
- 按本规格实现核心算法
- 开放用户配置项以支持灵活定制
- 实现完整的安全检查机制（回归检测、震荡检测）

---

## Table of Contents

1. 文档概述
2. 算法总体架构
3. Phase 0: 规律收敛阶段
4. Phase 1: 首次 Prompt 生成
5. Phase 2: 测试与反思迭代
6. 用户配置规格
7. 老师 Prompt 模板规格
8. 状态机定义
9. 最佳实践来源
10. 附录：错误处理

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

---

## 2. 算法总体架构

### 2.1 三阶段流程

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

### 2.2 四层处理器

| 层级 | 名称 | 职责 |
|------|------|------|
| Layer 1 | Pattern Extractor | 从测试集提炼规律 |
| Layer 2 | Prompt Engineer | 基于规律生成 Prompt |
| Layer 3 | Quality Assessor | 评估 Prompt 输出质量 |
| Layer 4 | Reflection Agent | 分析失败原因，推荐策略 |

---

## 3. Phase 0: 规律收敛阶段

### 3.1 流程定义

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

### 3.2 数据结构定义

#### 3.2.1 Rule 结构

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

#### 3.2.2 RuleSystem 结构

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

### 3.3 算法实现

#### 3.3.1 规律提炼算法

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

#### 3.3.2 冲突检测算法

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

#### 3.3.3 冲突解决算法

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

#### 3.3.4 相似合并算法

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

## 4. Phase 1: 首次 Prompt 生成

### 4.1 流程定义

```
输入: RuleSystem + 核心目标 + 用户配置
    ↓
Step 1.1: 根据输出策略选择规律组合
    ↓
Step 1.2: 生成 Prompt（可能多个变体）
    ↓
输出: Prompt v1（或多版本）
```

### 4.2 输出策略处理

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

## 5. Phase 2: 测试与反思迭代

### 5.1 流程定义

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

### 5.2 并行测试实现

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

### 5.3 反思分类实现

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

### 5.4 梯度聚合实现（借鉴 TextGrad）

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

### 5.5 安全检查实现

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

## 6. 用户配置规格

### 6.1 输出策略配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `output_strategy` | enum | `"single"` | `"single"` / `"adaptive"` / `"multi"` |
| `conflict_alert_threshold` | int | `3` | 冲突数量达到此值时弹出推荐 |
| `auto_recommend` | bool | `true` | 是否启用智能推荐 |

### 6.2 Minibatch 配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `minibatch_enabled` | bool | `false` | 是否启用 Minibatch |
| `minibatch_size` | int | `10` | 每批测试数量 |
| `full_eval_interval` | int | `5` | 全量验证间隔轮数 |
| `minibatch_recommend_threshold` | int | `20` | 推荐启用阈值 |

### 6.3 震荡检测配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `oscillation_threshold` | int | `3` | 震荡判定轮数 |
| `oscillation_action` | enum | `"diversity_inject"` | `"diversity_inject"` / `"human_intervention"` / `"stop"` |

### 6.4 规律抽象配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_abstraction_level` | int | `3` | 规律抽象最大层级 |

### 6.5 迭代控制配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_iterations` | int | `20` | 最大迭代轮数 |
| `pass_threshold` | float | `0.95` | 通过率阈值 |
| `diversity_inject_after` | int | `3` | 连续失败多少次后触发多样性注入 |

---

## 7. 老师 Prompt 模板规格

### 7.1 规律提炼 Prompt

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

### 7.2 冲突检测 Prompt

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

### 7.3 反思分类 Prompt

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

## 8. 状态机定义

### 8.1 状态枚举

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

### 8.2 状态转换规则

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

## 9. 最佳实践来源

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

## 10. 附录：错误处理

### 10.1 HumanInterventionRequired 异常

触发条件：
- 规律冲突无法自动解决（超过最大抽象层级）
- 建议冲突无法仲裁
- 检测到优化震荡（且配置为人工介入）

处理方式：
- 保存当前状态到 Checkpoint
- 向用户展示问题详情
- 等待用户指导后继续

### 10.2 MaxIterationReached

触发条件：
- 迭代轮数达到 `max_iterations`

处理方式：
- 输出历史最佳 Prompt
- 生成优化报告

---

**文档结束**
