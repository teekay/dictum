use std::path::Path;

use crate::db;
use crate::error::Result;
use crate::format::{self, OutputFormat};

pub fn run(path: &Path, id: &str, fmt: Option<String>, is_tty: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;
    let decision = db::decisions::get(&conn, id)?;
    let links = db::links::get_for_decision(&conn, id)?;

    let format = OutputFormat::from_str_or_auto(fmt.as_deref(), is_tty);
    let output = format::format_decision(&decision, &links, &format)?;
    print!("{}", output);

    Ok(())
}
