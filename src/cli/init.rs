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

    // Write default config first so db::open can read the backend marker
    let config = Config::default();
    config.save(&dictum_dir)?;

    // Initialize database (open handles CREATE TABLE IF NOT EXISTS internally)
    let _store = crate::db::open(&dictum_dir)?;

    // Write .gitignore for backend-specific db artifacts
    let gitignore = crate::db::compiled_backend_gitignore();
    std::fs::write(dictum_dir.join(".gitignore"), gitignore)?;

    println!("Initialized dictum in {}", dictum_dir.display());
    Ok(())
}
