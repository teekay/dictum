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
    Ok(conn)
}

pub fn initialize(conn: &Connection) -> Result<()> {
    conn.execute_batch(schema::CREATE_DECISIONS_TABLE)?;
    conn.execute_batch(schema::CREATE_LINKS_TABLE)?;
    conn.execute_batch(schema::CREATE_LABELS_TABLE)?;
    Ok(())
}
