# Handoff

- Updated: 2026-11-26
- Current phase: Phase 1 (in progress, approved by the maintainer on 2026-11-25)
- Active change: none
- Branch and commit: `phase-1-audits`, last audit commit `5496049` (A7), not yet pushed
- Agent: Claude Code, Claude Opus 5.5

## Completed in this session

- A7 [baseline quality](research/audit-baseline-quality.md) (`5496049`): rustfmt fails on 20 files; Clippy reports 1,159 warnings, 1,109 of them rustc's `float_literal_f32_fallback` from the `setter!` macro's `impl Into<f32>` (a future hard error); `SAFETY:` comments are missing almost only in `anti-webui-macos`; complexity and duplication baselines from lizard and jscpd.
- Earlier in Phase 1: A2 [licences](research/audit-licenses.md), A3 [wgpu fork](research/audit-wgpu-fork.md), A4a [core](research/audit-capabilities-core.md), A4b [rendering](research/audit-capabilities-rendering.md), A4c [platform](research/audit-capabilities-platform.md), A6 [tests](research/audit-tests.md); toolchain pinned to 1.99.0 (`89c8afc`); Kache used locally (`4bcbd84`).

## Remaining work

1. Push `phase-1-audits` (`agent-git push`).
2. A5 platform matrix (`research/audit-platform-matrix.md`), then A8 and A9 in plan order.
3. Open the pull request with `gh pr create --repo AiraCometes/AntiWebUI --base main` when the maintainer wants the audits reviewed.

## Modified files

- `docs/fork/research/audit-baseline-quality.md` (new)
- `docs/fork/README.md`
- `docs/fork/handoff.md`

## Verification

| Command | Result |
|---|---|
| `cargo clippy --workspace --all-targets --locked` | 0 errors, 1,159 warnings (A7 baseline) |
| `cargo clippy … --target x86_64-pc-windows-msvc` (windows, video, wgpu) | Builds in 44 s; the Windows target is installed, so A5 can cross-check Windows here |
| `mise run check` | passes |

## Unresolved issues

- The agent sandbox sets `RUSTC_WRAPPER=""` (`~/.config/agent-guard/env`), so agent builds bypass Kache; builds in the maintainer's terminal use it. Keep `debug = "line-tables-only"`, and delete `target/llvm-cov-target` after coverage runs.
- `/home/airacometes/projects/.a6-mutants` (cargo-mutants output) is outside the repository; the agent cannot delete it.
- No macOS Rust target is installed, so Clippy has not seen `anti-webui-macos`; A5 needs `rustup target add aarch64-apple-darwin` (network) to try a cross-check.
- Whether to propose the `setter!` f32 fix upstream needs the maintainer's decision ([A7 open questions](research/audit-baseline-quality.md#open-questions)).
- Carry to Phase 2: the verification steps in Kuniwak's "バイブ形式手法のハマりどころ2選" (<https://blog.kuniwak.com/entry/vibe-fm>): state the oracle at the start of each change, before specs, design, and tests; check it with examples, counterexample search, and mutation testing; check model fidelity. Also using the grilling skill to settle which failures matter before the oracle is written. Decide these with the constitution and the OpenSpec change workflow; the tool-level checks (Kani `cover!`, reachability checks for Quint or Stateright, bounded results labelled with their bound) go to the Phase 4 spikes.
- Real macOS and Windows devices for A5 and A9: availability unknown.
- Open questions from Phase 0 still stand ([plan](quality/ci-and-quality-services.md), [decision log](decisions/README.md#decisions-awaiting-confirmation)).

## Relevant documents

- [Roadmap](roadmap.md)
- [Phase 1 audit plan](research/phase-1-audit-plan.md)
- [Technical-audit template](templates/technical-audit.md)
- [Test oracle and model-checking evaluation](research/candidate-test-oracles-and-model-checking.md)

## Recommended next action

Run A5 (`research/audit-platform-matrix.md`) from the [technical-audit template](templates/technical-audit.md): `cargo check --target x86_64-pc-windows-msvc` for the whole workspace, a macOS target if it can be added, and the build and test status per OS and display server from A4c and A6.
