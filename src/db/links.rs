use rusqlite::{params, Connection};

use crate::error::{DictumError, Result};
use crate::model::{Link, LinkKind};

pub fn insert(conn: &Connection, link: &Link) -> Result<()> {
    if link.source_id == link.target_id {
        return Err(DictumError::SelfLink);
    }
    conn.execute(
        "INSERT INTO links (source_id, target_id, kind, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            link.source_id,
            link.target_id,
            link.kind.to_string(),
            link.created_at,
        ],
    )
    .map_err(|e| match e {
        rusqlite::Error::SqliteFailure(err, _)
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            DictumError::LinkAlreadyExists
        }
        other => DictumError::Db(other),
    })?;
    Ok(())
}

pub fn delete(conn: &Connection, source_id: &str, kind: &LinkKind, target_id: &str) -> Result<()> {
    let rows = conn.execute(
        "DELETE FROM links WHERE source_id = ?1 AND target_id = ?2 AND kind = ?3",
        params![source_id, target_id, kind.to_string()],
    )?;
    if rows == 0 {
        return Err(DictumError::LinkNotFound);
    }
    Ok(())
}

pub fn get_for_decision(conn: &Connection, decision_id: &str) -> Result<Vec<Link>> {
    let mut stmt = conn.prepare(
        "SELECT source_id, target_id, kind, created_at FROM links
         WHERE source_id = ?1 OR target_id = ?1
         ORDER BY created_at",
    )?;
    let links = stmt
        .query_map(params![decision_id], |row| {
            Ok(Link {
                source_id: row.get(0)?,
                target_id: row.get(1)?,
                kind: row
                    .get::<_, String>(2)?
                    .parse::<LinkKind>()
                    .unwrap_or(LinkKind::Supports),
                created_at: row.get(3)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(links)
}

/// Get all refines links for building the tree
pub fn get_refines_links(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn
        .prepare("SELECT source_id, target_id FROM links WHERE kind = 'refines' ORDER BY created_at")?;
    let links = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(links)
}
