pub mod json;
pub mod text;
pub mod tree;

use crate::error::Result;
use crate::model::{Decision, Link};

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Jsonl,
}

impl OutputFormat {
    pub fn from_str_or_auto(s: Option<&str>, is_tty: bool) -> Self {
        match s {
            Some("json") => OutputFormat::Json,
            Some("jsonl") => OutputFormat::Jsonl,
            Some("text") => OutputFormat::Text,
            _ => {
                if is_tty {
                    OutputFormat::Text
                } else {
                    OutputFormat::Json
                }
            }
        }
    }
}

pub fn format_decision(decision: &Decision, links: &[Link], format: &OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Text => Ok(text::format_decision(decision, links)),
        OutputFormat::Json => json::format_decision(decision, links),
        OutputFormat::Jsonl => json::format_decision_jsonl(decision),
    }
}

pub fn format_decision_list(decisions: &[Decision], format: &OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Text => Ok(text::format_decision_list(decisions)),
        OutputFormat::Json => json::format_decision_list(decisions),
        OutputFormat::Jsonl => json::format_decision_list_jsonl(decisions),
    }
}
