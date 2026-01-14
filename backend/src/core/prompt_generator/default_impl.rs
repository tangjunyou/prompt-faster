use crate::core::prompt_generator::{
    EXT_CANDIDATE_INDEX, EXT_OPTIMIZATION_GOAL, GeneratorError, TEMPLATE_VARIANT_COUNT,
};
use crate::core::traits::PromptGenerator;
use crate::domain::models::{FailureArchiveEntry, Rule, TestCase, failure_fingerprint_v1};
use crate::domain::types::{EXT_FAILURE_ARCHIVE, OptimizationContext};
use async_trait::async_trait;
use std::collections::{BTreeMap, BTreeSet};
use tracing::{info, warn};

#[derive(Debug, Default)]
pub struct DefaultPromptGenerator;

impl DefaultPromptGenerator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PromptGenerator for DefaultPromptGenerator {
    async fn generate(&self, ctx: &OptimizationContext) -> Result<String, GeneratorError> {
        let mut missing = Vec::new();
        let candidate_index = read_required_u32(ctx, EXT_CANDIDATE_INDEX, &mut missing)?;

        let current_prompt_blank = ctx.current_prompt.trim().is_empty();
        if current_prompt_blank {
            let optimization_goal = read_required_string(ctx, EXT_OPTIMIZATION_GOAL, &mut missing)?;
            if !missing.is_empty() {
                missing.sort();
                missing.dedup();
                return Err(GeneratorError::MissingContext { keys: missing });
            }
            validate_candidate_index(candidate_index)?;

            info!(
                candidate_index = candidate_index,
                test_case_count = ctx.test_cases.len(),
                "layer2 进入初始 Prompt 生成（current_prompt 为空）"
            );

            let prompt = build_initial_prompt(candidate_index, &optimization_goal, &ctx.test_cases);
            reject_duplicate_candidate(ctx, candidate_index, &prompt)?;
            return Ok(prompt);
        }

        if !missing.is_empty() {
            missing.sort();
            missing.dedup();
            return Err(GeneratorError::MissingContext { keys: missing });
        }
        validate_candidate_index(candidate_index)?;

        let rules = &ctx.rule_system.rules;
        if rules.is_empty() {
            return Err(GeneratorError::InvalidRules {
                reason: "ctx.rule_system.rules 为空（非初始生成阶段不允许无依据生成）".to_string(),
            });
        }

        let classification = classify_rules(rules)?;
        info!(
            candidate_index = candidate_index,
            rules_total = rules.len(),
            rules_success = classification.success.len(),
            rules_failure = classification.failure.len(),
            rules_all_passed = classification.all_passed.len(),
            "layer2 生成候选 Prompt"
        );

        if !classification.all_passed.is_empty() {
            return Err(GeneratorError::AllPassed {
                rule_ids: classification
                    .all_passed
                    .iter()
                    .map(|r| r.id.clone())
                    .collect(),
            });
        }

        if classification.failure.is_empty() {
            return Err(GeneratorError::InvalidRules {
                reason: "缺少 failure 规律，无法生成可解释的修复型候选".to_string(),
            });
        }

        let keep = build_keep_section(&classification.success);
        let fix = build_fix_section(&classification.failure);
        let summary = summarize_test_cases(&ctx.test_cases);

        let prompt = render_candidate_variant(candidate_index, &keep, &fix, &summary);

        reject_duplicate_candidate(ctx, candidate_index, &prompt)?;

        Ok(prompt)
    }

    fn name(&self) -> &str {
        "default_prompt_generator"
    }
}

fn reject_duplicate_candidate(
    ctx: &OptimizationContext,
    candidate_index: u32,
    prompt: &str,
) -> Result<(), GeneratorError> {
    // 1) 内容重复：若候选与 current_prompt 规范化后完全一致，则拒绝
    if !ctx.current_prompt.trim().is_empty()
        && failure_fingerprint_v1(prompt) == failure_fingerprint_v1(&ctx.current_prompt)
    {
        return Err(GeneratorError::DuplicateCandidate {
            candidate_index,
            fingerprint: failure_fingerprint_v1(prompt),
        });
    }

    // 2) 命中失败档案：fingerprint 以 prompt 为输入，可在候选生成阶段预先比对
    let Some(v) = ctx.extensions.get(EXT_FAILURE_ARCHIVE) else {
        return Ok(());
    };
    let archive: Vec<FailureArchiveEntry> =
        serde_json::from_value(v.clone()).map_err(|e| GeneratorError::InvalidContext {
            reason: format!("{EXT_FAILURE_ARCHIVE} 反序列化失败：{e}"),
        })?;

    let fingerprint = failure_fingerprint_v1(prompt);
    if archive.iter().any(|e| e.failure_fingerprint == fingerprint) {
        return Err(GeneratorError::DuplicateCandidate {
            candidate_index,
            fingerprint,
        });
    }

    Ok(())
}

struct RuleClassification<'a> {
    success: Vec<&'a Rule>,
    failure: Vec<&'a Rule>,
    all_passed: Vec<&'a Rule>,
}

fn classify_rules(rules: &[Rule]) -> Result<RuleClassification<'_>, GeneratorError> {
    let mut success = Vec::new();
    let mut failure = Vec::new();
    let mut all_passed = Vec::new();

    for r in rules {
        let polarity = rule_polarity(r)?;
        match polarity {
            "success" => success.push(r),
            "failure" => failure.push(r),
            "all_passed" => all_passed.push(r),
            other => {
                return Err(GeneratorError::InvalidRules {
                    reason: format!(
                        "未知 polarity={other:?}（约定仅允许 success|failure|all_passed）"
                    ),
                });
            }
        }
    }

    Ok(RuleClassification {
        success,
        failure,
        all_passed,
    })
}

fn rule_polarity(rule: &Rule) -> Result<&str, GeneratorError> {
    let v = rule
        .tags
        .extra
        .get("polarity")
        .ok_or_else(|| GeneratorError::InvalidRules {
            reason: format!("rule.id={} 缺少 tags.extra[\"polarity\"]", rule.id),
        })?;
    let s = v.as_str().ok_or_else(|| GeneratorError::InvalidRules {
        reason: format!(
            "rule.id={} tags.extra[\"polarity\"] 不是 string（实际={v:?}）",
            rule.id
        ),
    })?;
    Ok(s)
}

fn read_required_u32(
    ctx: &OptimizationContext,
    key: &str,
    missing: &mut Vec<String>,
) -> Result<u32, GeneratorError> {
    let v = match ctx.extensions.get(key) {
        Some(v) => v,
        None => {
            missing.push(key.to_string());
            return Ok(0);
        }
    };

    let n = v.as_u64().ok_or_else(|| GeneratorError::InvalidContext {
        reason: format!("ctx.extensions[{key:?}] 必须为 number(u32)，实际={v:?}"),
    })?;
    u32::try_from(n).map_err(|_| GeneratorError::InvalidContext {
        reason: format!("ctx.extensions[{key:?}] 超出 u32 范围，实际={n}"),
    })
}

fn read_required_string(
    ctx: &OptimizationContext,
    key: &str,
    missing: &mut Vec<String>,
) -> Result<String, GeneratorError> {
    let v = match ctx.extensions.get(key) {
        Some(v) => v,
        None => {
            missing.push(key.to_string());
            return Ok(String::new());
        }
    };
    let s = v.as_str().ok_or_else(|| GeneratorError::InvalidContext {
        reason: format!("ctx.extensions[{key:?}] 必须为 string，实际={v:?}"),
    })?;
    Ok(s.to_string())
}

fn validate_candidate_index(candidate_index: u32) -> Result<(), GeneratorError> {
    if candidate_index >= TEMPLATE_VARIANT_COUNT {
        return Err(GeneratorError::InvalidContext {
            reason: format!(
                "ctx.extensions[{EXT_CANDIDATE_INDEX:?}] 超出模板变体范围（0..{}），实际={}",
                TEMPLATE_VARIANT_COUNT, candidate_index
            ),
        });
    }
    Ok(())
}

fn build_keep_section(success_rules: &[&Rule]) -> String {
    if success_rules.is_empty() {
        return "（无 success 规律可保留；请保持现有 Prompt 的已知正确行为，并避免引入不必要改动。）".to_string();
    }

    let mut formats: BTreeSet<String> = BTreeSet::new();
    let mut structures: BTreeSet<String> = BTreeSet::new();
    let mut concepts: BTreeSet<String> = BTreeSet::new();
    let mut must_include: BTreeSet<String> = BTreeSet::new();
    let mut must_exclude: BTreeSet<String> = BTreeSet::new();

    for r in success_rules {
        formats.extend(r.tags.output_format.iter().cloned());
        structures.extend(r.tags.output_structure.iter().cloned());
        concepts.extend(r.tags.key_concepts.iter().cloned());
        must_include.extend(r.tags.must_include.iter().cloned());
        must_exclude.extend(r.tags.must_exclude.iter().cloned());
    }

    let mut lines = Vec::new();
    lines.push("必须保留已通过用例的成功特性（不要为了修复失败而破坏这些特性）：".to_string());

    if !formats.is_empty() {
        lines.push(format!("1) 输出格式偏好：{}", join_set(&formats)));
    }
    if !structures.is_empty() {
        lines.push(format!("2) 输出结构偏好：{}", join_set(&structures)));
    }
    if !concepts.is_empty() {
        lines.push(format!("3) 关键关注点：{}", join_set(&concepts)));
    }
    if !must_include.is_empty() {
        lines.push(format!("4) 必须包含：{}", join_set(&must_include)));
    }
    if !must_exclude.is_empty() {
        lines.push(format!("5) 必须排除：{}", join_set(&must_exclude)));
    }

    if lines.len() == 1 {
        lines.push("1) 保持当前 Prompt 的输出结构/格式/关键约束不变。".to_string());
    }

    lines.join("\n")
}

fn build_fix_section(failure_rules: &[&Rule]) -> String {
    use crate::domain::models::RuleIR;

    if failure_rules.is_empty() {
        warn!("failure 规律为空（不应到达此处），返回兜底修复区");
        return "必须修复失败模式：补全并强化失败相关约束（failure 规律为空，无法给出更具体指令）。"
            .to_string();
    }

    let mut by_dimension: BTreeMap<String, Vec<&Rule>> = BTreeMap::new();
    for r in failure_rules {
        by_dimension
            .entry(failure_dimension(r))
            .or_default()
            .push(*r);
    }

    let mut dims: Vec<(String, usize, Vec<&Rule>)> = Vec::new();
    for (dimension, mut rules) in by_dimension {
        let mut source_ids: BTreeSet<String> = BTreeSet::new();
        for r in &rules {
            for tc_id in &r.source_test_cases {
                source_ids.insert(tc_id.clone());
            }
        }

        rules.sort_by(|a, b| {
            b.source_test_cases
                .len()
                .cmp(&a.source_test_cases.len())
                .then_with(|| a.id.cmp(&b.id))
        });

        dims.push((dimension, source_ids.len(), rules));
    }

    dims.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut lines = Vec::new();
    lines.push(
        "必须修复至少 1 条 failure 规律（按失败维度→证据数排序），并将修复要求写成可执行指令："
            .to_string(),
    );

    for (dimension, evidence_count, rules) in dims.into_iter().take(5) {
        lines.push(format!(
            "【失败维度：{}（证据用例数≈{}）】",
            dimension, evidence_count
        ));

        for r in rules.into_iter().take(3) {
            let desc = r.description.trim();
            let desc = if desc.is_empty() {
                "（无描述）"
            } else {
                desc
            };
            lines.push(format!("- 规则 {}：{}", r.id, desc));
            if !r.source_test_cases.is_empty() {
                lines.push(format!(
                    "  - 证据：{} 个用例（IDs 已省略，仅计数）",
                    r.source_test_cases.len()
                ));
            }
            if !r.tags.output_format.is_empty() {
                lines.push(format!("  - 格式偏好：{}", join_vec(&r.tags.output_format)));
            }
            if !r.tags.output_structure.is_empty() {
                lines.push(format!(
                    "  - 结构偏好：{}",
                    join_vec(&r.tags.output_structure)
                ));
            }
            if !r.tags.must_include.is_empty() {
                lines.push(format!("  - 必须包含：{}", join_vec(&r.tags.must_include)));
            }
            if !r.tags.must_exclude.is_empty() {
                lines.push(format!("  - 必须排除：{}", join_vec(&r.tags.must_exclude)));
            }
            if !r.tags.key_concepts.is_empty() {
                lines.push(format!(
                    "  - 关键关注点：{}",
                    join_vec(&r.tags.key_concepts)
                ));
            }
            if let Some(RuleIR { constraints, .. }) = &r.ir {
                if !constraints.is_empty() {
                    lines.push(format!("  - 约束（IR）：{}", join_vec(constraints)));
                }
            }
            lines.push("  - 修复要求：将上述内容转为“必须/不得/格式/字段”级别的明确约束，并在最终输出中逐条满足。".to_string());
        }
    }

    lines.join("\n")
}

fn failure_dimension(rule: &Rule) -> String {
    if let Some(d) = rule.tags.semantic_focus.first() {
        if !d.trim().is_empty() {
            return d.trim().to_string();
        }
    }
    "unknown".to_string()
}

fn render_candidate_variant(candidate_index: u32, keep: &str, fix: &str, summary: &str) -> String {
    match candidate_index {
        0 => [
            "【变体 0｜基础版】你是一个严格、可复现的助手。请根据输入完成任务，并遵守以下约束。",
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【修复项（failure 规律）】",
            fix,
            "",
            "【输出要求】",
            "- 输出必须满足所有保持项与修复项；若存在冲突，优先保持已通过用例的输出结构，再在不破坏结构的前提下修复失败。",
            "- 不要输出与任务无关的解释或多余内容（除非任务要求）。",
        ]
        .join("\n"),
        1 => [
            "【变体 1｜结构优先版】你是一个严格的助手。请先锁定输出结构，再在结构内满足约束。",
            "",
            "【输出结构锁定】",
            "- 保持现有输出的标题/列表/表格等结构风格不被破坏；优先保证结构一致性。",
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【修复项（failure 规律）】",
            fix,
            "",
            "【自检】",
            "- 在提交最终输出前，逐条核对：保持项已满足；至少一条 failure 规律已被显式修复。",
        ]
        .join("\n"),
        2 => [
            "【变体 2｜失败聚焦版】你是一个严格的助手。请用“失败修复清单”驱动输出。",
            "",
            "【失败修复清单（按证据集中度排序）】",
            fix,
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【注意】",
            "- 不要引入新的输出格式/结构，除非失败修复必须且不会破坏已通过用例的特性。",
        ]
        .join("\n"),
        3 => [
            "【变体 3｜示例驱动版】你是一个严格的助手。请用“抽象示例/反例结构”校准输出（不泄露真实输入）。",
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【修复项（failure 规律）】",
            fix,
            "",
            "【抽象示例（仅结构，不含真实内容）】",
            "- ✅ 示例结构：<标题/字段顺序固定> → <主体> → <结尾/校验字段>",
            "- ❌ 反例结构：<缺字段/乱序/夹杂闲聊/无法解析>",
            "",
            "【输出要求】",
            "- 输出必须匹配“示例结构”的结构特征，同时满足保持项与修复项。",
        ]
        .join("\n"),
        4 => [
            "【变体 4｜检查清单版】你是一个严格的助手。先把要求转成 checklist，再产出最终输出。",
            "",
            "【Checklist（不要在最终输出中逐条列出，只用于你自检）】",
            "- [ ] 已满足所有保持项（success）",
            "- [ ] 已显式修复至少 1 条 failure 规律",
            "- [ ] 输出结构/格式可被自动评估与解析",
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【修复项（failure 规律）】",
            fix,
        ]
        .join("\n"),
        5 => [
            "【变体 5｜最小改动版】你是一个严格的助手。用最小增量满足修复项，避免破坏保持项。",
            "",
            "【策略】",
            "- 不改变已有输出结构/字段顺序/格式风格（除非 failure 修复必须）。",
            "- 仅添加/强化必要约束与字段，避免新增无关内容。",
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【修复项（failure 规律）】",
            fix,
        ]
        .join("\n"),
        6 => [
            "【变体 6｜强约束版】你是一个严格的助手。把修复项写成“必须/不得”级别的强约束并严格执行。",
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【强约束修复（failure 规律）】",
            fix,
            "",
            "【禁止】",
            "- 不要输出任何无法被解析/评估的额外解释文本（除非任务要求）。",
        ]
        .join("\n"),
        7 => [
            "【变体 7｜解释优先版】你是一个严格的助手。先做内部推理/自检，再只输出最终结果。",
            "",
            "【内部自检（不要输出自检过程）】",
            "- 逐条核对保持项与修复项是否满足；若冲突，优先保持 success 结构，再以最小改动修复 failure。",
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【修复项（failure 规律）】",
            fix,
        ]
        .join("\n"),
        8 => [
            "【变体 8｜维度权重版】你是一个严格的助手。对测试集常见质量维度/约束给予更高优先级。",
            "",
            "【测试集摘要（用于权重，不含真实输入）】",
            summary,
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【修复项（failure 规律）】",
            fix,
            "",
            "【要求】",
            "- 对摘要中出现的“常见质量维度/约束”给予更高优先级，确保输出可评估、可解析。",
        ]
        .join("\n"),
        9 => [
            "【变体 9｜冲突仲裁版】你是一个严格的助手。若保持项与修复项冲突，必须按优先级仲裁并用最小改动落地。",
            "",
            "【冲突仲裁规则】",
            "- 优先级 1：保持已通过用例的输出结构稳定（success）。",
            "- 优先级 2：在不破坏结构的前提下修复 failure；必要时选择最小折中方案。",
            "- 若仍不可兼得：选择对通过率回归风险最低的折中，并确保输出可评估。",
            "",
            "【保持项（success 规律）】",
            keep,
            "",
            "【修复项（failure 规律）】",
            fix,
        ]
        .join("\n"),
        _ => unreachable!("candidate_index 已在上游校验为 0..TEMPLATE_VARIANT_COUNT"),
    }
}

fn build_initial_prompt(
    candidate_index: u32,
    optimization_goal: &str,
    test_cases: &[TestCase],
) -> String {
    let summary = summarize_test_cases(test_cases);
    match candidate_index {
        0 => [
            "【初始变体 0｜基础版】你是一个严格的助手。请根据用户输入完成任务。",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【测试集概览（仅摘要，不包含真实输入）】",
            &summary,
            "",
            "【要求】",
            "- 输出必须可被自动评估；保持结构稳定、可解析、无多余闲聊。",
        ]
        .join("\n"),
        1 => [
            "【初始变体 1｜结构优先版】你是一个严格的助手。优先保证输出结构清晰且稳定。",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【测试集概览（仅摘要，不包含真实输入）】",
            &summary,
            "",
            "【输出约束】",
            "- 使用一致的标题/列表层级；避免同一字段多种写法。",
            "- 若存在格式要求，请显式遵守（例如 JSON/表格/编号列表）。",
        ]
        .join("\n"),
        2 => [
            "【初始变体 2｜失败预防版】你是一个严格的助手。请对失败风险点进行预防式约束（基于测试集摘要）。",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【测试集概览（仅摘要，不包含真实输入）】",
            &summary,
            "",
            "【自检清单（不要输出自检过程）】",
            "- 输出是否结构稳定、可复现？",
            "- 是否覆盖了测试集常见约束/质量维度？",
        ]
        .join("\n"),
        3 => [
            "【初始变体 3｜示例驱动版】你是一个严格的助手。用抽象示例/反例结构校准输出（不泄露真实输入）。",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【抽象示例（仅结构）】",
            "- ✅ 示例：<标题/字段固定> → <主体> → <校验字段>",
            "- ❌ 反例：<字段缺失/乱序/夹杂闲聊/不可解析>",
            "",
            "【测试集概览（仅摘要）】",
            &summary,
        ]
        .join("\n"),
        4 => [
            "【初始变体 4｜检查清单版】你是一个严格的助手。先生成 checklist（仅用于你自检），再输出最终结果。",
            "",
            "【Checklist（不要输出）】",
            "- [ ] 输出结构稳定、可解析",
            "- [ ] 满足测试集摘要中的常见约束/质量维度",
            "- [ ] 不输出多余解释（除非任务要求）",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【测试集概览（仅摘要）】",
            &summary,
        ]
        .join("\n"),
        5 => [
            "【初始变体 5｜最小改动版】你是一个严格的助手。优先稳定结构，用最小增量满足约束。",
            "",
            "【策略】",
            "- 优先确定输出结构（标题/列表/字段顺序）。",
            "- 再补充必要约束，避免引入无关内容。",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【测试集概览（仅摘要）】",
            &summary,
        ]
        .join("\n"),
        6 => [
            "【初始变体 6｜强约束版】你是一个严格的助手。把约束写成“必须/不得”并严格执行。",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【测试集概览（仅摘要）】",
            &summary,
            "",
            "【禁止】",
            "- 不要输出不可解析/不可评估的额外闲聊。",
        ]
        .join("\n"),
        7 => [
            "【初始变体 7｜解释优先版】你是一个严格的助手。先做内部推理/自检，再只输出最终结果。",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【内部自检（不要输出）】",
            "- 逐条核对是否覆盖常见约束/质量维度；输出结构稳定且可评估。",
            "",
            "【测试集概览（仅摘要）】",
            &summary,
        ]
        .join("\n"),
        8 => [
            "【初始变体 8｜维度权重版】你是一个严格的助手。对测试集常见质量维度给予更高优先级。",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【测试集概览（仅摘要）】",
            &summary,
            "",
            "【要求】",
            "- 优先满足“常见质量维度/常见约束”，确保输出可评估、可解析。",
        ]
        .join("\n"),
        9 => [
            "【初始变体 9｜冲突仲裁版】你是一个严格的助手。若约束之间冲突，必须按优先级仲裁并选择最小折中。",
            "",
            "【优先级】",
            "- 1) 输出结构稳定、可解析",
            "- 2) 覆盖测试集常见约束/质量维度",
            "- 3) 最小化无关输出",
            "",
            "【优化目标】",
            optimization_goal,
            "",
            "【测试集概览（仅摘要）】",
            &summary,
        ]
        .join("\n"),
        _ => unreachable!("candidate_index 已在上游校验为 0..TEMPLATE_VARIANT_COUNT"),
    }
}

fn summarize_test_cases(test_cases: &[TestCase]) -> String {
    use crate::domain::models::TaskReference;

    let mut constraints: BTreeSet<String> = BTreeSet::new();
    let mut quality_dimensions: BTreeSet<String> = BTreeSet::new();
    let mut exact_count = 0usize;
    let mut constrained_count = 0usize;
    let mut hybrid_count = 0usize;

    for tc in test_cases {
        match &tc.reference {
            TaskReference::Exact { .. } => {
                exact_count += 1;
            }
            TaskReference::Constrained {
                core_request: _,
                constraints: cs,
                quality_dimensions: qs,
            } => {
                constrained_count += 1;
                constraints.extend(cs.iter().map(|c| c.name.clone()));
                quality_dimensions.extend(qs.iter().map(|q| q.name.clone()));
            }
            TaskReference::Hybrid {
                exact_parts: _,
                constraints: cs,
            } => {
                hybrid_count += 1;
                constraints.extend(cs.iter().map(|c| c.name.clone()));
            }
        }
    }

    let mut lines = Vec::new();
    lines.push(format!("用例数量：{}", test_cases.len()));
    lines.push(format!(
        "用例类型：exact={} constrained={} hybrid={}",
        exact_count, constrained_count, hybrid_count
    ));
    if !constraints.is_empty() {
        lines.push(format!("常见约束：{}", join_set(&constraints)));
    }
    if !quality_dimensions.is_empty() {
        lines.push(format!("常见质量维度：{}", join_set(&quality_dimensions)));
    }

    lines.join("\n")
}

fn join_set(set: &BTreeSet<String>) -> String {
    join_str_iter(set.iter().map(|s| s.as_str()))
}

fn join_vec(vec: &[String]) -> String {
    join_str_iter(vec.iter().map(|s| s.as_str()))
}

fn join_str_iter<'a>(iter: impl IntoIterator<Item = &'a str>) -> String {
    let mut out = String::new();
    for (i, s) in iter.into_iter().enumerate() {
        if i > 0 {
            out.push('、');
        }
        out.push_str(s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{OutputLength, RuleSystem, RuleTags, TaskReference};
    use crate::domain::types::{EXT_FAILURE_ARCHIVE, ExecutionTargetConfig, OptimizationConfig};
    use serde_json::json;
    use std::collections::HashMap;

    fn make_ctx(
        current_prompt: &str,
        rules: Vec<Rule>,
        extensions: HashMap<String, serde_json::Value>,
        test_cases: Vec<TestCase>,
    ) -> OptimizationContext {
        OptimizationContext {
            task_id: "task-1".to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: current_prompt.to_string(),
            rule_system: RuleSystem {
                rules,
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 0,
            },
            iteration: 0,
            state: crate::domain::models::IterationState::Idle,
            test_cases,
            config: OptimizationConfig::default(),
            checkpoints: vec![],
            extensions,
        }
    }

    fn rule_with_polarity(
        id: &str,
        polarity: &str,
        output_format: Vec<&str>,
        output_structure: Vec<&str>,
        semantic_focus: Vec<&str>,
        key_concepts: Vec<&str>,
        source_test_cases: Vec<&str>,
    ) -> Rule {
        let mut extra = HashMap::new();
        extra.insert("polarity".to_string(), json!(polarity));
        Rule {
            id: id.to_string(),
            description: format!("{polarity} rule"),
            tags: RuleTags {
                output_format: output_format.into_iter().map(|s| s.to_string()).collect(),
                output_structure: output_structure
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                output_length: OutputLength::Flexible,
                semantic_focus: semantic_focus.into_iter().map(|s| s.to_string()).collect(),
                key_concepts: key_concepts.into_iter().map(|s| s.to_string()).collect(),
                must_include: vec![],
                must_exclude: vec![],
                tone: None,
                extra,
            },
            source_test_cases: source_test_cases
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            abstraction_level: 0,
            parent_rules: vec![],
            verified: false,
            verification_score: 0.0,
            ir: None,
        }
    }

    fn make_test_case(id: &str) -> TestCase {
        TestCase {
            id: id.to_string(),
            input: HashMap::new(),
            reference: TaskReference::Exact {
                expected: "ok".to_string(),
            },
            split: None,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn generate_mixed_success_and_failure_contains_keep_and_fix() {
        let generator = DefaultPromptGenerator::new();

        let success = rule_with_polarity(
            "r1",
            "success",
            vec!["json"],
            vec!["table"],
            vec![],
            vec!["exact_match"],
            vec!["tc_ok"],
        );
        let failure = rule_with_polarity(
            "r2",
            "failure",
            vec![],
            vec![],
            vec!["format"],
            vec!["format"],
            vec!["tc_bad", "tc_bad2"],
        );

        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(0));

        let ctx = make_ctx("prompt", vec![success, failure], ext, vec![]);
        let out = generator.generate(&ctx).await.unwrap();

        assert!(out.contains("【保持项"));
        assert!(out.contains("json"));
        assert!(out.contains("【修复项") || out.contains("失败修复清单"));
        assert!(out.contains("规则 r2"));
    }

    #[tokio::test]
    async fn missing_candidate_index_returns_missing_context() {
        let generator = DefaultPromptGenerator::new();
        let ctx = make_ctx("prompt", vec![], HashMap::new(), vec![]);

        let err = generator.generate(&ctx).await.unwrap_err();
        match err {
            GeneratorError::MissingContext { keys } => {
                assert!(keys.contains(&EXT_CANDIDATE_INDEX.to_string()));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn initial_prompt_with_goal_generates_non_empty() {
        let generator = DefaultPromptGenerator::new();
        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(1));
        ext.insert(EXT_OPTIMIZATION_GOAL.to_string(), json!("提升通过率"));

        let ctx = make_ctx("", vec![], ext, vec![make_test_case("tc1")]);
        let out = generator.generate(&ctx).await.unwrap();
        assert!(!out.trim().is_empty());
        assert!(out.contains("提升通过率"));
    }

    #[tokio::test]
    async fn initial_prompt_missing_goal_returns_missing_context() {
        let generator = DefaultPromptGenerator::new();
        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(2));

        let ctx = make_ctx("   ", vec![], ext, vec![make_test_case("tc1")]);
        let err = generator.generate(&ctx).await.unwrap_err();
        match err {
            GeneratorError::MissingContext { keys } => {
                assert!(keys.contains(&EXT_OPTIMIZATION_GOAL.to_string()));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn candidate_index_variants_are_reproducible_and_different() {
        let generator = DefaultPromptGenerator::new();

        let success = rule_with_polarity(
            "r1",
            "success",
            vec!["json"],
            vec!["table"],
            vec![],
            vec!["exact_match"],
            vec!["tc_ok"],
        );
        let failure = rule_with_polarity(
            "r2",
            "failure",
            vec![],
            vec![],
            vec!["format"],
            vec!["format"],
            vec!["tc_bad"],
        );

        let mut ext0 = HashMap::new();
        ext0.insert(EXT_CANDIDATE_INDEX.to_string(), json!(0));
        let ctx0 = make_ctx(
            "prompt",
            vec![success.clone(), failure.clone()],
            ext0,
            vec![],
        );

        let mut ext1 = HashMap::new();
        ext1.insert(EXT_CANDIDATE_INDEX.to_string(), json!(1));
        let ctx1 = make_ctx(
            "prompt",
            vec![success.clone(), failure.clone()],
            ext1,
            vec![],
        );

        let mut ext2 = HashMap::new();
        ext2.insert(EXT_CANDIDATE_INDEX.to_string(), json!(2));
        let ctx2 = make_ctx("prompt", vec![success, failure], ext2, vec![]);

        let out0a = generator.generate(&ctx0).await.unwrap();
        let out0b = generator.generate(&ctx0).await.unwrap();
        assert_eq!(out0a, out0b);

        let out1 = generator.generate(&ctx1).await.unwrap();
        let out2 = generator.generate(&ctx2).await.unwrap();

        assert_ne!(out0a, out1);
        assert_ne!(out0a, out2);
        assert_ne!(out1, out2);

        assert!(out0a.contains("【变体 0｜基础版】"));
        assert!(out1.contains("【变体 1｜结构优先版】"));
        assert!(out2.contains("【变体 2｜失败聚焦版】"));
    }

    #[tokio::test]
    async fn candidate_index_out_of_range_returns_invalid_context() {
        let generator = DefaultPromptGenerator::new();

        let failure = rule_with_polarity(
            "r2",
            "failure",
            vec![],
            vec![],
            vec!["format"],
            vec![],
            vec!["tc_bad"],
        );

        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(10));

        let ctx = make_ctx("prompt", vec![failure], ext, vec![]);
        let err = generator.generate(&ctx).await.unwrap_err();
        assert!(matches!(err, GeneratorError::InvalidContext { .. }));
    }

    #[tokio::test]
    async fn initial_candidate_index_out_of_range_returns_invalid_context() {
        let generator = DefaultPromptGenerator::new();

        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(999));
        ext.insert(EXT_OPTIMIZATION_GOAL.to_string(), json!("提升通过率"));

        let ctx = make_ctx("", vec![], ext, vec![make_test_case("tc1")]);
        let err = generator.generate(&ctx).await.unwrap_err();
        assert!(matches!(err, GeneratorError::InvalidContext { .. }));
    }

    #[tokio::test]
    async fn all_ten_variants_are_reproducible_and_distinct() {
        let generator = DefaultPromptGenerator::new();

        let success = rule_with_polarity(
            "r1",
            "success",
            vec!["json"],
            vec!["table"],
            vec![],
            vec!["exact_match"],
            vec!["tc_ok"],
        );
        let failure = rule_with_polarity(
            "r2",
            "failure",
            vec![],
            vec![],
            vec!["format"],
            vec![],
            vec!["tc_bad"],
        );

        let mut outputs = Vec::new();
        for i in 0u32..TEMPLATE_VARIANT_COUNT {
            let mut ext = HashMap::new();
            ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(i));
            let ctx = make_ctx(
                "prompt",
                vec![success.clone(), failure.clone()],
                ext,
                vec![make_test_case("tc1")],
            );

            let out_a = generator.generate(&ctx).await.unwrap();
            let out_b = generator.generate(&ctx).await.unwrap();
            assert_eq!(out_a, out_b);
            outputs.push(out_a);
        }

        for i in 0..outputs.len() {
            for j in (i + 1)..outputs.len() {
                assert_ne!(
                    outputs[i], outputs[j],
                    "variant {i} and {j} unexpectedly identical"
                );
            }
        }
    }

    #[tokio::test]
    async fn initial_all_ten_variants_are_reproducible_and_distinct() {
        let generator = DefaultPromptGenerator::new();

        let mut outputs = Vec::new();
        for i in 0u32..TEMPLATE_VARIANT_COUNT {
            let mut ext = HashMap::new();
            ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(i));
            ext.insert(EXT_OPTIMIZATION_GOAL.to_string(), json!("提升通过率"));

            let ctx = make_ctx("", vec![], ext, vec![make_test_case("tc1")]);
            let out_a = generator.generate(&ctx).await.unwrap();
            let out_b = generator.generate(&ctx).await.unwrap();
            assert_eq!(out_a, out_b);
            outputs.push(out_a);
        }

        for i in 0..outputs.len() {
            for j in (i + 1)..outputs.len() {
                assert_ne!(
                    outputs[i], outputs[j],
                    "initial variant {i} and {j} unexpectedly identical"
                );
            }
        }
    }

    #[tokio::test]
    async fn all_passed_returns_structured_error() {
        let generator = DefaultPromptGenerator::new();

        let all_passed = rule_with_polarity(
            "r_all",
            "all_passed",
            vec![],
            vec![],
            vec![],
            vec![],
            vec!["tc_ok"],
        );

        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(0));

        let ctx = make_ctx("prompt", vec![all_passed], ext, vec![]);
        let err = generator.generate(&ctx).await.unwrap_err();
        match err {
            GeneratorError::AllPassed { rule_ids } => {
                assert!(rule_ids.contains(&"r_all".to_string()));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn summarize_test_cases_mixed_constrained_and_hybrid() {
        use crate::domain::models::{Constraint, QualityDimension};

        let constrained = TestCase {
            id: "tc_constrained".to_string(),
            input: HashMap::new(),
            reference: TaskReference::Constrained {
                core_request: Some("do something".to_string()),
                constraints: vec![Constraint {
                    name: "no_hallucination".to_string(),
                    description: "no hallucination".to_string(),
                    params: None,
                    weight: Some(1.0),
                }],
                quality_dimensions: vec![QualityDimension {
                    name: "clarity".to_string(),
                    description: "clarity".to_string(),
                    weight: 1.0,
                }],
            },
            split: None,
            metadata: None,
        };

        let mut exact_parts = HashMap::new();
        exact_parts.insert("k1".to_string(), "v1".to_string());
        let hybrid = TestCase {
            id: "tc_hybrid".to_string(),
            input: HashMap::new(),
            reference: TaskReference::Hybrid {
                exact_parts,
                constraints: vec![Constraint {
                    name: "json_only".to_string(),
                    description: "json only".to_string(),
                    params: None,
                    weight: Some(1.0),
                }],
            },
            split: None,
            metadata: None,
        };

        let summary = summarize_test_cases(&[constrained, hybrid]);
        assert!(summary.contains("constrained=1"));
        assert!(summary.contains("hybrid=1"));
        assert!(summary.contains("常见约束："));
        assert!(summary.contains("no_hallucination"));
        assert!(summary.contains("json_only"));
        assert!(summary.contains("常见质量维度："));
        assert!(summary.contains("clarity"));
    }

    #[tokio::test]
    async fn missing_polarity_key_returns_invalid_rules() {
        let generator = DefaultPromptGenerator::new();

        let bad = Rule {
            id: "r_missing".to_string(),
            description: "missing polarity".to_string(),
            tags: RuleTags {
                output_format: vec![],
                output_structure: vec![],
                output_length: OutputLength::Flexible,
                semantic_focus: vec![],
                key_concepts: vec![],
                must_include: vec![],
                must_exclude: vec![],
                tone: None,
                extra: HashMap::new(),
            },
            source_test_cases: vec!["tc1".to_string()],
            abstraction_level: 0,
            parent_rules: vec![],
            verified: false,
            verification_score: 0.0,
            ir: None,
        };

        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(0));

        let ctx = make_ctx("prompt", vec![bad], ext, vec![]);
        let err = generator.generate(&ctx).await.unwrap_err();
        assert!(matches!(err, GeneratorError::InvalidRules { .. }));
    }

    #[tokio::test]
    async fn non_initial_with_empty_rules_returns_invalid_rules() {
        let generator = DefaultPromptGenerator::new();
        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(0));

        let ctx = make_ctx("prompt", vec![], ext, vec![]);
        let err = generator.generate(&ctx).await.unwrap_err();
        assert!(matches!(err, GeneratorError::InvalidRules { .. }));
    }

    #[tokio::test]
    async fn invalid_polarity_returns_invalid_rules() {
        let generator = DefaultPromptGenerator::new();

        let bad = rule_with_polarity(
            "r_bad",
            "weird",
            vec![],
            vec![],
            vec![],
            vec![],
            vec!["tc1"],
        );

        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(0));
        let ctx = make_ctx("prompt", vec![bad], ext, vec![]);
        let err = generator.generate(&ctx).await.unwrap_err();
        assert!(matches!(err, GeneratorError::InvalidRules { .. }));
    }

    #[tokio::test]
    async fn generate_rejects_duplicate_candidate_when_fingerprint_hits_failure_archive() {
        let generator = DefaultPromptGenerator::new();

        let success = rule_with_polarity(
            "r1",
            "success",
            vec!["json"],
            vec!["table"],
            vec![],
            vec!["exact_match"],
            vec!["tc_ok"],
        );
        let failure = rule_with_polarity(
            "r2",
            "failure",
            vec![],
            vec![],
            vec!["format"],
            vec!["format"],
            vec!["tc_bad"],
        );

        let mut ext0 = HashMap::new();
        ext0.insert(EXT_CANDIDATE_INDEX.to_string(), json!(0));
        let ctx0 = make_ctx(
            "prompt",
            vec![success.clone(), failure.clone()],
            ext0,
            vec![],
        );
        let candidate_prompt = generator.generate(&ctx0).await.unwrap();
        let fp = failure_fingerprint_v1(&candidate_prompt);

        let entry = FailureArchiveEntry::new(&candidate_prompt, "tc1", "format:bad");
        let mut ext = HashMap::new();
        ext.insert(EXT_CANDIDATE_INDEX.to_string(), json!(0));
        ext.insert(
            EXT_FAILURE_ARCHIVE.to_string(),
            serde_json::to_value(vec![entry]).unwrap(),
        );

        let ctx = make_ctx("prompt", vec![success, failure], ext, vec![]);
        let err = generator.generate(&ctx).await.unwrap_err();

        match err.clone() {
            GeneratorError::DuplicateCandidate {
                candidate_index,
                fingerprint,
            } => {
                assert_eq!(candidate_index, 0);
                assert_eq!(fingerprint, fp);
            }
            other => panic!("unexpected error: {other:?}"),
        }

        // 不得泄露 prompt/testcase input 全文（这里只检查一个明显片段）
        assert!(!err.to_string().contains("【保持项"));
    }
}
