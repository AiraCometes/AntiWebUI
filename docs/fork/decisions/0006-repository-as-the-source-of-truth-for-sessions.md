---
status: accepted
date: 2026-11-24
decision-makers: AiraCometes (maintainer)
---

# Keep agent instructions and session state in the repository

## Context and problem statement

Work spans many sessions, models, and agents (Claude Code, Codex, Pi). Context windows run out and conversations are lost. The `handoff` skill from `mattpocock/skills` writes its document to `$TMPDIR`, outside the repository. Several agents read different instruction files (`AGENTS.md`, `CLAUDE.md`).

## Decision drivers

- Repository as the source of truth; work must resume from the repository alone
- Progressive context loading: small always-loaded files, everything else behind pointers
- No duplicated instructions

## Considered options

- `AGENTS.md` canonical, `CLAUDE.md` imports it; session state in a tracked `docs/fork/handoff.md`
- Separate `AGENTS.md` and `CLAUDE.md`; handoff in `$TMPDIR` as the skill does

## Decision outcome

Chosen option: `AGENTS.md` is canonical. `CLAUDE.md` only imports it (`@AGENTS.md`). `docs/fork/handoff.md` holds the latest session state, following the [handoff template](../templates/handoff.md).

- The handoff file holds only the latest state; git history keeps the older ones. It points to specs, ADRs, and audits and repeats none of them.
- The `handoff` skill may be used to draft the content. The result is written to `docs/fork/handoff.md`.
- Session startup steps live in `AGENTS.md`.

### Consequences

- Good, because any agent can resume from a clean clone.
- Good, because one file states the rules for every agent.
- Bad, because the handoff file changes in most sessions, which adds small commits.

### Confirmation

Review checks that a session that changes phase or active change updates `docs/fork/handoff.md`. `AGENTS.md` stays short enough to read in full at session start.
