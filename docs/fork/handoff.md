# Handoff

- Updated: 2026-11-26
- Current phase: Phase 1 (in progress, approved by the maintainer on 2026-11-25)
- Active change: none
- Branch and commit: `phase-1-audits`, A9 committed on top of A8, pushed to `origin`
- Agent: Claude Code, Claude Opus 5.5

## Completed in this session

- A9 [upstream baseline](performance/2026-11-26-upstream-baseline.md): cold dev build 58 s and cold release 149 s at `jobs = 4`; `showcase` 21.0 MB stripped; headless frame costs on llvmpipe (a text change at 20,000 rows is 18.5 ms, mostly layout). Windowed metrics are pending a run of `benchmarks/phase1/windowed.py` in a desktop session.
- Japanese edition policy added to the [documentation policy](documentation-policy.md#japanese-edition): English is the master, the Japanese edition matches it in content and structure, checked by a machine; timing and location are open (maintainer, 2026-11-26).
- [ADR-0007](decisions/0007-upstream-design-documents-across-the-fork-lifecycle.md): `docs/design` stays unchanged while soft-forked; the fork takes it over at the hard fork, and keeps adopting upstream features after it without duplication (maintainer, 2026-11-26).
- A8 [upstream design documents](research/audit-design-docs.md) (`633c627`), drafted by Pi in three parts and merged: 14 of 43 documents match the code, 28 mostly match, 1 is partly stale; 23 new stale statements are tabled, and the open items of `07-roadmap.md` are listed with their status.
- A4b corrected: frames are held back only on Linux dmabuf, not on macOS or Windows (found during A8).
- Added `aarch64-apple-darwin` to `rust-toolchain.toml` (maintainer, 2026-11-26), so every session can type-check `anti-webui-macos`.
- A5 [platform matrix](research/audit-platform-matrix.md) (`5f6918b`), drafted by Pi and completed with cross-checks: Windows and macOS targets type-check except `anti-webui-code` and `anti-webui-markdown` (their `tree-sitter` C build scripts need a target C toolchain); the `svg` example fails with `--no-default-features`.
- A7 [baseline quality](research/audit-baseline-quality.md) (`5496049`): rustfmt fails on 20 files; Clippy reports 1,159 warnings, 1,109 of them rustc's `float_literal_f32_fallback` from the `setter!` macro's `impl Into<f32>` (a future hard error); `SAFETY:` comments are missing almost only in `anti-webui-macos`; complexity and duplication baselines from lizard and jscpd.
- Earlier in Phase 1: A2 [licences](research/audit-licenses.md), A3 [wgpu fork](research/audit-wgpu-fork.md), A4a [core](research/audit-capabilities-core.md), A4b [rendering](research/audit-capabilities-rendering.md), A4c [platform](research/audit-capabilities-platform.md), A6 [tests](research/audit-tests.md); toolchain pinned to 1.99.0 (`89c8afc`); Kache used locally (`4bcbd84`).

## Remaining work

1. The maintainer runs `benchmarks/phase1/windowed.py` from a desktop terminal (commands in the [A9 report](performance/2026-11-26-upstream-baseline.md#pending-windowed-measurements)); add the results to the report.
2. Phase 1 exit: name the Phase 2 starting point here, and ask the maintainer to approve moving to Phase 2 ([exit criteria](research/phase-1-audit-plan.md#exit-criteria)).
3. Open the pull request with `gh pr create --repo AiraCometes/AntiWebUI --base main` when the maintainer wants the audits reviewed.

## Modified files

- `docs/fork/research/audit-baseline-quality.md`, `audit-platform-matrix.md`, `audit-design-docs.md` (new); `audit-capabilities-rendering.md`
- `docs/fork/README.md`
- `rust-toolchain.toml`, `docs/fork/quality/tool-adoption-matrix.md`
- `docs/fork/handoff.md`

## Verification

| Command | Result |
|---|---|
| `cargo clippy --workspace --all-targets --locked` | 0 errors, 1,159 warnings (A7 baseline) |
| `cargo clippy … --target x86_64-pc-windows-msvc` (windows, video, wgpu) | Builds in 44 s; the Windows target is installed, so A5 can cross-check Windows here |
| `mise run check` | passes |

## Unresolved issues

- The handoff of A6 recorded `RUSTC_WRAPPER=""` in the agent sandbox; in the A7–A8 session it was `kache`, and cc-rs also ran C compiles through it (`kache cc`). Check `env | rg RUSTC_WRAPPER` before relying on either. Keep `debug = "line-tables-only"`, and delete `target/llvm-cov-target` after coverage runs.
- `/home/airacometes/projects/.a6-mutants` (cargo-mutants output) is outside the repository; the agent cannot delete it.
- The `setter!` f32 fix is not proposed upstream (maintainer, 2026-11-26); the fork's style API handles it ([A7](research/audit-baseline-quality.md#the-f32-literal-fallback)).
- Carry to Phase 2: the verification steps in Kuniwak's "バイブ形式手法のハマりどころ2選" (<https://blog.kuniwak.com/entry/vibe-fm>): state the oracle at the start of each change, before specs, design, and tests; check it with examples, counterexample search, and mutation testing; check model fidelity. Also using the grilling skill to settle which failures matter before the oracle is written. Decide these with the constitution and the OpenSpec change workflow; the tool-level checks (Kani `cover!`, reachability checks for Quint or Stateright, bounded results labelled with their bound) go to the Phase 4 spikes.
- Real macOS and Windows devices for A5 and A9: availability unknown.
- Open questions from Phase 0 still stand ([plan](quality/ci-and-quality-services.md), [decision log](decisions/README.md#decisions-awaiting-confirmation)).

## Relevant documents

- [Roadmap](roadmap.md)
- [Phase 1 audit plan](research/phase-1-audit-plan.md)
- [Technical-audit template](templates/technical-audit.md)
- [Test oracle and model-checking evaluation](research/candidate-test-oracles-and-model-checking.md)

## Recommended next action

With the windowed results in, check the [Phase 1 exit criteria](research/phase-1-audit-plan.md#exit-criteria), write the Phase 2 starting point here, and open the audit pull request with `gh pr create --repo AiraCometes/AntiWebUI --base main` once the maintainer wants it reviewed.
