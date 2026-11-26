# Handoff

- Updated: 2026-11-27
- Current phase: Phase 1 closed by the maintainer on 2026-11-27; its pull request is [#6](https://github.com/AiraCometes/AntiWebUI/pull/6). Phase 2 is next
- Active change: none
- Branch and commit: `phase-1-audits`, pushed to `origin` (the A9 windowed results are the last commit)
- Agent: Claude Code, Claude Opus 5.5

## Completed in this session

- Phase 1 closed (maintainer, 2026-11-27). Phase 2 starts with a design literature review, whose findings feed the gap analysis ([roadmap](roadmap.md#phase-2-requirements-engineering)).
- A9 [upstream baseline](performance/2026-11-26-upstream-baseline.md): builds (cold dev 58 s, cold release 149 s at `jobs = 4`), binary size (`showcase` 21.0 MB stripped), headless frame cost on llvmpipe, and windowed runs on WSLg with llvmpipe and with the Iris Xe iGPU through `GALLIUM_DRIVER=d3d12 WGPU_BACKEND=gl`. WSLg adds about 24 ms of latency on either GPU. Raw data beside the report.
- A8 [upstream design documents](research/audit-design-docs.md): 14 of 43 documents match the code, 28 mostly match, 1 is partly stale; 23 new stale statements; the open items of `07-roadmap.md`. A4b corrected along the way (frames are held back only on Linux dmabuf).
- A5 [platform matrix](research/audit-platform-matrix.md): Windows and macOS type-check from Linux except `anti-webui-code` and `anti-webui-markdown`; `aarch64-apple-darwin` added to `rust-toolchain.toml`.
- A7 [baseline quality](research/audit-baseline-quality.md): rustfmt, Clippy (1,109 `float_literal_f32_fallback` warnings from `setter!`, a future hard error), `unsafe`, complexity, duplication.
- [ADR-0007](decisions/0007-upstream-design-documents-across-the-fork-lifecycle.md): `docs/design` unchanged while soft-forked, taken over at the hard fork; upstream features still adopted after it without duplication.
- Japanese edition policy in the [documentation policy](documentation-policy.md#japanese-edition).
- Earlier in Phase 1: A1–A4c and A6 (see the [fork README](README.md)).

## Phase 1 exit criteria

From the [audit plan](research/phase-1-audit-plan.md#exit-criteria):

| Criterion | State |
|---|---|
| Every workstream file exists, follows the template, and labels every claim | Done: A1–A8 in `research/`, A9 in `performance/` |
| Every checklist item has a status and evidence | Done: the checklists in A4a, A4b, and A4c |
| Baseline violations (A7) and measurements (A9) are recorded with commands | Done |
| The handoff names the Phase 2 starting point | Done: below |
| The maintainer approves moving to Phase 2 | Done: Phase 1 closed on 2026-11-27 |

## Phase 2 starting point

Phase 2 is requirements engineering ([roadmap](roadmap.md#phase-2-requirements-engineering)). Start with the design literature review: choose textbook-grade documents and websites on UI framework design, analyse them, and record what applies (features to add, directions, capabilities beyond upstream). Then the [gap analysis](templates/gap-analysis.md), filled from that review and from the "Inputs for gap analysis" section of each audit; the A8 [open items of `07-roadmap.md`](research/audit-design-docs.md#open-items-in-07-roadmapmd) are the first candidates. Then competitor research, grilling sessions, and the project constitution. Carry in:

- The verification steps from Kuniwak's "バイブ形式手法のハマりどころ2選" (<https://blog.kuniwak.com/entry/vibe-fm>): state the oracle at the start of each change, before specs, design, and tests; check it with examples, counterexample search, and mutation testing; check model fidelity; use the grilling skill to settle which failures matter before the oracle is written. Decide these with the constitution and the OpenSpec change workflow; the tool-level checks go to the Phase 4 spikes.
- "Officially supported platforms" from the [decision log](decisions/README.md#decisions-awaiting-confirmation), due in Phase 2.
- A true idle measurement needs an example that asks for nothing ([A9](performance/2026-11-26-upstream-baseline.md#windowed-wslg-llvmpipe-and-the-igpu)).

## Remaining work

1. Review and merge the Phase 1 pull request, [#6](https://github.com/AiraCometes/AntiWebUI/pull/6).
2. Start Phase 2 as described in [Phase 2 starting point](#phase-2-starting-point).

## Verification

| Command | Result |
|---|---|
| `mise run check` | passes |
| `benchmarks/phase1/windowed.py --runs 5`, default and `GALLIUM_DRIVER=d3d12 WGPU_BACKEND=gl` | 15 of 15 runs each, by the maintainer on WSLg |

## Unresolved issues

- The A6 handoff recorded `RUSTC_WRAPPER=""` in the agent sandbox; in the A7–A9 session it was `kache`, and cc-rs ran C compiles through it. Check `env | rg RUSTC_WRAPPER` before relying on either. Keep `debug = "line-tables-only"`, and delete `target/llvm-cov-target` after coverage runs.
- `/home/airacometes/projects/.a6-mutants` (cargo-mutants output) is outside the repository; the agent cannot delete it.
- Attachments the maintainer pastes into T3 Code are stored under `~/.t3/userdata/attachments/`, which the safety hook does not let the agent read; ask for the file in the repository instead.
- Real macOS and Windows devices: no native Linux machine; Windows is this machine's host; a Mac's availability is unknown.
- Open questions from Phase 0 still stand ([plan](quality/ci-and-quality-services.md), [decision log](decisions/README.md#decisions-awaiting-confirmation)).

## Relevant documents

- [Roadmap](roadmap.md)
- [Phase 1 audit plan](research/phase-1-audit-plan.md)
- [Gap-analysis template](templates/gap-analysis.md)
- [Test oracle and model-checking evaluation](research/candidate-test-oracles-and-model-checking.md)

## Recommended next action

After the Phase 1 pull request is merged, start the design literature review as described in [Phase 2 starting point](#phase-2-starting-point).
