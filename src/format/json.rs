use serde_json::Value;

use crate::error::Result;
use crate::model::{Decision, Link};

fn decision_to_value(decision: &Decision, links: Option<&[Link]>) -> Value {
    let mut v = serde_json::to_value(decision).unwrap_or(Value::Null);
    if let Some(links) = links {
        if let Value::Object(ref mut map) = v {
            map.insert(
                "links".to_string(),
                serde_json::to_value(links).unwrap_or(Value::Array(vec![])),
            );
        }
    }
    v
}

pub fn format_decision(decision: &Decision, links: &[Link]) -> Result<String> {
    let v = decision_to_value(decision, Some(links));
    Ok(serde_json::to_string_pretty(&v)?)
}

pub fn format_decision_jsonl(decision: &Decision) -> Result<String> {
    Ok(serde_json::to_string(decision)?)
}

pub fn format_decision_list(decisions: &[Decision]) -> Result<String> {
    let values: Vec<Value> = decisions
        .iter()
        .map(|d| decision_to_value(d, None))
        .collect();
    Ok(serde_json::to_string_pretty(&values)?)
}

pub fn format_decision_list_jsonl(decisions: &[Decision]) -> Result<String> {
    let mut out = String::new();
    for d in decisions {
        out.push_str(&serde_json::to_string(d)?);
        out.push('\n');
    }
    Ok(out)
}

/// Format for export: each line is a decision with its links
pub fn format_export_line(decision: &Decision, links: &[Link]) -> Result<String> {
    let v = decision_to_value(decision, Some(links));
    Ok(serde_json::to_string(&v)?)
}
