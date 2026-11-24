# Agent instructions

This repository is a soft fork of `bea4dev/AntiWebUI`, growing into an independent native Rust GUI framework. The repository is the source of truth: decisions, requirements, and progress live in files, never only in a conversation.

## Session startup

1. Read [`docs/fork/handoff.md`](docs/fork/handoff.md) for the current phase, active change, and next action.
2. Check the phase and its exit condition in [`docs/fork/roadmap.md`](docs/fork/roadmap.md). Work stays inside the current phase; the next phase starts only with the maintainer's approval.
3. Run `git status` and `git log --oneline -10`; compare with the handoff's branch and commit.
4. If there is an active OpenSpec change, run `openspec status --change <name>` and read its files.
5. Load only the specs, ADRs, audits, and source that the task touches. Find them through [`docs/fork/README.md`](docs/fork/README.md) and [`docs/fork/decisions/README.md`](docs/fork/decisions/README.md).
6. Continue from the handoff's remaining work. Skip work it records as done.

## Session end

Update [`docs/fork/handoff.md`](docs/fork/handoff.md) from [its template](docs/fork/templates/handoff.md) before the session ends, or before switching to a new session. It holds only the latest state and points to other documents.

## Ownership of files

- Upstream-owned: `src/`, `docs/design/`, `README.md`, `Cargo.toml`, `Cargo.lock`, `image/`, licence files. Keep edits to them small, and list them in the PR description ([ADR-0002](docs/fork/decisions/0002-track-upstream-as-a-soft-fork.md)).
- Fork-owned: `AGENTS.md`, `CLAUDE.md`, `docs/fork/`, `openspec/`, `mise.toml`, `hk.pkl`, `.rumdl.toml`, `.taplo.toml`, and future fork directories.

## Writing

- Everything in the repository is English: documents, OpenSpec artifacts, comments, rustdoc, commits, issues, PRs. Conversation with the maintainer may be Japanese.
- Follow [`docs/fork/documentation-policy.md`](docs/fork/documentation-policy.md): evidence labels on technical claims, one canonical place per fact, templates in `docs/fork/templates/`.
- Record a structural decision as an ADR in `docs/fork/decisions/` and add it to the index.

## Checks

- `mise run check` runs every fast check on fork-owned files. The hk pre-commit hook runs the same checks on staged files.
- After `mise.toml` changes, run `mise install` on the host. Tools installed only inside an agent sandbox are not visible to hooks that run outside it.

## Specifications

OpenSpec in `openspec/` is the only specification system ([ADR-0003](docs/fork/decisions/0003-openspec-as-the-single-specification-system.md)). A change from GitHub issue `#N` is named `N-<kebab-case-summary>`.

## Upstream

Bring upstream in with merge commits from `upstream/main`. A pull request to `bea4dev/AntiWebUI` is sent only after the maintainer approves it.
