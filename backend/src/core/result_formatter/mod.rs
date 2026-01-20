use serde_json::json;

use crate::domain::models::{IterationSummaryEntry, TaskResultView};
use crate::domain::types::unix_ms_to_iso8601;
use crate::shared::time::now_millis;

fn export_timestamp() -> String {
    unix_ms_to_iso8601(now_millis())
}

fn code_fence_for(content: &str) -> String {
    let mut max_run = 0usize;
    let mut current = 0usize;
    for ch in content.chars() {
        if ch == '`' {
            current += 1;
            max_run = max_run.max(current);
        } else {
            current = 0;
        }
    }
    let fence_len = (max_run + 1).max(3);
    "`".repeat(fence_len)
}

fn format_pass_rate(pass_rate: Option<f64>) -> String {
    pass_rate
        .map(|value| format!("{:.2}%", value * 100.0))
        .unwrap_or_else(|| "—".to_string())
}

fn summary_table_rows(entries: &[IterationSummaryEntry]) -> String {
    if entries.is_empty() {
        return "暂无已完成迭代".to_string();
    }
    let mut out = String::new();
    out.push_str("| 轮次 | 通过率 | 状态 |\n| --- | --- | --- |\n");
    for entry in entries {
        let pass_rate = format_pass_rate(entry.pass_rate);
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            entry.round, pass_rate, entry.status
        ));
    }
    out
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn wrap_cdata(value: &str) -> String {
    if value.contains("]]>") {
        value.replace("]]>", "]]]]><![CDATA[>")
    } else {
        value.to_string()
    }
}

pub fn format_as_markdown(result: &TaskResultView) -> String {
    let exported_at = export_timestamp();
    let best_prompt = result
        .best_prompt
        .as_deref()
        .unwrap_or("暂无可用最佳 Prompt");
    let fence = code_fence_for(best_prompt);
    let completed_at = result.completed_at.as_deref().unwrap_or("—");
    let pass_rate = format_pass_rate(result.pass_rate);

    let mut out = String::new();
    out.push_str("# 优化结果导出\n\n");
    out.push_str(&format!("- 任务名称: {}\n", result.task_name));
    out.push_str(&format!("- 状态: {}\n", result.status));
    out.push_str(&format!("- 通过率: {}\n", pass_rate));
    out.push_str(&format!("- 总迭代轮次: {}\n", result.total_iterations));
    out.push_str(&format!("- 完成时间: {}\n", completed_at));
    out.push_str(&format!("- 导出时间: {}\n\n", exported_at));
    out.push_str("## 最佳 Prompt\n\n");
    out.push_str(&fence);
    out.push('\n');
    out.push_str(best_prompt);
    out.push('\n');
    out.push_str(&fence);
    out.push_str("\n\n## 迭代摘要\n\n");
    out.push_str(&summary_table_rows(&result.iteration_summary));
    out
}

pub fn format_as_json(result: &TaskResultView) -> String {
    let exported_at = export_timestamp();
    let payload = json!({
        "taskName": result.task_name,
        "status": result.status,
        "bestPrompt": result.best_prompt,
        "passRate": result.pass_rate,
        "totalIterations": result.total_iterations,
        "completedAt": result.completed_at,
        "createdAt": result.created_at,
        "iterationSummary": result.iteration_summary,
        "exportedAt": exported_at,
    });
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

pub fn format_as_xml(result: &TaskResultView) -> String {
    let exported_at = export_timestamp();
    let mut out = String::new();
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push_str("<optimizationResult>");
    out.push_str(&format!(
        "<taskName>{}</taskName>",
        xml_escape(&result.task_name)
    ));
    out.push_str(&format!("<status>{}</status>", xml_escape(&result.status)));
    out.push_str(&format!(
        "<passRate>{}</passRate>",
        result
            .pass_rate
            .map(|value| value.to_string())
            .unwrap_or_default()
    ));
    out.push_str(&format!(
        "<totalIterations>{}</totalIterations>",
        result.total_iterations
    ));
    out.push_str(&format!(
        "<completedAt>{}</completedAt>",
        xml_escape(result.completed_at.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "<createdAt>{}</createdAt>",
        xml_escape(&result.created_at)
    ));
    out.push_str(&format!(
        "<exportedAt>{}</exportedAt>",
        xml_escape(&exported_at)
    ));
    out.push_str("<bestPrompt><![CDATA[");
    out.push_str(&wrap_cdata(result.best_prompt.as_deref().unwrap_or("")));
    out.push_str("]]></bestPrompt>");
    out.push_str("<iterationSummary>");
    for entry in &result.iteration_summary {
        out.push_str("<entry>");
        out.push_str(&format!("<round>{}</round>", entry.round));
        out.push_str(&format!(
            "<passRate>{}</passRate>",
            entry
                .pass_rate
                .map(|value| value.to_string())
                .unwrap_or_default()
        ));
        out.push_str(&format!("<status>{}</status>", xml_escape(&entry.status)));
        out.push_str("</entry>");
    }
    out.push_str("</iterationSummary>");
    out.push_str("</optimizationResult>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::Reader;
    use quick_xml::events::Event;

    fn sample_result() -> TaskResultView {
        TaskResultView {
            task_id: "task-1".to_string(),
            task_name: "Demo Task".to_string(),
            status: "completed".to_string(),
            best_prompt: Some("Hello ```world```".to_string()),
            pass_rate: Some(0.8),
            total_iterations: 2,
            completed_at: Some("2025-01-01T00:00:00Z".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            iteration_summary: vec![IterationSummaryEntry {
                round: 2,
                pass_rate: Some(0.8),
                status: "completed".to_string(),
            }],
        }
    }

    #[test]
    fn test_format_as_markdown_contains_sections() {
        let result = sample_result();
        let output = format_as_markdown(&result);
        assert!(output.contains("# 优化结果导出"));
        assert!(output.contains("## 最佳 Prompt"));
        assert!(output.contains("## 迭代摘要"));
        assert!(output.contains("````"));
    }

    #[test]
    fn test_format_as_markdown_contains_fields() {
        let result = sample_result();
        let output = format_as_markdown(&result);
        assert!(output.contains("任务名称: Demo Task"));
        assert!(output.contains("状态: completed"));
        assert!(output.contains("通过率: 80.00%"));
        assert!(output.contains("总迭代轮次: 2"));
        assert!(output.contains("完成时间: 2025-01-01T00:00:00Z"));
    }

    #[test]
    fn test_format_as_json_is_valid() {
        let result = sample_result();
        let output = format_as_json(&result);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["taskName"], "Demo Task");
        assert_eq!(parsed["iterationSummary"][0]["round"], 2);
    }

    #[test]
    fn test_format_as_xml_contains_cdata() {
        let result = sample_result();
        let output = format_as_xml(&result);
        assert!(output.contains("<![CDATA[Hello ```world```]]>"));
        assert!(output.contains("<optimizationResult>"));
    }

    #[test]
    fn test_format_as_xml_is_well_formed() {
        let result = sample_result();
        let output = format_as_xml(&result);
        let mut reader = Reader::from_str(&output);
        reader.trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(err) => panic!("XML 解析失败: {err}"),
            }
            buf.clear();
        }
    }

    #[test]
    fn test_format_as_xml_handles_cdata_end() {
        let mut result = sample_result();
        result.best_prompt = Some("a ]]> b".to_string());
        let output = format_as_xml(&result);
        assert!(output.contains("]]]]><![CDATA[>"));
    }
}
