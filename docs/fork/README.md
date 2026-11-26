# Fork documentation

Documents owned by this fork. Upstream documents stay in [`docs/design/`](../design/README.md) (Japanese, unchanged). Agents start at [`AGENTS.md`](../../AGENTS.md).

## Start here

| Document | Read it when |
|---|---|
| [Project brief](project-brief.md) | You need the goals and scope |
| [Roadmap](roadmap.md) | You need the current phase and what comes next |
| [Handoff](handoff.md) | You resume work from an earlier session |
| [Principles](principles.md) | You make or review a design choice |
| [Documentation policy](documentation-policy.md) | You write or place any document |
| [Decision log](decisions/README.md) | You need why something is the way it is |

## Reference

| Document | Contents |
|---|---|
| [Tool adoption matrix](quality/tool-adoption-matrix.md) | Every candidate tool, its version, licence, and adoption phase; agent skills; quality gate tiers |
| [CI and quality services plan](quality/ci-and-quality-services.md) | CI matrix and software GPU devices; Renovate, CodeQL, secret scanning, dependency review, Codecov, SonarQube Cloud, CodeRabbit; Kache with GHCR |
| [Phase 0 inspection](research/phase-0-inspection.md) | Repository, toolchain, licence, and upstream facts at the baseline commit |
| [Phase 1 audit plan](research/phase-1-audit-plan.md) | Workstreams and exit criteria for the upstream audit |
| [Audit A1: dependencies](research/audit-dependencies.md) | Crate layers, features, dependency counts and duplicates, crate and module cycles, core's reach to GPU and OS crates |
| [Audit A2: licences](research/audit-licenses.md) | Crate licences by feature, advisories, bundled fonts and assets, copied code, libraries loaded at run time, third-party notices |
| [Audit A3: wgpu fork](research/audit-wgpu-fork.md) | What `bea4dev/wgpu` changes against v30.0.1, where it is used, build without it, maintenance cost, upstream wgpu state |
| [Audit A4a: core capabilities](research/audit-capabilities-core.md) | Reactivity, node tree, styles, layout and the pixel model, text, editing and IME, transforms, input, scrolling; taffy fixture counts; state machines |
| [Audit A4b: rendering capabilities](research/audit-capabilities-rendering.md) | wgpu renderer, damage tracking, layers and texture caches, glyph atlas, GPU canvas, shaders and filters, animation, frame pacing, GPU-dependent tests |
| [Audit A4c: platform capabilities](research/audit-capabilities-platform.md) | Platform abstraction, Wayland, X11, macOS, and Windows backends, multi-window, accessibility, drag and drop, fullscreen, IME, clipboard, cursor, scale, wayland-shell |
| [Audit A5: platform matrix](research/audit-platform-matrix.md) | What builds, runs, and is tested per OS and display server; cross-checks for Windows and macOS; backend choice at run time; feature and cfg gates |
| [Audit A6: tests](research/audit-tests.md) | Test inventory per crate, results and failures on Linux, GPU skip behaviour, run time, line coverage, the oracle of each test file, a mutation sample |
| [Audit A7: baseline quality](research/audit-baseline-quality.md) | rustfmt and Clippy baselines, the f32 literal fallback, `unsafe` and `SAFETY:` counts, lint suppressions, complexity, duplication |
| [Audit A8: upstream design documents](research/audit-design-docs.md) | An English summary of each `docs/design` file, how far the code matches it, its limits and open items; the open items in `07-roadmap.md`; stale statements |
| [Test oracle and model-checking evaluation](research/candidate-test-oracles-and-model-checking.md) | taffy as a differential layout oracle; proptest-state-machine, Quint + ITF, Stateright, Loom, and Shuttle for state transitions and concurrency |
| [Templates](templates/) | Constitution, audit, competitor analysis, gap analysis, spike, benchmark report, ADR, handoff |

## Phase 0 outputs

| Output | File |
|---|---|
| Project brief | [project-brief.md](project-brief.md) |
| Provisional principles | [principles.md](principles.md) |
| Development roadmap | [roadmap.md](roadmap.md) |
| Tool adoption matrix | [quality/tool-adoption-matrix.md](quality/tool-adoption-matrix.md) |
| Documentation policy | [documentation-policy.md](documentation-policy.md) |
| Document templates | [templates/](templates/) |
| Agent instructions | [`AGENTS.md`](../../AGENTS.md), [`CLAUDE.md`](../../CLAUDE.md) |
| Decision log | [decisions/README.md](decisions/README.md) |
| Phase 1 audit plan | [research/phase-1-audit-plan.md](research/phase-1-audit-plan.md) |
| Minimal toolchain | `mise.toml`, `hk.pkl`, `.rumdl.toml`, `.taplo.toml`, `openspec/` |
