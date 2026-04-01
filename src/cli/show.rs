use std::path::Path;

use crate::db;
use crate::error::Result;
use crate::format::{self, OutputFormat};

pub fn run(path: &Path, id: &str, fmt: Option<String>, is_tty: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let store = db::open(&dictum_dir)?;
    let decision = store.decision_get(id)?;
    let links = store.links_for_decision(id)?;

    let format = OutputFormat::from_str_or_auto(fmt.as_deref(), is_tty);
    let output = format::format_decision(&decision, &links, &format)?;
    print!("{}", output);

    Ok(())
}
