use rusqlite::{params, Connection, Row};

use crate::error::{DictumError, Result};
use crate::model::{Decision, Kind, Level, Status, Weight};

fn decision_from_row(row: &Row) -> rusqlite::Result<Decision> {
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
        kind: row
            .get::<_, String>(9)?
            .parse::<Kind>()
            .unwrap_or(Kind::Choice),
        weight: row
            .get::<_, String>(10)?
            .parse::<Weight>()
            .unwrap_or(Weight::Should),
        rebuttal: row.get(11)?,
        scope: row.get(12)?,
    })
}

const SELECT_COLS: &str = "id, title, body, level, status, superseded_by, author, created_at, updated_at, kind, weight, rebuttal, scope";

pub fn insert(conn: &Connection, decision: &Decision) -> Result<()> {
    conn.execute(
        "INSERT INTO decisions (id, title, body, level, status, superseded_by, author, created_at, updated_at, kind, weight, rebuttal, scope)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
            decision.kind.to_string(),
            decision.weight.to_string(),
            decision.rebuttal,
            decision.scope,
        ],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> Result<Decision> {
    let sql = format!("SELECT {} FROM decisions WHERE id = ?1", SELECT_COLS);
    let mut stmt = conn.prepare(&sql)?;

    let decision = stmt
        .query_row(params![id], decision_from_row)
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
    pub kind: Option<Kind>,
    pub weight: Option<Weight>,
    pub scope: Option<String>,
}

pub fn list(conn: &Connection, filter: &ListFilter) -> Result<Vec<Decision>> {
    let mut sql = format!(
        "SELECT DISTINCT d.{} FROM decisions d",
        SELECT_COLS.replace(", ", ", d.")
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
    if let Some(ref kind) = filter.kind {
        param_values.push(kind.to_string());
        conditions.push(format!("d.kind = ?{}", param_values.len()));
    }
    if let Some(ref weight) = filter.weight {
        param_values.push(weight.to_string());
        conditions.push(format!("d.weight = ?{}", param_values.len()));
    }
    if let Some(ref scope) = filter.scope {
        param_values.push(scope.clone());
        conditions.push(format!("d.scope = ?{}", param_values.len()));
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
        .query_map(params.as_slice(), decision_from_row)?
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
    let sql = format!(
        "SELECT {} FROM decisions WHERE title LIKE ?1 OR body LIKE ?1 OR rebuttal LIKE ?1 OR scope LIKE ?1 ORDER BY created_at DESC",
        SELECT_COLS
    );
    let mut stmt = conn.prepare(&sql)?;
    let decisions = stmt
        .query_map(params![pattern], decision_from_row)?
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
            kind: None,
            weight: None,
            scope: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        db::initialize(&conn).unwrap();
        conn
    }

    fn make_decision(id: &str, kind: Kind, weight: Weight, scope: Option<&str>) -> Decision {
        Decision {
            id: id.to_string(),
            title: format!("Decision {}", id),
            body: None,
            level: Level::Tactical,
            status: Status::Active,
            superseded_by: None,
            author: "test".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            labels: Vec::new(),
            kind,
            weight,
            rebuttal: None,
            scope: scope.map(|s| s.to_string()),
        }
    }

    #[test]
    fn filter_by_kind() {
        let conn = test_db();
        insert(&conn, &make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        insert(&conn, &make_decision("d-2", Kind::Choice, Weight::Should, None)).unwrap();
        insert(&conn, &make_decision("d-3", Kind::Rule, Weight::Should, None)).unwrap();

        let results = list(&conn, &ListFilter {
            kind: Some(Kind::Rule),
            level: None, status: None, label: None, weight: None, scope: None,
        }).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|d| d.kind == Kind::Rule));
    }

    #[test]
    fn filter_by_weight() {
        let conn = test_db();
        insert(&conn, &make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        insert(&conn, &make_decision("d-2", Kind::Choice, Weight::Should, None)).unwrap();

        let results = list(&conn, &ListFilter {
            weight: Some(Weight::Must),
            level: None, status: None, label: None, kind: None, scope: None,
        }).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d-1");
    }

    #[test]
    fn filter_by_scope() {
        let conn = test_db();
        insert(&conn, &make_decision("d-1", Kind::Rule, Weight::Must, Some("auth"))).unwrap();
        insert(&conn, &make_decision("d-2", Kind::Rule, Weight::Must, Some("logging"))).unwrap();
        insert(&conn, &make_decision("d-3", Kind::Rule, Weight::Must, None)).unwrap();

        let results = list(&conn, &ListFilter {
            scope: Some("auth".to_string()),
            level: None, status: None, label: None, kind: None, weight: None,
        }).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d-1");
    }

    #[test]
    fn filter_combined_kind_and_weight() {
        let conn = test_db();
        insert(&conn, &make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        insert(&conn, &make_decision("d-2", Kind::Rule, Weight::Should, None)).unwrap();
        insert(&conn, &make_decision("d-3", Kind::Choice, Weight::Must, None)).unwrap();

        let results = list(&conn, &ListFilter {
            kind: Some(Kind::Rule),
            weight: Some(Weight::Must),
            level: None, status: None, label: None, scope: None,
        }).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d-1");
    }

    #[test]
    fn new_fields_roundtrip_through_db() {
        let conn = test_db();
        let d = Decision {
            rebuttal: Some("unless audit requires it".to_string()),
            scope: Some("auth".to_string()),
            ..make_decision("d-1", Kind::Assumption, Weight::May, None)
        };
        insert(&conn, &d).unwrap();

        let got = get(&conn, "d-1").unwrap();
        assert_eq!(got.kind, Kind::Assumption);
        assert_eq!(got.weight, Weight::May);
        assert_eq!(got.rebuttal.as_deref(), Some("unless audit requires it"));
        assert_eq!(got.scope.as_deref(), Some("auth"));
    }
}
