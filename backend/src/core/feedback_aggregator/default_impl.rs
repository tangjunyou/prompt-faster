use crate::core::feedback_aggregator::AggregatorError;
use crate::domain::models::{
    ArbitrationMethod, ArbitrationResult, ConflictType, FailureType, ReflectionResult, Suggestion,
    SuggestionConflict, SuggestionType, UnifiedReflection, UnifiedSuggestion,
};
use crate::domain::types::{
    CandidateStats, EXT_BEST_CANDIDATE_STATS, EXT_CONSECUTIVE_NO_IMPROVEMENT,
    EXT_CURRENT_PROMPT_STATS, EXT_EVALUATIONS_BY_TEST_CASE_ID, METRIC_EPS, OptimizationContext,
};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct DefaultFeedbackAggregator;

#[async_trait]
impl crate::core::traits::FeedbackAggregator for DefaultFeedbackAggregator {
    async fn aggregate(
        &self,
        ctx: &OptimizationContext,
        reflections: &[ReflectionResult],
    ) -> Result<UnifiedReflection, AggregatorError> {
        if reflections.is_empty() {
            return Err(AggregatorError::InvalidReflections(
                "reflections 为空".to_string(),
            ));
        }

        if reflections
            .iter()
            .all(|rr| rr.failed_test_case_ids.is_empty())
        {
            return Err(AggregatorError::InvalidReflections(
                "failed_test_case_ids 全为空：无法追溯失败用例（可能是上游未正确填充）".to_string(),
            ));
        }

        // 必需：用于追溯 failure_points / failed_test_case_ids。
        let evaluations_by_id = read_evaluations_by_test_case_id(ctx)?;

        // 校验：failed_test_case_ids 必须可追溯（只暴露 ID，不泄露 prompt/input 原文）。
        let mut missing = Vec::new();
        for rr in reflections {
            for id in &rr.failed_test_case_ids {
                if !evaluations_by_id.contains_key(id) {
                    missing.push(id.clone());
                }
            }
        }
        if !missing.is_empty() {
            missing.sort();
            missing.dedup();
            return Err(AggregatorError::InvalidReflections(format!(
                "无法追溯失败用例评估结果：missing_test_case_ids={missing:?}"
            )));
        }

        let (primary_failure_type, failure_type_distribution) = vote_failure_type(reflections);

        let (unified_suggestions, source_suggestion_samples) = merge_suggestions(reflections);

        let conflicts = detect_conflicts(&source_suggestion_samples, &unified_suggestions);
        let has_conflicts = !conflicts.is_empty();

        let arbitration_result = if has_conflicts {
            Some(self.arbitrate(ctx, &conflicts).await?)
        } else {
            None
        };

        let (recommended_action, extra) = choose_recommended_action(
            ctx,
            primary_failure_type.clone(),
            &unified_suggestions,
            has_conflicts,
        );

        Ok(UnifiedReflection {
            primary_failure_type,
            unified_suggestions: unified_suggestions.clone(),
            has_conflicts,
            conflicts,
            arbitration_result,
            source_count: reflections.len() as u32,
            failure_type_distribution,
            recommended_action,
            extra,
        })
    }

    async fn arbitrate(
        &self,
        _ctx: &OptimizationContext,
        conflicts: &[SuggestionConflict],
    ) -> Result<ArbitrationResult, AggregatorError> {
        if conflicts.is_empty() {
            return Err(AggregatorError::ArbitrationFailed(
                "conflicts 为空".to_string(),
            ));
        }

        // MVP：确定性仲裁（更高 confidence 胜；相等则 KeepAll）
        let mut chosen = Vec::<UnifiedSuggestion>::new();
        let mut reasoning_lines = Vec::<String>::new();

        for (idx, c) in conflicts.iter().enumerate() {
            let a = &c.suggestion_a;
            let b = &c.suggestion_b;
            let (keep_a, keep_b) = match a
                .confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
            {
                std::cmp::Ordering::Greater => (true, false),
                std::cmp::Ordering::Less => (false, true),
                std::cmp::Ordering::Equal => (true, true),
            };

            if keep_a {
                chosen.push(to_unified(a, 1, 1));
            }
            if keep_b {
                chosen.push(to_unified(b, 1, 1));
            }

            reasoning_lines.push(format!(
                "conflict[{idx}]: type={:?} kept={}{}",
                c.conflict_type,
                if keep_a { "A" } else { "" },
                if keep_b { "B" } else { "" }
            ));
        }

        // 去重（按 type+content）
        chosen = dedup_unified_suggestions(chosen);

        // 按 confidence desc 重新编号 priority
        chosen.sort_by(|x, y| {
            y.confidence
                .partial_cmp(&x.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    suggestion_type_rank(&x.suggestion_type)
                        .cmp(&suggestion_type_rank(&y.suggestion_type))
                })
                .then_with(|| x.content.cmp(&y.content))
        });
        for (i, s) in chosen.iter_mut().enumerate() {
            s.priority = (i + 1) as u32;
        }

        Ok(ArbitrationResult {
            chosen_suggestions: chosen,
            reasoning: reasoning_lines.join("; "),
            method: ArbitrationMethod::Voting,
        })
    }

    fn name(&self) -> &str {
        "default_feedback_aggregator"
    }
}

fn read_evaluations_by_test_case_id(
    ctx: &OptimizationContext,
) -> Result<HashMap<String, crate::domain::models::EvaluationResult>, AggregatorError> {
    let v = ctx
        .extensions
        .get(EXT_EVALUATIONS_BY_TEST_CASE_ID)
        .ok_or_else(|| {
            AggregatorError::InvalidReflections(format!(
                "ctx.extensions 缺少必需字段：{key:?}",
                key = EXT_EVALUATIONS_BY_TEST_CASE_ID
            ))
        })?;

    serde_json::from_value(v.clone()).map_err(|e| {
        AggregatorError::InvalidReflections(format!(
            "{key} 反序列化失败：{e}",
            key = EXT_EVALUATIONS_BY_TEST_CASE_ID
        ))
    })
}

fn vote_failure_type(reflections: &[ReflectionResult]) -> (FailureType, HashMap<String, u32>) {
    let mut counts: HashMap<&'static str, u32> = HashMap::new();
    for rr in reflections {
        *counts
            .entry(failure_type_key(&rr.failure_type))
            .or_insert(0) += 1;
    }

    let mut best_key = "undetermined";
    let mut best_count = 0u32;
    for (k, c) in &counts {
        if *c > best_count || (*c == best_count && *k < best_key) {
            best_key = k;
            best_count = *c;
        }
    }

    let primary = match best_key {
        "rule_incomplete" => FailureType::RuleIncomplete,
        "rule_incorrect" => FailureType::RuleIncorrect,
        "expression_issue" => FailureType::ExpressionIssue,
        "edge_case" => FailureType::EdgeCase,
        _ => FailureType::Undetermined,
    };

    let dist = counts
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect::<HashMap<_, _>>();

    (primary, dist)
}

fn failure_type_key(ft: &FailureType) -> &'static str {
    match ft {
        FailureType::RuleIncomplete => "rule_incomplete",
        FailureType::RuleIncorrect => "rule_incorrect",
        FailureType::ExpressionIssue => "expression_issue",
        FailureType::EdgeCase => "edge_case",
        FailureType::Undetermined => "undetermined",
    }
}

fn merge_suggestions(
    reflections: &[ReflectionResult],
) -> (Vec<UnifiedSuggestion>, Vec<Suggestion>) {
    let mut groups: HashMap<(SuggestionType, String), Vec<Suggestion>> = HashMap::new();
    let mut samples: Vec<Suggestion> = Vec::new();

    for rr in reflections {
        for s in &rr.suggestions {
            // 仅记录脱敏内容：content 由上游（LLM/规则）生成，可能包含 prompt 原文。
            // 这里不主动拼接更多原文；但也不在错误里回显 content。
            let key = (s.suggestion_type.clone(), normalize_text(&s.content));
            groups.entry(key).or_default().push(s.clone());
            samples.push(s.clone());
        }
    }

    let mut unified = Vec::new();
    for ((suggestion_type, norm), items) in groups {
        if norm.trim().is_empty() {
            continue;
        }
        let support_count = items.len() as u32;
        let mut conf_sum = 0.0;
        for s in &items {
            conf_sum += sanitize_confidence(s.confidence);
        }
        let confidence = if support_count == 0 {
            0.0
        } else {
            conf_sum / support_count as f64
        };

        unified.push(UnifiedSuggestion {
            suggestion_type,
            content: items[0].content.trim().to_string(),
            confidence,
            support_count,
            priority: 0,
        });
    }

    unified.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                suggestion_type_rank(&a.suggestion_type)
                    .cmp(&suggestion_type_rank(&b.suggestion_type))
            })
            .then_with(|| a.content.cmp(&b.content))
    });

    for (i, s) in unified.iter_mut().enumerate() {
        s.priority = (i + 1) as u32;
    }

    (unified, samples)
}

fn detect_conflicts(
    samples: &[Suggestion],
    unified_suggestions: &[UnifiedSuggestion],
) -> Vec<SuggestionConflict> {
    // 最小冲突识别：
    // - AddRule vs RemoveRule
    // - ModifyRule vs RemoveRule
    // - 同一内容的互斥建议（按归一化 content）
    let mut by_norm: HashMap<String, Vec<Suggestion>> = HashMap::new();
    for s in samples {
        by_norm
            .entry(normalize_text(&s.content))
            .or_default()
            .push(s.clone());
    }

    let mut conflicts = Vec::new();

    for (_k, items) in by_norm {
        if items.len() < 2 {
            continue;
        }
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                if let Some(conflict_type) =
                    conflict_type_for_pair(&items[i].suggestion_type, &items[j].suggestion_type)
                {
                    conflicts.push(SuggestionConflict {
                        suggestion_a: items[i].clone(),
                        suggestion_b: items[j].clone(),
                        conflict_type,
                        description: "检测到互斥的建议类型".to_string(),
                    });
                }
            }
        }
    }

    // 兜底：若 unified_suggestions 中存在 AddRule 与 RemoveRule，但未能按内容定位，则生成一个可诊断冲突。
    let has_add_rule = unified_suggestions
        .iter()
        .any(|u| matches!(u.suggestion_type, SuggestionType::AddRule));
    let has_remove_rule = unified_suggestions
        .iter()
        .any(|u| matches!(u.suggestion_type, SuggestionType::RemoveRule));

    if has_add_rule && has_remove_rule && conflicts.is_empty() {
        let a = Suggestion {
            suggestion_type: SuggestionType::AddRule,
            content: "(redacted)".to_string(),
            confidence: 0.0,
            expected_impact: None,
        };
        let b = Suggestion {
            suggestion_type: SuggestionType::RemoveRule,
            content: "(redacted)".to_string(),
            confidence: 0.0,
            expected_impact: None,
        };
        conflicts.push(SuggestionConflict {
            suggestion_a: a,
            suggestion_b: b,
            conflict_type: ConflictType::DirectContradiction,
            description: "同时存在 AddRule 与 RemoveRule 建议，但缺少可定位目标（请人工确认）"
                .to_string(),
        });
    }

    conflicts
}

fn conflict_type_for_pair(a: &SuggestionType, b: &SuggestionType) -> Option<ConflictType> {
    use SuggestionType::*;
    match (a, b) {
        (AddRule, RemoveRule) | (RemoveRule, AddRule) => Some(ConflictType::DirectContradiction),
        (ModifyRule, RemoveRule) | (RemoveRule, ModifyRule) => {
            Some(ConflictType::ResourceCompetition)
        }
        _ => None,
    }
}

fn to_unified(s: &Suggestion, support_count: u32, priority: u32) -> UnifiedSuggestion {
    UnifiedSuggestion {
        suggestion_type: s.suggestion_type.clone(),
        content: s.content.trim().to_string(),
        confidence: sanitize_confidence(s.confidence),
        support_count,
        priority,
    }
}

fn dedup_unified_suggestions(mut items: Vec<UnifiedSuggestion>) -> Vec<UnifiedSuggestion> {
    let mut seen: HashMap<(SuggestionType, String), UnifiedSuggestion> = HashMap::new();
    for s in items.drain(..) {
        let key = (s.suggestion_type.clone(), normalize_text(&s.content));
        match seen.get_mut(&key) {
            Some(existing) => {
                // 合并：support_count 相加，confidence 取 max（仲裁路径下的确定性规则）
                existing.support_count = existing.support_count.saturating_add(s.support_count);
                if s.confidence > existing.confidence {
                    existing.confidence = s.confidence;
                    existing.content = s.content;
                }
            }
            None => {
                seen.insert(key, s);
            }
        }
    }
    seen.into_values().collect()
}

fn choose_recommended_action(
    ctx: &OptimizationContext,
    primary_failure_type: FailureType,
    unified_suggestions: &[UnifiedSuggestion],
    has_conflicts: bool,
) -> (
    crate::domain::models::RecommendedAction,
    HashMap<String, serde_json::Value>,
) {
    use crate::domain::models::RecommendedAction;

    let mut extra: HashMap<String, serde_json::Value> = HashMap::new();

    let (low, high) = (
        ctx.config.evaluator.confidence_low_threshold,
        ctx.config.evaluator.confidence_high_threshold,
    );
    let max_conf = unified_suggestions
        .iter()
        .map(|s| s.confidence)
        .fold(0.0f64, |a, b| a.max(b));
    extra.insert(
        "confidence_gate".to_string(),
        serde_json::json!({
            "low_threshold": low,
            "high_threshold": high,
            "max_confidence": max_conf,
        }),
    );

    // 低置信度：直接请求人工介入（避免自动策略导致回归/震荡）
    if max_conf < low {
        extra.insert(
            "strategy_reason".to_string(),
            serde_json::json!("low_confidence"),
        );
        return (
            RecommendedAction::RequestHumanIntervention {
                reason: "建议置信度低于 low_threshold".to_string(),
            },
            extra,
        );
    }

    // 冲突存在：优先请求人工介入（仲裁结果仅用于展示/提示，不自动硬选）
    if has_conflicts {
        extra.insert(
            "strategy_reason".to_string(),
            serde_json::json!("has_conflicts"),
        );
        return (
            RecommendedAction::RequestHumanIntervention {
                reason: "检测到互斥建议，需要人工确认".to_string(),
            },
            extra,
        );
    }

    // 策略切换（MVP：无提升且达到阈值 → InjectDiversity）
    if let (Ok(cur), Ok(best)) = (
        read_stats(ctx, EXT_CURRENT_PROMPT_STATS),
        read_stats(ctx, EXT_BEST_CANDIDATE_STATS),
    ) {
        let best_is_better = is_better_stats(best, cur);
        extra.insert(
            "best_is_better".to_string(),
            serde_json::Value::Bool(best_is_better),
        );
        if !best_is_better {
            let threshold = ctx.config.iteration.diversity_inject_after;
            let consecutive = read_optional_u32(ctx, EXT_CONSECUTIVE_NO_IMPROVEMENT);
            match consecutive {
                Some(current) => {
                    extra.insert(
                        "strategy_reason".to_string(),
                        serde_json::json!("no_improvement_consecutive_gate"),
                    );
                    extra.insert(
                        "diversity_injection_gate".to_string(),
                        serde_json::json!({
                            "source": "extensions",
                            "threshold": threshold,
                            "current": current,
                        }),
                    );
                    if current >= threshold {
                        extra.insert(
                            "strategy_reason".to_string(),
                            serde_json::json!("no_improvement_and_consecutive_threshold_reached"),
                        );
                        return (RecommendedAction::InjectDiversity, extra);
                    }
                }
                None => {
                    // 兼容：若编排层尚未注入连续计数，保持既有近似（iteration >= threshold）但必须可诊断。
                    extra.insert(
                        "strategy_reason".to_string(),
                        serde_json::json!(
                            "missing_consecutive_no_improvement_fallback_to_iteration"
                        ),
                    );
                    extra.insert(
                        "diversity_injection_gate".to_string(),
                        serde_json::json!({
                            "source": "fallback_iteration",
                            "threshold": threshold,
                            "iteration": ctx.iteration,
                        }),
                    );
                    if ctx.iteration >= threshold {
                        return (RecommendedAction::InjectDiversity, extra);
                    }
                }
            }
        }
    } else {
        extra.insert(
            "strategy_reason".to_string(),
            serde_json::json!("missing_candidate_stats"),
        );
    }

    // 失败类型 → 默认行动（确定性）
    let mut action = match primary_failure_type {
        FailureType::RuleIncomplete | FailureType::RuleIncorrect => {
            RecommendedAction::UpdateRulesAndRegenerate
        }
        FailureType::ExpressionIssue | FailureType::EdgeCase => RecommendedAction::RefineExpression,
        FailureType::Undetermined => RecommendedAction::RequestHumanIntervention {
            reason: "失败类型无法判定（undetermined）".to_string(),
        },
    };

    // 置信度门控：中等置信度时禁止规则层建议驱动（只允许表达层）
    if max_conf < high {
        let has_rule_level = unified_suggestions.iter().any(|s| {
            matches!(
                s.suggestion_type,
                SuggestionType::AddRule | SuggestionType::ModifyRule | SuggestionType::RemoveRule
            )
        });
        if has_rule_level {
            extra.insert(
                "strategy_reason".to_string(),
                serde_json::json!("confidence_mid_expression_only"),
            );
            action = RecommendedAction::RefineExpression;
        }
    }

    (action, extra)
}

fn sanitize_confidence(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    v.clamp(0.0, 1.0)
}

fn normalize_text(s: &str) -> String {
    let mut out = String::new();
    let mut last_was_ws = false;
    for ch in s.trim().chars() {
        if ch.is_whitespace() {
            if !last_was_ws {
                out.push(' ');
                last_was_ws = true;
            }
        } else {
            out.push(ch);
            last_was_ws = false;
        }
    }
    out
}

fn suggestion_type_rank(t: &SuggestionType) -> u32 {
    // 稳定排序：规则层优先于表达层，便于在 UI 中“先看到结构性改动”。
    match t {
        SuggestionType::AddRule => 10,
        SuggestionType::ModifyRule => 11,
        SuggestionType::RemoveRule => 12,
        SuggestionType::ChangeFormat => 20,
        SuggestionType::Rephrase => 21,
        SuggestionType::AddExample => 22,
        SuggestionType::AddConstraint => 23,
    }
}

fn read_stats(ctx: &OptimizationContext, key: &str) -> Result<CandidateStats, AggregatorError> {
    let v = ctx.extensions.get(key).ok_or_else(|| {
        AggregatorError::InvalidReflections(format!("ctx.extensions 缺少必需字段：{key:?}"))
    })?;
    serde_json::from_value(v.clone())
        .map_err(|e| AggregatorError::InvalidReflections(format!("{key} 反序列化失败：{e}")))
}

fn is_better_stats(best: CandidateStats, current: CandidateStats) -> bool {
    if best.pass_rate > current.pass_rate + METRIC_EPS {
        return true;
    }
    (best.pass_rate - current.pass_rate).abs() <= METRIC_EPS
        && best.mean_score > current.mean_score + METRIC_EPS
}

fn read_optional_u32(ctx: &OptimizationContext, key: &str) -> Option<u32> {
    let v = ctx.extensions.get(key)?;
    let n = v.as_u64()?;
    u32::try_from(n).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::FeedbackAggregator;
    use crate::domain::models::{EvaluationResult, Severity};
    use crate::domain::types::{ExecutionTargetConfig, OptimizationConfig, OptimizationContext};
    use std::collections::HashMap;

    fn base_ctx_with_evals() -> OptimizationContext {
        let mut ctx = OptimizationContext {
            task_id: "t1".to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: "p".to_string(),
            rule_system: crate::domain::models::RuleSystem {
                rules: vec![],
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 0,
            },
            iteration: 1,
            state: crate::domain::models::IterationState::Reflecting,
            run_control_state: Default::default(),
            test_cases: vec![],
            config: OptimizationConfig::default(),
            checkpoints: vec![],
            extensions: HashMap::new(),
        };

        let ev = EvaluationResult {
            passed: false,
            score: 0.0,
            dimensions: HashMap::new(),
            failure_points: vec![crate::domain::models::FailurePoint {
                dimension: "format".to_string(),
                description: "bad".to_string(),
                severity: Severity::Major,
                expected: None,
                actual: None,
            }],
            evaluator_type: "e".to_string(),
            confidence: Some(0.9),
            reasoning: None,
            diversity_analysis: None,
            extra: HashMap::new(),
        };
        let mut map = HashMap::new();
        map.insert("tc1".to_string(), ev);
        ctx.extensions.insert(
            EXT_EVALUATIONS_BY_TEST_CASE_ID.to_string(),
            serde_json::to_value(map).unwrap(),
        );

        ctx
    }

    #[tokio::test]
    async fn aggregate_empty_reflections_returns_error() {
        let agg = DefaultFeedbackAggregator;
        let ctx = base_ctx_with_evals();
        let err = agg.aggregate(&ctx, &[]).await.unwrap_err();
        assert!(matches!(err, AggregatorError::InvalidReflections(_)));
    }

    #[tokio::test]
    async fn aggregate_all_failed_test_case_ids_empty_returns_error() {
        let agg = DefaultFeedbackAggregator;
        let ctx = base_ctx_with_evals();

        let rr = ReflectionResult {
            failure_type: FailureType::ExpressionIssue,
            analysis: "a".to_string(),
            root_cause: "r".to_string(),
            suggestions: vec![Suggestion {
                suggestion_type: SuggestionType::Rephrase,
                content: "rephrase".to_string(),
                confidence: 0.9,
                expected_impact: None,
            }],
            failed_test_case_ids: vec![],
            related_rule_ids: vec![],
            evaluation_ref: None,
            extra: HashMap::new(),
        };

        let err = agg.aggregate(&ctx, &[rr]).await.unwrap_err();
        assert!(matches!(err, AggregatorError::InvalidReflections(_)));
    }

    #[tokio::test]
    async fn arbitrate_empty_conflicts_returns_error() {
        let agg = DefaultFeedbackAggregator;
        let ctx = base_ctx_with_evals();
        let err = agg.arbitrate(&ctx, &[]).await.unwrap_err();
        assert!(matches!(err, AggregatorError::ArbitrationFailed(_)));
    }

    #[tokio::test]
    async fn arbitrate_equal_confidence_keeps_all() {
        let agg = DefaultFeedbackAggregator;
        let ctx = base_ctx_with_evals();
        let conflicts = vec![SuggestionConflict {
            suggestion_a: Suggestion {
                suggestion_type: SuggestionType::AddRule,
                content: "rule: X".to_string(),
                confidence: 0.5,
                expected_impact: None,
            },
            suggestion_b: Suggestion {
                suggestion_type: SuggestionType::RemoveRule,
                content: "rule: X".to_string(),
                confidence: 0.5,
                expected_impact: None,
            },
            conflict_type: ConflictType::DirectContradiction,
            description: "d".to_string(),
        }];

        let out = agg.arbitrate(&ctx, &conflicts).await.unwrap();
        assert_eq!(out.method, ArbitrationMethod::Voting);
        assert_eq!(out.chosen_suggestions.len(), 2);
        assert!(
            out.chosen_suggestions
                .iter()
                .any(|s| s.suggestion_type == SuggestionType::AddRule && s.content == "rule: X")
        );
        assert!(
            out.chosen_suggestions
                .iter()
                .any(|s| s.suggestion_type == SuggestionType::RemoveRule && s.content == "rule: X")
        );
    }

    #[tokio::test]
    async fn confidence_gate_low_threshold_is_inclusive() {
        let agg = DefaultFeedbackAggregator;
        let mut ctx = base_ctx_with_evals();
        ctx.config.evaluator.confidence_low_threshold = 0.5;
        ctx.config.evaluator.confidence_high_threshold = 0.8;

        let rr = ReflectionResult {
            failure_type: FailureType::ExpressionIssue,
            analysis: "a".to_string(),
            root_cause: "r".to_string(),
            suggestions: vec![Suggestion {
                suggestion_type: SuggestionType::Rephrase,
                content: "rephrase".to_string(),
                confidence: 0.5,
                expected_impact: None,
            }],
            failed_test_case_ids: vec!["tc1".to_string()],
            related_rule_ids: vec![],
            evaluation_ref: None,
            extra: HashMap::new(),
        };

        let out = agg.aggregate(&ctx, &[rr]).await.unwrap();
        assert!(!matches!(
            out.recommended_action,
            crate::domain::models::RecommendedAction::RequestHumanIntervention { .. }
        ));
    }

    #[tokio::test]
    async fn confidence_gate_high_threshold_is_inclusive_for_rule_level_action() {
        let agg = DefaultFeedbackAggregator;
        let mut ctx = base_ctx_with_evals();
        ctx.config.evaluator.confidence_low_threshold = 0.5;
        ctx.config.evaluator.confidence_high_threshold = 0.8;

        let rr = ReflectionResult {
            failure_type: FailureType::RuleIncorrect,
            analysis: "a".to_string(),
            root_cause: "r".to_string(),
            suggestions: vec![Suggestion {
                suggestion_type: SuggestionType::AddRule,
                content: "rule: Y".to_string(),
                confidence: 0.8,
                expected_impact: None,
            }],
            failed_test_case_ids: vec!["tc1".to_string()],
            related_rule_ids: vec![],
            evaluation_ref: None,
            extra: HashMap::new(),
        };

        let out = agg.aggregate(&ctx, &[rr]).await.unwrap();
        assert!(matches!(
            out.recommended_action,
            crate::domain::models::RecommendedAction::UpdateRulesAndRegenerate
        ));
    }

    #[tokio::test]
    async fn aggregate_detects_conflict_add_vs_remove() {
        let agg = DefaultFeedbackAggregator;
        let ctx = base_ctx_with_evals();

        let rr = ReflectionResult {
            failure_type: FailureType::RuleIncomplete,
            analysis: "a".to_string(),
            root_cause: "r".to_string(),
            suggestions: vec![
                Suggestion {
                    suggestion_type: SuggestionType::AddRule,
                    content: "rule: X".to_string(),
                    confidence: 0.9,
                    expected_impact: None,
                },
                Suggestion {
                    suggestion_type: SuggestionType::RemoveRule,
                    content: "rule: X".to_string(),
                    confidence: 0.8,
                    expected_impact: None,
                },
            ],
            failed_test_case_ids: vec!["tc1".to_string()],
            related_rule_ids: vec![],
            evaluation_ref: None,
            extra: HashMap::new(),
        };

        let out = agg.aggregate(&ctx, &[rr]).await.unwrap();
        assert!(out.has_conflicts);
        assert!(!out.conflicts.is_empty());
        match out.recommended_action {
            crate::domain::models::RecommendedAction::RequestHumanIntervention { .. } => {}
            other => panic!("expected RequestHumanIntervention, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn aggregate_recommends_inject_diversity_when_no_improvement_and_threshold_reached() {
        let agg = DefaultFeedbackAggregator;
        let mut ctx = base_ctx_with_evals();
        ctx.config.iteration.diversity_inject_after = 3;
        ctx.extensions.insert(
            EXT_CONSECUTIVE_NO_IMPROVEMENT.to_string(),
            serde_json::json!(3),
        );

        ctx.extensions.insert(
            EXT_CURRENT_PROMPT_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.9, "mean_score": 0.8 }),
        );
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.9, "mean_score": 0.7 }),
        );

        let rr = ReflectionResult {
            failure_type: FailureType::ExpressionIssue,
            analysis: "a".to_string(),
            root_cause: "r".to_string(),
            suggestions: vec![Suggestion {
                suggestion_type: SuggestionType::Rephrase,
                content: "rephrase".to_string(),
                confidence: 0.9,
                expected_impact: None,
            }],
            failed_test_case_ids: vec!["tc1".to_string()],
            related_rule_ids: vec![],
            evaluation_ref: None,
            extra: HashMap::new(),
        };

        let out = agg.aggregate(&ctx, &[rr]).await.unwrap();
        match out.recommended_action {
            crate::domain::models::RecommendedAction::InjectDiversity => {}
            other => panic!("expected InjectDiversity, got {other:?}"),
        }
        assert_eq!(
            out.extra
                .get("strategy_reason")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "no_improvement_and_consecutive_threshold_reached"
        );
    }

    #[tokio::test]
    async fn aggregate_does_not_inject_diversity_when_consecutive_below_threshold() {
        let agg = DefaultFeedbackAggregator;
        let mut ctx = base_ctx_with_evals();
        ctx.config.iteration.diversity_inject_after = 3;
        ctx.extensions.insert(
            EXT_CONSECUTIVE_NO_IMPROVEMENT.to_string(),
            serde_json::json!(2),
        );
        ctx.extensions.insert(
            EXT_CURRENT_PROMPT_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.9, "mean_score": 0.8 }),
        );
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.9, "mean_score": 0.7 }),
        );

        let rr = ReflectionResult {
            failure_type: FailureType::ExpressionIssue,
            analysis: "a".to_string(),
            root_cause: "r".to_string(),
            suggestions: vec![Suggestion {
                suggestion_type: SuggestionType::Rephrase,
                content: "rephrase".to_string(),
                confidence: 0.9,
                expected_impact: None,
            }],
            failed_test_case_ids: vec!["tc1".to_string()],
            related_rule_ids: vec![],
            evaluation_ref: None,
            extra: HashMap::new(),
        };

        let out = agg.aggregate(&ctx, &[rr]).await.unwrap();
        assert!(!matches!(
            out.recommended_action,
            crate::domain::models::RecommendedAction::InjectDiversity
        ));
        assert_eq!(
            out.extra
                .get("strategy_reason")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "no_improvement_consecutive_gate"
        );
    }

    #[tokio::test]
    async fn aggregate_inject_diversity_fallback_is_diagnostic_when_extension_missing() {
        let agg = DefaultFeedbackAggregator;
        let mut ctx = base_ctx_with_evals();
        ctx.iteration = 3;
        ctx.config.iteration.diversity_inject_after = 3;
        ctx.extensions.insert(
            EXT_CURRENT_PROMPT_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.9, "mean_score": 0.8 }),
        );
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.9, "mean_score": 0.7 }),
        );

        let rr = ReflectionResult {
            failure_type: FailureType::ExpressionIssue,
            analysis: "a".to_string(),
            root_cause: "r".to_string(),
            suggestions: vec![Suggestion {
                suggestion_type: SuggestionType::Rephrase,
                content: "rephrase".to_string(),
                confidence: 0.9,
                expected_impact: None,
            }],
            failed_test_case_ids: vec!["tc1".to_string()],
            related_rule_ids: vec![],
            evaluation_ref: None,
            extra: HashMap::new(),
        };

        let out = agg.aggregate(&ctx, &[rr]).await.unwrap();
        assert!(matches!(
            out.recommended_action,
            crate::domain::models::RecommendedAction::InjectDiversity
        ));
        assert_eq!(
            out.extra
                .get("strategy_reason")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "missing_consecutive_no_improvement_fallback_to_iteration"
        );
        let gate = out
            .extra
            .get("diversity_injection_gate")
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            gate.get("source").and_then(|v| v.as_str()),
            Some("fallback_iteration")
        );
    }
}
