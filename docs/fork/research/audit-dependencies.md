# Audit: workspace and dependency graph

- Date: 2026-11-25
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98` (`upstream/main` was still at this commit on 2026-11-25)
- Auditor: Claude Code, Claude Opus 5.5
- Plan item: [A1](phase-1-audit-plan.md#workstreams)

## Scope

All 21 workspace crates in `Cargo.toml`: crate layers, features, optional and target-specific dependencies, external dependency counts and duplicates, crate and module cycles, and the dependencies from `anti-webui-core` to GPU or OS crates. The upstream layer rules in `docs/design/00-overview.md` ("レイヤー構成") are the yardstick.

Left out: licences (A2), the wgpu fork itself (A3), what each crate does (A4a–A4c), and whether each platform builds (A5). Unused dependencies were not checked, because no tool for it is pinned (see [open questions](#open-questions)).

## Method

Toolchain: cargo and rustc 1.99.0 on Linux x86_64 (WSL2). Tools pinned in `mise.toml`. Every command ran with `--locked` against the baseline `Cargo.lock`.

| Question | Command |
|---|---|
| Crates, features, declared dependencies | `cargo metadata --format-version 1 --locked`, read with a script |
| Resolved dependencies per target | `cargo tree -e normal --target <triple> --prefix none` for `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, and `aarch64-apple-darwin` |
| Why a crate is pulled in | `cargo tree -i <crate> --target <triple>` |
| Duplicate versions | `cargo tree --workspace -d -e normal --target x86_64-unknown-linux-gnu` |
| Module graph | `cargo modules dependencies -p <crate> --lib --no-externs --no-sysroot` (with `--target` for the macOS and Windows backends) |
| Which core modules other crates use | `rg -o 'anti_webui_core::[a-z_]+' src/<crate>/src` (a lower bound: grouped `use` lists count once) |
| Feature extremes build | `cargo check --locked` with `-p anti-webui-core --no-default-features`, `-p anti-webui-app --no-default-features`, `-p anti-webui --no-default-features`, `-p anti-webui --all-features`, and `--workspace --all-features` |

Module cycles: `cargo modules … --acyclic` stops at the first cycle, and that cycle is always a type and its own method (for example `tree::NodeId` and `tree::NodeId::from`). So the graph was collapsed instead. Each item node was mapped to its nearest enclosing module, only `uses` edges were kept, and edges between a module and its own ancestor or descendant were dropped. The strongly connected components of that module graph are the module cycles reported below.

## Findings

### Crate layers

- Status: present
- Evidence: `cargo metadata`, `src/*/Cargo.toml`
- Behaviour: the normal (non-dev) dependencies between workspace crates form these layers. "opt" marks an optional dependency, and "linux", "macos", and "windows" mark target-gated ones.

| Layer | Crate | Workspace dependencies |
|---|---|---|
| 0 | `anti-webui-style` | none |
| 0 | `anti-webui-macros` (proc-macro) | none |
| 0 | `anti-webui-symbols-outlined` | none |
| 1 | `anti-webui-core` | style |
| 2 | `anti-webui-widgets` | core |
| 2 | `anti-webui-wgpu` | core |
| 2 | `anti-webui-video` | core |
| 2 | `anti-webui-pdf` | core |
| 2 | `anti-webui-math` | core |
| 2 | `anti-webui-wayland`, `anti-webui-x11` | core (linux) |
| 2 | `anti-webui-macos` | core (macos) |
| 2 | `anti-webui-windows` | core (windows) |
| 3 | `anti-webui-code` | core, widgets |
| 3 | `anti-webui-app` | core, wgpu; video (opt); wayland and x11 (opt, linux); macos (opt, macos); windows (opt, windows) |
| 4 | `anti-webui-markdown` | core; code and math (opt) |
| 4 | `anti-webui-wayland-shell` | app, core, wayland; widgets (opt); all linux |
| 5 | `anti-webui-internal` | app, core, macros, style, wgpu, widgets; code, markdown, math, pdf, wayland-shell (opt) |
| 6 | `anti-webui-dylib` (`crate-type = ["dylib"]`) | internal |
| 6 | `anti-webui` (facade) | internal; dylib (opt, feature `dynamic_linking`) |
| 7 | `anti-webui-md3e` | anti-webui, symbols-outlined |

- Dev-dependency edges add no cycle: `anti-webui-app` dev-depends on pdf and video, and `anti-webui-math` dev-depends on app.
- No workspace crate has a `build.rs`. No manifest sets `rust-version`, `[lints]`, or a `[profile]`.
- Label: Confirmed

### Upstream layer rules against the code

- Status: partial
- Evidence: `docs/design/00-overview.md` ("依存の向き"); the commands in [Method](#method)

| Rule in `00-overview.md` | What the code does | Label |
|---|---|---|
| `core` knows neither the GPU nor the OS | No GPU crate is reachable from core on any of the three targets. OS crates are reachable: `parley` → `fontique` pulls `yeslogic-fontconfig-sys` and `libloading` on Linux, the `windows` crate on Windows, and `objc2-core-text` on macOS, even with `--no-default-features`. Core itself calls `libc::malloc_trim` and `libc::mallopt` on Linux glibc (`src/anti-webui-core/src/images/worker.rs:169`–`188`) and has `cfg(windows)` or `cfg(unix)` branches in `drag.rs`, `text/mod.rs`, `video.rs`, and `platform.rs` | Confirmed |
| `wgpu` reads only `core::scene` | `anti-webui-wgpu` uses the core modules `video`, `shader`, `scene`, `geometry`, `damage`, `prelude`, `canvas`, `style`, and `images` | Confirmed |
| The Wayland backend uses only `core::event` and `core::platform`, and does not know wgpu | Also uses `geometry`, `window`, `layer`, and `shared`. No backend crate mentions wgpu. The X11, macOS, and Windows backends use the same set without `shared` | Confirmed |
| `app` wires platform, renderer, and core together | `anti-webui-app` depends on core, wgpu, and every backend behind features | Confirmed |
| The layer diagram | Seven crates are missing from it: video, pdf, markdown, code, math, wayland-shell, and symbols-outlined. `anti-webui-md3e`, drawn beside `app`, depends on the top facade `anti-webui` | Confirmed |

Reading: core reaches the OS only for font discovery and allocator tuning, and its platform branches are small. The C ABI plan ("core だけをラップすればよい") still holds for the GPU, but not for the OS. Label: Inferred, from the rows above.

### Features and optional dependencies

- Status: present
- Evidence: `[features]` in `src/*/Cargo.toml`
- Behaviour:
  - `anti-webui` defaults to `svg`, `svg-text`, `wayland`, `x11`, `macos`, and `windows`. Backend features are forwarded through `anti-webui-internal` to `anti-webui-app`. Each backend dependency is also target-gated, so the default turns on only the backends of the build target. Label: Confirmed (features); Inferred (only the build target's backends compile).
  - Opt-in features on the facade: `code`, `markdown`, `math`, `pdf`, `pdf-js` (`rquickjs`), `video`, `wayland-shell`, and 15 `shell-*` features, with `shell-all` enabling all of them. Label: Confirmed
  - `anti-webui-code` defaults to all 14 tree-sitter grammars, one feature each. `anti-webui-markdown` defaults to `mermaid`, `code`, and `math`; `mermaid` also turns on `anti-webui-core/svg`. `anti-webui-md3e` has a default `bundled-fonts` feature that turns on no dependency. Label: Confirmed
  - `anti-webui-wayland-shell` features: `dbus` pulls `zbus`, `async-executor`, `async-lock`, and `futures-lite`; `hyprland`, `niri`, and `sway` pull `serde_json`; `audio` pulls `libloading`. Label: Confirmed
- Builds: every feature extreme in [Method](#method) passes `cargo check` on Linux, with warnings. Warning counts belong to A7. Label: Confirmed
- Exact version pins: `hayro = "=0.9.0"` (pdf), `merman = "=0.10.0-alpha.1"` (markdown, a pre-release), and `ratex-font`, `ratex-layout`, `ratex-parser`, and `ratex-types = "=0.1.15"` (math). Label: Confirmed

### External dependency counts

- Status: present
- Evidence: `cargo tree` and `Cargo.lock`. Counts are distinct name and version pairs and include the 21 workspace crates.

| Set | Packages |
|---|---|
| `Cargo.lock` | 467 |
| Linux, default features, normal edges | 339 |
| Windows, default features, normal edges | 317 |
| macOS, default features, normal edges | 307 |
| Linux, all features, normal edges | 390 |
| All targets, all features, every edge kind | 460 |
| `anti-webui-core` alone, Linux, default features | 106 (82 with `--no-default-features`) |

- Sources: 382 registry packages, 21 path packages, and 10 git packages in `cargo metadata`. All 10 git packages are the patched `wgpu` family from `bea4dev/wgpu` branch `adapter-filter`, locked at `ff143aeeb126dce521b87f47547565c4b6a6da1e` (A3).
- Label: Confirmed

### Duplicate versions (Linux, default features)

- Status: present
- Evidence: `cargo tree -d` and `cargo tree -i <crate>@<version>`

| Crate | Versions and what pulls each |
|---|---|
| `read-fonts` | 0.41.0 (core, math, pdf, `fontique`, `harfrust`, `skrifa` 0.44); 0.46.0 (`skrifa` 0.49 ← `hayro-font`) |
| `skrifa` | 0.44.0 (`parley_engine`, `swash`, `usvg`); 0.49.0 (`hayro-font`) |
| `syn` | 2.0.119 (`anti-webui-macros`, `palette_derive`, `phf_macros`); 3.0.9 (`bytemuck_derive`, `cssparser-macros`, `displaydoc`) |
| `thiserror` | 1.0.69 (`ratex-lexer`, `ratex-parser`); 2.0.21 (`gpu-allocator`, `manatee`, `merman`) |
| `phf` | 0.11.3 (`ratex-font`); 0.13.1 (`hayro-font`, `htmlize`); 0.14.0 (`cssparser`, `web_atoms`) |
| `rustc-hash` | 1.1.0 (`naga`, `wgpu-core`); 2.1.3 (`dugong`, `hayro`) |
| `hashbrown` | 0.16.1 (`gpu-allocator`); 0.17.1 (`fontique`, `indexmap`) |
| `fearless_simd` | 0.7.0 (`vello_common`); 1.2.0 (`hayro`) |
| others | `bit-set`, `bit-vec`, `miniz_oxide`, `quick-xml`, `roxmltree`, and the `phf` and `thiserror` helper crates |

- 17 crate names have more than one version. Most second copies come from the PDF (`hayro`), math (`ratex`), and Markdown (`merman`) crates. The text stack and the workspace pin share one `read-fonts` (0.41.0).
- Label: Confirmed

### Crate cycles

- Status: absent (no cycles)
- Evidence: the layer table above; cargo rejects cycles in normal dependencies, and the dev-dependency edges were checked by hand.
- Label: Confirmed

### Module cycles

- Status: present
- Evidence: the module-graph method in [Method](#method). Linux host, default features; the macOS and Windows backends were analysed with `--target`.

| Crate | Modules | Modules in cycles (parent and child edges dropped) | Cycle members |
|---|---|---|---|
| `anti-webui-core` | 40 | 25, in one cycle | access, animation, damage, drag, event, images, layout, paint, platform, scene, scroll, settings, svg, svg_file, text, text::edit, text::editor, text::rich, text::selection, tree, ui, video, view, visibility, window |
| `anti-webui-style` | 7 | 4 | sheet, style, styled, transition |
| `anti-webui-wayland` | 11 | 7 | decor, drag, layer, lock, popup, present, state |
| `anti-webui-macos` | 11 | 5 | dnd, popup, remote, state, view |
| `anti-webui-x11` | 13 | 4 | popup, selection, state, xdnd |
| `anti-webui-wayland-shell` | 14 | 4 | clipboard, conn, toplevels, workspaces |
| `anti-webui-windows` | 11 | 3 | ime, manipulation, wndproc |
| `anti-webui-wgpu` | 13 | 2 | dmabuf, gbm |
| `anti-webui-md3e` | 51 | 2 | color, dynamic::scheme |
| app, code, markdown, math, pdf, video, widgets, and the rest | — | 0 | — |

- Core detail: outside the cycle are `canvas`, `channel`, `drawing`, `geometry`, `images::decode`, `images::worker`, `layer`, `overlay`, `paint::part`, `prelude`, `reactive`, `shader`, `shared`, and `task`. `reactive` and `geometry` use no other core module, and many modules use them (14 and 25). `tree` has the most two-way links: with `drag`, `layout`, `text`, `text::selection`, `view`, and `visibility`. Thirteen module pairs use each other directly, among them `event`↔`platform` and `platform`↔`window`.
- Counting parent and child edges as well adds small cycles in app, math, and pdf, such as `present`↔`present::dmabuf`.
- Label: Confirmed

## Baseline violations

The fork has no dependency rules of its own yet (Phase 3 sets them). Against the upstream rules in `00-overview.md`, the baseline is:

- Core reaches OS crates through `parley` and `fontique`, and calls `libc` directly on Linux glibc.
- The renderer uses 9 core modules, not only `scene`. The backends use 5 or 6, not only `event` and `platform`.
- 25 of 40 core modules form one module cycle. Eight other crates have smaller ones (table above).

Fork changes must not add to these numbers until Phase 3 replaces them with fork rules.

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| Core is one large module cycle, so it cannot be split into smaller crates, or given a narrow C ABI, without untangling `tree`, `view`, `drag`, `text`, and `platform` first | High | High, if Phase 3 wants core sub-crates or an FFI | [Module cycles](#module-cycles) |
| OS reach from core, through system font discovery, makes core harder to test headless and to wrap for other languages | Medium | Medium | [Upstream layer rules against the code](#upstream-layer-rules-against-the-code) |
| Exact pins on a pre-release (`merman =0.10.0-alpha.1`) and on `hayro` and `ratex-*` block semver updates and need a manual bump | Medium | High | [Features and optional dependencies](#features-and-optional-dependencies) |
| Ten crates come from a git fork of wgpu | High | Covered by A3 | [External dependency counts](#external-dependency-counts) |
| Duplicate crate versions add build time and binary size, mostly from optional crates | Low | High | [Duplicate versions](#duplicate-versions-linux-default-features) |
| Layer rules exist only in prose, so nothing stops new edges | Medium | High | `docs/design/00-overview.md`; no fitness function |

## Open questions

- Are there unused dependencies? Answered by cargo-shear or cargo-machete, which the [tool adoption matrix](../quality/tool-adoption-matrix.md) plans for later.
- Do the Windows and macOS builds of core have the same module graph? The `cfg(windows)` and `cfg(unix)` branches are small, but core was analysed only for Linux. Answered by `cargo modules dependencies -p anti-webui-core --target <triple>`.
- Which core OS reach can move behind a trait or a feature, for example font discovery? Phase 3 decides, using A4a's findings on text.

## Inputs for gap analysis

- Layer rules and their violations, for the Phase 3 dependency rules and for fitness functions in Phase 5.
- The core module cycle, for any Phase 3 plan to split core or wrap it in a C ABI.
- Feature and pin data, for the licence audit (A2) and the platform matrix (A5).
