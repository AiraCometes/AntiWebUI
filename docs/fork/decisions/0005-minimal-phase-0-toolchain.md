---
status: accepted
date: 2026-11-24
decision-makers: AiraCometes (maintainer)
---

# Start with a documentation-only toolchain in Phase 0

## Context and problem statement

The bootstrap brief lists about forty tools and asks that none be adopted without a reason. Upstream code fails `cargo fmt --check` in 30 hunks, and no lint, dependency, or CI baseline exists yet ([inspection](../research/phase-0-inspection.md#cargo-workspace-and-toolchain)). Phase 0 adds only documents and configuration. On this machine, commits that change hook or task configuration need a manual `verify-commits accept`.

## Decision drivers

- Pay for What You Use, applied to the development process
- No rule that fails on upstream code before its baseline is recorded
- Small, reviewable configuration

## Considered options

- Documentation-only toolchain: mise pins, mise tasks, and an hk pre-commit hook for fork-owned files
- No tooling in Phase 0
- Rust checks (rustfmt, clippy, cargo-deny) and CI in Phase 0

## Decision outcome

Chosen option: documentation-only toolchain.

- `mise.toml` pins `hk`, `openspec`, `typos`, `rumdl`, `taplo`, and `lychee`, and defines the `docs:*`, `spell`, `toml:check`, `spec:validate`, and `check` tasks.
- `hk.pkl` runs the same checks on staged fork-owned files at pre-commit. External links are checked only by `mise run docs:links-online`.
- `.rumdl.toml` and `.taplo.toml` limit the linters to fork-owned files.
- The Rust toolchain pin, rustfmt, clippy, cargo-deny, cargo-shear, nextest, `xtask`, and GitHub Actions CI come after the Phase 1 baseline, per the [tool adoption matrix](../quality/tool-adoption-matrix.md).

### Consequences

- Good, because the hook cannot fail on upstream code.
- Good, because there is one place to run checks (`mise run check`).
- Bad, because nothing checks Rust code in the fork until Phase 1 or 5.
- Bad, because the tools must be installed on the host once (`mise install`). Tools installed only inside the agent sandbox are invisible to hooks that run outside it.

### Confirmation

`mise run check` passes on the Phase 0 branch. `hk validate` accepts `hk.pkl`.
