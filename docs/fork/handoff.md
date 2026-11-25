# Handoff

- Updated: 2026-11-26
- Current phase: Phase 1 (in progress, approved by the maintainer on 2026-11-25)
- Active change: none
- Branch and commit: `phase-1-audits`, last audit commit `cb4df73` (A6), pushed to `origin`
- Agent: Claude Code, Claude Opus 5.5

## Completed in this session

- A2 [licences](research/audit-licenses.md) (`0b8a8b1`), A3 [wgpu fork](research/audit-wgpu-fork.md) (`6365cf7`), A4a [core](research/audit-capabilities-core.md) (`338397d`), A4b [rendering](research/audit-capabilities-rendering.md) (`8cf5299`), A4c [platform](research/audit-capabilities-platform.md) (`50e960e`), A6 [tests](research/audit-tests.md) (`cb4df73`). A4a–A4c were drafted by Pi and corrected after checking the code.
- Pinned the Rust toolchain to 1.99.0 in `rust-toolchain.toml` (`89c8afc`, accepted by the maintainer).
- Kache is used locally from Phase 1 to cut `target/` disk use ([matrix](quality/tool-adoption-matrix.md), [CI plan](quality/ci-and-quality-services.md); `4bcbd84`).

## Remaining work

1. A7 baseline quality (Clippy, rustfmt, `unsafe`, complexity, duplication, the `float_literal_f32_fallback` warnings A6 saw), then A5, A8, and A9 in plan order.
2. Open the pull request with `gh pr create --repo AiraCometes/AntiWebUI --base main` when the maintainer wants the audits reviewed.

## Modified files

- `docs/fork/research/audit-*.md` (A2, A3, A4a–A4c, A6)
- `docs/fork/README.md`
- `docs/fork/quality/tool-adoption-matrix.md`, `docs/fork/quality/ci-and-quality-services.md`
- `rust-toolchain.toml`, `AGENTS.md`
- `docs/fork/handoff.md`

## Verification

| Command | Result |
|---|---|
| `cargo nextest run --workspace --locked --no-fail-fast` | 1,595 run: 1,586 passed, 9 failed (sandbox and fonts, see A6) |
| `cargo llvm-cov nextest --workspace --locked --ignore-run-fail` | 73.3% lines; `target/llvm-cov-target` deleted afterwards |
| `mise run check` | passes |

## Unresolved issues

- 16 Wayland and wayland-shell tests fail inside the agent sandbox (Unix sockets, session bus). The maintainer can confirm them from a normal terminal with `cargo nextest run --locked -p anti-webui-wayland -p anti-webui-wayland-shell --all-features`.
- The agent sandbox sets `RUSTC_WRAPPER=""` (`~/.config/agent-guard/env`), so agent builds bypass Kache; builds in the maintainer's terminal use it. Keep `debug = "line-tables-only"`, and delete `target/llvm-cov-target` after coverage runs.
- `/home/airacometes/projects/.a6-mutants` (cargo-mutants output) is outside the repository; the agent cannot delete it.
- Carry to Phase 2: the verification steps in Kuniwak's "バイブ形式手法のハマりどころ2選" (<https://blog.kuniwak.com/entry/vibe-fm>): state the oracle at the start of each change, before specs, design, and tests; check it with examples, counterexample search, and mutation testing; check model fidelity. Also using the grilling skill to settle which failures matter before the oracle is written. Decide these with the constitution and the OpenSpec change workflow; the tool-level checks (Kani `cover!`, reachability checks for Quint or Stateright, bounded results labelled with their bound) go to the Phase 4 spikes.
- Real macOS and Windows devices for A5 and A9: availability unknown.
- Open questions from Phase 0 still stand ([plan](quality/ci-and-quality-services.md), [decision log](decisions/README.md#decisions-awaiting-confirmation)).

## Relevant documents

- [Roadmap](roadmap.md)
- [Phase 1 audit plan](research/phase-1-audit-plan.md)
- [Technical-audit template](templates/technical-audit.md)
- [Test oracle and model-checking evaluation](research/candidate-test-oracles-and-model-checking.md)

## Recommended next action

Run A7 (`research/audit-baseline-quality.md`) from the [technical-audit template](templates/technical-audit.md): `cargo clippy --workspace --all-targets --locked`, `cargo fmt --check`, the `unsafe` and `SAFETY:` counts, and duplication.
