use crate::core::rule_engine::RuleEngineError;
use crate::core::traits::RuleEngine;
use crate::domain::models::{
    EvaluationResult, ExecutionResult, FailurePoint, OutputLength, Rule, RuleConflict, RuleTags,
    TestCase,
};
use crate::domain::types::{EXT_USER_GUIDANCE, OptimizationContext, UserGuidance};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use tracing::{error, warn};
use uuid::Uuid;

const LAYER1_TEST_RESULTS_KEY: &str = "layer1_test_results";
const STRUCTURE_SCAN_LINES: usize = 100;

#[derive(Debug, Default)]
pub struct DefaultRuleEngine;

impl DefaultRuleEngine {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RuleEngine for DefaultRuleEngine {
    async fn extract_rules(
        &self,
        ctx: &OptimizationContext,
        test_cases: &[TestCase],
    ) -> Result<Vec<Rule>, RuleEngineError> {
        let layer1_results_value = ctx
            .extensions
            .get(LAYER1_TEST_RESULTS_KEY)
            .ok_or(RuleEngineError::MissingLayer1TestResults)?;

        let layer1_results: Layer1TestResults =
            serde_json::from_value(layer1_results_value.clone())
                .map_err(|e| RuleEngineError::InvalidLayer1TestResults(e.to_string()))?;

        let mut missing_test_case_ids = Vec::new();
        for tc in test_cases {
            if !layer1_results
                .evaluations_by_test_case_id
                .contains_key(&tc.id)
            {
                missing_test_case_ids.push(tc.id.clone());
            }
        }
        if !missing_test_case_ids.is_empty() {
            error!(missing_test_case_ids=?missing_test_case_ids, "layer1 缺少测试结果，无法抽取规律");
            return Err(RuleEngineError::MissingTestResults {
                missing_test_case_ids,
            });
        }

        let mut passed_ids = Vec::new();
        let mut failed_ids = Vec::new();
        for tc in test_cases {
            let eval = layer1_results
                .evaluations_by_test_case_id
                .get(&tc.id)
                .expect("checked above");
            if eval.passed {
                passed_ids.push(tc.id.clone());
            } else {
                failed_ids.push(tc.id.clone());
            }
        }

        if !test_cases.is_empty() && failed_ids.is_empty() {
            return Ok(vec![build_all_passed_rule(test_cases)]);
        }

        let mut rules = Vec::new();

        if !failed_ids.is_empty() {
            rules.extend(build_failure_rules(
                test_cases,
                &layer1_results.evaluations_by_test_case_id,
            ));
        }

        if !passed_ids.is_empty() {
            rules.push(build_success_rule(
                test_cases,
                &passed_ids,
                &layer1_results.evaluations_by_test_case_id,
                &layer1_results.executions_by_test_case_id,
            ));
        }

        if let Some(guidance) = read_optional_user_guidance(ctx) {
            annotate_rules_with_guidance(&mut rules, &guidance);
        }

        Ok(rules)
    }

    async fn detect_conflicts(
        &self,
        _ctx: &OptimizationContext,
        _rules: &[Rule],
    ) -> Result<Vec<RuleConflict>, RuleEngineError> {
        warn!("RuleEngine.detect_conflicts 目前为占位实现（返回空冲突集）");
        Ok(vec![])
    }

    async fn resolve_conflict(
        &self,
        _ctx: &OptimizationContext,
        conflict: &RuleConflict,
    ) -> Result<Rule, RuleEngineError> {
        warn!("RuleEngine.resolve_conflict 目前为占位实现（返回 rule1 原样）");
        Ok(conflict.rule1.clone())
    }

    async fn merge_similar_rules(
        &self,
        _ctx: &OptimizationContext,
        rules: &[Rule],
    ) -> Result<Vec<Rule>, RuleEngineError> {
        warn!("RuleEngine.merge_similar_rules 目前为占位实现（返回原 rules）");
        Ok(rules.to_vec())
    }

    fn name(&self) -> &str {
        "default_rule_engine"
    }
}

#[derive(Debug, Deserialize)]
struct Layer1TestResults {
    evaluations_by_test_case_id: HashMap<String, EvaluationResult>,
    #[serde(default)]
    executions_by_test_case_id: HashMap<String, ExecutionResult>,
}

fn build_all_passed_rule(test_cases: &[TestCase]) -> Rule {
    Rule {
        id: Uuid::new_v4().to_string(),
        description: "当前 Prompt 已满足所有测试用例".to_string(),
        tags: rule_tags_with_polarity(
            "all_passed",
            OutputLength::Flexible,
            vec![],
            vec![],
            vec![],
            vec![],
        ),
        source_test_cases: test_cases.iter().map(|tc| tc.id.clone()).collect(),
        abstraction_level: 0,
        parent_rules: vec![],
        verified: true,
        verification_score: 1.0,
        ir: None,
    }
}

fn build_failure_rules(
    test_cases: &[TestCase],
    evaluations_by_test_case_id: &HashMap<String, EvaluationResult>,
) -> Vec<Rule> {
    let mut by_dimension: BTreeMap<String, FailureAggregate> = BTreeMap::new();

    for tc in test_cases {
        let eval = evaluations_by_test_case_id
            .get(&tc.id)
            .expect("checked completeness");
        if eval.passed {
            continue;
        }

        if eval.failure_points.is_empty() {
            by_dimension
                .entry("unknown".to_string())
                .or_default()
                .push(&tc.id, None);
            continue;
        }

        for fp in &eval.failure_points {
            by_dimension
                .entry(fp.dimension.clone())
                .or_default()
                .push(&tc.id, Some(fp));
        }
    }

    by_dimension
        .into_iter()
        .map(|(dimension, agg)| {
            let mut examples = agg.examples;
            examples.sort();
            examples.dedup();

            let example_text = if examples.is_empty() {
                "（未提供 failure_points 细节）".to_string()
            } else {
                examples
                    .into_iter()
                    .take(3)
                    .map(|s| format!("「{s}」"))
                    .collect::<Vec<_>>()
                    .join("、")
            };

            let description = if dimension == "unknown" {
                format!(
                    "失败规律：存在失败用例但缺少 failure_points 细节。表现：{example_text}。建议：补全评估失败点并加强输出约束与示例。"
                )
            } else {
                format!(
                    "失败规律：当输出在「{dimension}」维度不满足要求时容易失败。常见表现：{example_text}。建议：明确并强化「{dimension}」相关约束/格式提示，必要时提供示例与反例。"
                )
            };

            Rule {
                id: Uuid::new_v4().to_string(),
                description,
                tags: rule_tags_with_polarity(
                    "failure",
                    OutputLength::Flexible,
                    vec![],
                    vec![],
                    vec![dimension.clone()],
                    vec![dimension.clone()],
                ),
                source_test_cases: agg.source_test_cases,
                abstraction_level: 0,
                parent_rules: vec![],
                verified: false,
                verification_score: 0.0,
                ir: None,
            }
        })
        .collect()
}

fn build_success_rule(
    test_cases: &[TestCase],
    passed_ids: &[String],
    evaluations_by_test_case_id: &HashMap<String, EvaluationResult>,
    executions_by_test_case_id: &HashMap<String, ExecutionResult>,
) -> Rule {
    let mut format_counts: HashMap<String, u32> = HashMap::new();
    let mut structure_counts: HashMap<String, u32> = HashMap::new();
    let mut concept_counts: HashMap<String, u32> = HashMap::new();
    let mut missing_execution_ids: Vec<String> = Vec::new();

    for tc in test_cases {
        let eval = evaluations_by_test_case_id
            .get(&tc.id)
            .expect("checked completeness");
        if !eval.passed {
            continue;
        }

        if let Some(exec) = executions_by_test_case_id.get(&tc.id) {
            for tag in infer_output_format_tags(&exec.output) {
                *format_counts.entry(tag).or_insert(0) += 1;
            }
            for tag in infer_output_structure_tags(&exec.output) {
                *structure_counts.entry(tag).or_insert(0) += 1;
            }
        } else {
            missing_execution_ids.push(tc.id.clone());
        }

        for c in infer_key_concepts_from_test_case(tc) {
            *concept_counts.entry(c).or_insert(0) += 1;
        }
    }

    if !missing_execution_ids.is_empty() {
        let sample_ids: Vec<String> = missing_execution_ids.iter().take(10).cloned().collect();
        warn!(
            missing_execution_count = missing_execution_ids.len(),
            missing_execution_sample_ids = ?sample_ids,
            "layer1 executions 缺失，success 规律将不包含部分输出格式/结构推断"
        );
    }

    let common_formats = top_tags(&format_counts, 3);
    let common_structures = top_tags(&structure_counts, 3);
    let common_concepts = top_tags(&concept_counts, 5);

    let mut parts = Vec::new();
    if !common_formats.is_empty() {
        parts.push(format!("输出格式偏向：{}", common_formats.join("、")));
    }
    if !common_structures.is_empty() {
        parts.push(format!("输出结构偏向：{}", common_structures.join("、")));
    }
    if !common_concepts.is_empty() {
        parts.push(format!("关键关注点：{}", common_concepts.join("、")));
    }

    let keep_text = if parts.is_empty() {
        "保持当前 Prompt 的表达方式与关键约束，避免破坏已通过用例。".to_string()
    } else {
        format!("保持项：{}。避免改动破坏已通过用例。", parts.join("；"))
    };

    Rule {
        id: Uuid::new_v4().to_string(),
        description: format!("成功规律：在已通过的用例中，当前 Prompt 能稳定满足要求。{keep_text}"),
        tags: rule_tags_with_polarity(
            "success",
            OutputLength::Flexible,
            common_formats,
            common_structures,
            vec![],
            common_concepts,
        ),
        source_test_cases: passed_ids.to_vec(),
        abstraction_level: 0,
        parent_rules: vec![],
        verified: false,
        verification_score: 0.0,
        ir: None,
    }
}

fn rule_tags_with_polarity(
    polarity: &str,
    output_length: OutputLength,
    output_format: Vec<String>,
    output_structure: Vec<String>,
    semantic_focus: Vec<String>,
    key_concepts: Vec<String>,
) -> RuleTags {
    let mut extra = HashMap::new();
    extra.insert(
        "polarity".to_string(),
        serde_json::Value::String(polarity.to_string()),
    );

    RuleTags {
        output_format,
        output_structure,
        output_length,
        semantic_focus,
        key_concepts,
        must_include: vec![],
        must_exclude: vec![],
        tone: None,
        extra,
    }
}

fn read_optional_user_guidance(ctx: &OptimizationContext) -> Option<UserGuidance> {
    ctx.extensions
        .get(EXT_USER_GUIDANCE)
        .and_then(|v| serde_json::from_value::<UserGuidance>(v.clone()).ok())
}

fn annotate_rules_with_guidance(rules: &mut [Rule], guidance: &UserGuidance) {
    let preview = guidance.content_preview();
    for rule in rules {
        rule.tags.extra.insert(
            "user_guidance_id".to_string(),
            serde_json::Value::String(guidance.id.clone()),
        );
        rule.tags.extra.insert(
            "user_guidance_preview".to_string(),
            serde_json::Value::String(preview.clone()),
        );
    }
}

fn infer_output_format_tags(output: &str) -> Vec<String> {
    let trimmed = output.trim_start();
    let mut tags = Vec::new();

    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        tags.push("json".to_string());
    }
    if trimmed.starts_with('#') || output.contains("```") {
        tags.push("markdown".to_string());
    }
    if tags.is_empty() {
        tags.push("plain_text".to_string());
    }
    tags
}

fn infer_output_structure_tags(output: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for line in output.lines().take(STRUCTURE_SCAN_LINES) {
        let l = line.trim_start();
        if l.starts_with('#') {
            tags.push("heading".to_string());
        }
        if l.starts_with("- ") || l.starts_with("* ") {
            tags.push("bullet_list".to_string());
        }
        if l.chars()
            .take(3)
            .collect::<String>()
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
            && l.contains(". ")
        {
            tags.push("numbered_list".to_string());
        }
        if l.starts_with('|') && l.matches('|').count() >= 2 {
            tags.push("table".to_string());
        }
    }
    tags.sort();
    tags.dedup();
    tags
}

fn infer_key_concepts_from_test_case(test_case: &TestCase) -> Vec<String> {
    use crate::domain::models::TaskReference;

    match &test_case.reference {
        TaskReference::Exact { .. } => vec!["exact_match".to_string()],
        TaskReference::Constrained {
            core_request: _,
            constraints,
            quality_dimensions,
        } => constraints
            .iter()
            .map(|c| c.name.clone())
            .chain(quality_dimensions.iter().map(|d| d.name.clone()))
            .collect(),
        TaskReference::Hybrid {
            exact_parts,
            constraints,
        } => exact_parts
            .keys()
            .cloned()
            .chain(constraints.iter().map(|c| c.name.clone()))
            .collect(),
    }
}

fn top_tags(counts: &HashMap<String, u32>, limit: usize) -> Vec<String> {
    let mut items: Vec<(String, u32)> = counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    items.into_iter().take(limit).map(|(k, _)| k).collect()
}

#[derive(Default)]
struct FailureAggregate {
    source_test_cases: Vec<String>,
    examples: Vec<String>,
}

impl FailureAggregate {
    fn push(&mut self, test_case_id: &str, fp: Option<&FailurePoint>) {
        self.source_test_cases.push(test_case_id.to_string());
        if let Some(fp) = fp {
            self.examples.push(fp.description.clone());
        }
        self.source_test_cases.sort();
        self.source_test_cases.dedup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{RuleSystem, Severity, TaskReference};
    use crate::domain::types::{ExecutionTargetConfig, OptimizationConfig};
    use serde_json::json;
    use std::collections::HashMap;

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

    fn make_ctx(extensions: HashMap<String, serde_json::Value>) -> OptimizationContext {
        OptimizationContext {
            task_id: "task-1".to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: "prompt".to_string(),
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
            test_cases: vec![],
            config: OptimizationConfig::default(),
            checkpoints: vec![],
            extensions,
        }
    }

    fn make_layer1_ext(
        evaluations_by_test_case_id: &HashMap<String, crate::domain::models::EvaluationResult>,
    ) -> HashMap<String, serde_json::Value> {
        let mut extensions = HashMap::new();
        extensions.insert(
            LAYER1_TEST_RESULTS_KEY.to_string(),
            json!({
                "evaluations_by_test_case_id": evaluations_by_test_case_id
            }),
        );
        extensions
    }

    #[tokio::test]
    async fn extract_rules_mixed_success_and_failure() {
        let engine = DefaultRuleEngine::new();

        let tc_ok = make_test_case("tc_ok");
        let tc_bad = make_test_case("tc_bad");
        let test_cases = vec![tc_ok.clone(), tc_bad.clone()];

        let mut evals = HashMap::new();
        evals.insert(
            tc_ok.id.clone(),
            crate::domain::models::EvaluationResult {
                passed: true,
                score: 1.0,
                dimensions: HashMap::new(),
                failure_points: vec![],
                evaluator_type: "unit-test".to_string(),
                confidence: None,
                reasoning: None,
                diversity_analysis: None,
                extra: HashMap::new(),
            },
        );
        evals.insert(
            tc_bad.id.clone(),
            crate::domain::models::EvaluationResult {
                passed: false,
                score: 0.0,
                dimensions: HashMap::new(),
                failure_points: vec![crate::domain::models::FailurePoint {
                    dimension: "format_compliance".to_string(),
                    description: "输出格式不符合要求".to_string(),
                    severity: Severity::Major,
                    expected: None,
                    actual: None,
                }],
                evaluator_type: "unit-test".to_string(),
                confidence: None,
                reasoning: None,
                diversity_analysis: None,
                extra: HashMap::new(),
            },
        );

        let ctx = make_ctx(make_layer1_ext(&evals));
        let rules = engine.extract_rules(&ctx, &test_cases).await.unwrap();

        let has_success = rules
            .iter()
            .any(|r| r.tags.extra.get("polarity").and_then(|v| v.as_str()) == Some("success"));
        let has_failure = rules
            .iter()
            .any(|r| r.tags.extra.get("polarity").and_then(|v| v.as_str()) == Some("failure"));

        assert!(has_success, "应包含 success 规律");
        assert!(has_failure, "应包含 failure 规律");
    }

    #[tokio::test]
    async fn extract_rules_all_passed_emits_all_passed_signal() {
        let engine = DefaultRuleEngine::new();

        let tc1 = make_test_case("tc1");
        let tc2 = make_test_case("tc2");
        let test_cases = vec![tc1.clone(), tc2.clone()];

        let mut evals = HashMap::new();
        for tc in [&tc1, &tc2] {
            evals.insert(
                tc.id.clone(),
                crate::domain::models::EvaluationResult {
                    passed: true,
                    score: 1.0,
                    dimensions: HashMap::new(),
                    failure_points: vec![],
                    evaluator_type: "unit-test".to_string(),
                    confidence: None,
                    reasoning: None,
                    diversity_analysis: None,
                    extra: HashMap::new(),
                },
            );
        }

        let ctx = make_ctx(make_layer1_ext(&evals));
        let rules = engine.extract_rules(&ctx, &test_cases).await.unwrap();

        assert_eq!(rules.len(), 1, "全通过时固定输出 1 条 all_passed 规则");
        assert_eq!(
            rules[0].tags.extra.get("polarity").and_then(|v| v.as_str()),
            Some("all_passed")
        );
        assert!(
            rules[0]
                .description
                .contains("当前 Prompt 已满足所有测试用例"),
            "all_passed 规则描述需明确"
        );
    }

    #[tokio::test]
    async fn extract_rules_missing_test_results_is_error() {
        let engine = DefaultRuleEngine::new();

        let tc1 = make_test_case("tc1");
        let tc2 = make_test_case("tc2");
        let test_cases = vec![tc1.clone(), tc2.clone()];

        let mut evals = HashMap::new();
        evals.insert(
            tc1.id.clone(),
            crate::domain::models::EvaluationResult {
                passed: true,
                score: 1.0,
                dimensions: HashMap::new(),
                failure_points: vec![],
                evaluator_type: "unit-test".to_string(),
                confidence: None,
                reasoning: None,
                diversity_analysis: None,
                extra: HashMap::new(),
            },
        );

        let ctx = make_ctx(make_layer1_ext(&evals));
        let err = engine.extract_rules(&ctx, &test_cases).await.unwrap_err();

        match err {
            RuleEngineError::MissingTestResults {
                missing_test_case_ids,
            } => {
                assert_eq!(missing_test_case_ids, vec![tc2.id]);
            }
            other => panic!("期望 MissingTestResults，实际：{other:?}"),
        }
    }

    #[tokio::test]
    async fn extract_rules_failure_points_duplicate_and_multiple_are_aggregated() {
        let engine = DefaultRuleEngine::new();

        let tc1 = make_test_case("tc1");
        let tc2 = make_test_case("tc2");
        let test_cases = vec![tc1.clone(), tc2.clone()];

        let mut evals = HashMap::new();
        evals.insert(
            tc1.id.clone(),
            crate::domain::models::EvaluationResult {
                passed: false,
                score: 0.0,
                dimensions: HashMap::new(),
                failure_points: vec![
                    crate::domain::models::FailurePoint {
                        dimension: "format_compliance".to_string(),
                        description: "输出格式不符合要求".to_string(),
                        severity: Severity::Major,
                        expected: None,
                        actual: None,
                    },
                    crate::domain::models::FailurePoint {
                        dimension: "format_compliance".to_string(),
                        description: "输出格式不符合要求".to_string(),
                        severity: Severity::Major,
                        expected: None,
                        actual: None,
                    },
                ],
                evaluator_type: "unit-test".to_string(),
                confidence: None,
                reasoning: None,
                diversity_analysis: None,
                extra: HashMap::new(),
            },
        );
        evals.insert(
            tc2.id.clone(),
            crate::domain::models::EvaluationResult {
                passed: false,
                score: 0.0,
                dimensions: HashMap::new(),
                failure_points: vec![crate::domain::models::FailurePoint {
                    dimension: "format_compliance".to_string(),
                    description: "缺少 Markdown 标题".to_string(),
                    severity: Severity::Minor,
                    expected: None,
                    actual: None,
                }],
                evaluator_type: "unit-test".to_string(),
                confidence: None,
                reasoning: None,
                diversity_analysis: None,
                extra: HashMap::new(),
            },
        );

        let ctx = make_ctx(make_layer1_ext(&evals));
        let rules = engine.extract_rules(&ctx, &test_cases).await.unwrap();

        assert_eq!(rules.len(), 1, "仅一个维度时应聚合为 1 条 failure 规律");
        let rule = &rules[0];
        assert_eq!(
            rule.tags.extra.get("polarity").and_then(|v| v.as_str()),
            Some("failure")
        );
        assert_eq!(rule.source_test_cases, vec![tc1.id, tc2.id]);
        assert!(
            rule.description.contains("format_compliance"),
            "描述应包含 failure_points.dimension"
        );
        assert_eq!(
            rule.description.matches("输出格式不符合要求").count(),
            1,
            "重复 failure_points.description 应被去重"
        );
    }

    #[tokio::test]
    async fn extract_rules_attaches_user_guidance_preview() {
        let engine = DefaultRuleEngine::new();

        let tc1 = make_test_case("tc1");
        let test_cases = vec![tc1.clone()];

        let mut evals = HashMap::new();
        evals.insert(
            tc1.id.clone(),
            crate::domain::models::EvaluationResult {
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
                evaluator_type: "unit-test".to_string(),
                confidence: None,
                reasoning: None,
                diversity_analysis: None,
                extra: HashMap::new(),
            },
        );

        let mut ext = make_layer1_ext(&evals);
        let guidance = UserGuidance::new("请优先保证结构化输出");
        ext.insert(
            EXT_USER_GUIDANCE.to_string(),
            serde_json::to_value(&guidance).unwrap(),
        );

        let ctx = make_ctx(ext);
        let rules = engine.extract_rules(&ctx, &test_cases).await.unwrap();

        assert!(!rules.is_empty());
        for rule in rules {
            let preview = rule
                .tags
                .extra
                .get("user_guidance_preview")
                .and_then(|v| v.as_str());
            assert_eq!(preview, Some(guidance.content_preview().as_str()));

            let gid = rule
                .tags
                .extra
                .get("user_guidance_id")
                .and_then(|v| v.as_str());
            assert_eq!(gid, Some(guidance.id.as_str()));
        }
    }
}
