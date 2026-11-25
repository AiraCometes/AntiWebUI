---
status: accepted
date: 2026-11-24
decision-makers: AiraCometes (maintainer)
---

# Track upstream as a soft fork, pinned to an audited baseline

## Context and problem statement

The repository is a fork of `bea4dev/AntiWebUI`. Upstream is young and fast: 98 commits between 2026-11-02 and 2026-11-19 ([inspection](../research/phase-0-inspection.md#git-state)). The goal is an independent framework that keeps upstream's assets (Reuse Before Rewrite). We must choose whether to keep merging upstream or cut the fork loose now.

## Decision drivers

- Reuse Before Rewrite: upstream keeps adding the capabilities the project wants
- Low merge cost while upstream changes quickly
- Freedom to restructure later, when the master architecture needs it

## Considered options

- Soft fork: keep merging `upstream/main`; fork work stays in fork-owned paths
- Hard fork now: stop merging at `6908e31` and cherry-pick upstream changes by hand

## Decision outcome

Chosen option: soft fork, at least through Phase 4.

- Baseline: upstream `6908e3119db4ecf7080f57de59870e083f2cbc98`. Phase 1 audits this commit. Later audits name the commit they read.
- Upstream changes come in through merge commits from `upstream/main`. They are never squashed or rebased, so later merges stay clean.
- Fork work goes in fork-owned paths: `docs/fork/`, `openspec/`, `AGENTS.md`, `CLAUDE.md`, tool config, and future fork crates and directories. Edits to upstream-owned files stay small and are listed in the PR description.
- Phase 4 decides how far upstream-owned files may be edited while tracking continues, and whether fork changes are offered upstream as pull requests at all ([decision log](README.md#decisions-awaiting-confirmation)). No pull request goes upstream without the maintainer's approval.
- The switch to a hard fork is decided in a later ADR, at the latest when the master architecture (Phase 3) needs changes that cannot coexist with upstream.

### Consequences

- Good, because upstream improvements keep arriving with little work.
- Good, because the audit has a fixed reference point.
- Bad, because structural changes to upstream code are postponed or need upstream agreement.
- Bad, because each upstream merge must be checked against fork rules (fitness functions apply to fork-owned paths first).

### Confirmation

PR review checks the list of upstream-owned files touched. Phase 5 adds automated upstream sync on GitHub Actions and a check that reports changes to upstream-owned paths.

## More information

The previous plan for automated sync (`upstream-mirror` branch, GitHub App token, merge-commit auto-merge) is on hold until Phase 5.
