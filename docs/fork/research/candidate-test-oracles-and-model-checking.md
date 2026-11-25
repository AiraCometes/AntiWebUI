# Candidate evaluation: layout oracles, model-based testing, and concurrency checking

- Date: 2026-11-25 (first version 2026-11-24)
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Status: Proposed. Each tool needs a Phase 4 spike before adoption. All of them are dev-dependencies or development tools, never runtime dependencies.

Maintainer direction (2026-11-25):

- taffy is used only as a differential-testing oracle, not as a layout engine.
- Quint Connect is not used, because it is no longer updated. The goal is to test state transitions and asynchronous code: events, focus, and gestures; the reactive system and the redraw scheduler; asynchronous work and cross-thread communication; and undo/redo and navigation history.

## Part 1: taffy as a differential layout oracle

### Facts

| Item | Finding | Label |
|---|---|---|
| Project | `DioxusLabs/taffy`, MIT, 0.16.0 released 2026-11-23 | Confirmed (GitHub and crates.io APIs) |
| Algorithms | CSS Block, Flexbox, and Grid, plus float and `calc()`, each behind a feature flag | Confirmed (`README.md`, `Cargo.toml` `[features]`) |
| Test fixtures | `test_fixtures/` has 700 `flex`, 595 `grid`, and 281 `block` entries, plus `blockflex`, `blockgrid`, `gridflex`, `float`, `contain`, `leaf` | Confirmed (GitHub contents API) |
| Rounding model | taffy computes in `f32` and snaps edges (`round_layout`, method B). Upstream converts every length to physical `i32` first and distributes remainders inside the algorithm (method C) | Confirmed (`docs/design/01-pixel-model.md` "3 方式の比較", `docs/design/03-layout.md`) |
| Upstream conformance tests today | `anti-webui-core/tests/data/flex_wrap.txt`: 82 wrap cases compared with Chromium, up to 1 px difference allowed where Chromium is fractional | Confirmed (`docs/design/03-layout.md` "折り返し") |
| Upstream feature gaps | Grid and `order` | Confirmed (`docs/design/03-layout.md` "未対応") |

### Design of the differential test

1. Build the same tree twice: once with the upstream style API, once with `taffy::Style`, from one shared fixture description.
2. Feed taffy with lengths that are already physical pixels at the tested scale, so both engines start from the same integers.
3. Compare each node's rectangle. Allow at most 1 px per edge, because of the B-versus-C rounding difference. Treat a larger difference as a failure.
4. Keep a reviewed list of known divergences (case, reason, link to an ADR or issue), so intentional differences do not hide new ones.
5. Run at scales 1.0, 1.25, 1.5, and 1.75 (`Scale(120)`, `Scale(150)`, `Scale(180)`, `Scale(210)`).
6. Generate random trees with proptest as well as the fixtures, and shrink any failure to a minimal tree.

Fixture source: taffy's fixtures are MIT, so copied fixtures keep taffy's copyright notice in the fixture folder. Taffy is pinned as a dev-dependency of a test crate. Only the features the comparison needs are enabled (`flexbox`, `block_layout`, later `grid`).

### Phases

- Phase 1 (A4a): count the fixtures that use only supported features, and record how upstream layout is tested today.
- Phase 4 spike: convert a sample (for example 50 flex fixtures), run the comparison, and record the divergence rate and its causes.
- Phase 5: the full harness at the pull-request gate.

## Part 2: state-transition and concurrency testing

### Where the targets live upstream

| Target | Upstream location | Threading | Label |
|---|---|---|---|
| Reactive system | `src/anti-webui-core/src/reactive.rs` (944 lines) | Single UI thread; built on `Rc` and `RefCell` | Confirmed |
| UI-thread channel | `src/anti-webui-core/src/channel.rs`: `Arc<Mutex<…>>` queues, an `AtomicUsize` work counter, and a waker | Cross-thread | Confirmed |
| Deferred results | `src/anti-webui-core/src/task.rs`, built on `ui_channel` | Cross-thread into the UI thread | Confirmed |
| Shared signals, windows, image workers | `shared.rs`, `window.rs`, `images/worker.rs` in `anti-webui-core` (std sync primitives and atomics) | Cross-thread | Confirmed |
| D-Bus and other async work | `anti-webui-wayland-shell` with `zbus` (`async-io`), `async-executor`, `futures-lite` behind the `dbus` feature | Async, cross-thread | Confirmed (`Cargo.toml`) |
| Undo/redo | `EditData::undo` and `redo` in `src/anti-webui-core/src/text/edit.rs`; `Editor::undo` and `redo` in `text/editor.rs`; `anti-webui-pdf/src/lib.rs` | Single thread | Confirmed |
| Focus, events, gestures | `src/anti-webui-core/src/view.rs`, `ui.rs`, `event.rs`, `scroll.rs`, `drag.rs`; `anti-webui-windows/src/manipulation.rs` | Single UI thread | Confirmed as locations; behaviour is for the Phase 1 audit |
| Navigation history | No general history found; `anti-webui-md3e/tests/navigation.rs` covers MD3E navigation components | — | Inferred from a search; Phase 1 confirms |
| Property testing | No crate depends on `proptest` or `quickcheck` | — | Confirmed |

Inference: most targets are single-threaded state machines. Only the channel, task, shared-signal, worker, and D-Bus code needs a concurrency checker.

### Candidates

| Tool | Kind | Version and activity | Licence | Notes |
|---|---|---|---|---|
| proptest-state-machine | Random state-machine testing in Rust: a reference model and the real system run the same random transitions, with shrinking | 0.9.0, 2026-10-10 (part of `proptest-rs/proptest`) | MIT OR Apache-2.0 | Confirmed (crates.io). Rust only, no extra toolchain |
| Quint + ITF | Specification language with simulator and model checker. `quint run --mbt --out-itf=… --n-traces=… --max-steps=…` writes traces in the Informal Trace Format. `--mbt` (experimental) adds `mbt::actionTaken` and `mbt::nondetPicks` to each state | Quint 0.34.0, 2026-11-22 | Apache-2.0 | Confirmed (Quint `docs/content/docs/quint.md`). Needs Node.js. `quint verify` uses Apalache, which needs a JVM (Unverified) |
| `itf` crate | Decodes ITF JSON into Rust types with serde | 0.4.0, 2025-05-28; repository last pushed 2025-05-28 | Apache-2.0 | Confirmed (crates.io, GitHub). Stale; the ITF subset needed (records, sets, maps, tuples, `#bigint`) is small enough to decode in-repo if needed |
| Loom | Exhaustive exploration of thread interleavings and the C11 memory model, by replacing `std::sync` and `std::thread` with `loom::` types under `cfg(loom)` | 0.7.3, 2026-11-08 | MIT | Confirmed (crates.io, GitHub). Exhaustive but exponential; suits small primitives |
| Shuttle | Randomised concurrency testing (random and PCT schedulers), with `shuttle::sync`, `shuttle::thread`, and `shuttle::future` replacements | 0.9.10, 2026-11-05; pushed 2026-11-24 | Apache-2.0 | Confirmed (crates.io, GitHub). Scales to larger tests than Loom, supports async; not exhaustive |
| Stateright | Model checker and actor framework in Rust: exhaustive BFS/DFS over a model written as a Rust `Model`, with linearizability checking | 0.31.0, 2025-07-27; repository last pushed 2025-07-27 | MIT | Confirmed (crates.io, GitHub). No Node.js or JVM; about 16 months without a commit |
| Quint Connect | Trace replay against a Rust driver | 0.1.2, 2026-05-25 | Apache-2.0 | Not used (maintainer direction) |

### How they fit together

The tools cover different layers. The aim is one tool per layer, not several tools per target.

| Layer | Question | Default tool | Alternative to decide in the spike |
|---|---|---|---|
| Single-thread state machines: focus, gestures, undo/redo, navigation history, redraw scheduler logic, reactive update order | Does the implementation match a simple reference model under any sequence of events? | proptest-state-machine | — |
| Design-level protocols: Zero Idle Rendering, device loss and recovery, focus restoration across overlays, task cancellation and ownership | Does the design keep its invariants in every reachable state, and does the code follow the design? | Quint (invariants with `quint run` and `quint verify`) plus ITF trace replay through a small in-repo driver | Stateright: the same role with models in Rust |
| Cross-thread primitives: `ui_channel` waker and work counter, shared signals, worker hand-off | Is there an interleaving or memory ordering that loses a wakeup, deadlocks, or races? | Loom | Shuttle, if Loom's state space explodes |
| Async and larger concurrent flows: D-Bus tasks, `spawn_window`, video and image pipelines | Do realistic schedules break ordering or shutdown? | Shuttle | — |

Overlap with planned tools: Kani proves properties of small Rust functions and unsafe code. Miri and ASan find undefined behaviour and memory errors. Neither explores event sequences or schedules, so they complement the tools above.

### Constraints

- Deterministic seams. Single-thread state machines are testable only if time, input, and the platform come in from outside: a virtual clock and a headless backend (both bootstrap-brief items). The Phase 3 architecture should state this for focus, gestures, the scheduler, and history.
- Upstream-owned code. Loom and Shuttle need the sync primitives behind a `cfg` switch, which means editing upstream files (`channel.rs`, `shared.rs`, …). Phase 4 decides whether to edit upstream-owned files at all, how to keep tracking upstream if they are edited, and whether to offer such changes upstream or keep them fork-only ([decision log](../decisions/README.md#decisions-awaiting-confirmation)). Until then, spikes work on copies in fork-owned test crates. Fork-owned concurrency code uses a swappable facade from the start.
- The reactive system is `Rc`/`RefCell` on one thread. Loom and Shuttle do not apply to it. Its update order is a sequential state machine, which is tested by proptest-state-machine or Quint.
- Quint adds Node.js (pinned through mise as `npm:@informalsystems/quint`), and optionally a JVM for Apalache. Stateright avoids both, but it is stale. The spike chooses one of Quint and Stateright, not both.
- Trace replay without Quint Connect needs a small in-repo driver: read the ITF trace, map `mbt::actionTaken` and `mbt::nondetPicks` to calls on the system under test, and compare the projected state after each step. Estimate: one module plus one trait per model. Label: Inferred, to be measured in the spike.

### Spikes (Phase 4)

1. proptest-state-machine on `EditData` undo/redo: the reference model is a stack of snapshots, and the check is that undo after edits restores the text and selection. Smallest target, existing code, no upstream edits.
2. Quint on the redraw scheduler's idle invariant: model the reasons to draw (input, signal change, animation, resize, explicit request) and check "no present without a pending reason" with `quint run`. Then replay ITF traces against a headless scheduler. Repeat the model in Stateright to compare effort, run time, and clarity, and choose one.
3. Loom and Shuttle on a copy of `ui_channel` in a fork-owned test crate: check for no lost wakeup and an exact work count. Compare state-space size, run time, and bugs found.

### Adoption gates

| Tool | Gate | Why |
|---|---|---|
| proptest-state-machine | Pull request | Cheap, deterministic seeds |
| Quint or Stateright invariant checks | Critical change (paths of modelled components) | Seconds to minutes per model |
| ITF trace replay | Pull request for modelled components | Fast once traces are generated; regenerate traces on spec changes |
| Loom | Critical change | Exhaustive runs can be slow |
| Shuttle | Critical change, with longer runs at phase completion | Randomised; more iterations find more bugs |
