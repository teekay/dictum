pub mod store;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "grafeo")]
pub mod grafeo;

pub use store::{ListFilter, Store};
#[allow(unused_imports)]
pub use store::Neighborhood;

use std::path::Path;

use crate::config::Config;
use crate::error::{DictumError, Result};

pub fn open(dictum_dir: &Path) -> Result<Box<dyn Store>> {
    check_backend_marker(dictum_dir)?;

    #[cfg(feature = "sqlite")]
    { return Ok(Box::new(sqlite::SqliteStore::open(dictum_dir)?)); }

    #[cfg(feature = "grafeo")]
    { return Ok(Box::new(grafeo::GrafeoStore::open(dictum_dir)?)); }
}

fn check_backend_marker(dictum_dir: &Path) -> Result<()> {
    let config = Config::load(dictum_dir)?;
    let expected = compiled_backend();
    if config.backend != expected {
        return Err(DictumError::BackendMismatch {
            found: config.backend,
            expected: expected.to_string(),
        });
    }
    Ok(())
}

pub fn compiled_backend() -> &'static str {
    #[cfg(feature = "sqlite")]
    { "sqlite" }
    #[cfg(feature = "grafeo")]
    { "grafeo" }
}

pub fn compiled_backend_gitignore() -> &'static str {
    #[cfg(feature = "sqlite")]
    { "dictum.db\ndictum.db-wal\ndictum.db-shm\n" }
    #[cfg(feature = "grafeo")]
    { "dictum.grafeo/\n" }
}
