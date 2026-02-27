use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::db;
use crate::error::Result;
use crate::format::json::format_export_line;
use crate::model::{Decision, Link};

pub fn run_export(path: &Path, output_file: Option<String>) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let conn = db::open(&dictum_dir)?;
    let decisions = db::decisions::get_all(&conn)?;

    let mut writer: Box<dyn Write> = match output_file {
        Some(ref f) => Box::new(std::fs::File::create(f)?),
        None => Box::new(io::stdout()),
    };

    for d in &decisions {
        let links = db::links::get_for_decision(&conn, &d.id)?;
        let line = format_export_line(d, &links)?;
        writeln!(writer, "{}", line)?;
    }

    if output_file.is_some() {
        eprintln!("Exported {} decisions", decisions.len());
    }

    Ok(())
}

pub fn run_import(path: &Path, input_file: Option<String>, dry_run: bool) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let reader: Box<dyn BufRead> = match input_file {
        Some(ref f) => Box::new(io::BufReader::new(std::fs::File::open(f)?)),
        None => {
            if atty::is(atty::Stream::Stdin) {
                eprintln!("Error: no input file specified and stdin is a terminal");
                eprintln!("Usage: dictum import -i <file>  or  cat file.jsonl | dictum import");
                std::process::exit(1);
            }
            Box::new(io::BufReader::new(io::stdin()))
        }
    };

    let conn = db::open(&dictum_dir)?;
    let mut count = 0;
    let mut link_count = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let value: serde_json::Value = serde_json::from_str(&line)?;

        // Extract decision fields
        let decision: Decision = serde_json::from_value(value.clone())?;

        if dry_run {
            println!("Would import: [{}] {}", decision.id, decision.title);
        } else {
            // Insert decision (ignore if already exists)
            match db::decisions::insert(&conn, &decision) {
                Ok(_) => {}
                Err(crate::error::DictumError::Db(rusqlite::Error::SqliteFailure(err, _)))
                    if err.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    // Already exists, skip
                    continue;
                }
                Err(e) => return Err(e),
            }

            // Insert labels
            for label in &decision.labels {
                db::labels::add(&conn, &decision.id, label)?;
            }

            // Insert links if present
            if let Some(links) = value.get("links") {
                if let Ok(links) = serde_json::from_value::<Vec<Link>>(links.clone()) {
                    for link in &links {
                        match db::links::insert(&conn, link) {
                            Ok(_) => link_count += 1,
                            Err(crate::error::DictumError::LinkAlreadyExists) => {}
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
        }
        count += 1;
    }

    if dry_run {
        eprintln!("Dry run: {} decisions would be imported", count);
    } else {
        eprintln!("Imported {} decisions, {} links", count, link_count);
    }

    Ok(())
}
