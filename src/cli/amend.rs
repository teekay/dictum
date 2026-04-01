use std::path::Path;

use crate::config::Config;
use crate::db;
use crate::error::Result;
use crate::format::OutputFormat;
use crate::id::generate_id;
use crate::model::{Decision, Kind, Link, LinkKind, Status, Weight};

pub struct AmendArgs {
    pub id: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub format: Option<String>,
    pub kind: Option<Kind>,
    pub weight: Option<Weight>,
    pub rebuttal: Option<String>,
    pub scope: Option<String>,
}

pub fn run(path: &Path, args: AmendArgs, is_tty: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let config = Config::load(&dictum_dir)?;
    let mut store = db::open(&dictum_dir)?;

    let old = store.decision_get(&args.id)?;

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
        kind: args.kind.unwrap_or(old.kind.clone()),
        weight: args.weight.unwrap_or(old.weight.clone()),
        rebuttal: args.rebuttal.or(old.rebuttal.clone()),
        scope: args.scope.or(old.scope.clone()),
    };

    store.decision_insert(&new_decision)?;

    for label in &old.labels {
        store.label_add(&new_id, label)?;
    }

    let link = Link {
        source_id: new_id.clone(),
        target_id: args.id.clone(),
        kind: LinkKind::Supersedes,
        created_at: now,
        reason: None,
    };
    store.link_insert(&link)?;

    store.decision_update_status(&args.id, &Status::Superseded, Some(&new_id))?;

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

    let mut store = db::open(&dictum_dir)?;

    store.decision_get(id)?;
    store.decision_update_status(id, &Status::Deprecated, None)?;

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
            let updated = store.decision_get(id)?;
            let output = serde_json::to_string(&updated)?;
            println!("{}", output);
        }
    }

    Ok(())
}
