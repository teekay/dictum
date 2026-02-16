use std::path::Path;

use crate::db;
use crate::error::Result;
use crate::format::{self, OutputFormat};

pub fn run(path: &Path, query: &str, fmt: Option<String>, is_tty: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;
    let decisions = db::decisions::search(&conn, query)?;

    let format = OutputFormat::from_str_or_auto(fmt.as_deref(), is_tty);
    let output = format::format_decision_list(&decisions, &format)?;
    print!("{}", output);

    Ok(())
}
