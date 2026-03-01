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

Filter by proposition type or obligation:

```bash
dictum list --kind rule --weight must --format json
dictum list --scope auth --format json
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

## Proposition kinds

Each decision has a `kind` that tells you how to reason about it:

- **principle** — an axiom. Never question it; derive other decisions from it.
- **constraint** — an external boundary. Work within it; don't try to remove it.
- **assumption** — believed true but unverified. Monitor for invalidation. If evidence contradicts it, flag immediately.
- **choice** — a deliberate pick among alternatives. Respect it but it can be revisited with new information.
- **rule** — enforce unconditionally. If your implementation would violate a rule, stop and flag it.
- **goal** — a desired outcome. Decisions should move toward goals, not away from them.

## Obligation weight

Each decision has a `weight` that tells you how binding it is:

- **must** — mandatory. No exceptions unless the `rebuttal` condition applies.
- **should** — strong preference. Deviate only with good reason, and mention it.
- **may** — permitted, not required. Use your judgement.

## Scope and rebuttal

- **scope** — limits where a decision applies (e.g. `auth`, `api`, `frontend`). A decision scoped to `auth` does not constrain logging choices.
- **rebuttal** — the condition under which the decision can be overridden. If the rebuttal condition is met, flag it and proceed accordingly.

## Link types

- **refines** — child implements or narrows a parent decision
- **supports** — provides evidence or rationale for another decision
- **supersedes** — replaces an older decision
- **conflicts** — tensions between decisions (both may be active)
- **requires** — one decision depends on another being in place
- **entails** — logical implication: if A then B must also hold
- **excludes** — mutual exclusion: A and B cannot both be active

## Vocabulary hygiene

You are the guardian of the project's ubiquitous language. Scopes, labels, and terminology in decision titles form the domain vocabulary. Your job is to keep it consistent.

**Before creating or amending a decision:**
1. Check existing scopes and labels (`dictum list --format json`, `dictum context --format json`)
2. If the user's term is a synonym of an established one (e.g. `database` when `db` is already in use), suggest the established term
3. If a term is genuinely new, use it — but mention you're introducing new vocabulary
4. If unsure whether two terms refer to the same concept, ask the human

**The human is the authority.** They pick the preferred term. Once they've chosen, use it consistently. Do not silently introduce synonyms.

**The tool is a dumb store.** Vocabulary lives in the decisions themselves — every scope, label, and title is evidence of what terms the team uses. There is no separate vocabulary registry. You enforce consistency by reading what exists and aligning to it.

## What to do

**If a decision applies to your current task:**
Follow it. Mention which decision guided your choice.

**If a user request contradicts an existing decision:**
Flag it clearly. Quote the decision ID, title, kind, and weight. Ask the user whether to:
1. Proceed with the request and amend/deprecate the old decision
2. Adjust the request to align with the existing decision
3. Do something else entirely

Do NOT silently override a decision. The whole point is to make the "why" explicit.

**If an assumption looks invalid:**
Flag it immediately. Quote the decision ID and explain what evidence contradicts it.

**If you make a significant choice and no decision covers it:**
Suggest the user add one:
```
dictum add "description of the decision" --level tactical --kind choice --weight should
```

## Output format

Always use `--format json` when calling dictum so you get structured, parseable output. The text format is for humans at a terminal.
