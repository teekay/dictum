---
description: Check project decisions before making choices. Use proactively.
allowed-tools: Bash(dictum *)
---

# Dictum — Decision Guardrails

This project tracks decisions using [dictum](https://github.com/teekay/dictum). Before making non-trivial choices during implementation, check existing decisions for guidance.

## When to check

- Starting a new feature or task
- Choosing between implementation approaches
- Making architectural or design decisions
- When a user request seems to contradict prior direction

## How to check

Load all active decisions into context:

```bash
dictum context --format json
```

Search for decisions related to your current work:

```bash
dictum query "search terms" --format json
```

View the decision hierarchy:

```bash
dictum tree
```

Show a specific decision and its links:

```bash
dictum show <id> --format json
```

## Decision levels

- **strategic** — high-level direction, rarely overridden (e.g. "We serve restaurant owners")
- **tactical** — how we achieve strategy (e.g. "Single-click booking flow")
- **operational** — day-to-day implementation choices (e.g. "Use PostgreSQL for persistence")

Higher levels take precedence. A tactical decision should not contradict a strategic one.

## What to do

**If a decision applies to your current task:**
Follow it. Mention which decision guided your choice.

**If a user request contradicts an existing decision:**
Flag it clearly. Quote the decision ID and title. Ask the user whether to:
1. Proceed with the request and amend/deprecate the old decision
2. Adjust the request to align with the existing decision
3. Do something else entirely

Do NOT silently override a decision. The whole point is to make the "why" explicit.

**If you make a significant choice and no decision covers it:**
Suggest the user add one:
```
dictum add "description of the decision" --level tactical
```

## Output format

Always use `--format json` when calling dictum so you get structured, parseable output. The text format is for humans at a terminal.
