use std::path::Path;

use crate::config::Config;
use crate::error::{DictumError, Result};

pub fn run(path: &Path) -> Result<()> {
    let dictum_dir = path.join(".dictum");

    if dictum_dir.exists() {
        return Err(DictumError::AlreadyInitialized(
            dictum_dir.display().to_string(),
        ));
    }

    std::fs::create_dir_all(&dictum_dir)?;

    // Initialize database
    let conn = crate::db::open(&dictum_dir)?;
    crate::db::initialize(&conn)?;

    // Write default config
    let config = Config::default();
    config.save(&dictum_dir)?;

    // Write .gitignore for the db file
    std::fs::write(dictum_dir.join(".gitignore"), "dictum.db\ndictum.db-wal\ndictum.db-shm\n")?;

    println!("Initialized dictum in {}", dictum_dir.display());
    Ok(())
}
