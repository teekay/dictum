---
description: Check project decisions before making choices. Use proactively.
allowed-tools: Bash(dictum *)
---

# Dictum — Decision Guardrails

Check existing decisions before making non-trivial implementation choices.

## Commands

```bash
# Load decisions for your task (preferred — saves tokens)
dictum context --format compact --scope db
dictum context --format compact --kind rule --weight must

# Load all active decisions
dictum context --format compact

# Search, show, or browse
dictum decision query "search terms" --format json
dictum decision show <id> --format json
dictum decision tree

# Generate an HTML report for human review (meetings, 1:1s, executive review)
dictum report -o report.html          # active decisions only
dictum report --all -o report.html    # include deprecated and superseded
```

Use `--format compact` for `context` (minified JSON). Use `--format json` for `show`/`query`.

## How to interpret results

**Levels** — strategic > tactical > operational. Higher levels take precedence.

**Kinds** — how to reason about each decision:

| kind | action |
|------|--------|
| principle | Axiom. Never question; derive from it. |
| constraint | External boundary. Work within it. |
| assumption | Monitor. Flag immediately if evidence contradicts it. |
| choice | Respect, but revisitable with new information. |
| rule | Enforce unconditionally. Stop and flag if you'd violate it. |
| goal | Move toward it, not away. |

**Weights** — `must`: mandatory (unless `rebuttal` condition applies). `should`: deviate only with good reason. `may`: use judgement.

**Scope** limits where a decision applies. **Rebuttal** is the override condition — flag if met.

## Rules

1. **Follow applicable decisions.** Mention which decision (by ID) guided your choice.
2. **Never silently override.** If a user request contradicts a decision, quote its ID/title/kind/weight and ask the user how to proceed.
3. **Flag invalid assumptions.** Quote the decision ID and the contradicting evidence.
4. **Suggest new decisions** for significant uncovered choices:
   ```
   dictum decision add "description" --level tactical --kind choice --weight should
   ```

## Vocabulary

Before creating/amending decisions, check existing scopes and labels in `dictum context`. Use established terms (e.g. `db` not `database` if `db` exists). Ask the human if unsure — they pick the preferred term.
