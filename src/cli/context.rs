use std::collections::HashMap;
use std::path::Path;

use crate::db;
use crate::db::ListFilter;
use crate::db::Store;
use crate::error::Result;
use crate::format::OutputFormat;
use crate::model::{Decision, Kind, Status, Weight};

pub struct ContextArgs {
    pub format: Option<String>,
    pub kind: Option<Kind>,
    pub weight: Option<Weight>,
    pub scope: Option<String>,
}

pub fn run(path: &Path, args: ContextArgs, is_tty: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let store = db::open(&dictum_dir)?;

    let decisions = store.decision_list(&ListFilter {
        level: None,
        status: Some(Status::Active),
        label: None,
        kind: args.kind,
        weight: args.weight,
        scope: args.scope,
    })?;

    let format = OutputFormat::from_str_or_auto(args.format.as_deref(), is_tty);

    match format {
        OutputFormat::Compact => print_compact_context(&*store, &decisions)?,
        OutputFormat::Json => print_json_context(&*store, &decisions)?,
        _ => print_text_context(&*store, &decisions)?,
    }

    Ok(())
}

fn print_compact_context(store: &dyn Store, decisions: &[Decision]) -> Result<()> {
    let decision_ids: std::collections::HashSet<&str> =
        decisions.iter().map(|d| d.id.as_str()).collect();

    let mut entries = Vec::new();
    for d in decisions {
        let mut obj = serde_json::json!({
            "id": d.id,
            "kind": d.kind.to_string(),
            "weight": d.weight.to_string(),
            "title": d.title,
        });
        let map = obj.as_object_mut().unwrap();

        if let Some(ref scope) = d.scope {
            map.insert("scope".to_string(), serde_json::Value::String(scope.clone()));
        }
        if let Some(ref body) = d.body {
            map.insert("body".to_string(), serde_json::Value::String(body.clone()));
        }
        if let Some(ref rebuttal) = d.rebuttal {
            map.insert("rebuttal".to_string(), serde_json::Value::String(rebuttal.clone()));
        }
        if !d.labels.is_empty() {
            map.insert("labels".to_string(), serde_json::to_value(&d.labels)?);
        }

        let links = store.links_for_decision(&d.id)?;
        let relevant_links: Vec<serde_json::Value> = links
            .iter()
            .filter(|l| {
                decision_ids.contains(l.source_id.as_str())
                    && decision_ids.contains(l.target_id.as_str())
            })
            .map(|l| {
                let mut link_obj = serde_json::json!({
                    "kind": l.kind.to_string(),
                    "target": if l.source_id == d.id { &l.target_id } else { &l.source_id },
                });
                if l.source_id != d.id {
                    link_obj.as_object_mut().unwrap().insert(
                        "dir".to_string(),
                        serde_json::Value::String("inbound".to_string()),
                    );
                }
                if let Some(ref reason) = l.reason {
                    link_obj.as_object_mut().unwrap().insert(
                        "reason".to_string(),
                        serde_json::Value::String(reason.clone()),
                    );
                }
                link_obj
            })
            .collect();
        if !relevant_links.is_empty() {
            map.insert("links".to_string(), serde_json::Value::Array(relevant_links));
        }

        entries.push(obj);
    }
    println!("{}", serde_json::to_string(&entries)?);
    Ok(())
}

fn print_json_context(store: &dyn Store, decisions: &[Decision]) -> Result<()> {
    let mut entries = Vec::new();
    for d in decisions {
        let links = store.links_for_decision(&d.id)?;
        let mut value = serde_json::to_value(d)?;
        if let serde_json::Value::Object(ref mut map) = value {
            let link_values: Vec<serde_json::Value> = links
                .iter()
                .map(|l| {
                    let mut obj = serde_json::json!({
                        "kind": l.kind.to_string(),
                        "source": l.source_id,
                        "target": l.target_id,
                    });
                    if let Some(ref reason) = l.reason {
                        obj.as_object_mut().unwrap().insert(
                            "reason".to_string(),
                            serde_json::Value::String(reason.clone()),
                        );
                    }
                    obj
                })
                .collect();
            if !link_values.is_empty() {
                map.insert("links".to_string(), serde_json::Value::Array(link_values));
            }
            map.remove("updated_at");
            map.remove("created_at");
            map.remove("status");
        }
        entries.push(value);
    }
    println!("{}", serde_json::to_string_pretty(&entries)?);
    Ok(())
}

fn print_text_context(store: &dyn Store, decisions: &[Decision]) -> Result<()> {
    if decisions.is_empty() {
        println!("No active decisions.");
        return Ok(());
    }

    let mut by_level: HashMap<String, Vec<&Decision>> = HashMap::new();
    for d in decisions {
        by_level.entry(d.level.to_string()).or_default().push(d);
    }

    let refines_links = store.links_of_kind(&crate::model::LinkKind::Refines)?;

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
