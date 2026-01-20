//! 诊断报告生成服务

use std::collections::{HashMap, HashSet};

use similar::{ChangeTag, TextDiff};
use sqlx::SqlitePool;
use thiserror::Error;

use crate::domain::models::{
    DiagnosticReport, DiagnosticSummary, DiffSegment, DiffSegmentType, FailedCaseDetail,
    FailedCaseSummary, FailureArchiveEntry, FailureReasonEntry, OptimizationTaskStatus,
    TurningPoint, TurningPointType,
};
use crate::domain::types::{
    EvaluationResultSummary, IterationArtifacts, IterationStatus, unix_ms_to_iso8601,
};
use crate::infra::db::repositories::{
    IterationRepo, IterationRepoError, IterationSummaryWithArtifactsAndEvaluations,
    OptimizationTaskRepo, OptimizationTaskRepoError, TestSetRepo, TestSetRepoError,
};

pub const FAILED_CASES_DEFAULT_LIMIT: usize = 50;
pub const FAILED_CASES_MAX_LIMIT: usize = 100;

#[derive(Debug, Error)]
pub enum DiagnosticServiceError {
    #[error("任务不存在或无权访问")]
    TaskNotFoundOrForbidden,
    #[error("任务未开始")]
    TaskNotStarted,
    #[error("任务尚未完成")]
    TaskNotCompleted,
    #[error("迭代数据不存在")]
    IterationNotFound,
    #[error("失败用例不存在")]
    FailedCaseNotFound,
    #[error("无效的 case_id")]
    InvalidCaseId,
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    #[error("仓库错误: {0}")]
    Repo(String),
}

#[derive(Debug, Clone)]
struct DiagnosticIteration {
    id: String,
    round: u32,
    status: IterationStatus,
    pass_rate: f64,
    evaluation_results: Vec<EvaluationResultSummary>,
    failure_archive: Vec<FailureArchiveEntry>,
    completed_at: Option<i64>,
    created_at: i64,
}

pub async fn generate_diagnostic_report(
    pool: &SqlitePool,
    user_id: &str,
    task_id: &str,
    failed_cases_limit: usize,
) -> Result<DiagnosticReport, DiagnosticServiceError> {
    let task = OptimizationTaskRepo::find_by_id_for_user(pool, user_id, task_id)
        .await
        .map_err(map_task_repo_error)?;

    if task.status == OptimizationTaskStatus::Draft {
        return Err(DiagnosticServiceError::TaskNotStarted);
    }
    if !matches!(
        task.status,
        OptimizationTaskStatus::Completed | OptimizationTaskStatus::Terminated
    ) {
        return Err(DiagnosticServiceError::TaskNotCompleted);
    }

    let iterations =
        IterationRepo::list_with_artifacts_and_results_by_task_id(pool, user_id, task_id, None)
            .await
            .map_err(map_iteration_repo_error)?;

    let iterations = to_diagnostic_iterations(iterations);
    let total_iterations = IterationRepo::count_by_task_id(pool, user_id, task_id)
        .await
        .map_err(map_iteration_repo_error)? as u32;

    let (failed_iterations, success_iterations) = count_iteration_status(&iterations);
    let common_failure_reasons = analyze_failure_patterns(&iterations);
    let turning_points = detect_turning_points(&iterations);
    let improvement_suggestions = generate_improvement_suggestions(&common_failure_reasons);
    let natural_language_explanation =
        generate_natural_language_explanation(&common_failure_reasons, &turning_points);

    let test_cases = load_test_cases(pool, &task.workspace_id, task_id).await?;
    let limit = failed_cases_limit.clamp(1, FAILED_CASES_MAX_LIMIT);
    let failed_cases = build_failed_case_summaries(&iterations, &test_cases, limit);

    Ok(DiagnosticReport {
        task_id: task.id,
        task_name: task.name,
        status: task_status_label(task.status).to_string(),
        summary: DiagnosticSummary {
            total_iterations,
            failed_iterations,
            success_iterations,
            common_failure_reasons,
            natural_language_explanation,
        },
        turning_points,
        improvement_suggestions,
        failed_cases,
    })
}

pub async fn get_failed_case_detail(
    pool: &SqlitePool,
    user_id: &str,
    task_id: &str,
    case_id: &str,
) -> Result<FailedCaseDetail, DiagnosticServiceError> {
    let (iteration_id, test_case_id) = parse_case_id(case_id)?;

    let task = OptimizationTaskRepo::find_by_id_for_user(pool, user_id, task_id)
        .await
        .map_err(map_task_repo_error)?;
    if task.status == OptimizationTaskStatus::Draft {
        return Err(DiagnosticServiceError::TaskNotStarted);
    }
    if !matches!(
        task.status,
        OptimizationTaskStatus::Completed | OptimizationTaskStatus::Terminated
    ) {
        return Err(DiagnosticServiceError::TaskNotCompleted);
    }

    let iteration = IterationRepo::get_by_id(pool, user_id, task_id, &iteration_id)
        .await
        .map_err(map_iteration_repo_error)?;

    let eval = iteration
        .evaluation_results
        .iter()
        .find(|item| item.test_case_id == test_case_id)
        .ok_or(DiagnosticServiceError::FailedCaseNotFound)?;

    if eval.passed {
        return Err(DiagnosticServiceError::FailedCaseNotFound);
    }

    let test_case = load_test_case(pool, &task.workspace_id, task_id, &test_case_id).await?;

    let input = test_case
        .as_ref()
        .map(|case| stringify_input(&case.input))
        .unwrap_or_else(|| "输入已被删除或不可用".to_string());
    let expected_output = test_case.as_ref().and_then(extract_expected_output);

    let actual_output = load_actual_output(pool, task_id, &iteration_id, &test_case_id).await;
    let diff_segments = match (expected_output.as_deref(), actual_output.as_deref()) {
        (Some(expected), Some(actual)) => build_diff_segments(expected, actual),
        _ => Vec::new(),
    };

    Ok(FailedCaseDetail {
        case_id: case_id.to_string(),
        test_case_id: Some(test_case_id),
        input,
        expected_output,
        actual_output,
        failure_reason: eval
            .failure_reason
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        iteration_round: iteration.round.max(0) as u32,
        prompt_used: select_prompt_from_artifacts(&iteration.artifacts),
        diff_segments,
    })
}

fn analyze_failure_patterns(iterations: &[DiagnosticIteration]) -> Vec<FailureReasonEntry> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    let mut total = 0u32;
    for iteration in iterations {
        for result in &iteration.evaluation_results {
            if result.passed {
                continue;
            }
            let raw = result.failure_reason.as_deref().unwrap_or("unknown");
            let normalized = normalize_failure_reason(raw);
            let key = if normalized.is_empty() {
                "unknown".to_string()
            } else {
                normalized
            };
            *counts.entry(key).or_insert(0) += 1;
            total += 1;
        }
    }

    if total == 0 {
        for iteration in iterations {
            for entry in &iteration.failure_archive {
                let normalized = normalize_failure_reason(&entry.failure_reason);
                let key = if normalized.is_empty() {
                    "unknown".to_string()
                } else {
                    normalized
                };
                *counts.entry(key).or_insert(0) += 1;
                total += 1;
            }
        }
        if total == 0 {
            return Vec::new();
        }
    }

    let mut entries: Vec<FailureReasonEntry> = counts
        .into_iter()
        .map(|(reason, count)| FailureReasonEntry {
            reason,
            count,
            percentage: (count as f64) * 100.0 / (total as f64),
        })
        .collect();

    entries.sort_by(|a, b| b.count.cmp(&a.count));
    entries.truncate(10);
    entries
}

fn detect_turning_points(iterations: &[DiagnosticIteration]) -> Vec<TurningPoint> {
    let mut out = Vec::new();
    let mut last_turning: Option<(TurningPointType, u32)> = None;
    let mut has_reached_half = false;
    let mut prev_pass_rate: Option<f64> = None;

    for iteration in iterations {
        let pass_rate = iteration.pass_rate;
        let mut candidate: Option<TurningPointType> = None;
        let mut description = String::new();

        if pass_rate >= 1.0 - 1e-6 {
            candidate = Some(TurningPointType::Breakthrough);
            description = "通过率达到 100%".to_string();
        } else if !has_reached_half && pass_rate >= 0.5 {
            candidate = Some(TurningPointType::Breakthrough);
            description = "首次达到 50% 通过率".to_string();
        } else if let Some(prev) = prev_pass_rate {
            let diff = pass_rate - prev;
            if diff >= 0.1 {
                candidate = Some(TurningPointType::Improvement);
                description = format!("通过率提升 {:.1}%", diff * 100.0);
            } else if diff <= -0.1 {
                candidate = Some(TurningPointType::Regression);
                description = format!("通过率下降 {:.1}%", diff.abs() * 100.0);
            }
        }

        if let Some(tp_type) = candidate {
            let is_consecutive_same = last_turning
                .as_ref()
                .map(|(last_type, last_round)| {
                    *last_type == tp_type && iteration.round == last_round + 1
                })
                .unwrap_or(false);
            if !is_consecutive_same {
                out.push(TurningPoint {
                    round: iteration.round,
                    event_type: tp_type.clone(),
                    description,
                    pass_rate_before: prev_pass_rate,
                    pass_rate_after: Some(pass_rate),
                    timestamp: unix_ms_to_iso8601(
                        iteration.completed_at.unwrap_or(iteration.created_at),
                    ),
                });
                last_turning = Some((tp_type, iteration.round));
            }
        }

        if pass_rate >= 0.5 {
            has_reached_half = true;
        }
        prev_pass_rate = Some(pass_rate);
    }

    out
}

fn generate_improvement_suggestions(failure_patterns: &[FailureReasonEntry]) -> Vec<String> {
    let mut suggestions = Vec::new();
    let mut seen = HashSet::new();
    for entry in failure_patterns {
        if let Some(category) = classify_failure_reason(&entry.reason) {
            for suggestion in suggestions_for_category(category) {
                if seen.insert(suggestion) {
                    suggestions.push(suggestion.to_string());
                }
                if suggestions.len() >= 5 {
                    return suggestions;
                }
            }
        }
    }
    suggestions
}

fn generate_natural_language_explanation(
    failure_patterns: &[FailureReasonEntry],
    turning_points: &[TurningPoint],
) -> String {
    if failure_patterns.is_empty() {
        return "所有用例均通过，暂无失败原因。".to_string();
    }

    let top = &failure_patterns[0];
    let suggestion = classify_failure_reason(&top.reason)
        .and_then(|category| suggestions_for_category(category).get(0).copied())
        .unwrap_or("补充任务说明与正反例约束");
    let mut explanation = format!(
        "主要失败原因是 {}，占比 {:.1}%。建议关注 {}。",
        top.reason, top.percentage, suggestion
    );

    if let Some(tp) = turning_points.first() {
        if let (Some(before), Some(after)) = (tp.pass_rate_before, tp.pass_rate_after) {
            explanation.push_str(&format!(
                " 在第 {} 轮出现关键转折，通过率从 {:.0}% 提升到 {:.0}%。",
                tp.round,
                before * 100.0,
                after * 100.0
            ));
        }
    }

    explanation
}

fn build_failed_case_summaries(
    iterations: &[DiagnosticIteration],
    test_cases: &HashMap<String, crate::domain::models::TestCase>,
    limit: usize,
) -> Vec<FailedCaseSummary> {
    let mut cases = Vec::new();
    let mut has_results = false;
    for iteration in iterations {
        for result in &iteration.evaluation_results {
            has_results = true;
            if result.passed {
                continue;
            }
            let test_case_id = result.test_case_id.clone();
            let input_preview = test_cases
                .get(&test_case_id)
                .map(|case| truncate_preview(&stringify_input(&case.input), 100))
                .unwrap_or_else(|| "输入已被删除或不可用".to_string());
            let failure_reason = result
                .failure_reason
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            cases.push(FailedCaseSummary {
                case_id: format!("{}:{}", iteration.id, test_case_id),
                input_preview,
                failure_reason,
                iteration_round: iteration.round,
                test_case_id: Some(test_case_id),
            });
        }
    }

    if !has_results {
        let mut seen = HashSet::new();
        for iteration in iterations {
            for entry in &iteration.failure_archive {
                if !seen.insert(entry.failure_fingerprint.clone()) {
                    continue;
                }
                let test_case_id = entry.test_case_id.clone();
                let input_preview = test_cases
                    .get(&test_case_id)
                    .map(|case| truncate_preview(&stringify_input(&case.input), 100))
                    .unwrap_or_else(|| "输入已被删除或不可用".to_string());
                cases.push(FailedCaseSummary {
                    case_id: format!("{}:{}", iteration.id, test_case_id),
                    input_preview,
                    failure_reason: entry.failure_reason.clone(),
                    iteration_round: iteration.round,
                    test_case_id: Some(test_case_id),
                });
            }
        }
    }

    cases.sort_by(|a, b| b.iteration_round.cmp(&a.iteration_round));
    cases.truncate(limit);
    cases
}

async fn load_test_cases(
    pool: &SqlitePool,
    workspace_id: &str,
    task_id: &str,
) -> Result<HashMap<String, crate::domain::models::TestCase>, DiagnosticServiceError> {
    let test_set_ids = OptimizationTaskRepo::list_test_set_ids_by_task_id(pool, task_id)
        .await
        .map_err(map_task_repo_error)?;
    let cases = TestSetRepo::list_cases_by_ids(pool, workspace_id, &test_set_ids)
        .await
        .map_err(map_test_set_repo_error)?;

    Ok(cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect())
}

async fn load_test_case(
    pool: &SqlitePool,
    workspace_id: &str,
    task_id: &str,
    test_case_id: &str,
) -> Result<Option<crate::domain::models::TestCase>, DiagnosticServiceError> {
    let test_set_ids = OptimizationTaskRepo::list_test_set_ids_by_task_id(pool, task_id)
        .await
        .map_err(map_task_repo_error)?;
    let case = TestSetRepo::find_case_by_id(pool, workspace_id, &test_set_ids, test_case_id)
        .await
        .map_err(map_test_set_repo_error)?;
    Ok(case)
}

fn to_diagnostic_iterations(
    iterations: Vec<IterationSummaryWithArtifactsAndEvaluations>,
) -> Vec<DiagnosticIteration> {
    iterations
        .into_iter()
        .map(|item| DiagnosticIteration {
            id: item.summary.id.clone(),
            round: item.summary.round.max(0) as u32,
            status: item.summary.status,
            pass_rate: item.summary.pass_rate,
            evaluation_results: item.evaluation_results,
            failure_archive: item.artifacts.failure_archive.clone().unwrap_or_default(),
            completed_at: item.completed_at,
            created_at: item.created_at,
        })
        .collect()
}

fn task_status_label(status: OptimizationTaskStatus) -> &'static str {
    match status {
        OptimizationTaskStatus::Draft => "draft",
        OptimizationTaskStatus::Running => "running",
        OptimizationTaskStatus::Paused => "paused",
        OptimizationTaskStatus::Completed => "completed",
        OptimizationTaskStatus::Terminated => "terminated",
    }
}

fn parse_case_id(case_id: &str) -> Result<(String, String), DiagnosticServiceError> {
    let mut parts = case_id.splitn(2, ':');
    let iteration_id = parts.next().unwrap_or("").trim();
    let test_case_id = parts.next().unwrap_or("").trim();
    if iteration_id.is_empty() || test_case_id.is_empty() {
        return Err(DiagnosticServiceError::InvalidCaseId);
    }
    Ok((iteration_id.to_string(), test_case_id.to_string()))
}

fn normalize_failure_reason(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.trim().to_lowercase().chars() {
        if ch.is_alphanumeric() || ch.is_whitespace() || ch.is_alphabetic() {
            out.push(ch);
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn classify_failure_reason(reason: &str) -> Option<&'static str> {
    let normalized = reason.to_lowercase();
    if normalized.contains("format") || normalized.contains("格式") {
        Some("format")
    } else if normalized.contains("length") || normalized.contains("长度") {
        Some("length")
    } else if normalized.contains("missing") || normalized.contains("缺失") {
        Some("missing_field")
    } else if normalized.contains("semantic") || normalized.contains("语义") {
        Some("semantic_drift")
    } else if normalized.contains("structure") || normalized.contains("结构") {
        Some("structure")
    } else {
        None
    }
}

fn suggestions_for_category(category: &str) -> &'static [&'static str] {
    match category {
        "format" => &["补充输出格式示例", "在 Prompt 中明确格式约束"],
        "length" => &["强调字数/长度要求", "提供截断/扩写规则"],
        "missing_field" => &["列出必需字段清单", "给出字段顺序"],
        "semantic_drift" => &["增加任务目标强调", "提供正反例"],
        "structure" => &["定义结构模板", "要求严格遵循结构"],
        _ => &[],
    }
}

fn count_iteration_status(iterations: &[DiagnosticIteration]) -> (u32, u32) {
    let mut failed = 0u32;
    let mut success = 0u32;
    for iteration in iterations {
        match iteration.status {
            IterationStatus::Failed | IterationStatus::Terminated => failed += 1,
            IterationStatus::Completed => success += 1,
            _ => {}
        }
    }
    (failed, success)
}

fn truncate_preview(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let total_len = input.chars().count();
    if total_len <= max_chars {
        return input.to_string();
    }
    let take_len = max_chars.saturating_sub(3);
    let mut out: String = input.chars().take(take_len).collect();
    out.push_str("...");
    out
}

fn stringify_input(input: &HashMap<String, serde_json::Value>) -> String {
    serde_json::to_string(input).unwrap_or_else(|_| "{}".to_string())
}

fn extract_expected_output(case: &crate::domain::models::TestCase) -> Option<String> {
    use crate::domain::models::TaskReference;
    match &case.reference {
        TaskReference::Exact { expected } => Some(expected.clone()),
        _ => None,
    }
}

fn select_prompt_from_artifacts(artifacts: &IterationArtifacts) -> Option<String> {
    artifacts
        .candidate_prompts
        .iter()
        .find(|prompt| prompt.is_best)
        .or_else(|| artifacts.candidate_prompts.first())
        .map(|prompt| prompt.content.clone())
}

fn build_diff_segments(expected: &str, actual: &str) -> Vec<DiffSegment> {
    let diff = TextDiff::from_chars(expected, actual);
    let mut segments = Vec::new();
    let mut expected_cursor = 0u32;
    let mut actual_cursor = 0u32;
    for change in diff.iter_all_changes() {
        let content = change.to_string();
        let len = content.chars().count() as u32;
        let segment_type = match change.tag() {
            ChangeTag::Delete => DiffSegmentType::Removed,
            ChangeTag::Insert => DiffSegmentType::Added,
            ChangeTag::Equal => DiffSegmentType::Unchanged,
        };
        let (start_index, end_index) = match change.tag() {
            ChangeTag::Insert => {
                let start = actual_cursor;
                let end = actual_cursor + len;
                actual_cursor = actual_cursor.saturating_add(len);
                (start, end)
            }
            ChangeTag::Delete => {
                let start = expected_cursor;
                let end = expected_cursor + len;
                expected_cursor = expected_cursor.saturating_add(len);
                (start, end)
            }
            ChangeTag::Equal => {
                let start = expected_cursor;
                let end = expected_cursor + len;
                expected_cursor = expected_cursor.saturating_add(len);
                actual_cursor = actual_cursor.saturating_add(len);
                (start, end)
            }
        };
        segments.push(DiffSegment {
            segment_type,
            content,
            start_index,
            end_index,
        });
    }
    segments
}

#[derive(Debug, serde::Deserialize)]
struct EvaluationResultDetail {
    #[serde(rename = "testCaseId", alias = "test_case_id")]
    test_case_id: String,
    #[serde(default)]
    actual_output: Option<String>,
    #[serde(default, alias = "actualOutput")]
    actual_output_alias: Option<String>,
}

async fn load_actual_output(
    pool: &SqlitePool,
    task_id: &str,
    iteration_id: &str,
    test_case_id: &str,
) -> Option<String> {
    let raw: Option<String> = sqlx::query_scalar(
        r#"
        SELECT evaluation_results
        FROM iterations
        WHERE id = ?1 AND task_id = ?2
        "#,
    )
    .bind(iteration_id)
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let raw = raw?;
    let parsed = serde_json::from_str::<Vec<EvaluationResultDetail>>(&raw).ok()?;
    parsed
        .into_iter()
        .find(|item| item.test_case_id == test_case_id)
        .and_then(|item| item.actual_output.or(item.actual_output_alias))
}

fn map_task_repo_error(err: OptimizationTaskRepoError) -> DiagnosticServiceError {
    match err {
        OptimizationTaskRepoError::NotFound => DiagnosticServiceError::TaskNotFoundOrForbidden,
        OptimizationTaskRepoError::DatabaseError(err) => DiagnosticServiceError::Database(err),
        other => DiagnosticServiceError::Repo(other.to_string()),
    }
}

fn map_iteration_repo_error(err: IterationRepoError) -> DiagnosticServiceError {
    match err {
        IterationRepoError::TaskNotFoundOrForbidden => {
            DiagnosticServiceError::TaskNotFoundOrForbidden
        }
        IterationRepoError::NotFound => DiagnosticServiceError::IterationNotFound,
        IterationRepoError::Database(err) => DiagnosticServiceError::Database(err),
        other => DiagnosticServiceError::Repo(other.to_string()),
    }
}

fn map_test_set_repo_error(err: TestSetRepoError) -> DiagnosticServiceError {
    match err {
        TestSetRepoError::DatabaseError(err) => DiagnosticServiceError::Database(err),
        other => DiagnosticServiceError::Repo(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_iteration(round: u32, pass_rate: f64) -> DiagnosticIteration {
        DiagnosticIteration {
            id: format!("iter-{round}"),
            round,
            status: IterationStatus::Completed,
            pass_rate,
            evaluation_results: Vec::new(),
            failure_archive: Vec::new(),
            completed_at: Some(1_700_000_000_000 + round as i64),
            created_at: 1_700_000_000_000 + round as i64,
        }
    }

    #[test]
    fn test_analyze_failure_patterns() {
        let iterations = vec![DiagnosticIteration {
            id: "iter-1".to_string(),
            round: 1,
            status: IterationStatus::Failed,
            pass_rate: 0.1,
            evaluation_results: vec![
                EvaluationResultSummary {
                    test_case_id: "t1".to_string(),
                    passed: false,
                    score: None,
                    failure_reason: Some("Format mismatch".to_string()),
                },
                EvaluationResultSummary {
                    test_case_id: "t2".to_string(),
                    passed: false,
                    score: None,
                    failure_reason: Some("format mismatch".to_string()),
                },
            ],
            failure_archive: Vec::new(),
            completed_at: Some(1),
            created_at: 1,
        }];

        let patterns = analyze_failure_patterns(&iterations);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].count, 2);
    }

    #[test]
    fn test_analyze_failure_patterns_empty() {
        let iterations = vec![base_iteration(1, 0.5)];
        let patterns = analyze_failure_patterns(&iterations);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_turning_points_breakthrough_priority() {
        let iterations = vec![
            base_iteration(1, 0.3),
            base_iteration(2, 0.6),
            base_iteration(3, 0.75),
            base_iteration(4, 1.0),
        ];

        let points = detect_turning_points(&iterations);
        assert_eq!(points[0].event_type, TurningPointType::Breakthrough);
        assert_eq!(points[0].round, 2);
        assert_eq!(
            points.last().unwrap().event_type,
            TurningPointType::Breakthrough
        );
    }

    #[test]
    fn test_detect_turning_points_regression() {
        let iterations = vec![
            base_iteration(1, 0.7),
            base_iteration(2, 0.5),
            base_iteration(3, 0.4),
        ];
        let points = detect_turning_points(&iterations);
        assert!(
            points
                .iter()
                .any(|p| p.event_type == TurningPointType::Regression)
        );
    }

    #[test]
    fn test_generate_natural_language_explanation_empty() {
        let explanation = generate_natural_language_explanation(&[], &[]);
        assert!(explanation.contains("暂无失败原因"));
    }

    #[test]
    fn test_build_diff_segments() {
        let segments = build_diff_segments("abc", "abXc");
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_build_failed_case_summaries_limit() {
        let mut iteration = base_iteration(1, 0.2);
        iteration.evaluation_results = (0..60)
            .map(|i| EvaluationResultSummary {
                test_case_id: format!("t{i}"),
                passed: false,
                score: None,
                failure_reason: Some("format".to_string()),
            })
            .collect();

        let mut test_cases = HashMap::new();
        for i in 0..60 {
            test_cases.insert(
                format!("t{i}"),
                crate::domain::models::TestCase {
                    id: format!("t{i}"),
                    input: HashMap::new(),
                    reference: crate::domain::models::TaskReference::Exact {
                        expected: "ok".to_string(),
                    },
                    split: None,
                    metadata: None,
                },
            );
        }

        let cases = build_failed_case_summaries(&[iteration], &test_cases, 50);
        assert_eq!(cases.len(), 50);
    }
}
