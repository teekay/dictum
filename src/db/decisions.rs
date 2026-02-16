use rusqlite::{params, Connection};

use crate::error::{DictumError, Result};
use crate::model::{Decision, Level, Status};

pub fn insert(conn: &Connection, decision: &Decision) -> Result<()> {
    conn.execute(
        "INSERT INTO decisions (id, title, body, level, status, superseded_by, author, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            decision.id,
            decision.title,
            decision.body,
            decision.level.to_string(),
            decision.status.to_string(),
            decision.superseded_by,
            decision.author,
            decision.created_at,
            decision.updated_at,
        ],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> Result<Decision> {
    let mut stmt = conn.prepare(
        "SELECT id, title, body, level, status, superseded_by, author, created_at, updated_at
         FROM decisions WHERE id = ?1",
    )?;

    let decision = stmt
        .query_row(params![id], |row| {
            Ok(Decision {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                level: row
                    .get::<_, String>(3)?
                    .parse::<Level>()
                    .unwrap_or(Level::Tactical),
                status: row
                    .get::<_, String>(4)?
                    .parse::<Status>()
                    .unwrap_or(Status::Active),
                superseded_by: row.get(5)?,
                author: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                labels: Vec::new(),
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DictumError::DecisionNotFound(id.to_string()),
            other => DictumError::Db(other),
        })?;

    // Load labels
    let labels = crate::db::labels::get_for_decision(conn, id)?;
    Ok(Decision { labels, ..decision })
}

pub struct ListFilter {
    pub level: Option<Level>,
    pub status: Option<Status>,
    pub label: Option<String>,
}

pub fn list(conn: &Connection, filter: &ListFilter) -> Result<Vec<Decision>> {
    let mut sql = String::from(
        "SELECT DISTINCT d.id, d.title, d.body, d.level, d.status, d.superseded_by, d.author, d.created_at, d.updated_at
         FROM decisions d",
    );
    let mut conditions = Vec::new();
    let mut param_values: Vec<String> = Vec::new();

    if filter.label.is_some() {
        sql.push_str(" LEFT JOIN labels l ON d.id = l.decision_id");
    }

    if let Some(ref level) = filter.level {
        param_values.push(level.to_string());
        conditions.push(format!("d.level = ?{}", param_values.len()));
    }
    if let Some(ref status) = filter.status {
        param_values.push(status.to_string());
        conditions.push(format!("d.status = ?{}", param_values.len()));
    }
    if let Some(ref label) = filter.label {
        param_values.push(label.clone());
        conditions.push(format!("l.label = ?{}", param_values.len()));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY d.created_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::types::ToSql> = param_values
        .iter()
        .map(|v| v as &dyn rusqlite::types::ToSql)
        .collect();

    let decisions = stmt
        .query_map(params.as_slice(), |row| {
            Ok(Decision {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                level: row
                    .get::<_, String>(3)?
                    .parse::<Level>()
                    .unwrap_or(Level::Tactical),
                status: row
                    .get::<_, String>(4)?
                    .parse::<Status>()
                    .unwrap_or(Status::Active),
                superseded_by: row.get(5)?,
                author: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                labels: Vec::new(),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    // Load labels for each decision
    let mut result = Vec::new();
    for d in decisions {
        let labels = crate::db::labels::get_for_decision(conn, &d.id)?;
        result.push(Decision { labels, ..d });
    }
    Ok(result)
}

pub fn update_status(
    conn: &Connection,
    id: &str,
    status: &Status,
    superseded_by: Option<&str>,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn.execute(
        "UPDATE decisions SET status = ?1, superseded_by = ?2, updated_at = ?3 WHERE id = ?4",
        params![status.to_string(), superseded_by, now, id],
    )?;
    if rows == 0 {
        return Err(DictumError::DecisionNotFound(id.to_string()));
    }
    Ok(())
}

pub fn search(conn: &Connection, query: &str) -> Result<Vec<Decision>> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, title, body, level, status, superseded_by, author, created_at, updated_at
         FROM decisions WHERE title LIKE ?1 OR body LIKE ?1
         ORDER BY created_at DESC",
    )?;
    let decisions = stmt
        .query_map(params![pattern], |row| {
            Ok(Decision {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                level: row
                    .get::<_, String>(3)?
                    .parse::<Level>()
                    .unwrap_or(Level::Tactical),
                status: row
                    .get::<_, String>(4)?
                    .parse::<Status>()
                    .unwrap_or(Status::Active),
                superseded_by: row.get(5)?,
                author: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                labels: Vec::new(),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut result = Vec::new();
    for d in decisions {
        let labels = crate::db::labels::get_for_decision(conn, &d.id)?;
        result.push(Decision { labels, ..d });
    }
    Ok(result)
}

pub fn get_all(conn: &Connection) -> Result<Vec<Decision>> {
    list(
        conn,
        &ListFilter {
            level: None,
            status: None,
            label: None,
        },
    )
}
