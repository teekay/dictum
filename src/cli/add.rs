use std::path::Path;

use crate::config::Config;
use crate::db;
use crate::error::Result;
use crate::format::OutputFormat;
use crate::id::generate_id;
use crate::model::{Decision, Kind, Level, Link, LinkKind, Status, Weight};

pub struct AddArgs {
    pub title: String,
    pub level: Level,
    pub parent: Option<String>,
    pub label: Vec<String>,
    pub body: Option<String>,
    pub author: Option<String>,
    pub format: Option<String>,
    pub kind: Kind,
    pub weight: Weight,
    pub rebuttal: Option<String>,
    pub scope: Option<String>,
}

pub fn run(path: &Path, args: AddArgs, is_tty: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let config = Config::load(&dictum_dir)?;
    let conn = db::open(&dictum_dir)?;

    let now = chrono::Utc::now().to_rfc3339();
    let author = args
        .author
        .or(config.default_author.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let id = generate_id(&config.prefix, &args.title, &now);

    let decision = Decision {
        id: id.clone(),
        title: args.title,
        body: args.body,
        level: args.level,
        status: Status::Active,
        superseded_by: None,
        author,
        created_at: now.clone(),
        updated_at: now.clone(),
        labels: args.label.clone(),
        kind: args.kind,
        weight: args.weight,
        rebuttal: args.rebuttal,
        scope: args.scope,
    };

    db::decisions::insert(&conn, &decision)?;

    // Add labels
    for label in &args.label {
        db::labels::add(&conn, &id, label)?;
    }

    // Add parent link if specified
    if let Some(ref parent_id) = args.parent {
        // Verify parent exists
        db::decisions::get(&conn, parent_id)?;

        let link = Link {
            source_id: id.clone(),
            target_id: parent_id.clone(),
            kind: LinkKind::Refines,
            created_at: now,
            reason: None,
        };
        db::links::insert(&conn, &link)?;
    }

    let format = OutputFormat::from_str_or_auto(args.format.as_deref(), is_tty);
    match format {
        OutputFormat::Text => println!("Added: {}", id),
        _ => {
            let output = serde_json::to_string(&decision)?;
            println!("{}", output);
        }
    }

    Ok(())
}
