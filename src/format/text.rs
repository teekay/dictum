use crate::model::{Decision, Link};

pub fn format_decision(decision: &Decision, links: &[Link]) -> String {
    let mut out = String::new();

    out.push_str(&format!("[{}] {}\n", decision.id, decision.title));
    out.push_str(&format!(
        "  Level: {}  Status: {}  Kind: {}  Weight: {}\n",
        decision.level, decision.status, decision.kind, decision.weight
    ));

    if let Some(ref scope) = decision.scope {
        out.push_str(&format!("  Scope: {}\n", scope));
    }

    if let Some(ref rebuttal) = decision.rebuttal {
        out.push_str(&format!("  Unless: {}\n", rebuttal));
    }

    out.push_str(&format!("  Author: {}\n", decision.author));
    out.push_str(&format!("  Created: {}\n", decision.created_at));

    if decision.updated_at != decision.created_at {
        out.push_str(&format!("  Updated: {}\n", decision.updated_at));
    }

    if let Some(ref superseded_by) = decision.superseded_by {
        out.push_str(&format!("  Superseded by: {}\n", superseded_by));
    }

    if !decision.labels.is_empty() {
        out.push_str(&format!("  Labels: {}\n", decision.labels.join(", ")));
    }

    if let Some(ref body) = decision.body {
        out.push_str(&format!("\n  {}\n", body));
    }

    if !links.is_empty() {
        out.push_str("\n  Links:\n");
        for link in links {
            if link.source_id == decision.id {
                out.push_str(&format!("    {} -> {}", link.kind, link.target_id));
            } else {
                out.push_str(&format!(
                    "    <- {} {}",
                    link.kind, link.source_id
                ));
            }
            if let Some(ref reason) = link.reason {
                out.push_str(&format!(" ({})", reason));
            }
            out.push('\n');
        }
    }

    out
}

pub fn format_decision_list(decisions: &[Decision]) -> String {
    if decisions.is_empty() {
        return "No decisions found.\n".to_string();
    }

    let mut out = String::new();
    for d in decisions {
        let labels = if d.labels.is_empty() {
            String::new()
        } else {
            format!(" [{}]", d.labels.join(", "))
        };
        out.push_str(&format!(
            "{} | {:12} | {:10} | {:10} | {}{}\n",
            d.id, d.level, d.status, d.kind, d.title, labels
        ));
    }
    out
}
