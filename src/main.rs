mod cli;
mod config;
mod db;
mod error;
mod format;
mod id;
mod model;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dictum", about = "Track decisions over time", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .dictum/ in current directory
    Init,

    /// Add a decision
    Add {
        /// Decision statement
        title: String,
        /// Level: strategic, tactical, or operational
        #[arg(long, default_value = "tactical")]
        level: String,
        /// Parent decision ID (creates a "refines" link)
        #[arg(long)]
        parent: Option<String>,
        /// Label(s) to tag the decision
        #[arg(long)]
        label: Vec<String>,
        /// Additional context or rationale
        #[arg(long)]
        body: Option<String>,
        /// Author name
        #[arg(long)]
        author: Option<String>,
        /// Output format: text, json, jsonl
        #[arg(long)]
        format: Option<String>,
    },

    /// Show a decision and its links
    Show {
        /// Decision ID
        id: String,
        /// Output format: text, json, jsonl
        #[arg(long)]
        format: Option<String>,
    },

    /// List decisions
    List {
        /// Show as hierarchy (refines links)
        #[arg(long)]
        tree: bool,
        /// Filter by level
        #[arg(long)]
        level: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by label
        #[arg(long)]
        label: Option<String>,
        /// Output format: text, json, jsonl
        #[arg(long)]
        format: Option<String>,
    },

    /// Create a relationship between decisions
    Link {
        /// Source decision ID
        source: String,
        /// Link kind: refines, supports, supersedes, conflicts, requires
        kind: String,
        /// Target decision ID
        target: String,
    },

    /// Remove a relationship between decisions
    Unlink {
        /// Source decision ID
        source: String,
        /// Link kind
        kind: String,
        /// Target decision ID
        target: String,
    },

    /// Supersede a decision with a new one
    Amend {
        /// Decision ID to supersede
        id: String,
        /// New decision statement
        #[arg(long)]
        title: Option<String>,
        /// Why it changed
        #[arg(long)]
        body: Option<String>,
        /// Output format: text, json, jsonl
        #[arg(long)]
        format: Option<String>,
    },

    /// Mark a decision as deprecated
    Deprecate {
        /// Decision ID
        id: String,
        /// Reason for deprecation
        #[arg(long)]
        reason: Option<String>,
        /// Output format: text, json, jsonl
        #[arg(long)]
        format: Option<String>,
    },

    /// Search decisions
    Query {
        /// Search text
        question: String,
        /// Output format: text, json, jsonl
        #[arg(long)]
        format: Option<String>,
    },

    /// Export full graph to JSONL
    Export {
        /// Output file (default: stdout)
        #[arg(short)]
        o: Option<String>,
    },

    /// Import from JSONL
    Import {
        /// Input file (default: stdin)
        #[arg(short)]
        i: Option<String>,
        /// Preview only
        #[arg(long)]
        dry_run: bool,
    },

    /// Visual tree of decisions (refines hierarchy)
    Tree,

    /// Dump active decisions as compact context for LLM agents
    Context {
        /// Output format: text, json
        #[arg(long)]
        format: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().expect("cannot determine current directory");
    let is_tty = atty::is(atty::Stream::Stdout);

    let result = match cli.command {
        Commands::Init => cli::init::run(&cwd),

        Commands::Add {
            title,
            level,
            parent,
            label,
            body,
            author,
            format,
        } => {
            let level = match level.parse() {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };
            cli::add::run(
                &cwd,
                cli::add::AddArgs {
                    title,
                    level,
                    parent,
                    label,
                    body,
                    author,
                    format,
                },
                is_tty,
            )
        }

        Commands::Show { id, format } => cli::show::run(&cwd, &id, format, is_tty),

        Commands::List {
            tree,
            level,
            status,
            label,
            format,
        } => cli::list::run(
            &cwd,
            cli::list::ListArgs {
                tree,
                level,
                status,
                label,
                format,
            },
            is_tty,
        ),

        Commands::Link {
            source,
            kind,
            target,
        } => cli::link::run_link(&cwd, &source, &kind, &target),

        Commands::Unlink {
            source,
            kind,
            target,
        } => cli::link::run_unlink(&cwd, &source, &kind, &target),

        Commands::Amend {
            id,
            title,
            body,
            format,
        } => cli::amend::run(
            &cwd,
            cli::amend::AmendArgs {
                id,
                title,
                body,
                format,
            },
            is_tty,
        ),

        Commands::Deprecate { id, reason, format } => {
            cli::amend::run_deprecate(&cwd, &id, reason, format, is_tty)
        }

        Commands::Query { question, format } => {
            cli::query::run(&cwd, &question, format, is_tty)
        }

        Commands::Export { o } => cli::io::run_export(&cwd, o),
        Commands::Import { i, dry_run } => cli::io::run_import(&cwd, i, dry_run),
        Commands::Tree => cli::list::run_tree(&cwd),
        Commands::Context { format } => cli::context::run(&cwd, format, is_tty),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
