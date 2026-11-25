# Phase 1 audit plan

- Status: approved by the maintainer on 2026-11-25.
- Baseline: upstream `6908e3119db4ecf7080f57de59870e083f2cbc98` ([ADR-0002](../decisions/0002-track-upstream-as-a-soft-fork.md)). If upstream moves during Phase 1, finish the audit on this commit, then record a delta for the new commits.
- Output template: [technical-audit](../templates/technical-audit.md). Measurements use the [benchmark-report](../templates/benchmark-report.md) template.

The audit records what exists and how good it is. It does not classify requirements (that is the Phase 2 gap analysis) and changes no Rust code.

## Workstreams

Each workstream fits in one session and produces one file. Run them in this order; later ones depend on earlier ones.

| ID | Workstream | Output | Method and tools | Depends on |
|---|---|---|---|---|
| A1 | Workspace and dependency graph: crate layers, features, optional dependencies, cycles, dependencies from core to GPU or OS crates | `research/audit-dependencies.md` | `cargo metadata`, `cargo tree -e features`, cargo-modules | — |
| A2 | Licence audit: crate licences, bundled fonts and assets, run-time loaded libraries (FFmpeg) | `research/audit-licenses.md` | cargo-deny (`licenses`, `advisories`, warnings only), asset review | A1 |
| A3 | wgpu fork: diff against `v30.0.1`, why it exists, maintenance cost, path to standard wgpu (upstream PR to `gfx-rs/wgpu`, or a fallback without the patch on non-DX12 backends) | `research/audit-wgpu-fork.md` | `gh api …/compare`, `docs/design/39-platform-windows.md`, build without the patch | A1 |
| A4a | Capabilities, core: signals and reactivity, node tree, style system, layout, text shaping, selection, editing, IME, coordinate transforms, input | `research/audit-capabilities-core.md` | `docs/design/01`–`04`, `08`, `12`; `anti-webui-core`, `anti-webui-style`; for layout, also count how many taffy fixtures map onto supported features ([evaluation](candidate-test-oracles-and-model-checking.md#phases)); for focus, gestures, undo/redo, history, the reactive system, and the redraw scheduler, record each state machine's states, inputs, and how time and platform reach it | A1 |
| A4b | Capabilities, rendering: wgpu renderer, damage tracking, compositing layers and texture caches, glyph atlas and subpixel rendering, GPU canvas, shaders and filters, animation, frame pacing | `research/audit-capabilities-rendering.md` | `docs/design/05`, `09`–`14`, `16`, `29`, `41`; `anti-webui-wgpu`, `anti-webui-app` | A1 |
| A4c | Capabilities, platform: Wayland, X11, macOS, and Windows backends, multi-window, accessibility API, drag and drop, fullscreen | `research/audit-capabilities-platform.md` | `docs/design/06`, `23`, `27`, `34`–`39`; backend crates | A1 |
| A5 | Platform matrix: for each OS and display server, what builds, what runs, what was tested, and how | `research/audit-platform-matrix.md` | Cross-checks (`cargo check --target …`), upstream docs, A4c | A4c |
| A6 | Tests and benchmarks: test inventory per crate, GPU tests that skip without an adapter, headless rendering, run time, coverage | `research/audit-tests.md` | cargo-nextest, cargo-llvm-cov, lavapipe if available | A1 |
| A7 | Technical debt and baseline violations: Clippy warnings, rustfmt diff, `unsafe` blocks with and without `SAFETY:` comments, complexity, duplication | `research/audit-baseline-quality.md` | Clippy, rustfmt, `rg`, rust-code-analysis or an alternative, jscpd | A1 |
| A8 | Upstream documents: an English summary of each `docs/design` file, how far the code matches it, known limits and open items in `07-roadmap.md` | `research/audit-design-docs.md` | Reading, with checks against code from A4 | A4a–A4c |
| A9 | Baseline measurements: build time (cold and warm), example binary size, startup time, idle wakeups and presents, resident memory, frame time for a few examples, on what this machine can run | `performance/<date>-upstream-baseline.md` | `cargo build --timings`, cargo-bloat, samply, `/proc` and `perf` counters, the upstream performance overlay (`docs/design/30-performance-overlay.md`) | A6 |

## Capability checklist

A4 must cover every upstream asset named in the bootstrap brief, with status (present, partial, absent) and evidence:

fractional-scaling handling; the integer physical-pixel model; the layout engine; signal reactivity; the node tree; the style system; the wgpu renderer; damage tracking; compositing layers and texture caches; the glyph atlas and subpixel rendering; text editing, selection, and IME; coordinate transforms and input; GPU canvas; animation; accessibility API; multi-window; OS backends; existing tests and benchmarks; existing design documents.

## Setup before the first workstream

1. Pin the Phase 1 tools in `mise.toml`: cargo-modules, cargo-deny, cargo-nextest, cargo-llvm-cov, cargo-bloat, and samply. This changes `mise.toml`, so it needs a `verify-commits accept` and a host `mise install`.
2. Decide whether to try OpenLore (formerly spec-gen) for the inventory. Use it only if A4a takes more than one session by hand.

## Limits of this machine

- WSL2 on Linux 6.18. Wayland through WSLg is likely; a GPU adapter through the WSL D3D12 bridge is Unverified. A9 records which adapter was used.
- macOS and Windows backends can only be compile-checked here. Running them needs real devices, which is an open question for the maintainer.

## Exit criteria

- Every workstream file exists, follows the template, and labels every claim.
- Every checklist item above has a status and evidence.
- Baseline violations (A7) and measurements (A9) are recorded with commands so they can be repeated.
- `docs/fork/handoff.md` names the Phase 2 starting point.
- The maintainer approves moving to Phase 2.
