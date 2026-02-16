pub mod add;
pub mod amend;
pub mod context;
pub mod init;
pub mod io;
pub mod link;
pub mod list;
pub mod query;
pub mod show;

use std::path::Path;

use crate::error::{DictumError, Result};

pub fn ensure_init(dictum_dir: &Path) -> Result<()> {
    if !dictum_dir.exists() {
        return Err(DictumError::NotInitialized);
    }
    Ok(())
}
