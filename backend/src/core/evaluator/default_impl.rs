use crate::core::evaluator::{
    EXT_TASK_EVALUATOR_CONFIG, EXTRA_EVALUATOR_FALLBACK_REASON, EXTRA_SELECTED_EVALUATORS,
    EXTRA_THRESHOLDS, EvaluatorError,
};
use crate::core::traits::{Evaluator, TeacherModel};
use crate::domain::models::{
    Constraint, DataSplit, DimensionScore, EvaluationResult,
    EvaluatorConfig as TaskEvaluatorConfig, EvaluatorType, FailurePoint, Severity, TaskReference,
    TestCase,
};
use crate::domain::types::{EXT_USER_GUIDANCE, OptimizationContext, UserGuidance};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Default)]
pub struct DefaultEvaluator {
    teacher_model: Option<Arc<dyn TeacherModel>>,
}

impl DefaultEvaluator {
    pub fn new(teacher_model: Option<Arc<dyn TeacherModel>>) -> Self {
        Self { teacher_model }
    }
}

#[async_trait]
impl Evaluator for DefaultEvaluator {
    async fn evaluate(
        &self,
        ctx: &OptimizationContext,
        test_case: &TestCase,
        output: &str,
    ) -> Result<EvaluationResult, EvaluatorError> {
        if test_case.id.trim().is_empty() {
            return Err(EvaluatorError::InvalidInput(
                "test_case.id 不能为空".to_string(),
            ));
        }

        validate_output_non_empty_if_needed(test_case, output)?;

        let task_cfg = read_task_evaluator_config(ctx)?;
        let eval_cfg = &ctx.config.evaluator;

        if eval_cfg.ensemble_enabled && matches!(task_cfg.evaluator_type, EvaluatorType::Auto) {
            return evaluate_with_ensemble(
                ctx,
                &task_cfg,
                test_case,
                output,
                self.teacher_model.as_ref(),
            )
            .await;
        }

        evaluate_single(
            ctx,
            &task_cfg,
            test_case,
            output,
            self.teacher_model.as_ref(),
        )
        .await
    }

    async fn evaluate_batch(
        &self,
        ctx: &OptimizationContext,
        results: &[(TestCase, String)],
    ) -> Result<Vec<EvaluationResult>, EvaluatorError> {
        if results.is_empty() {
            return Err(EvaluatorError::InvalidInput(
                "results 为空（需要至少 1 条 (TestCase, output)）".to_string(),
            ));
        }

        let allowed_ids: HashSet<&str> = ctx.test_cases.iter().map(|tc| tc.id.as_str()).collect();
        let mut unknown_ids = Vec::new();
        let mut duplicates = Vec::new();
        let mut seen: HashSet<&str> = HashSet::with_capacity(results.len());
        for (idx, (tc, _)) in results.iter().enumerate() {
            if tc.id.trim().is_empty() {
                return Err(EvaluatorError::InvalidInput(format!(
                    "results[{idx}].TestCase.id 不能为空"
                )));
            }
            if !allowed_ids.contains(tc.id.as_str()) {
                unknown_ids.push(tc.id.clone());
            }
            if !seen.insert(tc.id.as_str()) {
                duplicates.push(tc.id.clone());
            }
        }
        if !unknown_ids.is_empty() {
            unknown_ids.sort();
            unknown_ids.dedup();
            return Err(EvaluatorError::InvalidInput(format!(
                "results 中包含 ctx.test_cases 未知 test_case_id：{unknown_ids:?}"
            )));
        }
        if !duplicates.is_empty() {
            duplicates.sort();
            duplicates.dedup();
            return Err(EvaluatorError::InvalidInput(format!(
                "results 中包含重复 test_case_id：{duplicates:?}"
            )));
        }

        let mut out = Vec::with_capacity(results.len());
        for (tc, output) in results {
            let r = self.evaluate(ctx, tc, output).await?;
            out.push(r);
        }

        Ok(out)
    }

    fn name(&self) -> &str {
        "default_evaluator"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitFilter {
    /// 不过滤：用于 `data_split.enabled = false` 的默认行为。
    All,
    /// 仅用于排序/统计：Validation + Unassigned（Holdout 不参与 racing/排序）。
    ValidationAndUnassigned,
}

#[derive(Debug, Clone)]
pub struct EvaluationStats {
    pub passed_count: usize,
    pub total_count: usize,
    pub pass_rate: f64,
    pub mean_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateRankingEntry {
    pub candidate_index: usize,
    pub pass_rate: f64,
    pub mean_score: f64,
}

pub fn rank_candidates(entries: &[CandidateRankingEntry]) -> Vec<CandidateRankingEntry> {
    let mut out = entries.to_vec();
    out.sort_by(|a, b| {
        // pass_rate desc
        let pr_a = sanitize_score(a.pass_rate);
        let pr_b = sanitize_score(b.pass_rate);
        match pr_b.partial_cmp(&pr_a).unwrap_or(std::cmp::Ordering::Equal) {
            std::cmp::Ordering::Equal => {
                // mean_score desc
                let ms_a = sanitize_score(a.mean_score);
                let ms_b = sanitize_score(b.mean_score);
                match ms_b.partial_cmp(&ms_a).unwrap_or(std::cmp::Ordering::Equal) {
                    std::cmp::Ordering::Equal => a.candidate_index.cmp(&b.candidate_index),
                    other => other,
                }
            }
            other => other,
        }
    });
    out
}

fn sanitize_score(v: f64) -> f64 {
    if v.is_finite() { v } else { 0.0 }
}

pub fn build_evaluations_by_test_case_id(
    results: &[(TestCase, String)],
    evaluations: &[EvaluationResult],
) -> Result<HashMap<String, EvaluationResult>, EvaluatorError> {
    if results.len() != evaluations.len() {
        return Err(EvaluatorError::InvalidInput(format!(
            "results 与 evaluations 长度不一致：results_len={} evaluations_len={}",
            results.len(),
            evaluations.len()
        )));
    }

    let mut map = HashMap::with_capacity(results.len());
    for (idx, ((tc, _), ev)) in results.iter().zip(evaluations.iter()).enumerate() {
        if tc.id.trim().is_empty() {
            return Err(EvaluatorError::InvalidInput(format!(
                "results[{idx}].TestCase.id 为空"
            )));
        }
        if map.insert(tc.id.clone(), ev.clone()).is_some() {
            return Err(EvaluatorError::InvalidInput(format!(
                "results 中包含重复 test_case_id：{id:?}",
                id = tc.id
            )));
        }
    }
    Ok(map)
}

pub fn split_filter_for_stats(ctx: &OptimizationContext) -> SplitFilter {
    if ctx.config.data_split.enabled {
        SplitFilter::ValidationAndUnassigned
    } else {
        SplitFilter::All
    }
}

pub fn summarize_for_stats(
    filter: SplitFilter,
    results: &[(TestCase, String)],
    evaluations: &[EvaluationResult],
) -> Result<EvaluationStats, EvaluatorError> {
    if results.len() != evaluations.len() {
        return Err(EvaluatorError::InvalidInput(format!(
            "results 与 evaluations 长度不一致：results_len={} evaluations_len={}",
            results.len(),
            evaluations.len()
        )));
    }

    let mut total = 0usize;
    let mut passed = 0usize;
    let mut score_sum = 0.0f64;

    for i in 0..results.len() {
        let split = results[i].0.split.unwrap_or(DataSplit::Unassigned);
        if !split_allowed(filter, split) {
            continue;
        }
        total += 1;
        if evaluations[i].passed {
            passed += 1;
        }
        score_sum += clamp_01(evaluations[i].score);
    }

    if total == 0 {
        return Err(EvaluatorError::InvalidInput(
            "参与统计的用例数为 0（可能是 split 过滤导致）".to_string(),
        ));
    }

    Ok(EvaluationStats {
        passed_count: passed,
        total_count: total,
        pass_rate: passed as f64 / total as f64,
        mean_score: score_sum / total as f64,
    })
}

fn split_allowed(filter: SplitFilter, split: DataSplit) -> bool {
    match filter {
        SplitFilter::All => true,
        SplitFilter::ValidationAndUnassigned => match split {
            DataSplit::Holdout => false,
            DataSplit::Train => false,
            DataSplit::Validation | DataSplit::Unassigned => true,
        },
    }
}

fn clamp_01(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    v.clamp(0.0, 1.0)
}

fn read_task_evaluator_config(
    ctx: &OptimizationContext,
) -> Result<TaskEvaluatorConfig, EvaluatorError> {
    let v = ctx
        .extensions
        .get(EXT_TASK_EVALUATOR_CONFIG)
        .ok_or_else(|| {
            EvaluatorError::InvalidInput(format!(
                "ctx.extensions 缺少必需字段：{EXT_TASK_EVALUATOR_CONFIG:?}"
            ))
        })?;

    serde_json::from_value(v.clone()).map_err(|e| {
        EvaluatorError::InvalidInput(format!(
            "ctx.extensions[{EXT_TASK_EVALUATOR_CONFIG:?}] 不是合法的 EvaluatorConfig：{e}"
        ))
    })
}

fn validate_output_non_empty_if_needed(
    test_case: &TestCase,
    output: &str,
) -> Result<(), EvaluatorError> {
    let out_blank = output.trim().is_empty();
    if !out_blank {
        return Ok(());
    }

    match &test_case.reference {
        TaskReference::Exact { expected } => {
            if expected.trim().is_empty() {
                return Ok(());
            }
            Err(EvaluatorError::InvalidInput(format!(
                "test_case_id={} output 为空，但 expected 非空",
                test_case.id
            )))
        }
        TaskReference::Constrained { constraints, .. } => {
            if constraints.is_empty() {
                return Ok(());
            }
            Err(EvaluatorError::InvalidInput(format!(
                "test_case_id={} output 为空，但 constraints 非空",
                test_case.id
            )))
        }
        TaskReference::Hybrid {
            exact_parts,
            constraints,
        } => {
            if exact_parts.is_empty() && constraints.is_empty() {
                return Ok(());
            }
            Err(EvaluatorError::InvalidInput(format!(
                "test_case_id={} output 为空，但 exact_parts/constraints 非空",
                test_case.id
            )))
        }
    }
}

async fn evaluate_single(
    ctx: &OptimizationContext,
    task_cfg: &TaskEvaluatorConfig,
    test_case: &TestCase,
    output: &str,
    teacher_model: Option<&Arc<dyn TeacherModel>>,
) -> Result<EvaluationResult, EvaluatorError> {
    let mut thresholds = BTreeMap::<String, serde_json::Value>::new();
    thresholds.insert(
        "pass_threshold".to_string(),
        json!(ctx.config.iteration.pass_threshold),
    );

    let (mut result, selected_evaluators, fallback_reason) = match task_cfg.evaluator_type {
        EvaluatorType::Auto => evaluate_auto(task_cfg, test_case, output).await,
        EvaluatorType::ExactMatch => {
            thresholds.insert(
                "exact_match_case_sensitive".to_string(),
                json!(task_cfg.exact_match.case_sensitive),
            );
            evaluate_exact_match(task_cfg, test_case, output)
                .await
                .map(|r| (r, vec!["exact_match"], None))
        }
        EvaluatorType::ConstraintCheck => {
            thresholds.insert(
                "constraint_check_strict".to_string(),
                json!(task_cfg.constraint_check.strict),
            );
            evaluate_constraint_check(task_cfg, test_case, output)
                .await
                .map(|r| (r, vec!["constraint_check"], None))
        }
        EvaluatorType::SemanticSimilarity => {
            thresholds.insert(
                "semantic_similarity_threshold_percent".to_string(),
                json!(task_cfg.semantic_similarity.threshold_percent),
            );
            evaluate_semantic_similarity(task_cfg, test_case, output)
                .await
                .map(|r| (r, vec!["semantic_similarity"], None))
        }
        EvaluatorType::TeacherModel => {
            let teacher_model = teacher_model.ok_or_else(|| {
                EvaluatorError::ModelFailure(
                    "TeacherModel 未注入（无法执行 TeacherModelEvaluator）".to_string(),
                )
            })?;
            thresholds.insert(
                "llm_judge_samples".to_string(),
                json!(llm_judge_samples(ctx, task_cfg)),
            );
            evaluate_teacher_model(ctx, task_cfg, test_case, output, teacher_model)
                .await
                .map(|r| (r, vec!["teacher_model"], None))
        }
        other => {
            return Err(EvaluatorError::InvalidInput(format!(
                "未支持的 evaluator_type（请使用扩展工厂创建对应 Evaluator）：{other:?}"
            )));
        }
    }?;

    let selected_evaluators: Vec<String> = selected_evaluators
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    // Enrich thresholds based on selected evaluators (for diagnostics).
    if selected_evaluators.iter().any(|s| s == "exact_match") {
        thresholds.insert(
            "exact_match_case_sensitive".to_string(),
            json!(task_cfg.exact_match.case_sensitive),
        );
    }
    if selected_evaluators.iter().any(|s| s == "constraint_check") {
        thresholds.insert(
            "constraint_check_strict".to_string(),
            json!(task_cfg.constraint_check.strict),
        );
    }
    if selected_evaluators
        .iter()
        .any(|s| s == "semantic_similarity")
    {
        thresholds.insert(
            "semantic_similarity_threshold_percent".to_string(),
            json!(task_cfg.semantic_similarity.threshold_percent),
        );
    }
    if selected_evaluators.iter().any(|s| s == "teacher_model") {
        thresholds.insert(
            "llm_judge_samples".to_string(),
            json!(llm_judge_samples(ctx, task_cfg)),
        );
    }

    result.extra.insert(
        EXTRA_SELECTED_EVALUATORS.to_string(),
        json!(selected_evaluators),
    );
    result
        .extra
        .insert(EXTRA_THRESHOLDS.to_string(), json!(thresholds));
    if let Some(reason) = fallback_reason {
        result
            .extra
            .insert(EXTRA_EVALUATOR_FALLBACK_REASON.to_string(), json!(reason));
    }

    Ok(result)
}

async fn evaluate_auto(
    task_cfg: &TaskEvaluatorConfig,
    test_case: &TestCase,
    output: &str,
) -> Result<(EvaluationResult, Vec<&'static str>, Option<String>), EvaluatorError> {
    match &test_case.reference {
        TaskReference::Exact { .. } | TaskReference::Hybrid { .. } => {
            evaluate_exact_match(task_cfg, test_case, output)
                .await
                .map(|r| (r, vec!["exact_match"], None))
        }
        TaskReference::Constrained { .. } => evaluate_constraint_check(task_cfg, test_case, output)
            .await
            .map(|r| (r, vec!["constraint_check"], None)),
    }
}

async fn evaluate_with_ensemble(
    ctx: &OptimizationContext,
    task_cfg: &TaskEvaluatorConfig,
    test_case: &TestCase,
    output: &str,
    teacher_model: Option<&Arc<dyn TeacherModel>>,
) -> Result<EvaluationResult, EvaluatorError> {
    let mut selected = Vec::new();
    let mut thresholds = BTreeMap::<String, serde_json::Value>::new();
    thresholds.insert(
        "pass_threshold".to_string(),
        json!(ctx.config.iteration.pass_threshold),
    );
    thresholds.insert(
        "confidence_high_threshold".to_string(),
        json!(ctx.config.evaluator.confidence_high_threshold),
    );
    thresholds.insert(
        "confidence_low_threshold".to_string(),
        json!(ctx.config.evaluator.confidence_low_threshold),
    );

    let mut parts: Vec<EvaluationResult> = Vec::new();

    // Hard checks: based on TaskReference type.
    let mut fallback_reason: Option<String> = None;
    match &test_case.reference {
        TaskReference::Exact { .. } => {
            selected.push("exact_match");
            thresholds.insert(
                "exact_match_case_sensitive".to_string(),
                json!(task_cfg.exact_match.case_sensitive),
            );
            parts.push(evaluate_exact_match(task_cfg, test_case, output).await?);
        }
        TaskReference::Hybrid { .. } => {
            selected.push("exact_match");
            thresholds.insert(
                "exact_match_case_sensitive".to_string(),
                json!(task_cfg.exact_match.case_sensitive),
            );
            parts.push(evaluate_exact_match(task_cfg, test_case, output).await?);

            selected.push("constraint_check");
            thresholds.insert(
                "constraint_check_strict".to_string(),
                json!(task_cfg.constraint_check.strict),
            );
            parts.push(evaluate_constraint_check(task_cfg, test_case, output).await?);
        }
        TaskReference::Constrained { core_request, .. } => {
            selected.push("constraint_check");
            thresholds.insert(
                "constraint_check_strict".to_string(),
                json!(task_cfg.constraint_check.strict),
            );
            parts.push(evaluate_constraint_check(task_cfg, test_case, output).await?);

            if core_request.is_some() {
                selected.push("semantic_similarity");
                thresholds.insert(
                    "semantic_similarity_threshold_percent".to_string(),
                    json!(task_cfg.semantic_similarity.threshold_percent),
                );
                parts.push(evaluate_semantic_similarity(task_cfg, test_case, output).await?);
            } else {
                fallback_reason = Some("core_request 缺失，跳过 semantic_similarity".to_string());
            }
        }
    }

    if let Some(tm) = teacher_model {
        selected.push("teacher_model");
        thresholds.insert(
            "llm_judge_samples".to_string(),
            json!(llm_judge_samples(ctx, task_cfg)),
        );
        parts.push(evaluate_teacher_model(ctx, task_cfg, test_case, output, tm).await?);
    } else {
        // TeacherModel 是可选依赖：未注入时允许降级，但必须可诊断。
        let reason = "TeacherModel 未注入，跳过 teacher_model".to_string();
        fallback_reason = Some(match fallback_reason {
            Some(existing) => format!("{existing}; {reason}"),
            None => reason,
        });
    }

    let mut aggregated = aggregate_ensemble(
        parts,
        selected,
        thresholds,
        ctx.config.evaluator.hard_checks_weight,
        ctx.config.evaluator.agreement_weight,
        ctx.config.evaluator.variance_penalty,
    )?;

    if let Some(reason) = fallback_reason {
        aggregated
            .extra
            .insert(EXTRA_EVALUATOR_FALLBACK_REASON.to_string(), json!(reason));
    }

    // For ensemble, the final result carries selection metadata in extra.
    // (parts are internal and do not need to duplicate it.)
    Ok(aggregated)
}

fn aggregate_ensemble(
    parts: Vec<EvaluationResult>,
    selected: Vec<&'static str>,
    thresholds: BTreeMap<String, serde_json::Value>,
    hard_checks_weight: f64,
    agreement_weight: f64,
    variance_penalty: f64,
) -> Result<EvaluationResult, EvaluatorError> {
    if parts.is_empty() {
        return Err(EvaluatorError::Internal(
            "EnsembleEvaluator parts 为空（不应发生）".to_string(),
        ));
    }

    let mut dimensions = HashMap::new();
    let mut failure_points = Vec::new();
    let mut scores = Vec::new();

    let mut hard_total = 0usize;
    let mut hard_passed = 0usize;
    for p in &parts {
        for (k, v) in &p.dimensions {
            dimensions.insert(k.clone(), v.clone());
        }
        failure_points.extend(p.failure_points.iter().cloned());
        scores.push(clamp_01(p.score));
        let is_hard = p.evaluator_type == "exact_match" || p.evaluator_type == "constraint_check";
        if is_hard {
            hard_total += 1;
            if p.passed {
                hard_passed += 1;
            }
        }
    }

    let hard_pass_ratio = if hard_total == 0 {
        1.0
    } else {
        hard_passed as f64 / hard_total as f64
    };
    let mean_score = scores.iter().sum::<f64>() / scores.len() as f64;

    let variance = if scores.len() <= 1 {
        0.0
    } else {
        let mu = mean_score;
        scores.iter().map(|s| (s - mu) * (s - mu)).sum::<f64>() / scores.len() as f64
    };

    let agreement_score = 1.0 - clamp_01(variance);
    let confidence = clamp_01(
        clamp_01(hard_checks_weight) * hard_pass_ratio
            + clamp_01(agreement_weight) * agreement_score
            - clamp_01(variance_penalty) * clamp_01(variance),
    );

    let passed = hard_pass_ratio >= 1.0;
    let mut extra = HashMap::new();
    extra.insert(
        EXTRA_SELECTED_EVALUATORS.to_string(),
        json!(selected.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
    );
    extra.insert(EXTRA_THRESHOLDS.to_string(), json!(thresholds));

    Ok(EvaluationResult {
        passed,
        score: clamp_01(if passed { mean_score } else { mean_score * 0.5 }),
        dimensions,
        failure_points,
        evaluator_type: "ensemble".to_string(),
        confidence: Some(confidence),
        reasoning: None,
        diversity_analysis: None,
        extra,
    })
}

async fn evaluate_exact_match(
    task_cfg: &TaskEvaluatorConfig,
    test_case: &TestCase,
    output: &str,
) -> Result<EvaluationResult, EvaluatorError> {
    let case_sensitive = task_cfg.exact_match.case_sensitive;

    let (passed, score, failure_points) = match &test_case.reference {
        TaskReference::Exact { expected } => {
            let ok = compare_text(expected, output, case_sensitive);
            let fps = if ok {
                vec![]
            } else {
                vec![FailurePoint {
                    dimension: "exact_match".to_string(),
                    description: "输出与期望不一致".to_string(),
                    severity: Severity::Major,
                    expected: Some(expected.clone()),
                    actual: Some(output.to_string()),
                }]
            };
            (ok, if ok { 1.0 } else { 0.0 }, fps)
        }
        TaskReference::Hybrid { exact_parts, .. } => {
            if exact_parts.is_empty() {
                (true, 1.0, vec![])
            } else {
                let mut miss = Vec::new();
                let mut hit = 0usize;
                for (k, expected) in exact_parts {
                    if contains_text(output, expected, case_sensitive) {
                        hit += 1;
                    } else {
                        miss.push(FailurePoint {
                            dimension: "exact_part_missing".to_string(),
                            description: format!("缺少 Hybrid.exact_parts[{k}] 对应内容"),
                            severity: Severity::Major,
                            expected: Some(expected.clone()),
                            actual: Some(output.to_string()),
                        });
                    }
                }
                let total = exact_parts.len().max(1);
                let s = hit as f64 / total as f64;
                (miss.is_empty(), s, miss)
            }
        }
        TaskReference::Constrained { .. } => {
            return Err(EvaluatorError::InvalidInput(
                "ExactMatchEvaluator 不支持 TaskReference::Constrained".to_string(),
            ));
        }
    };

    let mut dimensions = HashMap::new();
    dimensions.insert(
        "exact_match".to_string(),
        DimensionScore {
            score,
            passed,
            weight: None,
            details: None,
        },
    );

    Ok(EvaluationResult {
        passed,
        score,
        dimensions,
        failure_points,
        evaluator_type: "exact_match".to_string(),
        confidence: Some(if passed { 1.0 } else { 0.0 }),
        reasoning: None,
        diversity_analysis: None,
        extra: HashMap::new(),
    })
}

async fn evaluate_constraint_check(
    task_cfg: &TaskEvaluatorConfig,
    test_case: &TestCase,
    output: &str,
) -> Result<EvaluationResult, EvaluatorError> {
    let strict = task_cfg.constraint_check.strict;
    let constraints: &[Constraint] = match &test_case.reference {
        TaskReference::Constrained { constraints, .. } => constraints,
        TaskReference::Hybrid { constraints, .. } => constraints,
        TaskReference::Exact { .. } => {
            return Err(EvaluatorError::InvalidInput(
                "ConstraintCheckEvaluator 不支持 TaskReference::Exact".to_string(),
            ));
        }
    };

    if constraints.is_empty() {
        return Ok(EvaluationResult {
            passed: true,
            score: 1.0,
            dimensions: HashMap::new(),
            failure_points: vec![],
            evaluator_type: "constraint_check".to_string(),
            confidence: Some(1.0),
            reasoning: None,
            diversity_analysis: None,
            extra: HashMap::new(),
        });
    }

    let mut dimensions = HashMap::new();
    let mut failure_points = Vec::new();
    let mut passed_count = 0usize;

    for c in constraints {
        let (ok, fp_dim, details) = evaluate_constraint(c, output)?;
        if ok {
            passed_count += 1;
        } else {
            failure_points.push(FailurePoint {
                dimension: fp_dim.to_string(),
                description: details.clone(),
                severity: Severity::Major,
                expected: None,
                actual: Some(output.to_string()),
            });
        }

        dimensions.insert(
            fp_dim.to_string(),
            DimensionScore {
                score: if ok { 1.0 } else { 0.0 },
                passed: ok,
                weight: c.weight,
                details: Some(details),
            },
        );
    }

    let total = constraints.len().max(1);
    let ratio = passed_count as f64 / total as f64;
    let passed = if strict { passed_count == total } else { true };

    Ok(EvaluationResult {
        passed,
        score: ratio,
        dimensions,
        failure_points,
        evaluator_type: "constraint_check".to_string(),
        confidence: Some(1.0),
        reasoning: None,
        diversity_analysis: None,
        extra: HashMap::new(),
    })
}

fn evaluate_constraint(
    c: &Constraint,
    output: &str,
) -> Result<(bool, &'static str, String), EvaluatorError> {
    match c.name.as_str() {
        "length" => {
            let obj = c
                .params
                .as_ref()
                .and_then(|v| v.as_object())
                .ok_or_else(|| {
                    EvaluatorError::InvalidInput("length.params 必须为 object".to_string())
                })?;
            let min = obj
                .get("minChars")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);
            let max = obj
                .get("maxChars")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);
            let len = output.chars().count();
            if let Some(min) = min {
                if len < min {
                    return Ok((
                        false,
                        "length",
                        format!("长度过短：len={len} < minChars={min}"),
                    ));
                }
            }
            if let Some(max) = max {
                if len > max {
                    return Ok((
                        false,
                        "length",
                        format!("长度过长：len={len} > maxChars={max}"),
                    ));
                }
            }
            Ok((true, "length", format!("长度满足要求：len={len}")))
        }
        "must_include" => {
            let obj = c
                .params
                .as_ref()
                .and_then(|v| v.as_object())
                .ok_or_else(|| {
                    EvaluatorError::InvalidInput("must_include.params 必须为 object".to_string())
                })?;
            let keywords = obj
                .get("keywords")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    EvaluatorError::InvalidInput(
                        "must_include.params.keywords 必须为 array".to_string(),
                    )
                })?;
            let mut missing = Vec::new();
            for kw in keywords {
                let s = kw.as_str().ok_or_else(|| {
                    EvaluatorError::InvalidInput(
                        "must_include.params.keywords[*] 必须为 string".to_string(),
                    )
                })?;
                if !output.contains(s) {
                    missing.push(s.to_string());
                }
            }
            if missing.is_empty() {
                Ok((true, "must_include", "必含关键词满足".to_string()))
            } else {
                Ok((
                    false,
                    "must_include",
                    format!("缺少必含关键词：{missing:?}"),
                ))
            }
        }
        "must_exclude" => {
            let obj = c
                .params
                .as_ref()
                .and_then(|v| v.as_object())
                .ok_or_else(|| {
                    EvaluatorError::InvalidInput("must_exclude.params 必须为 object".to_string())
                })?;
            let keywords = obj
                .get("keywords")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    EvaluatorError::InvalidInput(
                        "must_exclude.params.keywords 必须为 array".to_string(),
                    )
                })?;
            let mut found = Vec::new();
            for kw in keywords {
                let s = kw.as_str().ok_or_else(|| {
                    EvaluatorError::InvalidInput(
                        "must_exclude.params.keywords[*] 必须为 string".to_string(),
                    )
                })?;
                if output.contains(s) {
                    found.push(s.to_string());
                }
            }
            if found.is_empty() {
                Ok((true, "must_exclude", "禁止内容未出现".to_string()))
            } else {
                Ok((false, "must_exclude", format!("检测到禁止内容：{found:?}")))
            }
        }
        "format" => {
            let obj = c
                .params
                .as_ref()
                .and_then(|v| v.as_object())
                .ok_or_else(|| {
                    EvaluatorError::InvalidInput("format.params 必须为 object".to_string())
                })?;
            let fmt = obj.get("format").and_then(|v| v.as_str()).ok_or_else(|| {
                EvaluatorError::InvalidInput("format.params.format 必须为 string".to_string())
            })?;
            match fmt {
                "json" => {
                    let ok = serde_json::from_str::<serde_json::Value>(output).is_ok();
                    if ok {
                        Ok((true, "format", "输出为合法 JSON".to_string()))
                    } else {
                        Ok((false, "format", "输出不是合法 JSON".to_string()))
                    }
                }
                "markdown" => {
                    let ok = looks_like_markdown(output);
                    if ok {
                        Ok((true, "format", "输出看起来像 Markdown".to_string()))
                    } else {
                        Ok((false, "format", "输出不满足 Markdown 形态特征".to_string()))
                    }
                }
                "plain_text" => Ok((true, "format", "plain_text 不做强校验（MVP）".to_string())),
                other => Ok((
                    false,
                    "constraint_unknown",
                    format!("未知 format={other:?}（MVP 仅支持 json/markdown/plain_text）"),
                )),
            }
        }
        other => Ok((
            false,
            "constraint_unknown",
            format!(
                "未知约束 name={other:?}（MVP 仅支持 length/must_include/must_exclude/format）"
            ),
        )),
    }
}

fn looks_like_markdown(s: &str) -> bool {
    let t = s.trim();
    if t.contains("```") {
        return true;
    }
    for line in t.lines().take(20) {
        let l = line.trim_start();
        if l.starts_with("#")
            || l.starts_with("- ")
            || l.starts_with("* ")
            || l.starts_with("> ")
            || l.starts_with("1. ")
        {
            return true;
        }
    }
    t.contains("**") || t.contains("_") || t.contains("](")
}

async fn evaluate_semantic_similarity(
    task_cfg: &TaskEvaluatorConfig,
    test_case: &TestCase,
    output: &str,
) -> Result<EvaluationResult, EvaluatorError> {
    let threshold = task_cfg.semantic_similarity.threshold_percent as f64 / 100.0;
    let core_request = match &test_case.reference {
        TaskReference::Constrained { core_request, .. } => core_request.clone(),
        _ => {
            return Err(EvaluatorError::InvalidInput(
                "SemanticSimilarityEvaluator 仅支持 TaskReference::Constrained".to_string(),
            ));
        }
    };

    let core_request = core_request.ok_or_else(|| {
        EvaluatorError::InvalidInput(
            "TaskReference::Constrained.core_request 缺失（但 evaluator_type=SemanticSimilarity）"
                .to_string(),
        )
    })?;

    let score = jaccard_similarity(&tokenize(&core_request), &tokenize(output));
    let passed = score >= threshold;

    let mut dimensions = HashMap::new();
    dimensions.insert(
        "semantic_similarity".to_string(),
        DimensionScore {
            score,
            passed,
            weight: None,
            details: Some(format!("jaccard={score:.3} threshold={threshold:.3}")),
        },
    );

    let failure_points = if passed {
        vec![]
    } else {
        vec![FailurePoint {
            dimension: "semantic_similarity".to_string(),
            description: format!("语义相似度低于阈值：{score:.3} < {threshold:.3}"),
            severity: Severity::Minor,
            expected: Some(core_request),
            actual: Some(output.to_string()),
        }]
    };

    Ok(EvaluationResult {
        passed,
        score: clamp_01(score),
        dimensions,
        failure_points,
        evaluator_type: "semantic_similarity".to_string(),
        confidence: Some(1.0),
        reasoning: None,
        diversity_analysis: None,
        extra: HashMap::new(),
    })
}

fn tokenize(s: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut cur = String::new();
    for ch in s.chars() {
        if ch.is_whitespace() {
            flush_token(&mut out, &mut cur);
            continue;
        }
        let is_cjk = matches!(ch as u32, 0x4E00..=0x9FFF);
        if ch.is_ascii_alphanumeric() || ch.is_alphabetic() || is_cjk {
            cur.extend(ch.to_lowercase());
        } else {
            flush_token(&mut out, &mut cur);
        }
    }
    flush_token(&mut out, &mut cur);
    out
}

fn flush_token(out: &mut HashSet<String>, cur: &mut String) {
    if !cur.is_empty() {
        out.insert(std::mem::take(cur));
    }
}

fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count() as f64;
    let union = (a.len() + b.len()) as f64 - inter;
    if union <= 0.0 { 0.0 } else { inter / union }
}

fn compare_text(expected: &str, actual: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        expected.trim() == actual.trim()
    } else {
        expected.trim().to_lowercase() == actual.trim().to_lowercase()
    }
}

fn contains_text(haystack: &str, needle: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        haystack.contains(needle)
    } else {
        haystack.to_lowercase().contains(&needle.to_lowercase())
    }
}

#[derive(Debug, Deserialize)]
struct TeacherJudgeResponse {
    passed: bool,
    score: f64,
    #[serde(default)]
    confidence: Option<f64>,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    failure_points: Vec<TeacherFailurePoint>,
}

#[derive(Debug, Deserialize)]
struct TeacherFailurePoint {
    dimension: String,
    description: String,
    #[serde(default)]
    severity: Option<String>,
}

async fn evaluate_teacher_model(
    ctx: &OptimizationContext,
    _task_cfg: &TaskEvaluatorConfig,
    test_case: &TestCase,
    output: &str,
    teacher_model: &Arc<dyn TeacherModel>,
) -> Result<EvaluationResult, EvaluatorError> {
    let samples = llm_judge_samples(ctx, _task_cfg).max(1);
    let mut parsed = Vec::with_capacity(samples as usize);

    let user_guidance = read_optional_user_guidance(ctx);
    let prompt = build_teacher_judge_prompt(test_case, output, user_guidance.as_deref());
    for _ in 0..samples {
        let raw = teacher_model_generate_with_timeout(ctx, teacher_model, &prompt).await?;
        let v: TeacherJudgeResponse = parse_teacher_judge_response(&raw)?;
        parsed.push(v);
    }

    let passed_votes = parsed.iter().filter(|p| p.passed).count();
    let passed = passed_votes * 2 >= parsed.len();
    let score = parsed.iter().map(|p| clamp_01(p.score)).sum::<f64>() / parsed.len() as f64;
    let confidence = if parsed.len() == 1 {
        parsed[0].confidence.or(Some(1.0))
    } else {
        let mu = score;
        let variance = parsed
            .iter()
            .map(|p| {
                let s = clamp_01(p.score);
                (s - mu) * (s - mu)
            })
            .sum::<f64>()
            / parsed.len() as f64;
        Some(clamp_01(1.0 - variance))
    };

    let mut failure_points = Vec::new();
    if !passed {
        for p in &parsed {
            for fp in &p.failure_points {
                failure_points.push(FailurePoint {
                    dimension: fp.dimension.clone(),
                    description: fp.description.clone(),
                    severity: parse_severity(fp.severity.as_deref()),
                    expected: None,
                    actual: Some(output.to_string()),
                });
            }
        }
    }

    Ok(EvaluationResult {
        passed,
        score,
        dimensions: HashMap::new(),
        failure_points,
        evaluator_type: "teacher_model".to_string(),
        confidence,
        reasoning: parsed.iter().find_map(|p| p.reasoning.clone()),
        diversity_analysis: None,
        extra: HashMap::new(),
    })
}

fn build_teacher_judge_prompt(
    test_case: &TestCase,
    output: &str,
    user_guidance: Option<&str>,
) -> String {
    let guidance_section = user_guidance.map(|g| {
        format!(
            "\n\n【用户特别引导】\n{}\n\n【评估要求】\n- 评估时参考用户引导，但不得忽略 Reference 的硬性约束。\n",
            g
        )
    });
    format!(
        "你是评估器。请根据 test_case.reference 判断 output 是否满足要求。\\n\\n要求：只返回 JSON（不要输出其它文本）。\\nJSON schema: {{\"passed\":bool,\"score\":number(0..1),\"confidence\"?:number(0..1),\"reasoning\"?:string,\"failure_points\"?:[{{\"dimension\":string,\"description\":string,\"severity\"?:\"Critical\"|\"Major\"|\"Minor\"}}]}}\\n\\nTestCaseId: {}\\n\\nReference: {}\\n\\nOutput: {}\\n{}",
        test_case.id,
        serde_json::to_string(&test_case.reference)
            .unwrap_or_else(|_| "<unserializable reference>".to_string()),
        output,
        guidance_section.unwrap_or_default()
    )
}

fn read_optional_user_guidance(ctx: &OptimizationContext) -> Option<String> {
    ctx.extensions
        .get(EXT_USER_GUIDANCE)
        .and_then(|v| serde_json::from_value::<UserGuidance>(v.clone()).ok())
        .map(|g| g.content)
}

fn parse_severity(s: Option<&str>) -> Severity {
    match s {
        Some("Critical") | Some("critical") => Severity::Critical,
        Some("Minor") | Some("minor") => Severity::Minor,
        _ => Severity::Major,
    }
}

fn llm_judge_samples(ctx: &OptimizationContext, task_cfg: &TaskEvaluatorConfig) -> u32 {
    // 任务级优先（更贴近用户配置），算法级为兜底（也允许编排层同步覆盖）。
    let task = task_cfg.teacher_model.llm_judge_samples;
    if task > 0 {
        return task;
    }
    ctx.config.evaluator.llm_judge_samples.max(1)
}

fn teacher_timeout(ctx: &OptimizationContext) -> Duration {
    // P2(必做子集)：避免 TeacherModel 调用无上限卡死；先用预算中的 max_duration_secs 作为可选覆盖。
    // 未配置预算时使用 60s 的保守默认值。
    let secs = ctx.config.budget.max_duration_secs.unwrap_or(60);
    Duration::from_secs(secs.max(1))
}

async fn teacher_model_generate_with_timeout(
    ctx: &OptimizationContext,
    teacher_model: &Arc<dyn TeacherModel>,
    prompt: &str,
) -> Result<String, EvaluatorError> {
    let d = teacher_timeout(ctx);
    timeout(d, teacher_model.generate(prompt))
        .await
        .map_err(|_| EvaluatorError::Timeout(format!("TeacherModel 调用超时（>{:?}）", d)))?
        .map_err(|e| EvaluatorError::ModelFailure(e.to_string()))
}

fn parse_teacher_judge_response(raw: &str) -> Result<TeacherJudgeResponse, EvaluatorError> {
    if let Ok(v) = serde_json::from_str::<TeacherJudgeResponse>(raw) {
        return Ok(v);
    }
    let extracted = extract_json_object(raw).ok_or_else(|| {
        EvaluatorError::ModelFailure(format!(
            "TeacherModel judge 输出不是合法 JSON，raw_excerpt={:?}",
            truncate(raw, 400)
        ))
    })?;

    serde_json::from_str::<TeacherJudgeResponse>(extracted).map_err(|e| {
        EvaluatorError::ModelFailure(format!(
            "TeacherModel judge 输出不是合法 JSON（{e}），raw_excerpt={:?}",
            truncate(raw, 400)
        ))
    })
}

fn truncate(s: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (n, ch) in s.chars().enumerate() {
        if n >= max_chars {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

fn extract_json_object(raw: &str) -> Option<&str> {
    // 1) fenced ```json ... ```
    if let Some(start) = raw.find("```") {
        let rest = &raw[start + 3..];
        if let Some(end) = rest.find("```") {
            let inner = &rest[..end];
            // drop optional language tag line
            let inner = inner.trim_start();
            let inner = inner
                .strip_prefix("json")
                .map(|s| s.trim_start_matches(['\n', '\r']))
                .unwrap_or(inner);
            if inner.trim_start().starts_with('{') {
                // fallthrough to brace scan on inner slice
                if let Some(extracted) = extract_json_object(inner) {
                    return Some(extracted);
                }
            }
        }
    }

    // 2) brace scan for first balanced {...}
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut depth = 0i32;
            let mut in_str = false;
            let mut escape = false;
            let start = i;
            while i < bytes.len() {
                let b = bytes[i];
                if in_str {
                    if escape {
                        escape = false;
                    } else if b == b'\\' {
                        escape = true;
                    } else if b == b'"' {
                        in_str = false;
                    }
                    i += 1;
                    continue;
                }
                match b {
                    b'"' => in_str = true,
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            let end = i + 1;
                            return Some(&raw[start..end]);
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            return None;
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{
        Constraint, EvaluatorConfig as TaskEvaluatorConfig, EvaluatorType, RuleSystem,
    };
    use crate::domain::types::{ExecutionTargetConfig, OptimizationConfig};

    fn make_ctx(
        test_cases: Vec<TestCase>,
        task_cfg: TaskEvaluatorConfig,
        algo_data_split_enabled: bool,
        pass_threshold: f64,
    ) -> OptimizationContext {
        let mut extensions = HashMap::new();
        extensions.insert(
            EXT_TASK_EVALUATOR_CONFIG.to_string(),
            serde_json::to_value(task_cfg).unwrap(),
        );

        let mut cfg = OptimizationConfig::default();
        cfg.data_split.enabled = algo_data_split_enabled;
        cfg.iteration.pass_threshold = pass_threshold;

        OptimizationContext {
            task_id: "task-1".to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: "p".to_string(),
            rule_system: RuleSystem {
                rules: vec![],
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 0,
            },
            iteration: 0,
            state: crate::domain::models::IterationState::Idle,
            run_control_state: Default::default(),
            test_cases,
            config: cfg,
            checkpoints: vec![],
            extensions,
        }
    }

    fn make_exact_case(id: &str, expected: &str) -> TestCase {
        TestCase {
            id: id.to_string(),
            input: HashMap::new(),
            reference: TaskReference::Exact {
                expected: expected.to_string(),
            },
            split: None,
            metadata: None,
        }
    }

    fn make_constrained_case(
        id: &str,
        split: Option<DataSplit>,
        constraints: Vec<Constraint>,
    ) -> TestCase {
        TestCase {
            id: id.to_string(),
            input: HashMap::new(),
            reference: TaskReference::Constrained {
                core_request: Some("友好 简洁".to_string()),
                constraints,
                quality_dimensions: vec![],
            },
            split,
            metadata: None,
        }
    }

    fn make_hybrid_case(
        id: &str,
        exact_parts: Vec<(&str, &str)>,
        constraints: Vec<Constraint>,
    ) -> TestCase {
        let mut map = HashMap::new();
        for (k, v) in exact_parts {
            map.insert(k.to_string(), v.to_string());
        }
        TestCase {
            id: id.to_string(),
            input: HashMap::new(),
            reference: TaskReference::Hybrid {
                exact_parts: map,
                constraints,
            },
            split: None,
            metadata: None,
        }
    }

    fn task_cfg(t: EvaluatorType) -> TaskEvaluatorConfig {
        TaskEvaluatorConfig {
            evaluator_type: t,
            ..TaskEvaluatorConfig::default()
        }
    }

    fn c_length(min: u64, max: u64) -> Constraint {
        Constraint {
            name: "length".to_string(),
            description: "len".to_string(),
            params: Some(json!({"minChars": min, "maxChars": max})),
            weight: None,
        }
    }

    fn c_must_include(words: Vec<&str>) -> Constraint {
        Constraint {
            name: "must_include".to_string(),
            description: "include".to_string(),
            params: Some(json!({"keywords": words})),
            weight: None,
        }
    }

    use crate::core::teacher_model::{ExampleTeacherModel, TeacherModelType, create_teacher_model};

    #[tokio::test]
    async fn exact_match_pass_and_fail() {
        let tc = make_exact_case("tc1", "OK");
        let ctx = make_ctx(
            vec![tc.clone()],
            task_cfg(EvaluatorType::ExactMatch),
            false,
            0.95,
        );

        let ev = DefaultEvaluator::new(None)
            .evaluate(&ctx, &tc, "OK")
            .await
            .unwrap();
        assert!(ev.passed);
        assert_eq!(ev.evaluator_type, "exact_match");
        assert!(ev.failure_points.is_empty());

        let ev2 = DefaultEvaluator::new(None)
            .evaluate(&ctx, &tc, "NO")
            .await
            .unwrap();
        assert!(!ev2.passed);
        assert!(!ev2.failure_points.is_empty());
        assert!(
            ev2.failure_points
                .iter()
                .any(|fp| fp.dimension == "exact_match")
        );
    }

    #[tokio::test]
    async fn constraint_check_generates_failure_points() {
        let tc = make_constrained_case(
            "tc1",
            None,
            vec![c_length(3, 10), c_must_include(vec!["欢迎"])],
        );
        let ctx = make_ctx(
            vec![tc.clone()],
            task_cfg(EvaluatorType::ConstraintCheck),
            false,
            0.95,
        );

        let ev = DefaultEvaluator::new(None)
            .evaluate(&ctx, &tc, "hi")
            .await
            .unwrap();
        assert!(!ev.passed);
        assert!(
            ev.failure_points
                .iter()
                .any(|fp| fp.dimension == "must_include")
        );
    }

    #[tokio::test]
    async fn hybrid_exact_parts_missing_generates_failure_point() {
        let tc = make_hybrid_case("tc1", vec![("a", "Hello"), ("b", "Bye")], vec![]);
        let ctx = make_ctx(
            vec![tc.clone()],
            task_cfg(EvaluatorType::ExactMatch),
            false,
            0.95,
        );

        let ev = DefaultEvaluator::new(None)
            .evaluate(&ctx, &tc, "Hello")
            .await
            .unwrap();
        assert!(!ev.passed);
        assert!(
            ev.failure_points
                .iter()
                .any(|fp| fp.dimension == "exact_part_missing")
        );
    }

    #[tokio::test]
    async fn task_reference_mismatch_returns_invalid_input() {
        let tc = make_exact_case("tc1", "OK");
        let ctx = make_ctx(
            vec![tc.clone()],
            task_cfg(EvaluatorType::ConstraintCheck),
            false,
            0.95,
        );

        let err = DefaultEvaluator::new(None)
            .evaluate(&ctx, &tc, "OK")
            .await
            .unwrap_err();
        match err {
            EvaluatorError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn evaluate_batch_keeps_order() {
        let tc1 = make_exact_case("tc1", "A");
        let tc2 = make_exact_case("tc2", "B");
        let ctx = make_ctx(
            vec![tc1.clone(), tc2.clone()],
            task_cfg(EvaluatorType::ExactMatch),
            false,
            0.95,
        );

        let results = vec![
            (tc1.clone(), "A".to_string()),
            (tc2.clone(), "NO".to_string()),
        ];
        let evs = DefaultEvaluator::new(None)
            .evaluate_batch(&ctx, &results)
            .await
            .unwrap();
        assert_eq!(evs.len(), 2);
        assert!(evs[0].passed);
        assert!(!evs[1].passed);
    }

    #[tokio::test]
    async fn split_filter_affects_stats_only() {
        let tc_train =
            make_constrained_case("train", Some(DataSplit::Train), vec![c_length(1, 10)]);
        let tc_val =
            make_constrained_case("val", Some(DataSplit::Validation), vec![c_length(1, 10)]);
        let tc_holdout =
            make_constrained_case("holdout", Some(DataSplit::Holdout), vec![c_length(1, 10)]);
        let tc_unassigned = make_constrained_case("u", None, vec![c_length(1, 10)]);

        let ctx = make_ctx(
            vec![
                tc_train.clone(),
                tc_val.clone(),
                tc_holdout.clone(),
                tc_unassigned.clone(),
            ],
            task_cfg(EvaluatorType::ConstraintCheck),
            true,
            0.95,
        );

        let results = vec![
            (tc_train, "ok".to_string()),
            (tc_val, "ok".to_string()),
            (tc_holdout, "ok".to_string()),
            (tc_unassigned, "ok".to_string()),
        ];

        let evs = DefaultEvaluator::new(None)
            .evaluate_batch(&ctx, &results)
            .await
            .unwrap();
        assert_eq!(evs.len(), 4);

        let filter = split_filter_for_stats(&ctx);
        let stats = summarize_for_stats(filter, &results, &evs).unwrap();
        assert_eq!(stats.total_count, 2); // Validation + Unassigned
        assert_eq!(stats.passed_count, 2);

        let ctx0 = make_ctx(
            ctx.test_cases.clone(),
            task_cfg(EvaluatorType::ConstraintCheck),
            true,
            0.0,
        );
        let stats0 = summarize_for_stats(split_filter_for_stats(&ctx0), &results, &evs).unwrap();
        assert!(stats0.pass_rate >= ctx0.config.iteration.pass_threshold);

        let ctx2 = make_ctx(
            ctx.test_cases.clone(),
            task_cfg(EvaluatorType::ConstraintCheck),
            true,
            1.0,
        );
        let stats2 = summarize_for_stats(split_filter_for_stats(&ctx2), &results, &evs).unwrap();
        assert!(stats2.pass_rate >= ctx2.config.iteration.pass_threshold);
    }

    #[tokio::test]
    async fn missing_task_evaluator_config_returns_invalid_input() {
        let tc = make_exact_case("tc1", "OK");
        let mut ctx = make_ctx(
            vec![tc.clone()],
            task_cfg(EvaluatorType::ExactMatch),
            false,
            0.95,
        );
        ctx.extensions.clear();

        let err = DefaultEvaluator::new(None)
            .evaluate(&ctx, &tc, "OK")
            .await
            .unwrap_err();
        match err {
            EvaluatorError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn summarize_for_stats_total_zero_returns_invalid_input() {
        let tc_train =
            make_constrained_case("train", Some(DataSplit::Train), vec![c_length(1, 10)]);
        let tc_holdout =
            make_constrained_case("holdout", Some(DataSplit::Holdout), vec![c_length(1, 10)]);
        let ctx = make_ctx(
            vec![tc_train.clone(), tc_holdout.clone()],
            task_cfg(EvaluatorType::ConstraintCheck),
            true,
            0.95,
        );
        let results = vec![(tc_train, "ok".to_string()), (tc_holdout, "ok".to_string())];
        let evs = DefaultEvaluator::new(None)
            .evaluate_batch(&ctx, &results)
            .await
            .unwrap();

        let err = summarize_for_stats(split_filter_for_stats(&ctx), &results, &evs).unwrap_err();
        match err {
            EvaluatorError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn evaluate_batch_unknown_id_returns_invalid_input() {
        let tc1 = make_exact_case("tc1", "OK");
        let ctx = make_ctx(
            vec![tc1.clone()],
            task_cfg(EvaluatorType::ExactMatch),
            false,
            0.95,
        );

        let other = make_exact_case("other", "OK");
        let results = vec![(other, "OK".to_string())];
        let err = DefaultEvaluator::new(None)
            .evaluate_batch(&ctx, &results)
            .await
            .unwrap_err();
        match err {
            EvaluatorError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn evaluate_batch_duplicate_id_returns_invalid_input() {
        let tc1 = make_exact_case("tc1", "OK");
        let ctx = make_ctx(
            vec![tc1.clone()],
            task_cfg(EvaluatorType::ExactMatch),
            false,
            0.95,
        );
        let results = vec![(tc1.clone(), "OK".to_string()), (tc1, "OK".to_string())];
        let err = DefaultEvaluator::new(None)
            .evaluate_batch(&ctx, &results)
            .await
            .unwrap_err();
        match err {
            EvaluatorError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn build_evaluations_by_test_case_id_duplicate_returns_invalid_input() {
        let tc1 = make_exact_case("tc1", "OK");
        let results = vec![(tc1.clone(), "OK".to_string()), (tc1, "OK".to_string())];
        let evs = vec![
            EvaluationResult {
                passed: true,
                score: 1.0,
                dimensions: HashMap::new(),
                failure_points: vec![],
                evaluator_type: "x".to_string(),
                confidence: None,
                reasoning: None,
                diversity_analysis: None,
                extra: HashMap::new(),
            },
            EvaluationResult {
                passed: true,
                score: 1.0,
                dimensions: HashMap::new(),
                failure_points: vec![],
                evaluator_type: "x".to_string(),
                confidence: None,
                reasoning: None,
                diversity_analysis: None,
                extra: HashMap::new(),
            },
        ];
        let err = build_evaluations_by_test_case_id(&results, &evs).unwrap_err();
        match err {
            EvaluatorError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn teacher_model_timeout_is_enforced() {
        let tc = make_exact_case("tc1", "OK");
        let mut ctx = make_ctx(
            vec![tc.clone()],
            task_cfg(EvaluatorType::TeacherModel),
            false,
            0.95,
        );
        ctx.config.budget.max_duration_secs = Some(1);
        let tm = Arc::new(
            ExampleTeacherModel::new("{\"passed\":true,\"score\":1}")
                .with_delay(Duration::from_millis(1500)),
        );
        let err = DefaultEvaluator::new(Some(tm))
            .evaluate(&ctx, &tc, "OK")
            .await
            .unwrap_err();
        match err {
            EvaluatorError::Timeout(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn teacher_model_parses_fenced_json() {
        let tc = make_exact_case("tc1", "OK");
        let ctx = make_ctx(
            vec![tc.clone()],
            task_cfg(EvaluatorType::TeacherModel),
            false,
            0.95,
        );
        let tm = Arc::new(ExampleTeacherModel::new(
            "当然可以：\n```json\n{\"passed\":true,\"score\":1,\"confidence\":1}\n```\n",
        ));
        let ev = DefaultEvaluator::new(Some(tm))
            .evaluate(&ctx, &tc, "OK")
            .await
            .unwrap();
        assert!(ev.passed);
        assert_eq!(ev.evaluator_type, "teacher_model");
    }

    #[tokio::test]
    async fn ensemble_includes_teacher_model_when_injected() {
        let tc = make_exact_case("tc1", "OK");
        let ctx = make_ctx(vec![tc.clone()], task_cfg(EvaluatorType::Auto), false, 0.95);
        let tm = create_teacher_model(TeacherModelType::Example);
        let ev = DefaultEvaluator::new(Some(tm))
            .evaluate(&ctx, &tc, "OK")
            .await
            .unwrap();
        assert_eq!(ev.evaluator_type, "ensemble");
        let selected = ev
            .extra
            .get(EXTRA_SELECTED_EVALUATORS)
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();
        assert!(selected.contains(&"teacher_model"));
    }

    #[test]
    fn teacher_model_prompt_includes_user_guidance() {
        let tc = make_exact_case("tc1", "OK");
        let prompt = build_teacher_judge_prompt(&tc, "OUT", Some("请更正式"));

        assert!(prompt.contains("【用户特别引导】"));
        assert!(prompt.contains("请更正式"));
        assert!(prompt.contains("评估时参考用户引导"));
    }
}
