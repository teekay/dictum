use rusqlite::{params, Connection};

use crate::error::Result;

pub fn add(conn: &Connection, decision_id: &str, label: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO labels (decision_id, label) VALUES (?1, ?2)",
        params![decision_id, label],
    )?;
    Ok(())
}

pub fn get_for_decision(conn: &Connection, decision_id: &str) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT label FROM labels WHERE decision_id = ?1 ORDER BY label")?;
    let labels = stmt
        .query_map(params![decision_id], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(labels)
}
