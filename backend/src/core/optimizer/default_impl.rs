use crate::core::evaluator::CandidateRankingEntry;
use crate::core::optimizer::OptimizerError;
use crate::core::optimizer::{
    EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT, EXT_BEST_CANDIDATE_STATS,
    EXT_CANDIDATE_RANKING, EXT_CURRENT_PROMPT_STATS, EXT_RECENT_PRIMARY_SCORES,
    EXTRA_ADOPT_BEST_CANDIDATE,
};
use crate::domain::models::{
    CandidateSource, IterationState, OptimizationResult, PromptCandidate, RecommendedAction,
    TerminationReason, UnifiedReflection,
};
use crate::domain::types::{CandidateStats, EXT_USER_GUIDANCE, METRIC_EPS, OptimizationContext, UserGuidance};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct DefaultOptimizer;

#[async_trait]
impl crate::core::traits::Optimizer for DefaultOptimizer {
    async fn optimize_step(
        &self,
        ctx: &OptimizationContext,
        unified_reflection: &UnifiedReflection,
    ) -> Result<OptimizationResult, OptimizerError> {
        let ranking = read_required::<Vec<CandidateRankingEntry>>(ctx, EXT_CANDIDATE_RANKING)?;
        if ranking.is_empty() {
            return Err(OptimizerError::StepFailed(format!(
                "{EXT_CANDIDATE_RANKING} 为空"
            )));
        }

        let best_candidate_index = read_best_candidate_index(ctx)?;
        let best_candidate_prompt = read_required::<String>(ctx, EXT_BEST_CANDIDATE_PROMPT)?;
        let current_stats = read_required::<CandidateStats>(ctx, EXT_CURRENT_PROMPT_STATS)?;
        let best_stats = read_required::<CandidateStats>(ctx, EXT_BEST_CANDIDATE_STATS)?;

        // 轻量一致性校验：best_candidate_index 应与 ranking 第一名一致（避免口径漂移/错配）。
        if let Some(top) = ranking.first() {
            if top.candidate_index != best_candidate_index {
                return Err(OptimizerError::InvalidState(format!(
                    "{EXT_BEST_CANDIDATE_INDEX}={best_candidate_index} 与 {EXT_CANDIDATE_RANKING}[0].candidate_index={top_idx} 不一致",
                    top_idx = top.candidate_index
                )));
            }
        }

        let best_better = is_better(best_stats, current_stats);

        let mut extra: HashMap<String, serde_json::Value> = HashMap::new();
        extra.insert(
            EXTRA_ADOPT_BEST_CANDIDATE.to_string(),
            serde_json::Value::Bool(best_better),
        );
        extra.insert(
            "current_prompt_stats".to_string(),
            serde_json::to_value(current_stats).unwrap_or(serde_json::Value::Null),
        );
        extra.insert(
            "best_candidate_stats".to_string(),
            serde_json::to_value(best_stats).unwrap_or(serde_json::Value::Null),
        );
        extra.insert(
            "best_candidate_index".to_string(),
            serde_json::Value::Number(serde_json::Number::from(best_candidate_index as u64)),
        );
        if let Some(guidance) = read_optional_user_guidance(ctx) {
            let guidance_id = guidance.id.clone();
            let guidance_preview = guidance.content_preview();
            extra.insert(
                "user_guidance_id".to_string(),
                serde_json::Value::String(guidance_id),
            );
            extra.insert(
                "user_guidance_preview".to_string(),
                serde_json::Value::String(guidance_preview),
            );
        }

        let primary_source = candidate_source_for_action(&unified_reflection.recommended_action);

        let primary_content = if best_better {
            best_candidate_prompt
        } else {
            ctx.current_prompt.clone()
        };

        let primary_score = combined_score(if best_better {
            best_stats
        } else {
            current_stats
        });

        let improvement_summary = if best_better {
            Some(format!(
                "采用 best candidate（index={best_candidate_index}）作为下一轮 current_prompt：pass_rate {cur_pr:.3}→{best_pr:.3}，mean_score {cur_ms:.3}→{best_ms:.3}",
                cur_pr = current_stats.pass_rate,
                best_pr = best_stats.pass_rate,
                cur_ms = current_stats.mean_score,
                best_ms = best_stats.mean_score
            ))
        } else {
            Some(format!(
                "best candidate 未优于当前 Prompt：保持 current_prompt；best(pass_rate={best_pr:.3},mean_score={best_ms:.3}) vs current(pass_rate={cur_pr:.3},mean_score={cur_ms:.3})",
                best_pr = best_stats.pass_rate,
                best_ms = best_stats.mean_score,
                cur_pr = current_stats.pass_rate,
                cur_ms = current_stats.mean_score
            ))
        };

        let (mut should_terminate, mut termination_reason) =
            termination_from_state(ctx, unified_reflection, best_stats.pass_rate);

        // 震荡/停滞检测（可选）：编排层若注入 recent_primary_scores，则 optimize_step 也能产出一致终止信号。
        if !should_terminate {
            if let Ok(Some(mut scores)) = read_optional::<Vec<f64>>(ctx, EXT_RECENT_PRIMARY_SCORES)
            {
                scores.push(primary_score);
                if oscillation_detected_scores(ctx, &scores) {
                    should_terminate = true;
                    termination_reason = Some(TerminationReason::OscillationDetected);
                    extra.insert(
                        "termination_source".to_string(),
                        serde_json::json!("oscillation_from_extensions"),
                    );
                }
            }
        }

        Ok(OptimizationResult {
            primary: PromptCandidate {
                id: if best_better {
                    format!("candidate:{best_candidate_index}")
                } else {
                    "current".to_string()
                },
                content: primary_content,
                score: primary_score,
                source: primary_source,
                failure_fingerprints: vec![format!(
                    "primary_failure_type:{:?}",
                    unified_reflection.primary_failure_type
                )],
            },
            alternatives: Vec::new(),
            should_terminate,
            termination_reason,
            iteration: ctx.iteration,
            improvement_summary,
            extra,
        })
    }

    fn should_terminate(
        &self,
        ctx: &OptimizationContext,
        history: &[OptimizationResult],
    ) -> Option<TerminationReason> {
        let best_stats = read_required::<CandidateStats>(ctx, EXT_BEST_CANDIDATE_STATS).ok()?;
        let best_pass_rate = best_stats.pass_rate;

        // 1) AllTestsPassed / PassThresholdReached / MaxIterationsReached / UserStopped
        if let Some(reason) = base_termination_reason(ctx, best_pass_rate) {
            return Some(reason);
        }

        // 2) OscillationDetected（仅当 action=Stop；否则由编排层继续推进）
        let scores = history.iter().map(|r| r.primary.score).collect::<Vec<_>>();
        if oscillation_detected_scores(ctx, &scores) {
            return Some(TerminationReason::OscillationDetected);
        }

        None
    }

    fn name(&self) -> &str {
        "default_optimizer"
    }
}

fn read_best_candidate_index(ctx: &OptimizationContext) -> Result<usize, OptimizerError> {
    let v = ctx
        .extensions
        .get(EXT_BEST_CANDIDATE_INDEX)
        .ok_or_else(|| {
            OptimizerError::InvalidState(format!(
                "ctx.extensions 缺少必需字段：{key:?}",
                key = EXT_BEST_CANDIDATE_INDEX
            ))
        })?;

    match v {
        serde_json::Value::Number(n) => n.as_u64().map(|x| x as usize).ok_or_else(|| {
            OptimizerError::InvalidState(format!("{EXT_BEST_CANDIDATE_INDEX} 不是合法的非负整数"))
        }),
        _ => Err(OptimizerError::InvalidState(format!(
            "{EXT_BEST_CANDIDATE_INDEX} 类型不为 number"
        ))),
    }
}

fn read_required<T: serde::de::DeserializeOwned>(
    ctx: &OptimizationContext,
    key: &str,
) -> Result<T, OptimizerError> {
    let v = ctx.extensions.get(key).ok_or_else(|| {
        OptimizerError::InvalidState(format!("ctx.extensions 缺少必需字段：{key:?}"))
    })?;
    serde_json::from_value(v.clone())
        .map_err(|e| OptimizerError::InvalidState(format!("{key} 反序列化失败：{e}")))
}

fn is_better(best: CandidateStats, current: CandidateStats) -> bool {
    if best.pass_rate > current.pass_rate + METRIC_EPS {
        return true;
    }
    if approx_eq(best.pass_rate, current.pass_rate)
        && best.mean_score > current.mean_score + METRIC_EPS
    {
        return true;
    }
    false
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= METRIC_EPS
}

fn combined_score(stats: CandidateStats) -> f64 {
    // 让 score 与“通过率优先，其次均分”保持一致的单调性。
    let pr = clamp_01(stats.pass_rate);
    let ms = clamp_01(stats.mean_score);
    (pr * 0.7 + ms * 0.3).clamp(0.0, 1.0)
}

fn clamp_01(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    v.clamp(0.0, 1.0)
}

fn candidate_source_for_action(action: &RecommendedAction) -> CandidateSource {
    match action {
        RecommendedAction::UpdateRulesAndRegenerate => CandidateSource::RuleSystemUpdate,
        RecommendedAction::RefineExpression => CandidateSource::ExpressionRefinement,
        RecommendedAction::InjectDiversity => CandidateSource::DiversityInjection,
        RecommendedAction::RequestHumanIntervention { .. } => CandidateSource::ManualEdit,
        RecommendedAction::Terminate { .. } => CandidateSource::ExpressionRefinement,
    }
}

fn termination_from_state(
    ctx: &OptimizationContext,
    unified_reflection: &UnifiedReflection,
    best_pass_rate: f64,
) -> (bool, Option<TerminationReason>) {
    // 优先级（MUST）：
    // 1. AllTestsPassed
    // 2. PassThresholdReached
    // 3. MaxIterationsReached
    // 4. OscillationDetected (由 should_terminate 处理)
    // 5. UserStopped
    // 6. HumanInterventionRequired

    if let Some(reason) = base_termination_reason(ctx, best_pass_rate) {
        return (true, Some(reason));
    }

    if let RecommendedAction::RequestHumanIntervention { reason } =
        &unified_reflection.recommended_action
    {
        return (
            true,
            Some(TerminationReason::HumanInterventionRequired {
                reason: reason.clone(),
            }),
        );
    }

    if let RecommendedAction::Terminate { reason } = &unified_reflection.recommended_action {
        return (true, Some(reason.clone()));
    }

    (false, None)
}

fn base_termination_reason(
    ctx: &OptimizationContext,
    best_pass_rate: f64,
) -> Option<TerminationReason> {
    if approx_eq(best_pass_rate, 1.0) {
        return Some(TerminationReason::AllTestsPassed);
    }

    let threshold = ctx.config.iteration.pass_threshold;
    if best_pass_rate >= threshold {
        return Some(TerminationReason::PassThresholdReached {
            threshold,
            actual: best_pass_rate,
        });
    }

    let max = ctx.config.iteration.max_iterations;
    if ctx.iteration >= max {
        return Some(TerminationReason::MaxIterationsReached { max });
    }

    if ctx.state == IterationState::UserStopped {
        return Some(TerminationReason::UserStopped);
    }

    None
}

fn oscillation_detected_scores(ctx: &OptimizationContext, scores: &[f64]) -> bool {
    let threshold = ctx.config.oscillation.threshold as usize;
    if threshold == 0 || scores.len() < threshold + 1 {
        return false;
    }

    let action = &ctx.config.oscillation.action;
    if !matches!(action, crate::domain::types::OscillationAction::Stop) {
        return false;
    }

    // “停滞/震荡”最小判定：最近 threshold+1 轮没有出现严格提升（primary.score）
    // - 以 last_n 的 max 为基准，如果最后一轮 <= 之前历史最优（在 last_n 内）则视为停滞。
    let recent = &scores[(scores.len() - (threshold + 1))..];
    let mut best_before_last = 0.0f64;
    for s in &recent[..recent.len().saturating_sub(1)] {
        best_before_last = best_before_last.max(*s);
    }
    recent
        .last()
        .map(|s| *s <= best_before_last + METRIC_EPS)
        .unwrap_or(false)
}

fn read_optional<T: serde::de::DeserializeOwned>(
    ctx: &OptimizationContext,
    key: &str,
) -> Result<Option<T>, OptimizerError> {
    let Some(v) = ctx.extensions.get(key) else {
        return Ok(None);
    };
    serde_json::from_value(v.clone())
        .map(Some)
        .map_err(|e| OptimizerError::InvalidState(format!("{key} 反序列化失败：{e}")))
}

fn read_optional_user_guidance(ctx: &OptimizationContext) -> Option<UserGuidance> {
    ctx.extensions
        .get(EXT_USER_GUIDANCE)
        .and_then(|v| serde_json::from_value::<UserGuidance>(v.clone()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::Optimizer;
    use crate::domain::models::{FailureType, UnifiedReflection};
    use crate::domain::types::{ExecutionTargetConfig, OptimizationConfig, OptimizationContext};

    fn base_ctx() -> OptimizationContext {
        OptimizationContext {
            task_id: "t1".to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: "p0".to_string(),
            rule_system: crate::domain::models::RuleSystem {
                rules: vec![],
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 0,
            },
            iteration: 1,
            state: IterationState::Reflecting,
            run_control_state: Default::default(),
            test_cases: vec![],
            config: OptimizationConfig::default(),
            checkpoints: vec![],
            extensions: HashMap::new(),
        }
    }

    fn unified(action: RecommendedAction) -> UnifiedReflection {
        UnifiedReflection {
            primary_failure_type: FailureType::ExpressionIssue,
            unified_suggestions: vec![],
            has_conflicts: false,
            conflicts: vec![],
            arbitration_result: None,
            source_count: 1,
            failure_type_distribution: HashMap::new(),
            recommended_action: action,
            extra: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn optimize_step_sets_all_tests_passed_priority() {
        let opt = DefaultOptimizer;
        let mut ctx = base_ctx();
        ctx.extensions.insert(
            EXT_CANDIDATE_RANKING.to_string(),
            serde_json::json!([{ "candidate_index": 0, "pass_rate": 1.0, "mean_score": 1.0 }]),
        );
        ctx.extensions
            .insert(EXT_BEST_CANDIDATE_INDEX.to_string(), serde_json::json!(0));
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_PROMPT.to_string(),
            serde_json::json!("p1"),
        );
        ctx.extensions.insert(
            EXT_CURRENT_PROMPT_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.9, "mean_score": 0.9 }),
        );
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_STATS.to_string(),
            serde_json::json!({ "pass_rate": 1.0, "mean_score": 1.0 }),
        );

        let r = opt
            .optimize_step(&ctx, &unified(RecommendedAction::RefineExpression))
            .await
            .unwrap();
        assert!(r.should_terminate);
        assert!(matches!(
            r.termination_reason,
            Some(TerminationReason::AllTestsPassed)
        ));
    }

    #[tokio::test]
    async fn optimize_step_can_terminate_on_oscillation_when_scores_injected() {
        let opt = DefaultOptimizer;
        let mut ctx = base_ctx();
        ctx.config.iteration.pass_threshold = 0.99;
        ctx.config.oscillation.threshold = 1;
        ctx.config.oscillation.action = crate::domain::types::OscillationAction::Stop;

        ctx.extensions.insert(
            EXT_CANDIDATE_RANKING.to_string(),
            serde_json::json!([{ "candidate_index": 0, "pass_rate": 0.6, "mean_score": 0.6 }]),
        );
        ctx.extensions
            .insert(EXT_BEST_CANDIDATE_INDEX.to_string(), serde_json::json!(0));
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_PROMPT.to_string(),
            serde_json::json!("p1"),
        );
        ctx.extensions.insert(
            EXT_CURRENT_PROMPT_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.7, "mean_score": 0.7 }),
        );
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.6, "mean_score": 0.6 }),
        );
        ctx.extensions.insert(
            EXT_RECENT_PRIMARY_SCORES.to_string(),
            serde_json::json!([0.7]),
        );

        let r = opt
            .optimize_step(&ctx, &unified(RecommendedAction::RefineExpression))
            .await
            .unwrap();
        assert!(r.should_terminate);
        assert!(matches!(
            r.termination_reason,
            Some(TerminationReason::OscillationDetected)
        ));
        assert_eq!(
            r.extra
                .get("termination_source")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "oscillation_from_extensions"
        );
    }

    #[test]
    fn should_terminate_pass_threshold_reached() {
        let opt = DefaultOptimizer;
        let mut ctx = base_ctx();
        ctx.config.iteration.pass_threshold = 0.95;
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.96, "mean_score": 0.5 }),
        );
        let reason = opt.should_terminate(&ctx, &[]).unwrap();
        match reason {
            TerminationReason::PassThresholdReached { threshold, actual } => {
                assert!((threshold - 0.95).abs() < METRIC_EPS);
                assert!((actual - 0.96).abs() < METRIC_EPS);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn should_terminate_max_iterations_reached() {
        let opt = DefaultOptimizer;
        let mut ctx = base_ctx();
        ctx.iteration = 10;
        ctx.config.iteration.max_iterations = 10;
        ctx.config.iteration.pass_threshold = 0.99;
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.5, "mean_score": 0.5 }),
        );
        assert!(matches!(
            opt.should_terminate(&ctx, &[]),
            Some(TerminationReason::MaxIterationsReached { max: 10 })
        ));
    }

    #[test]
    fn should_terminate_oscillation_detected_when_action_stop() {
        let opt = DefaultOptimizer;
        let mut ctx = base_ctx();
        ctx.config.oscillation.threshold = 1;
        ctx.config.oscillation.action = crate::domain::types::OscillationAction::Stop;
        ctx.config.iteration.pass_threshold = 0.99;
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.5, "mean_score": 0.5 }),
        );

        let h = vec![
            OptimizationResult {
                primary: PromptCandidate {
                    id: "a".to_string(),
                    content: "p".to_string(),
                    score: 0.5,
                    source: CandidateSource::ExpressionRefinement,
                    failure_fingerprints: vec![],
                },
                alternatives: vec![],
                should_terminate: false,
                termination_reason: None,
                iteration: 1,
                improvement_summary: None,
                extra: HashMap::new(),
            },
            OptimizationResult {
                primary: PromptCandidate {
                    id: "b".to_string(),
                    content: "p".to_string(),
                    score: 0.5,
                    source: CandidateSource::ExpressionRefinement,
                    failure_fingerprints: vec![],
                },
                alternatives: vec![],
                should_terminate: false,
                termination_reason: None,
                iteration: 2,
                improvement_summary: None,
                extra: HashMap::new(),
            },
        ];

        assert!(matches!(
            opt.should_terminate(&ctx, &h),
            Some(TerminationReason::OscillationDetected)
        ));
    }

    #[tokio::test]
    async fn optimize_step_includes_user_guidance_preview_in_extra() {
        let opt = DefaultOptimizer;
        let mut ctx = base_ctx();
        ctx.extensions.insert(
            EXT_CANDIDATE_RANKING.to_string(),
            serde_json::json!([{ "candidate_index": 0, "pass_rate": 1.0, "mean_score": 1.0 }]),
        );
        ctx.extensions
            .insert(EXT_BEST_CANDIDATE_INDEX.to_string(), serde_json::json!(0));
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_PROMPT.to_string(),
            serde_json::json!("p1"),
        );
        ctx.extensions.insert(
            EXT_CURRENT_PROMPT_STATS.to_string(),
            serde_json::json!({ "pass_rate": 0.9, "mean_score": 0.9 }),
        );
        ctx.extensions.insert(
            EXT_BEST_CANDIDATE_STATS.to_string(),
            serde_json::json!({ "pass_rate": 1.0, "mean_score": 1.0 }),
        );

        let guidance = crate::domain::types::UserGuidance::new("请优先保证输出结构");
        ctx.extensions.insert(
            EXT_USER_GUIDANCE.to_string(),
            serde_json::to_value(&guidance).unwrap(),
        );

        let out = opt
            .optimize_step(&ctx, &unified(RecommendedAction::RefineExpression))
            .await
            .unwrap();

        assert_eq!(
            out.extra
                .get("user_guidance_preview")
                .and_then(|v| v.as_str()),
            Some(guidance.content_preview().as_str())
        );
        assert_eq!(
            out.extra
                .get("user_guidance_id")
                .and_then(|v| v.as_str()),
            Some(guidance.id.as_str())
        );
    }
}
