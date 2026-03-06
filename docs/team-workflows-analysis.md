# Team Workflows & Conflict Resolution — Analysis

Related decision: `d-1tqluy` (strategic goal)

## Current state

- Import is additive-only with insert-or-skip semantics
- Status changes (amend, deprecate) are silently lost on import if the record already exists locally
- IDs are content-hashed so identical decisions deduplicate, but metadata updates don't propagate

## Root cause

Decisions are mutable shared state with causal ordering. State transitions (active → superseded → deprecated) have causal dependencies between them. SQLite gives a snapshot of current state. JSONL gives a flat log. Neither captures the history of transitions in a way that's mergeable across independent timelines.

## Why Beads moved to Dolt

Dolt is "git for databases" — branch, merge, diff at the row level with three-way merge. The Beads author likely hit these walls:

1. **Row-level merge** — git merges text lines, not records. Two developers editing different fields of the same decision produce a text conflict in JSONL that git can't auto-resolve, even when there's no semantic conflict. Dolt merges by cell.
2. **Schema evolution** — as the model grows (new fields, new tables), Dolt handles schema diffs natively. With JSONL, you hand-roll migration logic in the import command.
3. **Referential integrity during merge** — decisions reference each other via links. Git can merge two JSONL files that independently create links to a decision that the other branch deprecated. Dolt's merge can enforce constraints post-merge.
4. **Queryability of history** — "what was the state of decisions when we shipped v2.3?" Dolt has `AS OF` queries. With JSONL+git you check out old commits and re-import.

## Gaps not yet hit but likely

- **Audit trail** — who changed what, when, and why. The current model stores `created_at`/`updated_at` with no history. Team trust requires provenance.
- **Branching decisions** — feature branches may introduce speculative decisions that shouldn't pollute master until merged. Export/import doesn't know about branches.
- **Partial sync** — a team member working on the "billing" scope shouldn't have to pull and merge the entire decision graph. Scoped sync requires something smarter than flat file exchange.
- **Conflict semantics** — what does it mean for two decisions to "conflict" at the merge level? Two people deprecating the same decision for different reasons? Two people amending it in different directions? You need a conflict model before you can build a resolution UI.

## Possible directions

1. **Upsert on import** — update existing records if the incoming version is newer (`updated_at` comparison). Fixes the immediate symptom but not the deeper problems.
2. **Wipe-and-reimport** — treat JSONL as source of truth, drop and reload. Simple but destructive, no conflict detection.
3. **JSONL-as-primary-store** — skip SQLite for shared state; the committed JSONL is canonical, local DB is a cache/index rebuilt on pull. Git handles text merge. Still suffers from line-level vs. record-level merge mismatches.
4. **Dolt backend** — delegate collaboration to a purpose-built tool. Solves merge, history, and integrity but adds a heavy dependency.
5. **Own the collaboration layer** — build merge semantics into dictum itself (CRDT-like or event-sourced). Maximum control, maximum effort.

## Open question

Does dictum own the collaboration layer or delegate it to git/Dolt? This choice shapes everything downstream.
