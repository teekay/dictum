pub mod decisions;
pub mod labels;
pub mod links;
pub mod schema;

use rusqlite::Connection;
use std::path::Path;

use crate::error::Result;

pub fn open(dictum_dir: &Path) -> Result<Connection> {
    let db_path = dictum_dir.join("dictum.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrate(&conn)?;
    initialize(&conn)?;
    Ok(conn)
}

pub fn initialize(conn: &Connection) -> Result<()> {
    conn.execute_batch(schema::CREATE_DECISIONS_TABLE)?;
    conn.execute_batch(schema::CREATE_LINKS_TABLE)?;
    conn.execute_batch(schema::CREATE_LABELS_TABLE)?;
    Ok(())
}

fn needs_migration(conn: &Connection) -> Result<bool> {
    let mut stmt = conn.prepare("PRAGMA table_info(decisions)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(!columns.iter().any(|c| c == "kind"))
}

fn migrate(conn: &Connection) -> Result<()> {
    // Check if decisions table exists at all (fresh DB or not yet initialized)
    let table_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='decisions'",
        [],
        |row| row.get(0),
    )?;

    if !table_exists {
        return Ok(());
    }

    if !needs_migration(conn)? {
        return Ok(());
    }

    // Migrate decisions table
    for sql in schema::MIGRATE_DECISIONS_V2 {
        conn.execute_batch(sql)?;
    }

    // Migrate links table — need to disable foreign keys for table recreation
    conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
    for sql in schema::MIGRATE_LINKS_V2 {
        conn.execute_batch(sql)?;
    }
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    Ok(())
}
