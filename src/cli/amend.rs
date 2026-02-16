use std::path::Path;

use crate::config::Config;
use crate::db;
use crate::error::Result;
use crate::format::OutputFormat;
use crate::id::generate_id;
use crate::model::{Decision, Link, LinkKind, Status};

pub struct AmendArgs {
    pub id: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub format: Option<String>,
}

pub fn run(path: &Path, args: AmendArgs, is_tty: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let config = Config::load(&dictum_dir)?;
    let conn = db::open(&dictum_dir)?;

    let old = db::decisions::get(&conn, &args.id)?;

    let now = chrono::Utc::now().to_rfc3339();
    let new_title = args.title.unwrap_or_else(|| old.title.clone());
    let new_id = generate_id(&config.prefix, &new_title, &now);

    let new_decision = Decision {
        id: new_id.clone(),
        title: new_title,
        body: args.body.or(old.body.clone()),
        level: old.level.clone(),
        status: Status::Active,
        superseded_by: None,
        author: old.author.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
        labels: old.labels.clone(),
    };

    db::decisions::insert(&conn, &new_decision)?;

    // Copy labels
    for label in &old.labels {
        db::labels::add(&conn, &new_id, label)?;
    }

    // Create supersedes link
    let link = Link {
        source_id: new_id.clone(),
        target_id: args.id.clone(),
        kind: LinkKind::Supersedes,
        created_at: now,
    };
    db::links::insert(&conn, &link)?;

    // Mark old as superseded
    db::decisions::update_status(&conn, &args.id, &Status::Superseded, Some(&new_id))?;

    let format = OutputFormat::from_str_or_auto(args.format.as_deref(), is_tty);
    match format {
        OutputFormat::Text => {
            println!("Amended: {} -> {}", args.id, new_id);
        }
        _ => {
            let output = serde_json::to_string(&new_decision)?;
            println!("{}", output);
        }
    }

    Ok(())
}

pub fn run_deprecate(
    path: &Path,
    id: &str,
    reason: Option<String>,
    fmt: Option<String>,
    is_tty: bool,
) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;

    // Verify exists
    let _decision = db::decisions::get(&conn, id)?;

    db::decisions::update_status(&conn, id, &Status::Deprecated, None)?;

    let format = OutputFormat::from_str_or_auto(fmt.as_deref(), is_tty);
    match format {
        OutputFormat::Text => {
            print!("Deprecated: {}", id);
            if let Some(reason) = reason {
                print!(" ({})", reason);
            }
            println!();
        }
        _ => {
            let updated = db::decisions::get(&conn, id)?;
            let output = serde_json::to_string(&updated)?;
            println!("{}", output);
        }
    }

    Ok(())
}
