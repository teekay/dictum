use std::path::Path;

use crate::db;
use crate::error::Result;
use crate::model::{Link, LinkKind, Status};

pub fn run_link(path: &Path, source_id: &str, kind: &str, target_id: &str, reason: Option<String>) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;
    let kind: LinkKind = kind.parse()?;

    // Verify both decisions exist
    db::decisions::get(&conn, source_id)?;
    db::decisions::get(&conn, target_id)?;

    let now = chrono::Utc::now().to_rfc3339();
    let link = Link {
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        kind: kind.clone(),
        created_at: now,
        reason,
    };
    db::links::insert(&conn, &link)?;

    // If supersedes, update the target status
    if kind == LinkKind::Supersedes {
        db::decisions::update_status(&conn, target_id, &Status::Superseded, Some(source_id))?;
    }

    println!("Linked: {} {} {}", source_id, link.kind, target_id);
    Ok(())
}

pub fn run_unlink(path: &Path, source_id: &str, kind: &str, target_id: &str) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;
    let kind: LinkKind = kind.parse()?;

    db::links::delete(&conn, source_id, &kind, target_id)?;

    println!("Unlinked: {} {} {}", source_id, kind, target_id);
    Ok(())
}
