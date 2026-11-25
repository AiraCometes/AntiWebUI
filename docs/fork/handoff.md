# Handoff

- Updated: 2026-11-25
- Current phase: Phase 1 (in progress, approved by the maintainer on 2026-11-25)
- Active change: none
- Branch and commit: `phase-1-quality-tools`, based on `a37d69f3a6c8efa8a000891eeeb8ed00fbace20c` (A1 merged as #4)
- Agent: Claude Code, Claude Opus 5.5

## Completed in this session

- A1 merged (#4): [dependency audit](research/audit-dependencies.md).
- Pinned cargo-shear and Gevatter as report-only tools, with the tasks `quality:unused-deps` and `quality:lifetimes` ([matrix](quality/tool-adoption-matrix.md)).
- Ran cargo-shear and added its 7 findings to the A1 audit.
- Ran Gevatter on the workspace: 3 `non_permissive_lifetimes` warnings in `src/anti-webui-pdf/src/edits.rs` (lines 274, 521, 554) and 18 internal `out-of-range` errors; A7 records them as the baseline.
- Pinned OpenLore 3.8.0 (CLI only) with the task `audit:index`; the maintainer chose to use it where it works.

## Remaining work

1. The maintainer runs `verify-commits accept` for the `mise.toml` commit and `mise install` on the host, then merges the pull request.
2. The maintainer decides whether A2, A3, A4a–A4c, A6, and A7 run in one agent or in parallel. All of them depend only on A1.
3. The remaining workstreams in the audit plan order.

## Modified files

- `mise.toml`
- `docs/fork/quality/tool-adoption-matrix.md`
- `docs/fork/research/audit-dependencies.md`
- `docs/fork/research/phase-1-audit-plan.md`
- `docs/fork/handoff.md`

## Verification

| Command | Result |
|---|---|
| `mise install` (agent sandbox) | cargo-shear 1.14.1 and cargo-gevatter 0.1.5 install |
| `mise run quality:unused-deps` | 7 findings, exit 0 |
| `mise run quality:lifetimes` (after the host driver install) | exit 0; findings above |
| `mise run audit:index` steps | `openlore analyze --no-embed` exit 0 in 31 s |
| `mise run check` | passes |

## Unresolved issues

- Gevatter and OpenLore are young tools; their output is a pointer, never evidence on its own.
- `.openlore/` is excluded only in this clone's `.git/info/exclude`; another clone needs the same line, or a fork-owned ignore rule.
- Real macOS and Windows devices for A5 and A9: availability unknown. Until then, those platforms are compile-checked only.
- Open questions from Phase 0 still stand: Renovate and the upstream-owned `Cargo.toml` and `Cargo.lock`, the GHCR 5,000-download deletion limit, and CodeRabbit's plan ([plan](quality/ci-and-quality-services.md)); upstream-owned file edits and upstream pull requests settle in Phase 4 ([decision log](decisions/README.md#decisions-awaiting-confirmation)).

## Relevant documents

- [Roadmap](roadmap.md)
- [Phase 1 audit plan](research/phase-1-audit-plan.md)
- [Technical-audit template](templates/technical-audit.md)
- [Tool adoption matrix](quality/tool-adoption-matrix.md)

## Recommended next action

After the A1 merge and the maintainer's answer on parallel work, run A2 (licence audit) with cargo-deny, from the [technical-audit template](templates/technical-audit.md).
