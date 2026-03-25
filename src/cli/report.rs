use std::io::Write;
use std::path::Path;

use serde_json::json;

use crate::db;
use crate::error::Result;
use crate::model::Status;

const REPORT_TEMPLATE: &str = include_str!("../assets/report.html");
const DATA_PLACEHOLDER: &str = "/*__DICTUM_DATA__*/null";

pub fn run(path: &Path, all: bool, output_file: Option<String>, template: Option<String>) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;

    let decisions = if all {
        db::decisions::get_all(&conn)?
    } else {
        db::decisions::list(
            &conn,
            &db::decisions::ListFilter {
                status: Some(Status::Active),
                level: None,
                label: None,
                kind: None,
                weight: None,
                scope: None,
            },
        )?
    };

    let project_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("PROJECT")
        .to_uppercase();

    let custom_template;
    let tmpl = match template {
        Some(ref t) => {
            custom_template = std::fs::read_to_string(t)?;
            &custom_template
        }
        None => REPORT_TEMPLATE,
    };

    let report_data = build_report_data(&conn, &decisions, &project_name)?;
    let json_str = serde_json::to_string(&report_data)?;
    if !tmpl.contains(DATA_PLACEHOLDER) {
        return Err(crate::error::DictumError::InvalidTemplate);
    }
    let html = tmpl.replacen(DATA_PLACEHOLDER, &json_str, 1);

    let mut writer: Box<dyn Write> = match output_file {
        Some(ref f) => Box::new(std::fs::File::create(f)?),
        None => Box::new(std::io::stdout()),
    };

    writer.write_all(html.as_bytes())?;

    if output_file.is_some() {
        eprintln!("Report generated: {} decisions", decisions.len());
    }

    Ok(())
}

fn build_report_data(
    conn: &rusqlite::Connection,
    decisions: &[crate::model::Decision],
    project_name: &str,
) -> Result<serde_json::Value> {
    let mut active = 0u32;
    let mut deprecated = 0u32;
    let mut superseded = 0u32;
    let mut by_kind = std::collections::BTreeMap::<String, u32>::new();
    let mut by_level = std::collections::BTreeMap::<String, u32>::new();

    let mut decision_values = Vec::new();

    for d in decisions {
        match d.status {
            Status::Active => active += 1,
            Status::Deprecated => deprecated += 1,
            Status::Superseded => superseded += 1,
            _ => {}
        }
        *by_kind.entry(d.kind.to_string()).or_default() += 1;
        *by_level.entry(d.level.to_string()).or_default() += 1;

        let links = db::links::get_for_decision(conn, &d.id)?;
        let links_json: Vec<serde_json::Value> = links
            .iter()
            .map(|l| {
                json!({
                    "source_id": l.source_id,
                    "target_id": l.target_id,
                    "kind": l.kind.to_string(),
                    "reason": l.reason,
                })
            })
            .collect();

        decision_values.push(json!({
            "id": d.id,
            "title": d.title,
            "body": d.body,
            "kind": d.kind.to_string(),
            "weight": d.weight.to_string(),
            "level": d.level.to_string(),
            "status": d.status.to_string(),
            "scope": d.scope,
            "rebuttal": d.rebuttal,
            "superseded_by": d.superseded_by,
            "labels": d.labels,
            "links": links_json,
        }));
    }

    Ok(json!({
        "meta": {
            "project_name": project_name,
            "total": decisions.len(),
            "active": active,
            "deprecated": deprecated,
            "superseded": superseded,
            "by_kind": by_kind,
            "by_level": by_level,
        },
        "decisions": decision_values,
    }))
}
