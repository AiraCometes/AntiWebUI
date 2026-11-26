# Audit: tests and benchmarks

- Date: 2026-11-26
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Auditor: Claude Opus 5.5 (runs, checks, and text); the per-file oracle classification was drafted by Pi (DeepSeek V4.1 Flash) and spot-checked
- Plan item: [A6](phase-1-audit-plan.md#workstreams)

## Scope

Every test in the workspace: unit tests in `src/*/src`, integration tests in `src/*/tests`, and the `harness = false` test binaries of `anti-webui-app`. For each crate: how many tests exist, how many run on Linux with default features, what fails here and why, how pixel tests behave without a GPU, how long the suite takes, line coverage, and the test oracle each file relies on. One mutation-testing sample checks how sensitive the undo/redo tests are.

Left out: build time, binary size, and run-time performance (A9); Clippy, rustfmt, and `unsafe` (A7); macOS and Windows test runs, which need real devices ([A5](phase-1-audit-plan.md#workstreams)).

## Method

Toolchain 1.99.0 from `rust-toolchain.toml`, `debug = "line-tables-only"`, on WSL2 (Linux 6.18) inside the agent sandbox. No `/dev/dri`; the only Vulkan driver is lavapipe (`/usr/share/vulkan/icd.d/lvp_icd.json`). FFmpeg is not installed.

```sh
# Static counts
rg -c '#\[test\]' src/<crate>/src src/<crate>/tests
rg -n -A1 '#\[ignore' src
# What runs, and the results
cargo nextest list --workspace --locked --message-format json
cargo nextest run --workspace --locked --no-fail-fast --success-output final --failure-output final
cargo nextest run --locked -p anti-webui-wayland-shell --all-features --no-fail-fast
cargo nextest run --locked -p anti-webui-pdf --all-features --no-fail-fast
# Coverage (the instrumented build in target/llvm-cov-target is about 19 GB; delete it afterwards)
cargo llvm-cov nextest --workspace --locked --ignore-run-fail --json --summary-only --output-path cov.json
# Mutation sample (cargo-mutants is not pinned in mise.toml; run once through mise x)
mise x cargo:cargo-mutants@27.1.0 -- cargo mutants --in-place -p anti-webui-core \
  -f src/anti-webui-core/src/text/edit.rs --re 'EditData::(snapshot|undo|redo)' --timeout 300 \
  -- --locked -- --skip text_with_letters_on_their_own_is_outlines
```

`--in-place` was needed because the sandbox refuses to copy `.claude/launch.json` into cargo-mutants' scratch tree. The tree was clean afterwards (`git status`). The skipped test fails on this machine without any mutation (see [Failures on this machine](#failures-on-this-machine)), and cargo-mutants needs a passing baseline.

Oracle classification: each integration test file and each crate's unit tests were sorted into one primary oracle (and optional secondary ones) from a fixed list, each with one `path:line` of evidence. The counts were re-derived, and the claims below marked Confirmed were read in the code.

## Findings

### Test inventory

| Crate | `#[test]` in `src/` | `#[test]` in `tests/` | Listed by nextest (Linux, default features) | `#[ignore]` | Not built here |
|---|---:|---:|---:|---:|---|
| `anti-webui-core` | 101 | 550 | 651 | 0 | — |
| `anti-webui-md3e` | 31 | 196 | 227 | 0 | — |
| `anti-webui-pdf` | 82 | 127 | 193 | 0 | 16 need the `js` feature |
| `anti-webui-app` | 50 | 141 | 190 | 1 | `macos`, `macos_spawn`, `windows` binaries (`harness = false`) are empty `fn main() {}` off their OS |
| `anti-webui-widgets` | 0 | 124 | 124 | 0 | — |
| `anti-webui-wayland-shell` | 41 | 23 | 10 | 1 | 53 need the `dbus`, `hyprland`, `niri`, `sway`, or other features |
| `anti-webui-markdown` | 12 | 33 | 39 | 6 | — |
| `anti-webui-wgpu` | 38 | 0 | 38 | 0 | — |
| `anti-webui-video` | 13 | 21 | 34 | 0 | — |
| `anti-webui-x11` | 13 | 9 | 22 | 0 | — |
| `anti-webui-math` | 14 | 7 | 21 | 0 | — |
| `anti-webui-code` | 13 | 6 | 18 | 1 | — |
| `anti-webui-style` | 15 | 0 | 15 | 0 | — |
| `anti-webui-wayland` | 5 | 5 | 10 | 0 | — |
| `anti-webui-symbols-outlined` | 0 | 3 | 3 | 0 | — |
| `anti-webui-macos` | 10 | 0 | — | — | macOS only |
| `anti-webui-windows` | 10 | 0 | — | — | Windows only |
| Total | 448 | 1,245 | 1,595 | 9 | 89 |

- Status: present
- Behaviour: 1,693 `#[test]` attributes. With default features on Linux, nextest lists 1,604 tests and ignores 9 of them. The 89 others are behind features (`js` in `anti-webui-pdf`, the bus and IPC features of `anti-webui-wayland-shell`) or behind `cfg(target_os)`. A plain `cargo test --workspace` therefore never runs 69 tests that build on Linux.
- Ignored tests: 17 `#[ignore]` attributes, 9 of them in what default features build. Reasons: 9 read this machine's services (logind, NetworkManager, BlueZ, PipeWire, the tray, a compositor, installed applications), 6 only time work (`time_a_long_file`, `time_typing`, `measure_the_backend`, …), 2 dump output for a manual comparison (`app/tests/svg.rs:11`, `markdown/tests/mermaid.rs:82`).
- Benchmarks: no `benches/` directory and no benchmark harness. Performance is checked by the ignored timing tests and by scripts that compare with a browser by hand: `anti-webui-pdf/tools/compare.py` (PDF.js in Chromium), `anti-webui-app/tools/svg_compare.py` and `svg_bench.py` (Chromium), `anti-webui-core/tools/chrome_layout.py` (records Chromium's layout into `tests/data/flex_wrap.txt`).
- Label: Confirmed (`rg` counts; `cargo nextest list --message-format json`; `--all-features` runs listing 209 pdf and 64 wayland-shell tests)

### Results on this machine

| Run | Tests | Passed | Failed | Skipped (`#[ignore]`) | Test time |
|---|---:|---:|---:|---:|---:|
| Workspace, default features | 1,595 | 1,586 | 9 | 9 | 34.6 s |
| `anti-webui-pdf --all-features` | 209 | 209 | 0 | 0 | 5.4 s |
| `anti-webui-wayland-shell --all-features` | 55 | 44 | 11 | 9 | 5.1 s |

- Status: present
- Behaviour: the suite is fast. The whole workspace runs in 34.6 s of test time (nextest, about 2 minutes with the build on a warm target directory). The slowest tests are pixel and physics tests: `render::shadows_are_a_real_gaussian_blur_outside_of_the_box` 25.0 s, `filters::a_radius_that_changes_a_little_changes_the_picture_a_little` 13.8 s, `kinetic::single_flicks_go_as_far_as_in_chrome` 12.2 s. Three more `kinetic` tests and `incremental::random_changes_match_a_full_update` take 4–8 s each.
- Label: Confirmed (nextest summaries and per-test durations)

### Failures on this machine

| Tests | Where | Error | Cause | Label |
|---|---|---|---|---|
| 5 Wayland protocol tests (`drag`, `fullscreen`, `layer`, `lock`, `popup_grab`) | `anti-webui-wayland/tests/*_protocol.rs`, for example `layer_protocol.rs:332` | `ListeningSocket::bind_absolute(...)`: `Operation not permitted` | The agent sandbox refuses to bind Unix sockets | Confirmed: the same tests pass in a normal WSL terminal (see below) |
| 11 wayland-shell bus and IPC tests | `anti-webui-wayland-shell/tests/common/mod.rs:47`, `hyprland_ipc.rs:54`, `niri_ipc.rs:26`, `sway_ipc.rs:44`, … | zbus `Invalid address`, `Operation not permitted`, a timeout waiting for the tray watcher | Same: no session bus and no Unix sockets inside the sandbox | Confirmed, same run |
| `core::svg::text_with_letters_on_their_own_is_outlines` | `anti-webui-core/tests/svg.rs:571` | no outlines produced | `usvg`'s default font families name Times New Roman and Arial, and this machine has only DejaVu (`fc-list`), so the text is dropped | Inferred from `fc-list` and `svg_file.rs:84-93` (`load_system_fonts` with no fallback family) |
| `app::images::svg_with_text_shows_it` | `anti-webui-app/tests/images.rs:335` | `dark` is 0 | Same as above: `font-family="sans-serif"` resolves to a font that is not installed | Inferred |
| `math::layout::tests::what_the_katex_fonts_lack_is_drawn_with_the_programs` | `anti-webui-math/src/layout.rs:1269` | width 1.77, expected more than 2.3 | No Japanese font is installed (`fc-list :lang=ja` is empty), so `日本` falls back to a narrow glyph | Inferred |
| `app::render::text_scaled_up_is_as_sharp_as_bigger_text` | `anti-webui-app/tests/render.rs:1162` | 1,053 pixels differ, limit 300 | Depends on the system's default font; the reference machine's font is unknown | Unverified |

- Status: partial. Every failure depends on the environment: 16 on what the sandbox allows, 4 on which fonts are installed. No failure points at a code defect. Outside the sandbox, in a normal WSL terminal on the same machine, `cargo nextest run --locked -p anti-webui-wayland -p anti-webui-wayland-shell --all-features` reported `65 tests run: 65 passed, 9 skipped` (maintainer's run, 2026-11-26).
- Quality notes: tests that read system fonts state their expectations for one machine's font set. Nothing in the repository pins that set (no font fixture, no `fc-list` check, no skip). A fork CI image must either install the expected fonts or the tests must bring their own.

### GPU and headless rendering

- Status: present
- Evidence: `anti_webui_app::headless::render_rgba` and `render_driven_rgba` create `Gpu::headless()` (`anti-webui-app/src/headless.rs:29`) and read the frame back as RGBA. `anti-webui-pdf/tests/view.rs` (87 tests) checks pixels from its own CPU rasterizer and needs no GPU.
- Behaviour: with lavapipe present, no test printed `skipping` or `skipped` for a missing GPU, so every pixel test drew. Without an adapter, 13 files return early and pass (8 in `anti-webui-app`: `drag`, `filters`, `font_axes`, `gpu_canvas`, `render`, `shaders`, `text_effects`, `video_gpu`; 5 in `anti-webui-md3e`: `buttons`, `components`, `feedback`, `icons`, `shapes`), for example `app/tests/drag.rs:15-25` and `:81`. Three files that render fail instead, because they `unwrap()` the result (`app/tests/images.rs:47`, `svg.rs:53`, `video.rs:174`). 25 video tests pass without FFmpeg by printing `skipped: video is unavailable` (`app/tests/video.rs:353` and the `anti-webui-video` tests).
- Quality notes: a green run says nothing about rendering unless the log is checked for skip messages. lavapipe is one Vulkan implementation; whether results match a real GPU (or DX12, Metal) is not tested here.
- Label: Confirmed for the skip code and the absence of skip messages. Inferred that lavapipe was the adapter used (the only installed ICD, no `/dev/dri`)

### Coverage

Line coverage from `cargo llvm-cov nextest`, Linux, default features, inside the sandbox (so the 20 failing tests count as run but end early):

| Crate | Lines covered | Lines | Coverage |
|---|---:|---:|---:|
| `anti-webui-core` | 21,133 | 23,170 | 91.2% |
| `anti-webui-pdf` | 8,403 | 10,261 | 81.9% |
| `anti-webui-md3e` | 9,327 | 10,144 | 91.9% |
| `anti-webui-wgpu` | 5,667 | 7,669 | 73.9% |
| `anti-webui-widgets` | 4,466 | 5,102 | 87.5% |
| `anti-webui-x11` | 477 | 4,068 | 11.7% |
| `anti-webui-wayland` | 148 | 3,546 | 4.2% |
| `anti-webui-markdown` | 2,940 | 3,316 | 88.7% |
| `anti-webui-app` | 1,072 | 3,114 | 34.4% |
| `anti-webui-video` | 377 | 2,403 | 15.7% |
| `anti-webui-wayland-shell` | 422 | 2,372 | 17.8% |
| `anti-webui-code` | 2,010 | 2,300 | 87.4% |
| `anti-webui-math` | 1,297 | 1,442 | 89.9% |
| `anti-webui-style` | 782 | 907 | 86.2% |
| `anti-webui-macros` | 24 | 46 | 52.2% |
| Workspace | 58,545 | 79,860 | 73.3% (functions 74.9%, regions 74.4%) |

- Status: present
- Behaviour: the platform-independent crates are well covered (core, md3e, widgets, markdown, math, code: 87–92%). The OS backends are not. `anti-webui-wayland` 4.2% and `anti-webui-x11` 11.7%, because their protocol tests failed in the sandbox and no compositor or X server ran. `anti-webui-video` 15.7%, because FFmpeg is absent. `anti-webui-wayland-shell` 17.8%, because most of its tests need non-default features. `anti-webui-macos` and `anti-webui-windows` are not compiled on Linux and do not appear.
- Label: Confirmed for these numbers inside the sandbox. The Wayland and wayland-shell figures are a lower bound: their protocol and bus tests pass outside the sandbox but were not measured for coverage there

### Test oracles

The oracle is the criterion a test uses to decide pass or fail. Totals by the primary oracle of each integration test file:

| Oracle | What decides pass or fail | Tests |
|---|---|---:|
| Example | Hand-written expected values for chosen inputs | 933 |
| Pixel | Pixel values of a rendered frame (GPU, or the PDF rasterizer) | 253 |
| Protocol recording | Messages exchanged with a fake compositor, bus, or IPC server | 22 |
| Live machine | This machine's real services; mostly `#[ignore]` | 14 |
| Recorded reference | Values recorded from another implementation | 10 |
| Self-consistency | Two paths of the same code must agree | 6 |
| Property | A rule checked over many or all inputs | 5 |
| Timing | Measures time only; `#[ignore]` | 2 |
| Total | | 1,245 |

Unit tests (448) are example-based in 14 crates. The exception is `anti-webui-md3e`, whose dynamic colour tests compare with values from Google's material-color-utilities (`md3e/src/dynamic/tests.rs:57`).

- Status: present, mostly example-based
- Recorded references: Chromium layout for 82 wrap cases (`core/tests/flex_wrap.rs:167`, data from `tools/chrome_layout.py`); material-color-utilities' Swift tests (`md3e/tests/dynamic_color.rs:22`, `SchemeTonalSpotTests.swift`); Chrome 155 touchpad scaling in some kinetic scrolling tests (`core/tests/kinetic.rs:673-726`, counted under example because most tests in that file are not).
- Self-consistency: `core/tests/incremental.rs` compares culled painting with `paint_everything` and the hit test with a brute-force search (`incremental.rs:80-92`). A shared bug in both paths passes. `core/tests/stacking.rs` checks the hit test against its own paint order (`:272`); `pdf/tests/edit.rs` saves and reopens (`:543`).
- Property: only 5 integration tests have a property as their primary oracle: `md3e/tests/public_api.rs` (an allowlist over every source file), `symbols-outlined/tests/fonts.rs` (3 tests over all 4,004 fonts), and `video/tests/lifetime.rs` (no growth over 30 rounds). No crate depends on proptest or quickcheck.
- Quality notes: example tests cover what their authors thought of. Few tests state a rule that holds for all inputs, and the recorded references cover flex wrapping, dynamic colour, and flick distances only. The per-file classification is in [the appendix](#appendix-oracle-per-integration-test-file).
- Label: Confirmed for the evidence lines listed; the bucket of a file with mixed tests is a judgement (Inferred), noted in the appendix

### Mutation sample: undo and redo

cargo-mutants 27.1.0 on `EditData::snapshot`, `undo`, and `redo` in `anti-webui-core/src/text/edit.rs`, with all `anti-webui-core` tests as the oracle. 9 mutants, 2 minutes:

| Result | Mutants |
|---|---|
| Caught | 7: removing `snapshot`, each of the 4 changes to the coalescing condition `kind != EditKind::Other && kind == self.last_edit` (`edit.rs:424`), `> 200` to `< 200` (`:432`), and `undo` and `redo` returning a default (`:574`, `:593`) |
| Missed | 2: `self.undo.len() > 200` changed to `== 200` or `>= 200` (`edit.rs:432`) |

- Status: partial
- Behaviour: the tests pin what undo and redo restore, how typing coalesces, and that redo is cleared. Nothing tests the 200-step history limit: dropping one step early, or (with `== 200`) letting the history grow past 201 entries, both pass.
- Quality notes: one sample of 9 mutants. It is not a mutation score for the crate. The unmutated core suite took 16 s, so a per-file mutation run on core is cheap.
- Label: Confirmed (cargo-mutants `caught.txt` and `missed.txt`)

## Baseline violations

- 20 tests fail in this environment: 16 because of the agent sandbox and 4 because of the installed fonts ([Failures on this machine](#failures-on-this-machine)). The 16 sandbox failures pass outside the sandbox, so the baseline outside it is the 4 font-dependent failures; inside the agent sandbox it is these 20. The font ones stay until they have a fixture.
- 25 video tests and 13 pixel-test files pass silently when FFmpeg or a GPU is missing.
- 69 tests that build on Linux run only with non-default features, so `cargo test --workspace` skips them.
- The build prints `float_literal_f32_fallback` future-incompatibility warnings in tests and crates (for example `anti-webui-widgets/src/menu.rs:648`, `anti-webui-math/tests/math.rs:155`). A7 counts them.

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| Pixel tests pass without drawing when no adapter is found, so a CI runner without a GPU reports green | High: rendering regressions go unseen | High on hosted CI runners | [GPU and headless rendering](#gpu-and-headless-rendering); `app/tests/drag.rs:15-25` |
| Tests read the system's fonts, so results change with the machine | Medium: false failures on CI, false passes elsewhere | Certain on any machine unlike the author's | [Failures on this machine](#failures-on-this-machine) |
| Feature-gated tests are outside the default run | Medium: 69 tests (bus, IPC, PDF JavaScript) can rot | Medium | [Test inventory](#test-inventory) |
| Almost every test is example-based; few properties, one self-consistency check for incremental painting | High for rare-input bugs in layout, text, and painting | Medium | [Test oracles](#test-oracles) |
| Limits such as the 200-step undo history have no test | Low per item; the pattern likely repeats | Inferred from one sample | [Mutation sample](#mutation-sample-undo-and-redo) |
| OS backends are covered 4–18% on Linux and 0% for macOS and Windows here | High for platform regressions | High | [Coverage](#coverage) |

## Open questions

- Resolved: the 16 sandbox failures pass outside the sandbox ([Failures on this machine](#failures-on-this-machine)).
- Which fonts did upstream's machine have for the 4 font-dependent tests? Answer: ask upstream, or find the smallest font set (for example Liberation or Noto) that makes them pass and record it.
- How do pixel results on lavapipe compare with a real GPU and with DX12 and Metal? Answer: the Phase 4 or Phase 5 CI matrix ([CI plan](../quality/ci-and-quality-services.md)).
- Do the `harness = false` macOS and Windows test binaries pass on real devices? Answer: A5 and the device question in the [audit plan](phase-1-audit-plan.md#limits-of-this-machine).

## Inputs for gap analysis

- Test oracles: the fork's requirements need an oracle per requirement. Today 75% of integration tests are examples, and the existing recorded references (Chromium layout, material-color-utilities, Chrome flick distances) are the models to extend ([candidate oracles](candidate-test-oracles-and-model-checking.md)).
- GPU tests need a "no adapter means fail" mode for CI, and a reviewed list of what lavapipe can stand in for.
- A font fixture for every test that shapes text.
- Mutation testing per changed file is cheap enough on core (16 s per run) to be a pull-request gate candidate ([tool matrix](../quality/tool-adoption-matrix.md)).

## Appendix: oracle per integration test file

"No GPU" says what the file does when `Gpu::headless()` fails: `skip-pass` (returns early and passes), `fail`, or `—` (does not use the GPU). Evidence lines were recorded for every file; the ones quoted in the findings above were re-read.

| File | Tests | Primary | Secondary | No GPU |
|---|---:|---|---|---|
| `app/tests/drag.rs` | 6 | pixel | example | skip-pass |
| `app/tests/filters.rs` | 32 | pixel | property | skip-pass |
| `app/tests/font_axes.rs` | 4 | pixel | example | skip-pass |
| `app/tests/gpu_canvas.rs` | 8 | pixel | example | skip-pass |
| `app/tests/images.rs` | 27 | pixel | self-consistency | fail |
| `app/tests/macos.rs`, `macos_spawn.rs`, `windows.rs` | 0 | live machine | example | — (only on their OS) |
| `app/tests/render.rs` | 25 | pixel | example | skip-pass |
| `app/tests/shaders.rs` | 13 | pixel | example | skip-pass |
| `app/tests/svg.rs` | 11 | pixel | example | fail |
| `app/tests/text_effects.rs` | 4 | pixel | property | skip-pass |
| `app/tests/video.rs` | 10 | pixel | example | fail (skip-pass without FFmpeg) |
| `app/tests/video_gpu.rs` | 1 | pixel | example | skip-pass |
| `code/tests/editor.rs` | 6 | example | timing | — |
| `core/tests/flex_wrap.rs` | 1 | recorded reference (Chromium) | property | — |
| `core/tests/incremental.rs` | 6 | self-consistency | property | — |
| `core/tests/stacking.rs` | 15 | example | self-consistency | — |
| `core/tests/filters.rs` | 15 | example | self-consistency | — |
| `core/tests/kinetic.rs` | 47 | example | recorded reference (Chrome 155) | — |
| `core/tests/text_editor_rows.rs` | 9 | example | property | — |
| Other `core/tests/*.rs` (26 files: accessibility, animation, content_visibility, damage, drag, drawing, font_axes, fonts, fullscreen, images, interaction, layout, offscreen, popups, portals, presence, rich_text, shaders, sheets, svg, text_drag, text_editor, transform, transition, video, windows) | 457 | example | — | — |
| `markdown/tests/mermaid.rs` | 2 | timing | — | — |
| `markdown/tests/editor.rs`, `split.rs`, `view.rs`, `math.rs` | 31 | example | timing | — |
| `math/tests/math.rs` | 7 | example | — | — |
| `md3e/tests/components.rs` | 15 | pixel | example | skip-pass |
| `md3e/tests/shapes.rs` | 10 | pixel | property | skip-pass |
| `md3e/tests/buttons.rs`, `feedback.rs`, `icons.rs` | 76 | example | pixel | skip-pass |
| `md3e/tests/dynamic_color.rs` | 9 | recorded reference (material-color-utilities) | example | — |
| `md3e/tests/public_api.rs` | 1 | property | — | — |
| Other `md3e/tests/*.rs` (containers, density, images, inputs, navigation, selection, sortable) | 85 | example | — | — |
| `pdf/tests/view.rs` | 87 | pixel (CPU rasterizer) | example | — |
| `pdf/tests/edit.rs` | 26 | example | self-consistency | — |
| `pdf/tests/script.rs` | 14 | example | — | — |
| `symbols-outlined/tests/fonts.rs` | 3 | property | self-consistency | — |
| `video/tests/lifetime.rs` | 1 | property | — | — (skip-pass without FFmpeg) |
| Other `video/tests/*.rs` (audio_reader, own_directory, reader, sound, stream) | 20 | example | — | — (skip-pass without FFmpeg) |
| `wayland/tests/*_protocol.rs` (5 files) | 5 | protocol recording | example | — |
| `wayland-shell/tests/*_bus.rs`, `*_ipc.rs`, `apps_files.rs` (12 files) | 17 | protocol recording | example | — |
| `wayland-shell/tests/audio_live.rs`, `capture_live.rs`, `dbus_live.rs`, `system_read.rs` | 5 | live machine | example | — |
| `wayland-shell/tests/process_run.rs` | 1 | example | live machine | — |
| `widgets/tests/*.rs` (9 files) | 124 | example | property (`long.rs`) | — |
| `x11/tests/server.rs` | 9 | live machine (Xvfb and XTEST) | example | — (skip-pass without Xvfb) |

Mixed files where the primary bucket is a judgement: `md3e/tests/shapes.rs` and `icons.rs` (mostly scene values, a few real pixels), `md3e/tests/public_api.rs` (a source scan, not behaviour), `app/tests/svg.rs` (the ignored `dump` feeds a Chromium comparison), `wayland-shell/tests/process_run.rs` (runs real commands).
