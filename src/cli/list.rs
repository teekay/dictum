use std::path::Path;

use crate::db;
use crate::db::decisions::ListFilter;
use crate::error::Result;
use crate::format::{self, OutputFormat};
use crate::model::{Level, Status};

pub struct ListArgs {
    pub tree: bool,
    pub level: Option<String>,
    pub status: Option<String>,
    pub label: Option<String>,
    pub format: Option<String>,
}

pub fn run(path: &Path, args: ListArgs, is_tty: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;

    let level = args.level.map(|l| l.parse::<Level>()).transpose()?;
    let status = args.status.map(|s| s.parse::<Status>()).transpose()?;

    let filter = ListFilter {
        level,
        status,
        label: args.label,
    };

    let decisions = db::decisions::list(&conn, &filter)?;
    let fmt = OutputFormat::from_str_or_auto(args.format.as_deref(), is_tty);

    if args.tree {
        let refines_links = db::links::get_refines_links(&conn)?;
        let output = crate::format::tree::format_tree(&decisions, &refines_links);
        print!("{}", output);
    } else {
        let output = format::format_decision_list(&decisions, &fmt)?;
        print!("{}", output);
    }

    Ok(())
}

pub fn run_tree(path: &Path) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;
    let decisions = db::decisions::get_all(&conn)?;
    let refines_links = db::links::get_refines_links(&conn)?;
    let output = crate::format::tree::format_tree(&decisions, &refines_links);
    print!("{}", output);

    Ok(())
}
