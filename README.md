# Dictum

A CLI tool for tracking decisions over time. Unlike issue trackers (the "how"), dictum captures the "why" — terse, authoritative statements of intent that form a queryable decision graph.

AI coding agents and humans use it to resolve ambiguity by checking prior decisions before guessing.

Inspired by Steve Yegge's [Beads](https://github.com/steveyegge/beads).

## Why

Issues are downstream of decisions. With the ever-expanding amount of code we generate, it's easy to lose track of "why" we're doing what we're doing.

The goal is to capture decisions made during a project's lifecycle and use them as guardrails, steering every change.

Decisions get lost in chat threads, meeting notes, and commit messages. Dictum gives them a permanent, structured home inside your project. Each decision has a level (strategic/tactical/operational), a status lifecycle, and can be linked to other decisions — forming a graph you can query, export, and version-control.

## What it does

- **Per-project decision store** in `.dictum/` (SQLite + TOML config)
- **Hash-based IDs** (e.g. `d-23mpuu`) derived from content, deterministic and short
- **Decision graph** with typed links: refines, supports, supersedes, conflicts, requires
- **Lifecycle management**: add, amend (supersede), deprecate
- **Filtering**: by level, status, label
- **Tree view**: visualize the refines-hierarchy
- **Full-text search** across titles and bodies
- **JSONL export/import** for portability and git-tracking
- **Auto-detecting output format**: human-readable text on TTY, JSON when piped

## Commands

```
dictum init                                  # Initialize .dictum/ in current directory
dictum add "statement" [options]             # Add a decision
  --level strategic|tactical|operational     #   (default: tactical)
  --parent <id>                              #   Creates a "refines" link
  --label <label>                            #   Tag it (repeatable)
  --body "rationale"                         #   Longer explanation
  --author "name"                            #   Who decided

dictum show <id>                             # Show decision + its links
dictum list [--tree] [--level X] [--status X] [--label X]
dictum tree                                  # Visual refines-hierarchy

dictum link <id> <kind> <id>                 # Create a relationship
dictum unlink <id> <kind> <id>               # Remove a relationship
  # kinds: refines, supports, supersedes, conflicts, requires

dictum amend <id> [--title "new"] [--body "why"]   # Supersede a decision
dictum deprecate <id> [--reason "why"]              # Mark as deprecated
dictum query "search text"                          # Search decisions

dictum export [-o file]                      # Export to JSONL (default: stdout)
dictum import [-i file] [--dry-run]          # Import from JSONL (default: stdin)
```

All commands that produce output accept `--format text|json|jsonl`.

## Build

Requires [Rust](https://rustup.rs/) (1.70+).

```
cd dictum
cargo build --release
```

The binary is at `target/release/dictum`. Copy it somewhere on your PATH:

```
cp target/release/dictum ~/.local/bin/
```

## Run

```
mkdir /tmp/my-project && cd /tmp/my-project
dictum init
dictum add "We serve restaurant owners" --level strategic
dictum add "Single-click booking" --level tactical --parent d-<id>
dictum tree
dictum list
dictum export | dictum import --dry-run
```

## Storage

Everything lives in `.dictum/` at your project root:

| File | Purpose | Git-tracked? |
|------|---------|--------------|
| `config.toml` | Prefix, default author, format prefs | Yes |
| `dictum.db` | SQLite database | No (in `.gitignore`) |
| `decisions.jsonl` | Portable export (via `dictum export`) | Yes |

The JSONL file is the portable format — use `dictum export` before committing and `dictum import` to restore on another machine.

## License

MIT
