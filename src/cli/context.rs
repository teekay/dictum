use std::collections::HashMap;
use std::path::Path;

use crate::db;
use crate::db::decisions::ListFilter;
use crate::error::Result;
use crate::format::OutputFormat;
use crate::model::{Decision, Status};

pub fn run(path: &Path, fmt: Option<String>, is_tty: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;

    // Only active decisions
    let decisions = db::decisions::list(
        &conn,
        &ListFilter {
            level: None,
            status: Some(Status::Active),
            label: None,
            kind: None,
            weight: None,
            scope: None,
        },
    )?;

    let format = OutputFormat::from_str_or_auto(fmt.as_deref(), is_tty);

    match format {
        OutputFormat::Json => {
            print_json_context(&conn, &decisions)?;
        }
        _ => {
            print_text_context(&conn, &decisions)?;
        }
    }

    Ok(())
}

fn print_json_context(
    conn: &rusqlite::Connection,
    decisions: &[Decision],
) -> Result<()> {
    let mut entries = Vec::new();
    for d in decisions {
        let links = db::links::get_for_decision(conn, &d.id)?;
        let mut value = serde_json::to_value(d)?;
        if let serde_json::Value::Object(ref mut map) = value {
            // Only include links that reference other active decisions
            let link_values: Vec<serde_json::Value> = links
                .iter()
                .map(|l| {
                    let mut obj = serde_json::json!({
                        "kind": l.kind.to_string(),
                        "source": l.source_id,
                        "target": l.target_id,
                    });
                    if let Some(ref reason) = l.reason {
                        obj.as_object_mut()
                            .unwrap()
                            .insert("reason".to_string(), serde_json::Value::String(reason.clone()));
                    }
                    obj
                })
                .collect();
            if !link_values.is_empty() {
                map.insert("links".to_string(), serde_json::Value::Array(link_values));
            }
            // Remove noise for LLM context
            map.remove("updated_at");
            map.remove("created_at");
            map.remove("status"); // all active, redundant
        }
        entries.push(value);
    }
    println!("{}", serde_json::to_string_pretty(&entries)?);
    Ok(())
}

fn print_text_context(
    conn: &rusqlite::Connection,
    decisions: &[Decision],
) -> Result<()> {
    if decisions.is_empty() {
        println!("No active decisions.");
        return Ok(());
    }

    // Group by level
    let mut by_level: HashMap<String, Vec<&Decision>> = HashMap::new();
    for d in decisions {
        by_level
            .entry(d.level.to_string())
            .or_default()
            .push(d);
    }

    let refines_links = db::links::get_refines_links(conn)?;

    // Build parent map: child_id -> parent_id
    let mut parent_of: HashMap<&str, &str> = HashMap::new();
    for (source, target) in &refines_links {
        parent_of.insert(source.as_str(), target.as_str());
    }

    println!("# Active Decisions\n");

    for level in &["strategic", "tactical", "operational"] {
        if let Some(decs) = by_level.get(*level) {
            println!("## {}\n", capitalize(level));
            for d in decs {
                print!("- [{}] ({}/{}) {}", d.id, d.kind, d.weight, d.title);
                if let Some(ref scope) = d.scope {
                    print!(" [scope: {}]", scope);
                }
                if let Some(parent_id) = parent_of.get(d.id.as_str()) {
                    print!(" (refines {})", parent_id);
                }
                println!();
                if let Some(ref body) = d.body {
                    println!("  {}", body);
                }
                if let Some(ref rebuttal) = d.rebuttal {
                    println!("  UNLESS: {}", rebuttal);
                }
                if !d.labels.is_empty() {
                    println!("  Labels: {}", d.labels.join(", "));
                }
            }
            println!();
        }
    }

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}
