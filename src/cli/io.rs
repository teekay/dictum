use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use crate::db;
use crate::error::{DictumError, Result};
use crate::format::json::format_export_line;
use crate::model::{Decision, Link};

pub fn run_export(path: &Path, output_file: Option<String>) -> Result<()> {
    let dictum_dir = path.join(".dictum");
    crate::cli::ensure_init(&dictum_dir)?;

    let store = db::open(&dictum_dir)?;
    let decisions = store.decision_get_all()?;

    let mut writer: Box<dyn Write> = match output_file {
        Some(ref f) => Box::new(std::fs::File::create(f)?),
        None => Box::new(io::stdout()),
    };

    for d in &decisions {
        let links = store.links_for_decision(&d.id)?;
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
            if std::io::stdin().is_terminal() {
                eprintln!("Error: no input file specified and stdin is a terminal");
                eprintln!("Usage: dictum import -i <file>  or  cat file.jsonl | dictum import");
                std::process::exit(1);
            }
            Box::new(io::BufReader::new(io::stdin()))
        }
    };

    let mut store = db::open(&dictum_dir)?;
    let mut count = 0;
    let mut link_count = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let value: serde_json::Value = serde_json::from_str(&line)?;
        let decision: Decision = serde_json::from_value(value.clone())?;

        if dry_run {
            println!("Would import: [{}] {}", decision.id, decision.title);
        } else {
            match store.decision_insert(&decision) {
                Ok(_) => {}
                Err(DictumError::DecisionAlreadyExists) => continue,
                Err(e) => return Err(e),
            }

            for label in &decision.labels {
                store.label_add(&decision.id, label)?;
            }

            if let Some(links) = value.get("links") {
                if let Ok(links) = serde_json::from_value::<Vec<Link>>(links.clone()) {
                    for link in &links {
                        match store.link_insert(link) {
                            Ok(_) => link_count += 1,
                            Err(DictumError::LinkAlreadyExists) => {}
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
