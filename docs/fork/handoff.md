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

## Remaining work

1. The maintainer runs `verify-commits accept` for the `mise.toml` commit and `mise install` on the host, runs `mise exec -- cargo gevatter toolchain install` once on the host, then merges the pull request.
2. Run `mise run quality:lifetimes` and record the Gevatter baseline in A7.
3. The maintainer decides whether A2, A3, A4a–A4c, A6, and A7 run in one agent or in parallel. All of them depend only on A1.
4. The remaining workstreams in the audit plan order.

## Modified files

- `mise.toml`
- `docs/fork/quality/tool-adoption-matrix.md`
- `docs/fork/research/audit-dependencies.md`
- `docs/fork/handoff.md`

## Verification

| Command | Result |
|---|---|
| `mise install` (agent sandbox) | cargo-shear 1.14.1 and cargo-gevatter 0.1.5 install |
| `mise run quality:unused-deps` | 7 findings, exit 0 |
| `cargo gevatter check` (agent sandbox) | fails on the first run: the driver download gets `invalid peer certificate: UnknownIssuer` behind the sandbox TLS proxy |
| `mise run check` | passes |

## Unresolved issues

- Gevatter needs one host run to download `gevatter_driver` 0.1.5 into `~/.gevatter`; the cached driver there is 0.1.1.
- OpenLore: deferred; try it only if A4a needs more than one session by hand (audit plan setup step 2).
- Real macOS and Windows devices for A5 and A9: availability unknown. Until then, those platforms are compile-checked only.
- Open questions from Phase 0 still stand: Renovate and the upstream-owned `Cargo.toml` and `Cargo.lock`, the GHCR 5,000-download deletion limit, and CodeRabbit's plan ([plan](quality/ci-and-quality-services.md)); upstream-owned file edits and upstream pull requests settle in Phase 4 ([decision log](decisions/README.md#decisions-awaiting-confirmation)).

## Relevant documents

- [Roadmap](roadmap.md)
- [Phase 1 audit plan](research/phase-1-audit-plan.md)
- [Technical-audit template](templates/technical-audit.md)
- [Tool adoption matrix](quality/tool-adoption-matrix.md)

## Recommended next action

After the A1 merge and the maintainer's answer on parallel work, run A2 (licence audit) with cargo-deny, from the [technical-audit template](templates/technical-audit.md).
