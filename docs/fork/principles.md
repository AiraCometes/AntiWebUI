# Provisional project principles

- Status: provisional. The [project constitution](templates/project-constitution.md) replaces this file in Phase 2, after the audit and grilling.
- Source: the bootstrap brief, 2026-11-24.

Each principle states its guarantee boundary. A principle without a check yet says so, and Phase 5 adds the check.

| Principle | Meaning | Guarantee boundary | Planned check |
|---|---|---|---|
| Native First | No WebView, Chromium, DOM, or JavaScript runtime in the framework | Framework crates. Optional modules that embed a script engine need a Phase 2 decision; upstream `anti-webui-pdf` has an optional `js` feature using `rquickjs` (Confirmed: `src/anti-webui-pdf/Cargo.toml`) | Dependency ban (cargo-deny) |
| Cross-platform First | Windows, macOS, and Linux are equal targets | Platform-specific features stay in OS backends, with capability detection and fallback | CI matrix; real-device checks at phase completion |
| Physical Pixel Precision | Layout in integer physical pixels; fractional scaling stays sharp | Layout and paint output (upstream model: `docs/design/01-pixel-model.md`) | Snapshot tests at several scale factors |
| Zero Idle Rendering | No surface acquire, GPU submit, present, or periodic wakeup without a reason | Framework-owned scheduling; OS and driver CPU/GPU usage is not covered | Headless test counting submits, presents, and wakeups |
| Zero Hot-path Allocation | No heap allocation in the steady-state redraw path after warm-up | Framework-owned hot path only; user callbacks and OS/driver calls are not covered | Allocation-counter test |
| Pay for What You Use | Unused features cost nothing at run time | Feature gates and lazy initialisation | Feature-matrix build; binary size budget |
| Locality First | Data and work stay close to where they are used | Design reviews | Review |
| Bounded Resources | RAM and VRAM have budgets and eviction | Framework-owned caches and pools | Budget tests |
| Reuse Before Rewrite | Keep upstream assets; every rewrite needs evidence | Classified in the gap analysis (REUSE … DROP) | REPLACE requires an ADR |
| Benchmark-driven Development | Performance claims need reproducible benchmarks | All published performance statements | [Benchmark report](templates/benchmark-report.md) |
| Architecture as Code | Structural rules are machine-checked | Crate and module dependencies, cycles, layering | Fitness functions (Phase 5) |
| Explicit Ownership | Every resource has one owner and a defined lifetime | GPU resources, nodes, signals, windows | Review; tests for drop order |
| Minimal Abstraction | An abstraction needs two real users | Public and internal APIs | Review |
| Bounded Complexity | Complexity and duplication do not grow unchecked | Fork-owned code; upstream baseline is recorded | Complexity and duplication baselines |
| Controlled Public API | Public API grows only on purpose | Public items of framework crates | cargo-semver-checks; API listing diff |
| Documentation as Code | Documents live beside the code and are checked | Fork-owned documents | `mise run check` |

## Fitness function candidates

From the bootstrap brief. Each becomes a check in Phase 5. Upstream violations found in Phase 1 are recorded as the baseline, and fork changes must not make them worse. Fixing the whole upstream baseline at once is a non-goal.

- No crate or module dependency outside the allowed graph; no cycles
- No dependency from core crates on GPU or OS-specific crates
- No dependency on design kits
- No unplanned growth of the public API
- No new dependency without review
- Every `unsafe` block has a `SAFETY:` comment
- No growth in complexity or duplication beyond the baseline
- Fractional-scaling guarantees keep holding
- No allocation in the steady-state redraw path
- RAM and VRAM budgets hold
- No redraw while idle

## Measured metrics

Startup time (cold and warm), binary size, resident and peak memory, GPU memory, idle wakeups, idle CPU and GPU activity, frame time (median, P95, P99), input latency, allocation count, large node-canvas performance, virtualised list performance, text rendering quality, fractional-scaling correctness, and multi-window resource efficiency. Targets are set after the Phase 1 baseline.
