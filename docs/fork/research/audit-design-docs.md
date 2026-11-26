# Audit: upstream design documents

- Date: 2026-11-26
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Auditor: Pi, DeepSeek V4.1 Flash (draft, in three parts); merged, checked, and corrected by Claude Code, Claude Opus 5.5
- Plan item: [A8](phase-1-audit-plan.md#workstreams)

## Scope

All 43 files in `docs/design/`: `README.md` and `00-overview.md` through `41-gpu-canvas.md`. For each: what it designs, a short English summary, how far the code matches it, and the limits and open items it lists. `07-roadmap.md` gets a table of every open item and whether the code still lacks it.

Left out: re-running the documents' measurements (performance, power, memory, and browser comparisons are upstream's, recorded as claims; A9 measures this machine); behaviour that needs a real compositor, GPU, or OS ([A5](audit-platform-matrix.md)); the statements A4a–A5 already compared, which are cited, not re-derived.

## Method

The 43 files (10,528 lines, about 870 KB, in Japanese) were split into three parts, `README.md` and 00–15, 16–29, and 30–41, each drafted by one Pi session from the same brief. Each session read the earlier audits first ([A4a](audit-capabilities-core.md), [A4b](audit-capabilities-rendering.md#code-against-the-design-documents), [A4c](audit-capabilities-platform.md#code-against-the-design-documents), [A5](audit-platform-matrix.md#documents-against-the-code-and-the-machine)), then each document, then checked three to six concrete statements per document against the code with `rg` and by reading the named lines. No build or test was run for this audit.

The coordinator then checked every row of the disagreement table against the code (`sed -n` on each cited line, `rg -c '#\[test\]'` for each count), and set a document's verdict to "mostly matches" wherever this audit or an earlier one records a stale statement in it.

Verdicts: **matches** (no checked statement is wrong), **mostly matches** (one or more stale statements, the design itself holds), **partly stale** (a section describes code that has changed shape).

Counts by `rg -c '#\[test\]'` count attributes; where [A6](audit-tests.md)'s `cargo nextest list` differs, A6 is right.

## Findings

### Overview

14 documents match the code, 28 mostly match, and 1 is partly stale. None describes a design that the code abandoned. The stale statements are of four kinds: counts that grew (tests, pages, crates, node kinds), "not done" lists that later documents lifted, platform notes written before a backend landed, and API sketches whose names changed.

| Document | Subject | Code match |
|---|---|---|
| [`README.md`](#readmemd-index-of-the-design-documents) | the table of contents and glossary for the 42 design documents, so that a reader can find the document a piece of code points at. | matches |
| [`00-overview.md`](#00-overviewmd-goals-crate-layers-linking-and-the-binding-plan) | the shape of the whole project: what it is for, which crates exist and in which direction they depend, how a binary links against them, and how a C ABI would be laid on top later. | mostly matches |
| [`01-pixel-model.md`](#01-pixel-modelmd-the-integer-physical-pixel-model) | when and how a logical `f32` length becomes a physical `i32` one, and the seven rules that follow from doing it that way. | matches |
| [`02-reactivity-and-view.md`](#02-reactivity-and-viewmd-the-reactive-system-and-the-view-builders) | the SolidJS-shaped developer experience: fine-grained signals, a retained node tree, and the builders that make one from closures. | mostly matches |
| [`03-layout.md`](#03-layoutmd-the-integer-flexbox-engine) | the self-written layout engine and the integer distribution algorithm inside it, and why taffy is not used. | mostly matches |
| [`04-text.md`](#04-textmd-text-selection-editing-and-ime) | the split of work between parley, fontique, and AntiWebUI, and the selection, editing and IME models built on top. | mostly matches |
| [`05-rendering.md`](#05-renderingmd-the-wgpu-renderer-damage-tracking-and-caching) | how a `Scene` becomes pixels: the primitive set, the glyph atlas, damage tracking with buffer age, per-OS presenters, the scroll layer with its torus texture, and the CPU-side differencing that keeps a 20,000-row list cheap. | partly stale |
| [`06-platform-wayland.md`](#06-platform-waylandmd-the-wayland-backend) | which Wayland protocols the backend uses and what it does with each: scale, the event loop, scroll translation, popups, keyboard, clipboard, IME, and the dmabuf presenter with explicit synchronisation. | mostly matches |
| [`07-roadmap.md`](#07-roadmapmd-milestones-m1m4) | the upstream author's own progress list, used here as the yardstick for what is finished. | mostly matches |
| [`08-scrolling-and-settings.md`](#08-scrolling-and-settingsmd-kinetic-scrolling-and-the-settings-path) | where inertia is computed and where it is not, the `Scroll` event that hides the difference, the fling model borrowed from Chromium, and one global `Signal<Settings>`. | mostly matches |
| [`09-shadows-stacking-popups.md`](#09-shadows-stacking-popupsmd-shadows-stacking-order-and-popups) | the three things a pull-down list needs — CSS-shaped shadows, CSS 2.1 Appendix E stacking, and popups that leave the window. | mostly matches |
| [`10-frame-pacing.md`](#10-frame-pacingmd-rendering-a-frame-ahead-and-measuring-frames) | how to keep the CPU from idling behind the GPU and the compositor (triple buffering), how to measure that, and what time an animation is given. | matches |
| [`11-gpu-fill.md`](#11-gpu-fillmd-making-a-large-fill-cheap) | why a 120-layer page was slower than Chrome at pure fills and what fixed it — an unblended solid path, splitting a box into non-overlapping solid parts, and rounding colours to 8 bits. | mostly matches |
| [`12-transform.md`](#12-transformmd-transforms-and-compositing-layers) | `translate`, `scale`, `rotate` and `transform_origin` as paint-time transforms, and the layer machinery that makes them cheap. | mostly matches |
| [`13-animation.md`](#13-animationmd-transitions-animated-and-enterexit) | the two animation APIs and the presence mechanism, plus the reactive-engine work that made thousands of simultaneous effects affordable. | matches |
| [`14-shaders.md`](#14-shadersmd-custom-shaders-for-fill-and-overlay) | WGSL shaders that paint inside a box, the two stages they can take, the values and helpers they get, and how they are compiled and cached. | mostly matches |
| [`15-md3e.md`](#15-md3emd-the-material-3-expressive-components) | a separate crate that renders MD3E looks on top of the public `anti-webui` API, and what that crate needed from core. | mostly matches |
| [`16-filters.md`](#16-filtersmd-filters-blur-and-backdrop) | A filter stage after `14`: built-in `blur`/`backdrop_blur` and a user "image expression" for `filter` and `fill`. | matches |
| [`17-font-axes.md`](#17-font-axesmd-variable-font-axes) | Variable-font axes as style properties: inheritance, transitions, `opsz`, and the glyph-atlas consequences. | mostly matches |
| [`18-icons.md`](#18-iconsmd-icons) | Material Symbols shipped as one one-glyph font per icon, with the API, the box, and registration. | mostly matches |
| [`19-buttons.md`](#19-buttonsmd-button-parts) | The MD3E button family added after `15`: icons, toggles, icon buttons, FABs, and button groups. | matches |
| [`20-md3e-components.md`](#20-md3e-componentsmd-the-rest-of-the-md3e-parts-and-the-showcase) | The remaining Material 3 Expressive parts, the core additions they needed, `Ink`, density, and the showcase. | mostly matches |
| [`21-widgets.md`](#21-widgetsmd-the-standard-parts) | `anti-webui-widgets`: quiet default parts built only on core's public API, with `Sheet`, `Theme`, and virtualised long lists. | mostly matches |
| [`22-images.md`](#22-imagesmd-images) | `img` and `background-image` as replaced elements, with the decode pipeline, caches, memory rules, and the renderer's primitive. | matches |
| [`23-drag-and-drop.md`](#23-drag-and-dropmd-grabbing-and-drag-and-drop) | Three layers of pointing — `on_grab`, in-application typed drag and drop, OS drag and drop — plus text dragging and sorting. | mostly matches |
| [`24-video.md`](#24-videomd-video) | A `<video>` on FFmpeg loaded at run time: decoding, presentation cadence, audio, and frame-accurate reading. | matches |
| [`25-svg.md`](#25-svgmd-path-drawing-and-inline-svg) | `Primitive::Path` and `svg` as ordinary nodes, with the sparse-band coverage scheme and the `usvg` file path. | matches |
| [`26-pdf.md`](#26-pdfmd-pdf) | A fast PDF viewer on `hayro`: worker rendering, partial pictures and zoom, text, links, forms and annotations, presentation mode, and form JavaScript. | matches |
| [`27-fullscreen.md`](#27-fullscreenmd-fullscreen) | Window fullscreen and element fullscreen in core, with the Wayland path and scanout preference. | mostly matches |
| [`28-markdown.md`](#28-markdownmd-markdown) | `rich_text`, `markdown_view` with Mermaid, `text_editor`, and the live-preview `markdown_editor`. | mostly matches |
| [`29-content-visibility.md`](#29-content-visibilitymd-content-visibility-deferring-what-is-not-seen) | `content-visibility` for elements: the `auto` relevance rule, scroll anchoring, and `estimate_contents`. | matches |
| [`30-performance-overlay.md`](#30-performance-overlaymd-performance-overlay) | a frame-time and render-ahead readout at the top right of a window. | mostly matches |
| [`31-code-editor.md`](#31-code-editormd-a-code-editor-syntax-highlighting) | a tree-sitter-highlighted code editor built on `text_editor`. | mostly matches |
| [`32-markdown-split.md`](#32-markdown-splitmd-a-split-markdown-editor-and-colored-code-blocks) | source on the left, preview on the right, and tree-sitter coloring of code blocks. | mostly matches |
| [`33-customization.md`](#33-customizationmd-changing-parts-without-forking) | how a program restyles standard controls and builds its own set from the public API. | mostly matches |
| [`34-accessibility.md`](#34-accessibilitymd-an-accessibility-surface) | the interface in core that an assistive-technology bridge would use, with no bridge itself. | mostly matches |
| [`35-platform-x11.md`](#35-platform-x11md-the-x11-backend) | the X11 backend from the window to DRI3/Present, input, popups, clipboard, drag and drop, and IME. | mostly matches |
| [`36-multi-window.md`](#36-multi-windowmd-several-windows-and-window-decorations) | windows on their own threads, handles, closing order, modal windows, and decorations. | mostly matches |
| [`37-wayland-shell.md`](#37-wayland-shellmd-a-wayland-shell) | a toolkit for writing a desktop shell: layer surfaces, outputs, and Linux desktop services. | matches |
| [`38-platform-macos.md`](#38-platform-macosmd-the-macos-backend) | the macOS backend: AppKit windows on the main thread, `IOSurface` presentation, input, IME, popups, clipboard, and multi-window. | matches |
| [`39-platform-windows.md`](#39-platform-windowsmd-the-windows-backend) | the Windows backend, including start-up cost and how it is measured. | mostly matches |
| [`40-math.md`](#40-mathmd-mathematics-latex) | `$…$` and `$$…$$` in Markdown, drawn without JavaScript or a WebView. | matches |
| [`41-gpu-canvas.md`](#41-gpu-canvasmd-a-gpu-canvas-gpu_canvas) | a box a program draws into with wgpu, plus `portable_canvas`, offscreen nodes and `gpu_info`. | mostly matches |

### Statements that disagree with the code

New ones only; those already in the earlier audits are in their tables, linked in [Method](#method). Every row was checked by the coordinator.

| Document and section | The document says | The code says | Label |
|---|---|---|---|
| `00-overview.md`, "レイヤー構成" | The layer diagram has 14 crates | 21 workspace members; the diagram lacks `anti-webui-symbols-outlined`, `-video`, `-pdf`, `-markdown`, `-code`, `-math`, and `-wayland-shell` | Confirmed (`Cargo.toml:4-26`) |
| `02-reactivity-and-view.md`, "ノードツリー" | Node kinds are `Element`, `Text`, `TextEdit`, `Fragment` | 13 kinds, including `Edit`, `Editor`, `Image`, `Video`, `Svg`, `SvgShape`, `SvgText`, `Drawing`, `Canvas`, `Popup` | Confirmed (`src/anti-webui-core/src/tree.rs:174-200`) |
| `04-text.md`, "編集モデル（TextEdit）" | The caret blinks every 500 ms | `CARET_BLINK = Duration::from_millis(530)` | Confirmed (`src/anti-webui-core/src/paint.rs:26`) |
| `04-text.md`, "IME（text-input-v3）" | Core emits `ImeRequest { enabled, surrounding, cursor_rect, content_type }` | `ImeState { surrounding, cursor, anchor, cursor_rect, multiline, purpose }`, sent as `PlatformRequest::SetIme(Option<ImeState>)` | Confirmed (`src/anti-webui-core/src/platform.rs:43`, `:128`) |
| `06-platform-wayland.md`, "使用プロトコル" | `zxdg_decoration_manager_v1` … "CSD は将来" | CSD exists: client-side decoration is requested and the bar is drawn in-window | Confirmed (`src/anti-webui-wayland/src/lib.rs:857`, `src/anti-webui-widgets/src/title_bar.rs:143`) |
| `08-scrolling-and-settings.md`, the settings table | The settings are the eight listed | `Settings` also carries `images.cache_bytes` (256 MiB) and `video.*` | Confirmed (`src/anti-webui-core/src/settings.rs:11`, `:30`, `:33`, `:107`) |
| `09-shadows-stacking-popups.md`, "今回やらないこと" | "Windows / macOS（どちらもポップアップ用の別ウィンドウで実現できる形にしてある）" | Popups exist on macOS, Windows and X11, not just Wayland | Confirmed (`src/anti-webui-macos/src/popup.rs:1`, `src/anti-webui-x11/src/popup.rs`, `src/anti-webui-windows/src/state.rs:48`) |
| `11-gpu-fill.md`, "残る課題" | Scroll-layer compositing is still on the glyph pipeline | `vs_layer`/`fs_layer` is a dedicated pipeline (`renderer.rs:875`) — noted in [A4b](audit-capabilities-rendering.md#code-against-the-design-documents), repeated here because `11` still says otherwise | Confirmed |
| `15-md3e.md`, "方針" | "依存は `anti-webui` だけにする" | The manifest also depends on `anti-webui-symbols-outlined` | Confirmed (`src/anti-webui-md3e/Cargo.toml:16`) |
| `17-font-axes.md`, 「やっていないこと」 | 「1 段落の中で軸を変えること」 is not done | A `span` carries a full `Style`, implements `Styled`, and merges its `font_axes` into shaping (`src/anti-webui-core/src/text/rich.rs:107`, `:408`–`410`) | Confirmed |
| `18-icons.md`, 「やっていないこと」 | 「ボタンの中のアイコン（`.icon()` など）」 is not done | `ButtonStyle::icon`/`trailing_icon` exist and are tested (`src/anti-webui-md3e/src/components/button.rs:264`, `:270`; `tests/buttons.rs:154`); only the standard widgets' `button()` still has none | Confirmed |
| `20-md3e-components.md`, 「やっていないこと」 | 「`flex-wrap`…（[07](../../design/07-roadmap.md) の M2）」 is not done | The engine has `flex-wrap` and `07-roadmap.md:37` marks it done in M1 ([A4a](audit-capabilities-core.md#layout-engine)); chips and button rows just do not use it | Confirmed |
| `20-md3e-components.md`, ショーケース | Six pages | Seven, with `dragging` (`src/anti-webui-md3e/examples/showcase/main.rs:18`–`27`) | Confirmed |
| `21-widgets.md`, 測ったもの | Eight example pages | Ten, adding `images` and `dragging` (`src/anti-webui/examples/widgets/main.rs:15`–`26`) | Confirmed |
| `28-markdown.md`, 「確かめたこと（M3、M4）」 | core `tests/text_editor.rs` 9, markdown `tests/editor.rs` 4 | 13 and 11 (`rg -c '#\[test\]'`) | Confirmed |
| `30-performance-overlay.md`, "分かっている限界" | Only the Wayland runner has the overlay | The overlay is in the shared app runner, with no cfg gate (`app/src/runner.rs:295`), so every backend that uses the runner draws it | Confirmed |
| `30-performance-overlay.md`, "分かっている限界" | The number of frames ahead is not shown under `Custom` | `Custom { max_advance }` prints its number (`app/src/pacing.rs:388`) | Confirmed |
| `31-code-editor.md`, "確かめたこと" | `core/tests/text_editor_rows.rs` 8 tests and `code/tests/editor.rs` 5 | 9 and 6 | Confirmed |
| `31-code-editor.md`, "行番号（`gutter`）" | `text_editor().gutter(Style)` | `gutter(Gutter)` (`core/src/view.rs:2352`); there is no `From<Style> for Gutter` | Confirmed |
| `32-markdown-split.md`, "確かめたこと" | `tests/split.rs` 5 tests | 9 (`markdown/tests/split.rs`) | Confirmed |
| `33-customization.md`, "確かめたこと" | The workspace's 1,428 tests pass | [A6](audit-tests.md) counted 1,693 `#[test]`, 1,604 listed with default features | Confirmed (via the test audit) |
| `34-accessibility.md`, "`Access` で言えること" | `role` has "40 余り" (40-odd) variants | 39 (`core/src/access.rs:16`–`92`) | Confirmed |
| `35-platform-x11.md`, "やらないこと" | Client-side decorations are not done | `x11/src/decor.rs:1`–`2` handles a frame of the window's own and its resize edges; `x11/tests/server.rs:764` checks the frame hint | Confirmed |

### Open items in `07-roadmap.md`

Every M1 box is `[x]`, so the open items are the four M1 known limits, the M2–M4 lines that are not annotated as done, and the one M3 item each backend still lists as "今後".

| Item | Roadmap section | Status in the code today | Evidence | Label |
|---|---|---|---|---|
| Real-hardware check of IME, clipboard and primary selection round trips on Wayland | "M1 の既知の制限" | Not settled by reading; the code paths exist | [A4c](audit-capabilities-platform.md#wayland-backend), [A4c](audit-capabilities-platform.md#clipboard-cursor-and-high-dpi-scale) | Unverified |
| Frame advance where the compositor runs on AMD and an NVIDIA dGPU is also enabled | "M1 の既知の制限" | Still unmeasured; `10-frame-pacing.md` repeats it under "未確認のこと" | `docs/design/10-frame-pacing.md` "未確認のこと"; `src/anti-webui-app/src/pacing.rs` | Unverified |
| Compositors without server-side decoration show no title bar unless `title_bar` is placed | "M1 の既知の制限" | Still true by construction | `src/anti-webui-widgets/src/title_bar.rs:143`; `docs/design/36-multi-window.md` | Confirmed |
| Performance of shaders, MD3E, filter/backdrop, font axes, icons and images at 60 Hz and with the iGPU compositing | "M1 の既知の制限" | Still unmeasured; `15-md3e.md` repeats it as "未計測" | `docs/design/15-md3e.md` "未計測" | Unverified |
| Custom shadows | M2 | Absent; `14` lists it under "やっていないこと" | `docs/design/14-shaders.md:241`; [A4b](audit-capabilities-rendering.md#code-against-the-design-documents) | Confirmed |
| Blur behind the window (`ext-background-effect-v1`) | M2 | Absent | `rg -ni 'background.effect' src/` finds nothing | Confirmed |
| MD3E input modes (date, time) | M2 | Built | `src/anti-webui-md3e/src/components/date_picker.rs:356`, `time_picker.rs:76` | Confirmed |
| MD3E locale | M2 | Absent; no locale anywhere in the crate | `rg -ni 'locale' src/anti-webui-md3e/src` finds nothing | Confirmed |
| CSS subset (stylesheet, classes, hot reload) | M2 | Absent; styles are Rust builders only | [A4a](audit-capabilities-core.md#style-system); `simplecss` is used only for SVG files (`src/anti-webui-core/src/svg_file.rs:1149`) | Confirmed |
| Grid | M2 | Absent | [A4a](audit-capabilities-core.md#layout-engine); no `Grid` in `src/anti-webui-style/src/types.rs` | Confirmed |
| CSD and multiple windows | M2 (annotated done 2026-11-07) | Built | `docs/design/36-multi-window.md`; `title_bar.rs:143`; `wayland/src/lib.rs:857` | Confirmed |
| Accessibility through AccessKit | M2 | Partly: core exposes `snapshot`, `watch`, `perform`, but no AccessKit bridge exists | `src/anti-webui-core/src/access.rs:297`, `:357`, `:304`; no `accesskit` in any `Cargo.toml` | Confirmed |
| Element inspector | M2 | Absent | `rg -ni inspector src/` finds nothing | Confirmed |
| `anti-webui-ffi` (C ABI), C++ / C# bindings | M3 | Absent; still future | `Cargo.toml` members; no `cdylib`/`staticlib` in any manifest | Confirmed |
| Windows: own swapchain and `Present1` dirty rects for damage tracking | M3 ("今後") | Built | `src/anti-webui-wgpu/src/dxgi.rs:286` (`DirtyRectsCount`), `:298` (`Present1`), `:361` (`CreateSwapChainForComposition`) | Confirmed |
| Windows: OLE drag and drop | M3 ("今後") | Absent; `Win32_System_Ole` is used only for `CF_UNICODETEXT` | `src/anti-webui-windows/src/clipboard.rs:11`; no `IDropTarget`/`DoDragDrop`/`RegisterDragDrop` in `src/anti-webui-windows/src` | Confirmed |
| Windows: TSF | M3 ("今後") | Absent; IMM32 is what is used | no `ITfThreadMgr` in `src/anti-webui-windows/src`; [A4c](audit-capabilities-platform.md#windows-backend) | Confirmed |
| Wayland shell L1–L4 "どれも手での確認待ち" | M4 | Built; the hand check cannot be settled here | `shell-*` features in `src/anti-webui/Cargo.toml`; `examples/shell_launcher` | Unverified |
| WebRTC, screen capture | M4 (last line) | Absent | `rg -ni 'webrtc\|screencast' src/` finds nothing | Confirmed |
| Notifications sent by an app | M4 (last line) | Absent as a client; the shell can be the daemon | `src/anti-webui-wayland-shell/src/notifications.rs:1` (owns `org.freedesktop.Notifications`) | Confirmed |

### Per document

#### `README.md`: index of the design documents

- Subject: the table of contents and glossary for the 42 design documents, so that a reader can find the document a piece of code points at.
- Summary: one row per document 00–41, each with a one-line description of what it designs, and a five-term glossary. The opening states the premise: a lightweight framework that draws with wgpu itself, without a web view, and that borrows the *ideas* of the web (flexbox, CSS-shaped styles, an inspector, hot reload) rather than its assets (HTML/JS). The rows are the only place the set of documents is enumerated, so a new document has to be added here as well as in `docs/design/`. The glossary fixes the five terms the rest of the design builds on: logical pixels (`f32`), physical pixels (`i32`), the scale as `n / 120`, a node, and an owner.
- Code match: matches.
  - The 42 files the table lists all exist, and no file in `docs/design/` is missing from the table (`ls docs/design` gives `README.md` and `00-`…`41-`). Confirmed.
  - "スケール | `n / 120`（Wayland `wp_fractional_scale_v1` と同じ表現）。`Scale(n)`": `Scale(pub u32)` with `ONE = Scale(120)` and `px` dividing by 120 (`src/anti-webui-core/src/geometry.rs:3`, `:8`, `:23`). Confirmed.
  - "物理ピクセル … レイアウト結果は `i32`": the layout rectangles are `RectI` of `i32` and the style values are `f32` ([A4a](audit-capabilities-core.md#layout-engine)). Confirmed.
  - Row 15 names `anti_webui_md3e`, which is a workspace member (`Cargo.toml:15`). Confirmed.
- Limits and open items: the index lists none of its own. No row carries a status, so nothing here says whether a document is still current, and the labels have aged as the documents grew — a reader has to open the document to find out what it now covers. Inferred (from the drift recorded per document below, none of which the index reflects).

#### `00-overview.md`: goals, crate layers, linking, and the binding plan

- Subject: the shape of the whole project: what it is for, which crates exist and in which direction they depend, how a binary links against them, and how a C ABI would be laid on top later.
- Summary: five goals — no bundled Chromium, fractional scaling correct from the start, browser-grade text, a SolidJS-flavoured builder API, and bindings for other languages. The crate diagram puts `core` (reactive system, node tree, style resolution, layout, text, selection, events) at the bottom, with `wgpu` and the four OS backends beside it, `app`/`widgets`/`md3e` above, and the `anti-webui` facade on top. The dependency direction is the point: core knows neither the GPU nor an OS; it takes platform-independent events and emits a `Scene` plus platform requests. Linking is `rlib` for release, an optional `dylib` for faster incremental builds (Bevy's trick, no ABI stability, never shipped), and a future `cdylib` + `staticlib` C ABI in a crate called `anti-webui-ffi` with generational handles, `(ptr, len)` UTF-8 strings, and UI-thread-only callbacks. Rejected: reusing HTML/JS, and shipping the dylib.
- Code match: mostly matches.
  - Every crate the diagram draws exists (`Cargo.toml:4-26`, `src/`): `anti-webui`, `-internal`, `-dylib`, `-app`, `-widgets`, `-md3e`, `-core`, `-style`, `-macros`, `-wgpu`, `-wayland`, `-x11`, `-macos`, `-windows`. Seven crates the diagram does not draw are also members: `-symbols-outlined`, `-video`, `-pdf`, `-markdown`, `-code`, `-math`, `-wayland-shell` (21 members in all). Confirmed.
  - "`core` は GPU も OS も知らない": `src/anti-webui-core/Cargo.toml:15-29` has no `wgpu`, no `wayland-client`, no `windows`, no `objc2`; the only OS-specific entry is `libc` for GNU/Linux (`:30`). Confirmed.
  - "`wayland` は … wgpu を知らない": `src/anti-webui-wayland/Cargo.toml:9-19` depends on core and `raw-window-handle` only, and `rg -n wgpu src/anti-webui-wayland/src` finds nothing. Confirmed.
  - The `dynamic_linking` feature and the `dylib` crate are as described: `dynamic_linking = ["dep:anti-webui-dylib"]` (`src/anti-webui/Cargo.toml:89`), the `#[allow(unused_imports)] use anti_webui_dylib;` that the document quotes (`src/anti-webui/src/lib.rs:24`), and `crate-type = ["dylib"]` (`src/anti-webui-dylib/Cargo.toml:9`). Confirmed.
  - "`anti-webui-macros` | `derive(ShaderParams)`": the crate's only export is that derive (`src/anti-webui-macros/src/lib.rs:13`). Confirmed.
  - `anti-webui-ffi` still does not exist, and no manifest declares `cdylib` or `staticlib`; the document calls it future work, so this is not a disagreement. Confirmed.
- Limits and open items: the document marks "多言語バインディング方針" and `anti-webui-ffi` as "（将来）"; neither exists today (Confirmed above). The layer diagram is stale in the sense that it stops at the crates of 2026-10; it names neither the format crates nor the shell crate. The document lists no other limit.

#### `01-pixel-model.md`: the integer physical-pixel model

- Subject: when and how a logical `f32` length becomes a physical `i32` one, and the seven rules that follow from doing it that way.
- Summary: style values are logical `f32`; style resolution and layout run only once the buffer size and the scale are both known, and at that point every length is converted to physical `i32`. Of the three ways to handle fractional scaling, it rejects float layout (edges blur) and edge-only snapping as Chromium and taffy do it (one box gets width 49 while its neighbour gets 50), and takes "integerise first", which costs up to 1 px of overflow at the container and buys the same physical size for the same specified size wherever the box sits. The seven rules: one rounding function, never collapse a non-zero border, trust the allocated buffer for the root, integer line boxes with sub-pixel glyph positions, integer geometry only, animations outside layout, and logical coordinates back to the OS.
- Code match: matches.
  - R1 is one function: `Scale::px` is `(logical as f64 * self.0 as f64 / 120.0).round()` (`src/anti-webui-core/src/geometry.rs:23`), and the scale is held as `n / 120` in a `u32` (`:3`). Confirmed.
  - R2: `Scale::px_min1` returns at least 1 for any positive length (`geometry.rs:28`). Confirmed.
  - R5: `pxf` gives the unrounded value "for paint-only quantities (radii, blur, font size)" (`geometry.rs:37`). Confirmed.
  - R7: `to_logical` is the inverse, "for coordinates handed back to the OS (R7)" (`geometry.rs:53`). Confirmed.
  - The worked example at scale 1.5 and the leftover-pixel distribution are checked in [A4a](audit-capabilities-core.md#layout-engine): method C, `distribute`, and `same_spec_same_size_at_fractional_scale`. Confirmed there.
  - "スケール変更時はすべてのキャッシュを破棄する" (`03-layout.md`) and the re-resolve on a scale change: [A4a](audit-capabilities-core.md#layout-engine), `ui.rs:944-948`. Confirmed there.
- Limits and open items: the document states the cost of its own choice — "合計 148.5 / 149 / 150（1px はみ出しうる）" — and the code still has that property, since a fixed-size row can be 1 px wider than its container (Confirmed, same mechanism as R1). It lists no other deferral. R4's "1/4 px 単位でサブピクセル配置する" is a renderer property; [A4b](audit-capabilities-rendering.md#glyph-atlas-and-subpixel-positioning) covers the atlas key.

#### `02-reactivity-and-view.md`: the reactive system and the view builders

- Subject: the SolidJS-shaped developer experience: fine-grained signals, a retained node tree, and the builders that make one from closures.
- Summary: a component function runs once, and anything that can change is a closure registered as an effect, so there is no virtual DOM and no diff. The primitives are `signal`, `memo`, `effect`, `batch`, `untrack`, `on_cleanup`, and `create_root`, over a thread-local `slotmap` arena of `Copy` handles, with values in `Rc<RefCell<...>>` borrowed only outside the runtime borrow so a closure can write another signal. Propagation is the three-colour Clean/Check/Dirty push-then-pull algorithm, which is what keeps a diamond from showing an intermediate value; a node re-subscribes on every run, so dependencies are dynamic. Nodes are created by builders through `IntoView`, belong to the creating owner, and are removed when it is disposed, clearing focus, selection, and hover if they pointed at them. `dyn_view`, `show` and keyed `for_each` cover dynamic children. Events are `Rc<dyn Fn>` on the node, dispatched after the tree borrow is released, inside `batch`, bubbling to the parent; `on_click` is the primary button's and `on_aux_click` the others'.
- Code match: mostly matches.
  - The primitives are as listed, with the three-colour engine and the diamond test: [A4a](audit-capabilities-core.md#signals-and-reactivity). Confirmed there.
  - `dyn_view` (`src/anti-webui-core/src/view.rs:1467`), `show` (`:1618`), keyed `for_each` (`:1745`) exist. Confirmed.
  - "同じ種類のリスナーを複数付けると、付けた順にすべて呼ばれる": each `Listeners` field is a single `Option<Handler<_>>` (`src/anti-webui-core/src/tree.rs:29-47`), and `on_hover` and its neighbours compose the new closure behind the old one through `also` / `also_bool` (`view.rs:316`, `:330`), so old then new — the order the document promises. Confirmed.
  - `on_active` (`view.rs:682`) and `on_focus_visible` (`:674`) exist, as does `stop_propagation` on the event types (`event.rs:365`, `:382`, `:409`, `:431`). Confirmed.
  - "ハンドラ呼び出しは `batch` で包む": the dispatch sites wrap the call, e.g. `batch(|| handler(&ev))` (`ui.rs:1057`), `batch(|| f(w, h))` (`:381`). Confirmed.
  - "ノード種別: `Element`、`Text`、`TextEdit`、`Fragment`" — the code has 13 kinds (`tree.rs:174-200`); `TextEdit` is spelled `Edit`, and `Editor`, `Image`, `Video`, `Svg`, `SvgShape`, `SvgText`, `Drawing`, `Canvas` and `Popup` have been added since. Confirmed (stale list, not a broken rule).
- Limits and open items: none stated as such. The reactive runtime panics when a handle outlives its owner, and the effect guard panics after 1,000,000 iterations ([A4a](audit-capabilities-core.md#baseline-violations)); the document does not mention either.

#### `03-layout.md`: the integer flexbox engine

- Subject: the self-written layout engine and the integer distribution algorithm inside it, and why taffy is not used.
- Summary: taffy rounds box edges (method B in `01`), so the engine is written here and every calculation is physical `i32`. v1 covers flex (row/column and reverses, wrap, `justify-content`, `align-content`, `align-items`, `align-self`, `gap`, `grow`, `shrink`, `basis`), block as vertical stacking, `display: none` and `contents`, `position: relative` and `absolute` with `inset` against the parent's padding box, `z-index`, and `overflow` as `visible | hidden | scroll`. Left out: grid and `order`. The algorithm is basis → clamp → free space → grow or shrink with a freeze loop, then the cross axis, then `justify-content`. Leftover space is shared by integer distribution: equal weights by `q = total / n` and `r = total % n`, unequal weights by Sainte-Laguë, so growing the container never shrinks a child (no Alabama paradox). Wrapping is compared against 82 recorded Chromium cases. Incremental layout keeps measurement and placement caches keyed on their inputs and drops them all on a scale change.
- Code match: mostly matches.
  - The engine, the integer model, `distribute`, the freeze loop, and the incremental caches: [A4a](audit-capabilities-core.md#layout-engine), with the test names and the 82-case oracle. Confirmed there.
  - "未対応（後回し）: Grid, `order`" — still absent: `Display` has `Block`, `Flex`, `None`, `Contents` only ([A4a](audit-capabilities-core.md#layout-engine)), and no `Grid` appears in `src/anti-webui-style/src/types.rs`. Confirmed.
  - The oracle file exists and the generator is checked in beside the crate: `src/anti-webui-core/tests/data/flex_wrap.txt` (779 lines) and `src/anti-webui-core/tools/chrome_layout.py`. Confirmed (the document's "`tools/chrome_layout.py`" is relative to the crate, as its other paths are).
  - `flex_1()` as `flex: 1 1 0%` with the `0%`-cannot-resolve fallback for an unsized container: the document dates the change to 2026-11-06 and `32-markdown-split.md`; I did not trace `flex_1` through the engine. Unverified.
  - "ヒットテスト … 物理 px（f32 → floor で整数）": `RectI::contains` compares the `f32` point with integer edges using `>=` and `<`, which gives the same answer without flooring ([A4a](audit-capabilities-core.md#layout-engine)). Confirmed there.
- Limits and open items: "未対応（後回し）: Grid, `order`" — both still missing (Confirmed above); grid blocks 52% of taffy's fixtures ([A4a](audit-capabilities-core.md#taffy-test-fixtures)). "高さが決まっていない縦の容器：折り返さない" is a stated limit, still in force (Inferred from `03` matching the code everywhere `A4a` looked, not re-read here). Block, percentage, and absolute layout still have no external oracle ([A4a](audit-capabilities-core.md#baseline-violations)).

#### `04-text.md`: text, selection, editing, and IME

- Subject: the split of work between parley, fontique, and AntiWebUI, and the selection, editing and IME models built on top.
- Summary: parley shapes and breaks lines, fontique finds fonts, and AntiWebUI owns the selection, the editing model, IME, and the clipboard. Three things are not obvious: an empty string is zero width because parley lays it out as one space; fallback lookups are prepared once per script because fontique otherwise re-queries fontconfig on every shape (176 µs → 10 µs); and on Windows three families are appended because DirectWrite returns one family per script. Text shadows and strokes are CSS-shaped, with any number of strokes, `paint_order`, and a layer per blurred shadow. Embedded fonts are registered into fontique and take the same path, with a generation counter in the shape cache. The selection is one per document, ordered by a `doc_order` from the position pass; hit-testing falls back from the line box to the nearest node by vertical then horizontal distance.
- Code match: mostly matches.
  - Locale, `prepare_fallbacks`, `STAND_INS` (the eight families), variable axes, `register_font` / `register_font_as`, the shape cache key, and the IME apply order: [A4a](audit-capabilities-core.md#text-shaping) and [A4a](audit-capabilities-core.md#selection-editing-and-ime). Confirmed there.
  - "空の文字列は幅 0": implemented as a special case — "parley lays out empty text as a space, for the height of its line. Like an empty text node in a browser, it takes no width, unless there are boxes in it" (`src/anti-webui-core/src/layout.rs:1892-1896`). Confirmed.
  - The Windows fallback additions are `["Segoe UI Symbol", "Segoe UI Emoji", "Cambria Math"]` (`src/anti-webui-core/src/text/mod.rs:193`). Confirmed.
  - `STAND_INS` holds all eight stand-in families the document lists (`text/mod.rs:39-48`). Confirmed.
  - `text_shadows`, `text_strokes`, `paint_order` exist as `Arc<[...]>` with shared empty defaults (`src/anti-webui-style/src/style.rs:212-215`, `:228-230`). Confirmed.
  - Stroke rasterisation goes through swash's `Stroke` with `Join::Round` (`src/anti-webui-wgpu/src/glyph_cache.rs:8`, `:263`), and the shadow layer key uses `NOT_A_NODE = 1 << 63` (`src/anti-webui-core/src/paint.rs:55`, `:1999`). Confirmed.
  - "キャレットは 500ms で点滅し": `CARET_BLINK = Duration::from_millis(530)` (`src/anti-webui-core/src/paint.rs:26`). Confirmed (530, not 500).
  - "core は IME 状態を `ImeRequest { enabled, surrounding, cursor_rect, content_type }` として出力し": the type is `ImeState { surrounding, cursor, anchor, cursor_rect, multiline, purpose }` and it travels as `PlatformRequest::SetIme(Option<ImeState>)` (`src/anti-webui-core/src/platform.rs:43`, `:128`). Confirmed (different name, different fields).
  - Primary selection on a finished selection: `ClipboardKind::Primary` exists (`event.rs:187`) and `04`'s claim is in [A4c](audit-capabilities-platform.md#clipboard-cursor-and-high-dpi-scale). Confirmed there.
- Limits and open items: "fontique 0.12.0 の時点の動作で、fontique 側で直ればこの処理は要らなくなる" — `prepare_fallbacks` is still needed and still there (Confirmed, `text/mod.rs:147`). "`Style` が大きくなると、デバッグビルドのスタックが増える … 今はぎりぎり収まっている" — `Style` is large and inherited lists are `Arc` ([A4a](audit-capabilities-core.md#style-system)); still true (Confirmed by the field list). "付くのはテキストのノードだけで、入力欄の文字には付かない" — a stated limit; I did not check it against `paint.rs`. Unverified.

#### `05-rendering.md`: the wgpu renderer, damage tracking, and caching

- Subject: how a `Scene` becomes pixels: the primitive set, the glyph atlas, damage tracking with buffer age, per-OS presenters, the scroll layer with its torus texture, and the CPU-side differencing that keeps a 20,000-row list cheap.
- Summary: core emits a `Scene` of clipped primitives in physical pixels; the renderer batches runs of one pipeline into one draw call and blends premultiplied in a non-sRGB space. Damage is a chunk-level diff of consecutive scenes (`ChunkKey(node, part)`, the caret as its own part), expanded by a shadow's blur or a glyph's bounding box and capped at 8 rectangles, with the union of the last *k* frames' damage for buffer age; an empty damage presents nothing. Each OS gets a presenter of its own because wgpu exposes neither buffer age nor damage: a dmabuf on Wayland, a DXGI flip-model swapchain on Windows, an `IOSurface` on macOS, and a plain wgpu surface elsewhere. A scrolled subtree becomes one `Primitive::Layer` with the coordinates independent of the scroll, so a scroll-only frame reuses the scene and moves one integer origin, and the layer's texture is a torus so already-painted pixels do not move. Rejected: tiles (memory grows with the content) and painting a scroll frame from scratch.
- Code match: partly stale, in detail only. The four disagreements and the long list of what matched are in [A4b](audit-capabilities-rendering.md#code-against-the-design-documents); I did not re-derive them. Added here: the debug switches the document lists all exist — `ANTIWEBUI_DEBUG_DAMAGE` (`src/anti-webui-app/src/present.rs:81`), `ANTIWEBUI_PRESENT=surface` (`:96`, `src/anti-webui-wgpu/src/lib.rs:443`), `ANTIWEBUI_FRAME_STATS` (`frame_stats.rs:20`), `ANTIWEBUI_DUMP_FRAME` (`runner.rs:197`). Confirmed.
- Limits and open items:
  - "残るコスト": a changed child still makes the parent walk all its children (about 3 ms at 20,000 rows); a Chromium-style simplified layout would fix it. Still true — the layout pass is the one [A4a](audit-capabilities-core.md#layout-engine) describes. Confirmed.
  - "残るコスト": a content change runs the layer diff in two places (presenter and renderer); tolerable only because the list is culled. Still true by construction (Inferred from the same code).
  - "残るコスト": select-all and copy walk every node. Still true (Unverified — not traced).
  - macOS: "合成は層の単位（レイヤー分割は今後）". Still true: the damage reaches the `IOSurface`, but the compositor still composites the whole layer (`docs/design/38-platform-macos.md`; [A4c](audit-capabilities-platform.md#macos-backend)). Confirmed.
  - "効かない場合": zoom, and scrolling while the content also changes. Both still true (Inferred; nothing in `renderer.rs` claims otherwise).

#### `06-platform-wayland.md`: the Wayland backend

- Subject: which Wayland protocols the backend uses and what it does with each: scale, the event loop, scroll translation, popups, keyboard, clipboard, IME, and the dmabuf presenter with explicit synchronisation.
- Summary: `wayland-client` and `wayland-protocols` directly, with `libxkbcommon` for keys. The document is a protocol table plus a rule for each: buffer size `round(logical × n / 120)` with `wp_viewporter.set_destination`; a loop of dispatch, apply, flush, then poll on the Wayland fd and a wake pipe with the next deadline; a `wl_pointer` frame collapsed into one `WindowEvent::Scroll` per frame with a table mapping `axis_source` to source and phase; keys through mmap'd keymaps and the backend's own key repeat; clipboard and primary selection handled on threads with a wake pipe. The largest part is the dmabuf presenter: pick the compositor's main device, allocate through GBM so direct scanout works, import into Vulkan with explicit modifiers, and synchronise with either `wp_linux_drm_syncobj_manager_v1` or implicit fences — the document records that NVIDIA's proprietary driver corrupts exported Vulkan buffers that are scanned out, and that loading every Vulkan driver wakes a sleeping dGPU for about 0.9 s, which is why the other GPUs' manifests are filtered out with `VK_LOADER_DRIVERS_SELECT`.
- Code match: mostly matches.
  - The protocol table, the 100 ms popup deadline, `AXIS_MOTION_GAIN = 5.0`, the `Held` phase from `zwp_pointer_gesture_hold_v1`, and dmabuf v4: [A4c](audit-capabilities-platform.md#code-against-the-design-documents). Confirmed there.
  - The driver filter exists as described: `pub unsafe fn limit_vulkan_drivers(main_device: u64)` in `src/anti-webui-wgpu/src/drivers.rs:93`. Confirmed (the function exists; I did not compare its driver → manifest table with the document's).
  - "`zxdg_decoration_manager_v1` … 無い場合は装飾なし。CSD は将来": CSD exists — `DecorationMode::Client` maps to `zxdg_toplevel_decoration_v1::Mode::ClientSide` (`src/anti-webui-wayland/src/lib.rs:857`), and the widgets crate draws the bar (`title_bar`, `src/anti-webui-widgets/src/title_bar.rs:143`). Confirmed (stale).
  - The two debug switches the document names exist: `ANTIWEBUI_ALLOCATOR=vulkan` forces the Vulkan allocation path (`src/anti-webui-wgpu/src/dmabuf.rs:280`, doc comment `:80`) and `ANTIWEBUI_SYNC=implicit` forces implicit synchronisation (`src/anti-webui-app/src/present/dmabuf.rs:622`). Confirmed.
- Limits and open items:
  - "CSD は将来" — now built (Confirmed above).
  - Every optional row ("任意") is a designed degradation, not a gap: without `wl_subcompositor` the bubble falls back to `xdg_popup`, without `wp_fractional_scale_v1` to integer scale, without dmabuf v4 to a wgpu surface. The code still has those fallbacks — `present.rs`'s `TargetKind::Surface` path is the surface fallback. Confirmed.
  - "Mutter では未確認" belongs to `09`; `06` itself records no unverified behaviour beyond that.

#### `07-roadmap.md`: milestones M1–M4

- Subject: the upstream author's own progress list, used here as the yardstick for what is finished.
- Summary: M1 is "基本 UI on Wayland" and every one of its 37 boxes is ticked, ending at `text_editor`. Below it sit four "M1 の既知の制限" — real-hardware Wayland checks, frame advance on an AMD-compositor/NVIDIA-dGPU machine, the title bar on compositors without server-side decoration, and unmeasured 60 Hz performance for shaders, MD3E, filters, font axes, icons, and images. M2 ("実用性") is a plain list of seven items, one of which (CSD and multiple windows) was finished on 2026-11-07 and written into the list as done. M3 ("多言語・多プラットフォーム") holds the C ABI and the three non-Wayland backends, all three of which were finished in 2026-11-07/08 and annotated in place. M4 ("おまけ機能") is PDF, Markdown, the code editor, the Wayland shell — all annotated as implemented — and a last line of "WebRTC、画面キャプチャ、通知". The document is a historical record as much as a plan: items move from open to closed by gaining a parenthetical date.
- Code match: mostly matches.
  - Every crate and API M1 names exists: the widgets (`button`, `checkbox`, `switch`, `slider`, `scroll_view`, `text_field`, `card` at `src/anti-webui-widgets/src/button.rs:114`, `choice.rs:193`, `:620`, `slider.rs:317`, `display.rs:7`, `:16`, `field.rs:43`), `portal` (`view.rs:2035`), `set_timeout` (`animation.rs:98`), `moves` (`view.rs:1778`), `enter`/`exit`/`appear` (`view.rs:1540`, `:1546`, `:1558`). Confirmed.
  - Headless drawing and synthetic-event tests: [A4b](audit-capabilities-rendering.md#tests-that-need-a-gpu-and-what-they-do-without-one). Confirmed there.
  - The three backends M3 says were built do exist and each implements the parts the roadmap names: [A4c](audit-capabilities-platform.md) and [the matrix](audit-platform-matrix.md#the-matrix). Confirmed there.
- Limits and open items: see the table below. The four "M1 の既知の制限" and the M2–M4 lines that are still open are all carried there.

#### `08-scrolling-and-settings.md`: kinetic scrolling and the settings path

- Subject: where inertia is computed and where it is not, the `Scroll` event that hides the difference, the fling model borrowed from Chromium, and one global `Signal<Settings>`.
- Summary: only Wayland (and probably X11) needs the application to compute inertia; macOS and Windows hand theirs over, and the event's `phase` absorbs the difference rather than the framework re-deriving the OS feel. A gesture latches to the innermost scroller that can still move in the direction of the first motion, and everything after it — inertia included — goes to the same nodes. Fractions of a pixel are carried per scroller. The fling is `PhysicsBasedFlingCurve`: speed from a least-squares fit over the last three samples, distance and duration from that speed, position as a function of time so a dropped frame does not change the path, with a consecutive-flick booster. Settings are one thread-local `Signal<Settings>` in a root of its own, `#[non_exhaustive]`, changed through a closure, taking effect from the next event.
- Code match: mostly matches.
  - The gesture latch, the carried fractions, the fling curve, `LIFT_TIMEOUT`/`MIN_SPEED`/`MAX_DURATION`/`BOOST_TIMEOUT`/`FLING_SPEED_RATIO`, the 47 kinetic tests and the Chromium comparison: [A4a](audit-capabilities-core.md#scrolling-and-settings). Confirmed there.
  - The default values in the document's table are the code's: `touchpad_factor: 1.0`, `wheel_step: 48.0`, `kinetic.enabled: true`, `velocity_factor: 1.0`, `deceleration: 2645.7`, `max_screens: 3.0`, `frame_advance: Auto`, `performance_overlay: false` (`src/anti-webui-core/src/settings.rs:87-110`). Confirmed.
  - "`rendering.frame_advance` … macOS / Windows: 未実装": confirmed at the presenter — `can_hold_back` returns `false` on macOS ("Frames go to the layer as soon as they are drawn", `src/anti-webui-app/src/present/iosurface.rs:85-88`) and Windows has no own presenter at all (`present.rs:15-17`, `present/no_dmabuf.rs:39`). Confirmed.
  - The `Settings` struct carries two groups the table does not mention: `images.cache_bytes` (256 MiB) and `video` (`settings.rs:11`, `:30`, `:33`, `:105`, `:107`). Confirmed.
  - "`Held` だけはハンドラに届けない" and "`MomentumEnded` が届く": [A4a](audit-capabilities-core.md#scrolling-and-settings) left both Unverified. Still Unverified.
- Limits and open items:
  - "今回やらないこと: 端で跳ね返る表現（オーバースクロール）" — absent ([A4a](audit-capabilities-core.md#scrolling-and-settings)). Confirmed.
  - "要素ごとの設定の上書き" — absent; settings are one global signal (`settings.rs:115`). Confirmed.
  - "macOS / Windows のバックエンド（イベントの形だけ先に決めてある）" — both exist now, and they pass the OS's inertia through ([A4c](audit-capabilities-platform.md#macos-backend)). The bullet is stale, not the code.
  - The document calls its 5.0 axis gain "暫定のもの"; `AXIS_MOTION_GAIN: f32 = 5.0` is still there (`event.rs:193`). Confirmed.

#### `09-shadows-stacking-popups.md`: shadows, stacking order, and popups

- Subject: the three things a pull-down list needs — CSS-shaped shadows, CSS 2.1 Appendix E stacking, and popups that leave the window.
- Summary: shadows follow CSS semantics (blur `B` is `σ = B / 2`, `spread` grows the shape and its radii, an outer shadow is not drawn under its box) and are evaluated with Evan Wallace's analytic-in-x / four-sample-in-y coverage rather than a distance function and `erf`, whose worst error was 0.248. An outer shadow is up to four rectangles around a hole, in the same pipeline as the box, because a separate pipeline cost six times more for 200 cards. Stacking follows the CSS painting order, with the code's own rule that `overflow: scroll` is always a stacking context (a documented difference from CSS). Popups are logically children of where they are declared and separately rooted for layout and paint, with three kinds — `Menu` (grab), `Tooltip` (pass-through), `Bubble` (a subsurface) — sizes rounded to whole logical pixels so the compositor's geometry is the box and not the shadow, and four closing paths that all end in `on_dismiss`.
- Code match: mostly matches.
  - The shadow formula, the `reach`, the hole and rim drawing, the 1-logical-px shrink of the hole for an opaque rounded background (`paint.rs:1407`), and the same-pipeline decision: [A4b](audit-capabilities-rendering.md#code-against-the-design-documents) and its matched list. Confirmed there.
  - `PopupKind` is `Menu`, `Tooltip`, `Bubble` with `Menu` the default (`src/anti-webui-core/src/platform.rs:63-72`), and `Placement` has `BelowStart` as its default plus 7 others (`:77-89`). Confirmed.
  - The stacking-context conditions are in `stacking_role` (`src/anti-webui-core/src/layout.rs:750-771`): the root, a popup, `z_index`, `opacity < 1`, a non-identity transform, and `overflow_x`/`overflow_y == Scroll` — the document's own six. The code adds three the table does not list: `blur > 0`, `backdrop_blur > 0`, and a filtered ancestor, which `16-filters.md` brought later — an omission, not a contradiction. Confirmed.
  - The 100 ms popup deadline and the frame callback handling per surface: [A4c](audit-capabilities-platform.md#code-against-the-design-documents). Confirmed there.
  - Popups on the other backends: `src/anti-webui-macos/src/popup.rs:1` ("Popups as borderless child windows"), `src/anti-webui-x11/src/popup.rs`, and popup windows with spares on Windows (`src/anti-webui-windows/src/state.rs:48`, `:56`). Confirmed.
- Limits and open items:
  - "今回やらないこと" — press-and-drag menu selection, tooltip delay, submenu auto-open: still absent. Inferred for all three (nothing in `platform.rs`'s popup types carries a delay or a submenu flag); not traced through the widgets.
  - "`xdg_wm_base` v3 未満での移動とサイズ変更" and "ウィンドウごとに違うスケールでのポップアップの描画" — both still limited; a popup without `reposition` stays where it opened. Inferred from `06`'s `reposition` note; not traced.
  - "Windows / macOS（どちらもポップアップ用の別ウィンドウで実現できる形にしてある）" — both are built now (Confirmed above); the bullet is stale.
  - "スクロール内容を独立したサブサーフェスにして合成する最適化" — still absent (Inferred).
  - "残るコスト": paint invalidation is not per surface, so a hover inside a popup repaints the window too. Still true by construction (`Document::dirty` is one set of flags). Confirmed.

#### `10-frame-pacing.md`: rendering a frame ahead, and measuring frames

- Subject: how to keep the CPU from idling behind the GPU and the compositor (triple buffering), how to measure that, and what time an animation is given.
- Summary: the shape is Chromium's — render and submit, hold the commit until the compositor's frame callback for the previous frame — which only the framework's own buffers allow. A frame's cost is recorded from `CLOCK_MONOTONIC` for each CPU step, the kernel's signal time of the GPU's sync file, and `wp_presentation` for the screen, giving skipped refreshes per consecutive frame. The measurements say advance helps exactly where CPU and GPU each fit in a refresh but not together, and hurts elsewhere by one refresh of latency. `Auto` therefore does not predict: it detects skipped refreshes, tries one frame ahead for a bounded number of frames, keeps the trial only if frames then come at least 1.1× faster, and drops it once frames fit in half a refresh. Animations get the frame's place on the refresh grid rather than its start time, estimated from `wp_presentation` timestamps, which cut the mismatch from RMS 1.08 ms to 0.007 ms. Rejected: predicting a deadline (`frame_timing()`), a separate display thread, and `wp_fifo_v1`.
- Code match: matches. [A4b](audit-capabilities-rendering.md#frame-pacing) reads the whole mechanism: `FrameAdvance` and its four values, the `AutoAdvance` thresholds, `Pacing::max_advance`, `FrameClock::frame_time` and `place`, `on_may_render_ahead`, and `can_hold_back`, with 33 + 7 + 2 + 7 unit tests. Confirmed there. Added here: `ANTIWEBUI_FRAME_ADVANCE` and `ANTIWEBUI_FRAME_TIME=start` exist as the document says (`src/anti-webui-app/src/pacing.rs:341`, `:349`, with their doc comments at `:298` and `:306`). Confirmed.
- Limits and open items:
  - "未確認のこと" — a compositor on AMD with an NVIDIA dGPU enabled; a hybrid split where the dGPU draws and the iGPU composites; compositors other than ShojiWM; 60 Hz numbers; real resizes and popups. All still open (`10` is the only place they would be recorded). Unverified.
  - "訂正" — the document states that its own "結果" table, and the layer and CPU rows of the Chromium comparison, were measured before the flush fix and understate the effect. The table is still there, corrected only by the note. Confirmed (by reading the document).
  - "今回やらないこと" — deadline prediction, a display thread, delaying the start, `wp_fifo_v1` / `wp_commit_timing_v1`, and rendering efficiency itself. Still absent ([A4b](audit-capabilities-rendering.md#frame-pacing) lists the same set). Confirmed.
  - "NVIDIA でも同じ方法で読めるはずだが未確認" (the sync-file signal time). Still Unverified.

#### `11-gpu-fill.md`: making a large fill cheap

- Subject: why a 120-layer page was slower than Chrome at pure fills and what fixed it — an unblended solid path, splitting a box into non-overlapping solid parts, and rounding colours to 8 bits.
- Summary: the earlier comparison in `10` was invalid because the benchmark's alpha change rounded to the same 8-bit value, so Chrome barely rastered. With a colour that always changes, 120 rounded boxes cost Chrome 79.9 fps and AntiWebUI 45.5. The fix is to stop running the general rounded-rect shader over every pixel: integer layout means a rectangle's edges need no anti-aliasing, so a solid rectangle is a 32-byte instance on its own `vs_solid`/`fs_solid` pipeline, opaque colours take a blend-disabled pipeline (also used for the damage clear), and clips are cut on the CPU. Boxes are split per shape — no radius and no border is one rectangle, a radius becomes three bands plus four corners — and the general shader's instance gains separate `area` and `shape`. Small solid boxes are not split. Rounding paint colours to 8 bits means an invisible change produces no damage, which is the same effect Chrome was getting. Correctness is checked against the general shader within one 8-bit step.
- Code match: mostly matches.
  - The solid pipeline, the splitting and its 1-step comparison test (`boxes_in_parts_look_the_same_as_through_the_quad_shader`), and the damage clear on the same pipeline are in [A4b](audit-capabilities-rendering.md#wgpu-renderer), whose matched list includes `11`'s claims. Confirmed there.
  - The one claim `11` makes that `05` contradicts — that layer compositing still runs on the glyph pipeline — is [A4b](audit-capabilities-rendering.md#code-against-the-design-documents)'s row: `vs_layer`/`fs_layer` is its own pipeline (`renderer.rs:875`). Confirmed there.
  - The measurement tables (AMD 880M, RTX 5070) are upstream's, on upstream's machines; I did not reproduce them. Confirmed as documented; Unverified as measurements.
- Limits and open items:
  - "残る課題: 角丸のクリップがかかり、クリップの角に届く箱は分割しない" — still true ([A4b](audit-capabilities-rendering.md#wgpu-renderer)). Confirmed.
  - "残る課題: スクロールレイヤーの合成は … 専用の経路にする余地がある" — done since: `vs_layer`/`fs_layer` exists (the row above). Confirmed (stale).
  - "残る課題: 影の 9 パッチ化はしていない" — still absent; the hole/rim drawing is what is there. Confirmed via [A4b](audit-capabilities-rendering.md#code-against-the-design-documents).

#### `12-transform.md`: transforms and compositing layers

- Subject: `translate`, `scale`, `rotate` and `transform_origin` as paint-time transforms, and the layer machinery that makes them cheap.
- Summary: moving a box with `left` costs a layout pass and rounds to whole pixels; a transform does neither. The chosen mechanism is a compositing layer per transformed node rather than a matrix per primitive, because a frame that only changes the transform then redraws one quad, glyphs can be rasterised at the scaled size instead of at every animation size, and grouping opacity and shader effects land in the compositing step for free. Identity transforms are not layers. Integer translations composite nearest-neighbour, fractional and scaled and rotated ones bilinear, and a transform that stops moving is snapped back to integers on one extra frame so text is not left blurry. The layer's scale `k` is the smallest integer at least the enlargement, at most 4, at least 2 when rotated, and never lowered while the layer lives. In core, a transformed node is a coordinate space: its descendants' `abs` is unaffected, outside clips live in `layer_clip`, and window coordinates come from `Document::window_rect` / `to_window` / `from_window`. Opacity is grouped the CSS way only when a node paints two or more things.
- Code match: mostly matches.
  - The space machinery, `step_out`, `layer_matrix`, `hit` inverting the matrix, the paint-time-only property, and the 20 tests in `tests/transform.rs`: [A4a](audit-capabilities-core.md#coordinate-transforms). Confirmed there.
  - `k ≥ 1`, `k ≤ 4`, `k ≥ 2` when rotated, and "never lower while the layer lives": [A4b](audit-capabilities-rendering.md#code-against-the-design-documents), which also covers the torus texture with a `Repeat` sampler and the dedicated `vs_layer` / `fs_layer` pipeline. Confirmed there.
  - `ComputedStyle::move_x` / `move_y` for FLIP, separate from `translate`: `13-animation.md`'s implementation section says so and [A4b](audit-capabilities-rendering.md#animation) confirms the presence mechanism; I did not open the style fields. Unverified.
- Limits and open items:
  - "やっていないこと: 3D の変形、`skew`、遠近法" — absent from `Affine`'s API ([A4a](audit-capabilities-core.md#coordinate-transforms)). Confirmed.
  - "変形したノードが多いときの直接描画" — absent; every transformed node is a layer. Confirmed ([A4b](audit-capabilities-rendering.md#compositing-layers-and-texture-caches)).
  - "テクスチャの使い回し" and "ミップマップ（大きな縮小）" — both absent; the document records that shrinking past 1/2 makes thin lines flicker. Confirmed by the same section.
  - "変形の中のスクロール要素を別レイヤーにすること" — absent; the document's rule is that such a scroller is drawn as content, and `stacking_role` (`layout.rs:750`) makes it a stacking context, not a second layer. Inferred (from those two facts; I did not trace a transformed scroller through paint).
  - Cost: one layer per display-sized texture (about 2 MB for a 400×260 logical card at 1.8, four times that while rotating). Still true (Inferred from `k` and the torus texture).

#### `13-animation.md`: transitions, `animated`, and enter/exit

- Subject: the two animation APIs and the presence mechanism, plus the reactive-engine work that made thousands of simultaneous effects affordable.
- Summary: `transition` decides the intermediate values when a style changes from any layer (`hover`, `focused`, `active`, `dyn_style`) and takes the `transition` written on the *new* style, as CSS does. `animated` is a signal that moves itself, with `to` for a timed move and `spring_to` for a spring given by stiffness and damping; a spring is integrated in fixed 1 ms steps from the delta to the target, so the frame rate does not change the path. Both are resolved inside normal style resolution, so layout invalidation, inherited colour, damage, and node removal all work unchanged, and a `dyn_style` reading them is called at most once per frame because the frame callbacks run in one `batch`. `enter`/`exit` keep a leaving node alive for its exit style and restore the same node if `when` comes back; `for_each` does the same per key and adds `moves` (FLIP) so the surviving rows slide instead of snapping. The reactive engine was changed twice to make this cheap: the flush queue became a depth-ordered heap, and dependencies are only re-linked when the read order changes.
- Code match: matches. [A4b](audit-capabilities-rendering.md#animation) reads all three mechanisms, the frame loop, and the costs, and records that `Prop` has 27 variants where the document's table lists 17 (`ImageOpacity`, `Blur`, `BackdropBlur`, `FontWeight`, `FontAxes`, `SvgFill`, `SvgStroke`, `StrokeWidth`, `FillOpacity`, `StrokeOpacity` are missing from the table) — doc ageing, not a contradiction. Confirmed there. Added here: `Spring` is `{ stiffness: 170.0, damping: 26.0, mass: 1.0 }` with `Spring::from_ratio`, and `animated` is `Copy` with `get`/`set`/`to`/`spring_to`/`stop` — [A4b](audit-capabilities-rendering.md#animation) names `animated` (`animation.rs:451`), `to` (`:474`), `spring_to` (`:490`), `Spring` (`:357`), `follow` (`:576`). Confirmed there.
- Limits and open items:
  - "やっていないこと: `Auto` の高さからの動き（CSS の `interpolate-size`）" — absent; the document says `Auto` does not interpolate. Confirmed ([A4b](audit-capabilities-rendering.md#animation)).
  - "キーフレーム、ループ、遅延" — absent as animation features; `set_timeout` is a plain timer. Confirmed.
  - "`transition` での `Spring`" — absent; `Transition` takes a timing only. Confirmed.
  - "`prefers-reduced-motion` に当たる設定" — absent from `Settings` (`src/anti-webui-core/src/settings.rs:10-19`). Confirmed.
  - The pause cap of 4 s and "the start time is `Instant::now()` when the node passes `resolve_node`", which the document states as a design point, are the clock seam [A4a](audit-capabilities-core.md#risks-and-technical-debt) records as a risk. Confirmed there.

#### `14-shaders.md`: custom shaders for `fill` and `overlay`

- Subject: WGSL shaders that paint inside a box, the two stages they can take, the values and helpers they get, and how they are compiled and cached.
- Summary: a `Shader<P>` is a `static` holding a name and WGSL source; `#[derive(ShaderParams)]` generates the matching `struct Params` and the byte layout, so WGSL's alignment rules never have to be handled by hand. There are two paint stages: `.fill(f)` goes above the outer shadow and below `bg`, `.overlay(f)` above the background, border and inner shadows and below children — used for gradient fills, progress bars, state layers and ripples. Both are clipped to the border box's rounded shape by the framework, so a shader need not know the shape. `f` may return `Option`, and `None` costs nothing, so an unpressed button is free. The shader gets `aw_box` (size, border, padding, radii, scale) and `Frag` (physical position, integer position, and how many pixels one output pixel spans when a layer is drawn at `k > 1`), which is what lets it anti-alias correctly inside a scaled layer. Shaders compile on first use, or ahead of time with `preload()` across the cores.
- Code match: mostly matches.
  - `Shader<P>` with `params` and `preload` (`src/anti-webui-core/src/shader.rs:189`, `:212`, `:225`), `preloads_from` (`:237`), `ShaderPaint` (`:244`), `Primitive::ShaderDraw` (`scene.rs:276`), and `check_shader` for GPU-free tests (`src/anti-webui-wgpu/src/custom_shader.rs:119`). Confirmed.
  - The helpers the document promises are all in the prelude: `aw_rounded_box_distance` (`custom_prelude.wgsl:77`), `aw_shape_distance` (`:83`), `aw_coverage` (`:89`), `aw_padding_rect` (`:94`), `aw_content_rect` (`:100`). Confirmed.
  - Multi-core `preload` compilation and the DX12 pipeline cache on disk: [A4b](audit-capabilities-rendering.md#code-against-the-design-documents), which also records that the document contradicts itself about the disk cache (`14`'s "やっていないこと" says it is not done, `14`'s "実行時の扱い" says DX12 keeps it, and the code matches the second). Confirmed there.
  - Only `fill` and `overlay` exist as stages; `16-filters.md` owns the ones that read their input or the backdrop. Confirmed by `Node::fill` / `Node::overlay` (`tree.rs:211`, `:213`).
- Limits and open items:
  - "やっていないこと: カスタム影" — absent; the document says it would be `Primitive::Shader` plus `cut` and `extent`. Confirmed.
  - "前フレームの結果を入力にすること（`Image::previous()`）" — absent. Confirmed.
  - "パイプラインキャッシュのディスク保存" — done for DX12, contradicting this bullet (see above). Confirmed.
  - "コンピュートシェーダー、WebGPU を直接触る `canvas`" — `gpu_canvas` exists as `41-gpu-canvas.md` and `NodeKind::Canvas` (`tree.rs:194`); whether it is WebGPU-direct I did not check. Unverified.

#### `15-md3e.md`: the Material 3 Expressive components

- Subject: a separate crate that renders MD3E looks on top of the public `anti-webui` API, and what that crate needed from core.
- Summary: the crate is deliberately downstream — `tests/public_api.rs` fails if it reaches past `anti_webui::prelude`, `::widgets`, and `check_shader`; behaviour is borrowed from the standard widgets by passing only a look (`switch_with` and its siblings); a component returns a plain `Div`; and the theme stays inside the crate as a signal rather than moving into core. Tokens come from Jetpack Compose's sources, file by file, and motion is Expressive's six spring pairs. Buttons are five sizes with the documented heights, paddings, radii and pressed radii, state layers 8/10/10/16%, ripples capped at three, and the pressed radius change uses the "standard effects" spring because Compose says a button must not bounce. `MaterialShape` is the 34-shape catalogue without `Puffy`, stored as how far the outline is from an inner point in 256 directions, so any shape can morph into any other; `Puffy` is excluded because its lobes make every direction cross the outline more than once. Colour schemes come from a seed or a wallpaper's pixels, porting material-color-utilities' 2021 roles, CAM16, HCT, quantisation and `Score`.
- Code match: mostly matches.
  - The components the document shows exist: `button`, `tonal_button`, `elevated_button`, `outlined_button`, `text_button`, `styled_button` (`src/anti-webui-md3e/src/components/button.rs:322-343`), `wavy_progress` and `wavy_progress_eased` (`progress.rs:86`, `:92`), `loading_indicator` (`loading.rs:59`), `shape` and `styled_shape` (`shape.rs:110`, `:135`). Confirmed.
  - Button size tokens: `ExtraSmall => (32.0, 16.0, 12.0, 8.0, …)` and `Small => (40.0, 16.0, 12.0, 8.0, …)` (`button.rs:48-63`), matching the document's table. Confirmed.
  - `MaterialShape::ALL: [MaterialShape; 34]` (`src/anti-webui-md3e/src/shapes.rs:60`) with the doc comment "All of the catalog is here but `Puffy`" (`:17`). Confirmed.
  - `ColorScheme::from_seed` (`src/anti-webui-md3e/src/color.rs:128`), `from_seed_with` (`:151`), `set_color_scheme` (`:299`), `set_color_scheme_animated` (`:317`), `seed_colors` (`src/anti-webui-md3e/src/dynamic/mod.rs:55`), and the licence header naming commit `194144d` (`dynamic/mod.rs:4-10`). Confirmed.
  - `tests/public_api.rs` exists, and `bundled-fonts` is the default feature with `fonts/OFL.txt` beside `RobotoFlex.ttf` (`src/anti-webui-md3e/Cargo.toml:10`, `:12`, `fonts/`). Confirmed.
  - "**公開 API だけで書く。** 依存は `anti-webui` だけにする": the manifest also depends on `anti-webui-symbols-outlined` (`src/anti-webui-md3e/Cargo.toml:16`), added with `18-icons.md`. Confirmed (the policy's letter is stale; its intent — no internal API — may hold, which `public_api.rs` would show).
- Limits and open items:
  - "やっていないこと: Material You の 2025 年版以降の配色の仕様と、fixed 系の役割" — absent; the port stops at the 2021 roles the document lists. Confirmed.
  - "子を形で切り抜くこと" — absent; children paint over the shape and are not clipped by it (`shape` docs). Confirmed.
  - "`Puffy`" — still missing from `MaterialShape::ALL` (Confirmed above).
  - "**未計測:** 内蔵 GPU をコンポジタの出力にした状態と 60 Hz。実ウィンドウでのリップル" — still unmeasured. Confirmed as documented; Unverified as measurement.
  - "動かし続けたときの無名メモリ … 原因は未調査" — still unexplained. Unverified.
  - "Compose の実物との画面比較はしていない" (both the shapes and the gallery) — still not done. Confirmed as documented.

#### `16-filters.md`: Filters, blur, and backdrop

- Subject: A filter stage after `14`: built-in `blur`/`backdrop_blur` and a user "image expression" for `filter` and `fill`.
- Summary: The API is an expression tree (`Image::content()`, `Image::backdrop()`, `.blur(r)`, `.through(&SHADER, p)`, `.inputs([…4])`, `.reach(r)`), not hand-built passes and textures, because partial redraw needs to know how far a pixel depends. Blur is Dual Kawase with a fixed one-texel offset and levels mixed by variance; σ ≤ 1.68 goes to a two-pass Gaussian, σ above 249 is treated as 249. A scene that reads a backdrop draws into a window-sized canvas and only damaged rectangles reach the surface, since a direct copy would need `COPY_SRC` on each modifier. A filter layer keeps its output until its content or expression changes, and damage is widened by `reach`. It rejects the manual API, Chromium's mirrored edge, KWin's offset-based radius, and the direct-copy target.
- Code match: matches.
  - The sigmas and the mix: `[1.68 … 248.8]` (`src/anti-webui-core/src/shader.rs:298`), zero levels at σ ≤ 1.68, the variance formula, the 249 clamp (`:304`–`317`) — Confirmed.
  - `content()` outside a `filter` and `backdrop()` outside a `fill` have no source and "read as transparent" (`src/anti-webui-wgpu/src/effects.rs:603`, `:632`, `:477`–`478` — Confirmed).
  - The Kawase chain, the `reach` expansion, and the backdrop canvas are [A4b](audit-capabilities-rendering.md#shaders-and-filters).
- Limits and open items:
  - `backdrop()` inside `filter`: no source there, so it draws nothing (`effects.rs:632` — Confirmed).
  - Recomputing only the changed part of a `filter`: still the whole visible range ([A4b](audit-capabilities-rendering.md#shaders-and-filters) — Confirmed absent).
  - Custom shadows / `.drop_shadow()`: absent (`rg drop_shadow src/` finds nothing — Confirmed).
  - `Image::previous()`: absent (Confirmed).
  - `ext-background-effect-v1`: absent (Confirmed).

#### `17-font-axes.md`: Variable font axes

- Subject: Variable-font axes as style properties: inheritance, transitions, `opsz`, and the glyph-atlas consequences.
- Summary: Each axis is its own value that inherits and transitions per axis, unlike CSS's single `font-variation-settings` list. A missing side of an interpolation is "the value when the axis is not set", so an axis is held as `value`/`amount` and the font default is looked up only while `amount < 1`. `wght` defaults to `font_weight`, `opsz` to the font size and follows it unless written, anything else to the font's default. Normalised coordinates are rounded to 1/64 in paint so a still scene stays identical, and an all-zero set is dropped. A full atlas clears instead of growing when it is at least 2048 px and most of its area comes and goes. It rejects CSS's whole-list replacement, an API to unset an axis, and unrounded values.
- Code match: mostly matches.
  - A four-byte tag makes a wrong length a compile error (`FontAxis(pub [u8; 4])`, `src/anti-webui-style/src/types.rs:458`), and the builder exists (`styled.rs:55` — Confirmed).
  - Defaults and interpolation: `wght` → `font_weight`, `opsz` → `optical_size`, others → `axis_default`, `opsz` added when unset (`src/anti-webui-core/src/text/mod.rs:236`–`262` — Confirmed).
  - The 1/64 rounding and the empty all-zero set are `rounded_coords` (`paint.rs:772`–`778` — Confirmed); shaping and clamping are [A4a](audit-capabilities-core.md#text-shaping), the atlas rule [A4b](audit-capabilities-rendering.md#glyph-atlas-and-subpixel-positioning).
- Limits and open items:
  - OpenType features (`ss01`): no font-feature API (`rg 'ss01|OpenType'` finds nothing — Confirmed absent).
  - Querying a font's axes: absent (Confirmed).
  - Axes inside one paragraph: **now done** — a `span` merges its `font_axes` into shaping and `Span: Styled` (`src/anti-webui-core/src/text/rich.rs:107`, `:408`–`410` — Confirmed). Stale.
  - MD3E's emphasised typography: belongs to `15-md3e.md` (Unverified).

#### `18-icons.md`: Icons

- Subject: Material Symbols shipped as one one-glyph font per icon, with the API, the box, and registration.
- Summary: `anti-webui-symbols-outlined` holds one glyph font per icon and a `static Symbol` per name; `icon`/`sized_icon` return an ordinary `Div`, so `font_axis`, `text_color`, and `on_click` work on them. Unused `static`s are dropped by the linker, so there is no name table and no extraction step. The box fixes width and height to the size, does not shrink, uses a line height of one, centre alignment, weight 400, no letter spacing or italic, and is not selectable; colour and axes inherit. Registration is once per icon, and it skips a full layout when the family was never asked for or known. It rejects embedding the whole family, app-side extraction, ligature-by-name lookup, and string lookup.
- Code match: mostly matches.
  - 4004 per-icon fonts; `Symbol` fields are public (`lib.rs:9`–`16`); constants refer to statics (`generated.rs:2809`–`2810`); the alias is `pub use self::GRADE as STAR;` (`:4627`) — Confirmed.
  - `icon` is 24 px and `sized_icon` sets exactly that box (`src/anti-webui-md3e/src/components/icon.rs:32`–`57`); one registration per font pointer per thread (`:13`–`17`), skipped when the family is unseen (`src/anti-webui-core/src/text/mod.rs:139`–`142`, `:377`–`391`) — Confirmed.
  - The named files hold 14, 3, and 11 tests (`md3e/tests/icons.rs`, `symbols-outlined/tests/fonts.rs`, `core/tests/fonts.rs` — Confirmed).
- Limits and open items:
  - Button icons: **lifted for MD3E** (`ButtonStyle::icon`/`trailing_icon`, `src/anti-webui-md3e/src/components/button.rs:264`, `:270`; used at `tests/buttons.rs:154` — Confirmed); the standard widgets' `button()` still has none (Confirmed absent).
  - Rounded, Sharp, and lookup by string: absent (Confirmed).
  - Right-to-left mirroring: the box sets no direction (`icon.rs:37`–`57` — Inferred).
  - Accessibility label: no role or label (`icon.rs:37`–`57` — Confirmed absent).

#### `19-buttons.md`: Button parts

- Subject: The MD3E button family added after `15`: icons, toggles, icon buttons, FABs, and button groups.
- Summary: Values come from Jetpack Compose's Material 3, with motion added where Compose has none. State layers, ripple, focus, disabled, and shadow come from one `surface.rs`, and every part returns a plain `Div`. Corners animate per corner so only a connected group's inner corners change; a bordered part keeps its border width when selected and only makes the colour transparent. Toggling changes shape with the fast spatial spring (800, 0.6) while colours switch at once, and the icon's `FILL` animates. A group widens the pressed button by 15% of its width (at most 24 px) and takes the same from neighbours in whole physical pixels. It rejects Compose's immediate shape and colour changes and Compose's group-managed child widths.
- Code match: matches.
  - Every named function exists (`button.rs:322`–`354`, `icon_button.rs:144`–`167`, `fab.rs:126`–`144`, `button_group.rs:46`, `:121` — Confirmed).
  - Icon sizes and gaps are 20/4, 20/8, 24/8, 32/12, 40/16 (`src/anti-webui-md3e/src/components/button.rs:52`–`100`); the icon-button table matches all five rows (`icon_button.rs:44`–`52`); the FAB table matches, icon 36 at Large and Level 3/4 (`fab.rs:55`–`75`, `:112`) — Confirmed.
  - Group constants 15% and 24 px (`button_group.rs:12`, `:14`); motion tokens (800, 0.6), (3800, 1.0), (1600, 1.0) (`src/anti-webui-md3e/src/motion.rs:7`, `:19`, `:23`); `tests/buttons.rs` has 45 tests — Confirmed. The performance table is upstream's (Unverified).
- Limits and open items:
  - A group overflow menu: no overflow menu in the group code (`rg overflow button_group.rs` finds only padding) (Confirmed absent).
  - Equal-width groups (Compose's `weight`): no `weight` in the group code (Confirmed absent).
  - The FAB menu and split button, deferred to `20`: both exist (`components/mod.rs:62`, `:87` — Confirmed).

#### `20-md3e-components.md`: The rest of the MD3E parts, and the showcase

- Subject: The remaining Material 3 Expressive parts, the core additions they needed, `Ink`, density, and the showcase.
- Summary: This is the catalogue — selection, notification, container, navigation, shape, and input parts — with Compose's tokens where they exist and the same motion tokens where Compose does not animate. Core gained `portal`/`Popup::modal`, `set_timeout`, `on_scrolled`/`on_resized`, `AboveCenter`/`BelowCenter`, multiple `node_ref`s, and `pointer_events_auto`, so a part can be declared anywhere and still show over the window without a surface. `Ink` separates the box that takes input from the box that paints, which is what lets a switch hover on its root and paint a 40 px circle. `Density::Compact` is read at build time per thread, because a closure-scoped setting would be lost when `dyn_view` rebuilds a part. It rejects duplicated state layers, a second `overlay` on one box, range-scoped density, and Compose's behaviour wherever a section has a 「Compose との違い」.
- Code match: mostly matches.
  - Every name in 「部品の一覧」 is exported (`src/anti-webui-md3e/src/components/mod.rs:37`–`94` — Confirmed, name by name).
  - Core additions exist: `portal` (`src/anti-webui-core/src/view.rs:2035`), `Popup::modal` (`:2063`), `on_scrolled`/`on_resized` (`:821`, `:834`), `AboveCenter`/`BelowCenter` (`platform.rs:82`, `:86`), `set_timeout`/`Timeout` (`animation.rs:82`, `:98`), `scroll_offset`/`scroll_to` (`view.rs:158`, `:173`) — Confirmed.
  - `Ink` has `new`, `listen`, `listen_centered`, `paint` (`src/anti-webui-md3e/src/components/surface.rs:242`–`379`); density is thread-local and read at build time (`density.rs:8`–`33`) — Confirmed.
  - Motion tokens as in `19` (`motion.rs:7`, `:19`, `:23`, `:27`); progress spans 240 (linear) and 40/48 (circular), with a 20 px indeterminate wavelength against 40 px determinate, and a (50, 1.0) value spring (`progress.rs:12`, `:16`, `:147`, `:148`, `:151`, `:154`); switch and text-field density numbers match (`switch.rs:28`–`50`, `text_field.rs:36`–`53`) — Confirmed.
  - The showcase has seven pages, one more than the table lists (`src/anti-webui-md3e/examples/showcase/main.rs:18`–`27` — Confirmed); its performance table is upstream's (Unverified).
- Limits and open items:
  - `flex-wrap`: **stale** — the engine has it, `07-roadmap.md:37` marks it done in M1 ([A4a](audit-capabilities-core.md#layout-engine)); chips and button rows just do not use it (Confirmed).
  - Colour schemes from a seed beyond 2021, and the fixed roles: belongs to `15` (Unverified).
  - Field-specific selection and IME looks: fields still use core's (`text_field.rs` adds none — Inferred).
  - The per-section 「Compose との違い」: comparing needs a Compose build (Unverified).

#### `21-widgets.md`: The standard parts

- Subject: `anti-webui-widgets`: quiet default parts built only on core's public API, with `Sheet`, `Theme`, and virtualised long lists.
- Summary: The look is quiet — a Primer-derived palette and ~120 ms transitions, no springs or shaders — and every part is a `Div`, `TextInput`, or `Show`. Styles layer as inherited values, then the part's `Sheet` (base, hover, focus, focus-visible, active, disabled, accent), then the element's own style, then dynamic and state styles, so a part's look never beats the caller's `.bg()`. `virtual_list`, `tree_view`, and `table` build only visible rows, positioned with `Length::Device` so rounding does not accumulate. It rejects CSS selectors and classes, MD3E's motion, and an icon crate in the default parts (symbols are font characters).
- Code match: mostly matches.
  - Every part in 「部品」 is exported (`src/anti-webui-widgets/src/*.rs`; `lib.rs:25`–`45` — Confirmed, name by name).
  - `virtual_list(count, row_height, row)` has that shape, `SPARE = 8` rows each side, `GROUP = 4` rounding, `Length::Device` placement (`src/anti-webui-widgets/src/long.rs:13`–`21`, `:81`, `:91`); `window_scale()` (`view.rs:463`), `Sheet::focus_visible`/`disabled` (`sheet.rs:12`, `:48`, `:60`), `accent_color`/`caret_color` (`styled.rs:293`, `:298`) — Confirmed.
  - The example now has ten pages, adding `images` and `dragging` (`src/anti-webui/examples/widgets/main.rs:15`–`26` — Confirmed); the benchmark table is upstream's (Unverified).
- Limits and open items:
  - Text-type-ahead in `select`/`menu`/`list_box`, a menubar, `table` multi-select and cell editing, `tree_view` lazy loading: all absent (Confirmed).
  - Non-fixed-height `virtual_list`: still fixed (`long.rs:13`–`21` — Confirmed).
  - A magnifier icon in `search_field`: still a font character (Confirmed).
  - `:focus-visible`/`:disabled` outside a `Sheet`: still sheet-only (`sheet.rs:12`, `:60` — Confirmed).
  - A theme transition for the whole `page`: not done (Unverified).
  - Real pointer and keyboard use, and symbol glyphs on a font-poor machine: still unverified (Unverified).

#### `22-images.md`: Images

- Subject: `img` and `background-image` as replaced elements, with the decode pipeline, caches, memory rules, and the renderer's primitive.
- Summary: The element is `img`, and the API mirrors HTML and CSS (`object_fit`, `object_position`, `aspect_ratio`, `image_rendering`, `density`, `bg_*`, `on_load`/`on_decoded`/`on_error`, plus a non-CSS `image_opacity`). A worker reads the header, layout asks for the size it needs, and only then is the image decoded, oriented, converted to sRGB, premultiplied, and shrunk to the smallest 1/2^n not below that size. Memory follows the screen: one bitmap per image, a 256 MB cache, a 64 MB per-frame upload cap, one decode at a time above 32 MB, and glibc `mallopt`/`malloc_trim`. Animated formats hold every frame up to 256 MiB and advance by timer, stopping off screen. It rejects DCT-stage scaling (the decoder cannot), native codecs (AVIF), and network loading.
- Code match: matches.
  - Cache 256 MiB and upload cap 64 MiB per frame (`src/anti-webui-core/src/settings.rs:105`, `images/mod.rs:20` — Confirmed).
  - `ImageSource` is `Path | Static | Bytes | Pixels` with `rgba`, `Bitmap::with_levels` exists (`src/anti-webui-style/src/types.rs:117`–`132`; `images/mod.rs:163`, `:192`); `.density`, `.on_decoded`, `Prop::ImageOpacity` (`view.rs:1032`, `:1044`; `transition.rs:112`); `releasable` gates pixel release (`images/mod.rs:57`, `:88`, `:107`) — Confirmed.
  - `HEAVY_BYTES = 32 << 20` (`images/decode.rs:18`); `M_MMAP_THRESHOLD`/`M_TRIM_THRESHOLD` and `malloc_trim(0)` (`images/worker.rs:169`, `:187`–`188`); the renderer's half is [A4b](audit-capabilities-rendering.md#images-and-shapes-on-the-gpu-path) — Confirmed. Numbers are upstream's (Unverified).
- Limits and open items:
  - Hand checks (scroll, resize, pointer): still synthetic only (Unverified).
  - `srcset` selection and `<picture>`: absent; only a `srcset` comment in `view.rs` (Confirmed).
  - `mask-image` and the other background fields: do not exist (`mask_image` absent — Confirmed).
  - `ImageSource` as a `filter` input: absent (Confirmed).
  - AVIF/HEIC/JPEG XL, network, disk cache: absent (Confirmed).
  - Incremental animation decode and DCT-stage JPEG scaling: still absent (Confirmed).

#### `23-drag-and-drop.md`: Grabbing, and drag and drop

- Subject: Three layers of pointing — `on_grab`, in-application typed drag and drop, OS drag and drop — plus text dragging and sorting.
- Summary: A press becomes a `GrabEvent` stream after 4 logical pixels; a grabbed element suppresses selection and the click, and Escape or lost focus sends `Cancel`. Layer two passes Rust types and picks the receiver by walking up from the pointer, so there is no `dragenter`/`dragleave` flapping and no `preventDefault`; the state lives in `Ui`, so a virtualised source row may vanish. Layer three is the same API plus outward data, and the preview is a `Primitive::Layer` whose `origin` changes, so it never relays out. Selected text drags without extra API, with Blink's thresholds, password rule, and `Move`/`Copy` policy. Sorting moves rows with `translate` and reorders once on release. It rejects string payloads for in-app drags, HTML's enter/leave model, deleting text on an OS `Move`, and `text/html`.
- Code match: mostly matches.
  - `on_grab`, `grab_threshold`, `draggable`, `drag_preview`, `drop_target`, `on_drop_text` (`src/anti-webui-core/src/view.rs:699`–`799`), `start_drag` (`src/anti-webui-core/src/drag.rs:657`) — Confirmed.
  - Gesture, preview, and text-drag state machines: [A4a](audit-capabilities-core.md#state-machines); per-platform drags and policy: [A4c](audit-capabilities-platform.md#drag-and-drop).
  - The 48/41 test counts are 53/42 in the code; that drift is already in [A4c](audit-capabilities-platform.md#code-against-the-design-documents) (Confirmed). `sortable_list`/`sortable_row` exist in both crates (Confirmed); the benchmarks are upstream's (Unverified).
- Limits and open items:
  - Tab tearing (`xdg-toplevel-drag`): absent (Confirmed).
  - Touch (`wl_touch`): no touch events ([A4c](audit-capabilities-platform.md#the-platform-abstraction) — Confirmed).
  - Horizontal autoscroll at a single-line field's edge: absent (Unverified).
  - An animation returning a cancelled preview: absent (Confirmed).
  - `Sheet`'s `drag_over`: absent (`sheet.rs` has none — Confirmed).
  - Building large payloads on request: `DragData` still holds everything up front (Confirmed).
  - Real-pointer tests and dragging text to another app by hand: still unverified (Unverified).

#### `24-video.md`: Video

- Subject: A `<video>` on FFmpeg loaded at run time: decoding, presentation cadence, audio, and frame-accurate reading.
- Summary: The program ships no FFmpeg code: the ABI is declared by hand, `dlopen`ed on a video thread, and only majors in a table are accepted, because a wrong offset crashes instead of failing. One demuxer thread feeds a video and an audio thread; the audio position is the clock. A whole-number frame ratio is pinned to whole refreshes, otherwise frames are chosen by time, and frames are made halfway between two refreshes so the GPU's completion lands inside the compositor's window. Hardware decode maps VA-API images to dmabuf with an explicit foreign-queue barrier; sub-surfaces are deferred because their placement takes integer logical pixels. `VideoReader`/`AudioReader` give frame- and sample-accurate reads with no clock and no threads. It rejects bundling or linking FFmpeg, sub-surfaces for now, mips, and HLS/DASH/DRM.
- Code match: matches.
  - The ABI table has one row (FFmpeg 9: avcodec 63, avformat 63, avutil 61, swresample 7, swscale 10) (`src/anti-webui-video/src/ffi.rs:110`–`118`); `LgplOnly` accepts only a licence starting with `LGPL` (`ffi.rs:343`; enum at `src/anti-webui-core/src/video.rs:49`) — Confirmed.
  - `MAX_FRAMES = 4`, `EXTRA_HW_FRAMES` (`player.rs:21`, `:23`), `VideoReader` (`reader.rs:46`), `AudioReader` (`sound.rs:38`), the pure cadence module (`cadence.rs`) — Confirmed.
  - The dmabuf path is `av_hwframe_map` plus a `VK_QUEUE_FAMILY_FOREIGN_EXT` barrier (`player.rs:1192`; `src/anti-webui-wgpu/src/dmabuf.rs:698`); no video code mentions sub-surfaces (`rg subsurface` finds nothing) — Confirmed.
  - `video` is off by default (`src/anti-webui/Cargo.toml:10`); the measurements are upstream's (Unverified).
- Limits and open items:
  - The LGPL build script and a real LGPL `libavcodec`: still unverified (Unverified).
  - FFmpeg 8 and older: still `Unavailable`; one row only (Confirmed).
  - Real videos, listening to audio, 60 Hz, HDR by eye: still unverified (Unverified).
  - NVIDIA: CPU decode; Vulkan Video is the noted restart point (Confirmed absent).
  - Windows/macOS run time and GPU decode: type-checked only (Unverified).

#### `25-svg.md`: Path drawing and inline SVG

- Subject: `Primitive::Path` and `svg` as ordinary nodes, with the sparse-band coverage scheme and the `usvg` file path.
- Summary: Shapes are nodes, so signals, `for_each`, `dyn_view`, `hover`, transitions, events, and cursors work on them, and `svg::parse` produces the same nodes. Coverage is computed on the CPU into sparse cells and bands (4 rows, 40 px blocks, 0.05 px flattening) and drawn one instance per rectangle, so a large fill costs almost nothing; per-draw data goes in an RGBA32F table. More than 48 new shapes in a frame are built on up to eight threads. `foreignObject` labels and simple `<text>` become `svg::text` before `usvg`, so system fonts are read only for harder `<text>`, and `STAND_INS` supplies metric stand-ins such as Arial. It rejects a whole-shape mask, triangulation with MSAA, and `vello_hybrid`.
- Code match: matches.
  - `svg::parse`, `measure_text`, `view_box`, `aspect`, `path_length` (`src/anti-webui-core/src/svg_file.rs:23`; `src/anti-webui-core/src/svg.rs:320`, `:429`, `:434`, `:584`) and the `foreignObject` handling (`svg_file.rs:548`, `:1140`, `:1182`) — Confirmed.
  - `STAND_INS` exists and is applied to family lists (`src/anti-webui-core/src/text/mod.rs:39`, `:115` — Confirmed).
  - Band constants, the 48-shape threshold, and the store are [A4b](audit-capabilities-rendering.md#images-and-shapes-on-the-gpu-path); the comparisons are upstream's (Unverified).
- Limits and open items:
  - Rotated or slanted `svg::text`: text does not rotate (Confirmed by `28` too).
  - A line on `svg::text`, `tspan` inside one text: absent (Confirmed).
  - `paint-order`, `<pattern>`, `<mask>`, `<filter>`, `<image>`, blending, non-rectangular `clip-path` as nodes: absent; `parse` falls back to a bitmap (the only `paint_order` in core is a clip's draw order, `paint.rs:1287` — Confirmed).
  - Rectangular `clip-path` in hit-testing and `svg::text`: still open (Unverified).
  - General `foreignObject` HTML, `width="100%"`/`max-width`: one style per label; `parse` keeps the `viewBox` size (Confirmed).
  - CSS `clip-path`/`mask-image` on ordinary nodes, `currentColor` in `img` SVG, app fonts there, parse caching, SMIL and CSS animation: absent (Confirmed).

#### `26-pdf.md`: PDF

- Subject: A fast PDF viewer on `hayro`: worker rendering, partial pictures and zoom, text, links, forms and annotations, presentation mode, and form JavaScript.
- Summary: The goal is to beat PDF.js on a UI thread that never blocks; the base is pure-Rust `hayro`, not PDFium or an offscreen browser. A page gets one whole picture and, past what it can hold, partial pictures with half a box of margin; requests are prioritised (text, whole pictures, partial pictures, look-ahead, search), two workers by default, stale requests dropped, up to eight when all are stuck. Zoom changes layout in the same frame and stretches the old picture; a pinch repaints only when it ends. Text comes from a custom `Device`, with an Encoding/CMap fallback for CID fonts that lack `ToUnicode`, and the viewer keeps its own selection and search instead of a DOM layer. Forms and annotations append one update to the original bytes and reopen, so one renderer draws both. It rejects PDFium, PDF.js's text layer, tiled pictures, forced thread termination, and a second drawing path.
- Code match: matches.
  - `hayro` is pinned exactly (`Cargo.toml:86`); `js` gates `rquickjs` (`src/anti-webui-pdf/Cargo.toml:22`) — Confirmed.
  - Defaults: two workers, 256 MiB of pictures (`src/anti-webui-pdf/src/lib.rs:925`–`926`); extra workers cap at eight (`MAX_STUCK_WORKERS = 8`, `engine.rs:26`, `:230`); panics are caught per job (`engine.rs:306`, `:426`, `:485`) — Confirmed.
  - The viewer is `select_none` and intercepts Ctrl+A/C with `prevent_default` (`lib.rs:1878`, `:1749`, `:1753`); `pdf_view` (`:913`), `pdf_viewer` (`:232`), `FitWidth` (`:103`), `present` (`:437`), `highlight_selection` (`:705`), `editing` (`:461`), `is_signed` (`:467`), `outline` (`:343`), `answer_dialog` (`:535`), `on_link`/`on_page_error`/`field_tint` (`:949`–`970`), `pdf_thumbnails` (`:46`) at 108 px (`thumbnails.rs:13`) — Confirmed.
  - The modules in 「構成」 all exist (Confirmed); `tests/script.rs` has the documented 14 tests (Confirmed; crate totals in [A6](audit-tests.md#test-inventory)).
  - Measurements and reader comparisons are upstream's (Unverified: those readers and samples are not here).
- Limits and open items:
  - A separate process for untrusted files, printing, two-page view, a context menu, slide transitions (`/Trans`), auto-advance (`/Dur`), a presenter screen: absent (Confirmed).
  - Page view beyond 16 million px, and streaming: still limited (Confirmed).
  - Text while typing, `/AP`-less annotations, `/Popup`, un-referenced widgets, FreeText `DA`, the selection box: as listed; no `/Popup` is written (`rg '"/Popup"'` finds nothing — Confirmed).
  - Right-to-left and vertical text, column order: still limited; `unicode-bidi` is unused (Confirmed).
  - AES, Acrobat, real IME and pointer input, transparent-heavy and CAD PDFs, pinch and text drag by hand, `hayro` 0.x stability: still unverified (Unverified).

#### `27-fullscreen.md`: Fullscreen

- Subject: Window fullscreen and element fullscreen in core, with the Wayland path and scanout preference.
- Summary: `set_window_fullscreen` covers F11; `NodeRef::request_fullscreen`/`exit_fullscreen` cover one element. Element fullscreen follows CSS: it fills the window whatever its own size, ignores ancestors' overflow, transform, and opacity, paints black behind itself, hides and unblocks the rest, puts popups above it, keeps event bubbling, exits on Escape unless prevented, and takes and returns the focus. Rather than `position: fixed`, the element becomes the root for layout, positioning, painting, and hit-testing, so the tree behind keeps its cached layout and scroll position. Switching waits for `WindowEvent::Fullscreen` with `Resized`, so a refused request stays pending. Output-level fullscreen, a `:fullscreen` pseudo-class, and a `::backdrop` colour are deferred.
- Code match: mostly matches.
  - The API is where the document puts it: `request_fullscreen`/`is_fullscreen` on `NodeRef` (`src/anti-webui-core/src/view.rs:87`, `:95`), `window_fullscreen`/`set_window_fullscreen`/`exit_fullscreen` (`:388`, `:394`, `:399` — Confirmed).
  - The element and window behaviour, all four backends, scanout modifiers, the 18 core tests, and the stale 「後回しにするもの」 line about missing X11/Windows/macOS backends: [A4c](audit-capabilities-platform.md#fullscreen).
- Limits and open items:
  - Fullscreen on a chosen output: absent (Confirmed).
  - A `:fullscreen` pseudo-class: absent; `dyn_style` with `is_fullscreen()` instead (Confirmed).
  - Setting the `::backdrop` colour: fixed black (Confirmed).
  - 「X11、Windows、macOS（バックエンド自体がまだ）」: stale — all three exist ([A4c](audit-capabilities-platform.md#fullscreen) — Confirmed).

#### `28-markdown.md`: Markdown

- Subject: `rich_text`, `markdown_view` with Mermaid, `text_editor`, and the live-preview `markdown_editor`.
- Summary: The parser and Mermaid live only in `anti-webui-markdown`; core gains `rich_text` (spans as ranges in one text node, never as nodes) and `text_editor` (one node per visible row, decorations that hide, replace, or style, and a row table of source byte positions). `markdown_view` turns blocks into ordinary nodes, with tables as columns (no grid), and far blocks in `content_visibility: auto` with `estimate_contents` (doc 29). The live preview hides markers only where the caret or selection touches them and returns blocks to source only when the selection is wholly inside. Mermaid goes through `merman` to SVG and `svg::parse` on one worker. It rejects a source-only editor mode and CSS grid for tables.
- Code match: mostly matches.
  - `rich_text`/`span`, `Span: Styled`, merged `font_axes` (`src/anti-webui-core/src/text/rich.rs:107`, `:408`); `text_editor` (`view.rs:2255`); `Row::Line`/`Row::Block` with `LineLook` marks, `raised`, `replace` (`text/editor.rs:54`–`84`) — Confirmed.
  - `markdown_view`/`markdown_editor` (`src/anti-webui-markdown/src/view.rs:78`, `:1104`); outermost blocks use `content_visibility_auto` with `estimate_contents` (`:1177`) — Confirmed.
  - `merman` pinned to `=0.10.0-alpha.1`, `mermaid`/`code` default (`Cargo.toml:16`, `:19`); GFM tables, tasks, strikethrough, footnotes (`parse.rs:107`–`112`) — Confirmed.
  - The 「確かめたこと（M3、M4）」 counts have drifted: core `tests/text_editor.rs` has 13 (not 9), markdown `tests/editor.rs` 11 (not 4) (Confirmed by count). Timings and memory are upstream's (Unverified).
- Limits and open items:
  - Table cells do not wrap, selection order across `inline` boxes, classic diagram theme only: still (Confirmed).
  - The 「`merman` より mermaid.js との違いが大きい」 line reads as a leftover from the `ariel-rs` period (Inferred from 「クレートの選択」).
  - A link opening when a drag starts on it: still (Unverified).
  - Estimated block heights shifting the scrollbar: still (Confirmed by `29`'s estimate).
  - Editor limits (bidi order, hanging indent, tables/diagrams missing from the drag image, reference links elsewhere, setext underlines, superscript spans): as listed (Confirmed).
  - The multi-byte crash is fixed: decoration ranges are floored to a character boundary (`text/editor.rs:337`, `:645`, `:826` — Confirmed).

#### `29-content-visibility.md`: content-visibility (deferring what is not seen)

- Subject: `content-visibility` for elements: the `auto` relevance rule, scroll anchoring, and `estimate_contents`.
- Summary: `hidden` always skips its content and `auto` skips it when nothing relates to it. A skipped box takes its remembered size when the width conditions match, else the estimate, else `contain_intrinsic_*`, else its padding and border. It relates when half a viewport of margin on each side meets the window, or when the focus, the selection, or a `scroll_into_view` target is inside; the decision is taken after positioning in the same update, so no frame shows an empty box, and only the `auto` list is scanned. Scroll anchoring keeps one anchor per scroller at the same place when a change of relevance would move what is shown. `estimate_contents` estimates text by per-style average character width, with CJK at 1 em and a 3-character word-wrap allowance, and caches the estimate until the content changes. It rejects the "everything above the viewport" correction tried first, and a measurement cache for estimates (it measured slower).
- Code match: matches.
  - `content_visibility_visible/auto/hidden`, `contain_intrinsic_size/width/height`, `estimate_contents` (`src/anti-webui-style/src/styled.rs:90`–`105`, `:256`–`265`); `check` is decided after positioning (`src/anti-webui-core/src/visibility.rs:32`), and a node to scroll to waits up to 8 updates (`WAITS`, `:11`) — Confirmed.
  - Skipped text stays selectable through `held` (`src/anti-webui-core/src/text/selection.rs:263`); core's `tests/content_visibility.rs` has 8 tests (Confirmed by count); `markdown_view` uses it for its outermost blocks (`src/anti-webui-markdown/src/view.rs:1177` — Confirmed).
  - The relevance and anchoring behaviour, the renderer seeing nothing, and the effect table are [A4b](audit-capabilities-rendering.md#content-visibility); the timings are upstream's (Unverified).
- Limits and open items:
  - Not creating the nodes of far blocks: still open ([A4b](audit-capabilities-rendering.md#content-visibility) — Confirmed absent).
  - Laying out in idle frames: absent (Confirmed).
  - Anchored estimates making the scrollbar drift: still (`28`, 「分かっている限界」 — Confirmed).

#### `30-performance-overlay.md`: Performance overlay

- Subject: a frame-time and render-ahead readout at the top right of a window.
- Summary: The overlay shows fps, CPU, GPU and latency, the `FrameAdvance` state and two 90-frame graphs, as primitives added to the scene rather than nodes, so it has no part in layout or hit testing and its numbers arrive a frame or two late, from the same `FrameRecord` as `ANTIWEBUI_FRAME_STATS`. It refuses to add frames: when the app stops it draws once more after 200 ms and leaves that frame out. Text is 11 logical pixels monospace, cut with `…`; `settings().debug.performance_overlay` turns it on and `ANTIWEBUI_PERF_OVERLAY` overrides the setting.
- Code match: mostly matches.
  - `PerformanceOverlay::paint` (`core/src/overlay.rs:141`) adds primitives under `ChunkKey { node: u64::MAX, part: 0 }` (`overlay.rs:19`–`22`), so the damage is its own box — Confirmed.
  - `SHOWN` 90, `AVERAGED` 30, `PAUSE_MS` 100 (`overlay.rs:13`, `:15`, `:17`); 11 px monospace with `…` (`:30`, `:216`, `:44`) — Confirmed.
  - `ANTIWEBUI_PERF_OVERLAY` beats the setting (`app/src/perf_overlay.rs:70`, `:86`); the last word is `LAST_WORD`, 200 ms (`:10`), asked for at `runner.rs:353`, and the overlay's own frame is dropped (`perf_overlay.rs:116`) — Confirmed.
  - Advance words `Auto: …`, `Enable`, `Disable`, `Custom n`, `(env)`, `, ahead` (`pacing.rs:372`–`393`; `perf_overlay.rs:169`) — Confirmed.
  - The `bench_pacing` cost figure (0.53–0.55 → 0.65–0.73 ms) — Unverified here.
- Limits and open items, from "分かっている限界":
  - 「Wayland のランナーだけ。ヘッドレスとほかのプラットフォームには、プラットフォームができたときに足す。」 — no longer true: the overlay is in the shared runner with no cfg gate (`runner.rs:295`) — Confirmed (read, not run).
  - 「先行の数（`Custom` で 2 枚以上）は出さず、先行したかどうかだけを出す。」 — no longer true: `Custom { max_advance }` prints its number (`pacing.rs:388`) — Confirmed.

#### `31-code-editor.md`: A code editor (syntax highlighting)

- Subject: a tree-sitter-highlighted code editor built on `text_editor`.
- Summary: The document measures the old editor (a node per line: 4.2 GB and 84 ms per key on sqlite3.c) and rebuilds it with nodes only for the rows around the view, one height-only gap box per run of skipped lines, estimated heights after CodeMirror 6's `HeightOracle`, `decorate_lines` for the built rows only, `on_edit` positions before decoration, and line numbers that ignore horizontal scroll. `anti-webui-code` adds tree-sitter highlighting: grammars compiled on a worker, at most 1 ms of parsing on the UI thread, and injections for embedded languages. It rejects `tree-sitter-highlight` and tree-house, and leaves out multiple carets, a minimap and LSP.
- Code match: mostly matches.
  - Rows are rebuilt when the window covers less than the view plus half a view each way (`text/editor.rs:2304`), and one view each way is built (`:2313`) — Confirmed.
  - `decorate_lines`, `blocks`, `on_edit`, `gutter` (`view.rs:2315`–`2352`); `Gutter` (`text/editor.rs:185`), `DecorateLines` (`:159`) — Confirmed.
  - 10,000-byte lines and the 20 MB highlight limit (`code/editor.rs:19`, `:21`); folding by indentation (`code/edits.rs:254`) with `⋯` (`editor.rs:276`); 100,000 matches (`search.rs:6`); every language a feature — Confirmed.
  - The 13 unit tests match; `code/tests/editor.rs` has 6 and `core/tests/text_editor_rows.rs` 9, against 5 and 8 — Confirmed. Node sizes and the speed tables — Unverified.
- Limits and open items, from "入れていないもの":
  - 「複数のキャレット、ミニマップ、LSP、括弧の自動の閉じ。」 — still absent (`rg 'minimap|LSP'` finds nothing in the crate) — Confirmed.
  - 「タブ文字の幅（今は parley の空白の幅）」 — not checked — Unverified.
  - 「折りたたみの印を行番号の横に出すこと」 — still absent: the mark is at the end of the folded line (`editor.rs:266`) — Confirmed.
  - 「長いファイルの色分けで、画面の位置から解析すること」 — still true: `highlight_limit` is the only knob (`editor.rs:150`) — Inferred.
  - 「`markdown_editor` の中でソースモードに切り替えること」 — still absent; the source side is the split editor — Confirmed.

#### `32-markdown-split.md`: A split Markdown editor, and colored code blocks

- Subject: source on the left, preview on the right, and tree-sitter coloring of code blocks.
- Summary: `markdown_split_editor` puts `code_editor` beside `markdown_view`, with a 5 px divider draggable to 15% and Escape to reset, and the scrolling side leading through a piecewise map between top-level blocks and their heights. A caret-line marker and double-click-to-source use the same map. A new `CodeHighlighter` colors snippets on the UI thread up to 64 KB, compiles grammars on a worker, and calls `ready()` so only blocks that were waiting rebuild. Two fixes are recorded: one grammar compile per language, and `flex_1` as `flex: 1 1 0%`, which stopped the preview remeasuring everything. Partial re-parsing, sub-block sync and a dark preview theme are out.
- Code match: mostly matches.
  - `markdown_split_editor` (`markdown/src/split.rs:33`), `.split` (`:100`), `NARROWEST = 0.15` (`:15`); `code` on by default (`markdown/Cargo.toml:19`) — Confirmed.
  - `CodeHighlighter` (`code/src/snippet.rs:29`), 64 KB (`:16`), 512 kept (`:18`), `ready()` (`:68`); `Anchors` is crate-private (`markdown/src/view.rs:147`) — Confirmed.
  - `EditorRef::top_line`/`scroll_to_line` (`view.rs:2209`, `:2216`), `NodeRef::offset_in` (`:132`), `is_default_prevented` (`event.rs:392`), `code_editor`'s `value`, `on_scrolled`, `on_key_down` (`code/src/editor.rs:101`, `:108`, `:115`) — Confirmed.
  - `flex_1` sets grow, shrink and `flex-basis: 0%` (`style/src/styled.rs:45`–`50`) — Confirmed.
  - `markdown/tests/split.rs` has 9 tests, not the 5 the document lists — Confirmed. The typing and start-up tables are measurements — Unverified.
- Limits and open items:
  - 「変わったブロックの近くだけを解析し直すこと」 — still absent: parsing is whole-document (`markdown/src/parse.rs:423`) — Inferred (no partial entry point).
  - 「合わせるのも、印を出すのも、いちばん外側のブロック単位で…」 — still true (`Anchors`, `view.rs:147`) — Confirmed.
  - 「ダブルクリックで飛ぶのは行の頭までで、押した文字の位置ではない」 — still true (`markdown/tests/split.rs:318`) — Confirmed.
  - 「左だけ、右だけの表示への切り替え」 — still absent — Confirmed.
  - 「右側の暗い配色」 — still true: `MarkdownTheme` has `Default` and no dark variant (`markdown/src/theme.rs`) — Confirmed.

#### `33-customization.md`: Changing parts without forking

- Subject: how a program restyles standard controls and builds its own set from the public API.
- Summary: Five levels run from theme colours (Lv1) through the outer box (Lv2) and named parts (Lv3) to a look written entire (Lv4); each moving control gets `x`, `styled_x` and `x_with`, and motion is added to the box the look returns so the look's own listeners still run. Core gains `Binding<T>`, `Interaction`, `follow`, `clock`, `logical_rect` and `close_window`; the widgets export `themed` and `focus_ring`. MD3E is re-seated on the standard motion, and a test pins it to the prelude, the widgets and `check_shader`. Internal `pub`s are `#[doc(hidden)]`, and `.disabled` now stacks. Table motion, a disabled signal from the look, and type-to-select are left out.
- Code match: mostly matches.
  - `Binding` (`reactive.rs:612`), `Copy` (`:636`), `From<Signal<T>>` (`:679`); `Interaction`/`.interaction` (`view.rs:418`, `:689`); `follow` (`animation.rs:576`), `clock` (`:591`); `logical_rect` (`view.rs:119`); `close_window` (`window.rs:1085`); `themed`/`focus_ring` (`widgets/src/theme.rs:173`, `:195`) — Confirmed.
  - `styled_switch`/`switch_with` (`choice.rs:625`, `:631`), `slider_with` (`slider.rs:355`), `tabs_with` (`structure.rs:205`), `tooltip_with` (`tooltip.rs:223`), `modal_with` (`dialog.rs:159`), `menu_with` (`menu.rs:522`), `list_box_with` (`select.rs:433`), `tree_view_with` (`long.rs:314`) — Confirmed.
  - Roles come from the motion layer (`choice.rs:640`–`650`); `.disabled` ORs its predicates (`view.rs:853`–`858`); MD3E's allowed imports (`md3e/tests/public_api.rs:7`); `tests/looks.rs` has 10 tests — Confirmed.
  - The 1,428 workspace tests are old: [A6](audit-tests.md) counted 1,693 `#[test]`, 1,604 listed with default features — Confirmed (via the test audit).
- Limits and open items, from "やらないこと（次）":
  - 「テーブルの動きの分離」 — still undone; cells are styled through `custom_column` — Inferred from the document's wording.
  - 「見た目からの無効のシグナル」 — still true: a look reads the same condition itself (`view.rs:853`) — Confirmed.
  - 「プルダウンやリストで、文字を打って項目へ飛ぶこと」 — still absent (`rg 'type_ahead|first letter'` finds none) — Confirmed by absence.

#### `34-accessibility.md`: An accessibility surface

- Subject: the interface in core that an assistive-technology bridge would use, with no bridge itself.
- Summary: Core keeps the surface only: what a node is, is called and its state; the tree and word of its changes; and requests (`Focus`, `Click`, `Increment`, `Decrement`, `ScrollIntoView`). Roles and states follow ARIA, names come from content, text nodes become `Text` and inputs `TextInput`, and a box that says nothing and cannot take focus is dropped with its contents given to the parent, as Chromium does. The tree is built only when `snapshot` is called or someone `watch`es. AccessKit and AT-SPI are left to another crate, as are caret positions inside text, `SetValue`, table roles, and MD3E's pickers and carousel.
- Code match: mostly matches.
  - `access` (`view.rs:868`), every builder in the table (`access.rs:151`–`218`), `Checked` (`:94`), `Live` (`:109`), `named_from_content` (`:70`, used at `:521`) — Confirmed.
  - `snapshot` (`:297`), `watch` (`:357`), `perform` (`:304`), `AccessTree` (`:254`), `AccessNode` (`:228`), `AccessAction` (`:277`) — Confirmed.
  - `Role` has 39 variants, not the "40 余り" the document says (`:16`–`92`) — Confirmed.
  - A `Status` node announces `Polite` by default (`:507`–`510`), which is how toasts and snackbars meet the document (`widgets/src/toast.rs:219`, `md3e/src/components/snackbar.rs:190`) — Confirmed.
  - Motion sets roles: `MenuItem` (`menu.rs:279`), `Tree`/`TreeItem` (`long.rs:382`, `:370`), `TabList` (`structure.rs:217`), `Slider` (`slider.rs:199`); 7 tests in `core/tests/accessibility.rs` — Confirmed (the count [A4c](audit-capabilities-platform.md#accessibility-api) gives).
- Limits and open items, from "やっていないこと":
  - 「AT-SPI への橋渡しのクレート」 — still absent: no crate depends on `accesskit` — Confirmed.
  - 「文字の中の位置」 — still absent: `AccessNode` (`access.rs:228`–`250`) has no runs or caret — Confirmed.
  - 「値を直接決める頼みごと（`SetValue`）」 — still absent — Confirmed.
  - 「テーブル。」 — still true: `Role::Table` appears only inside `access.rs` — Confirmed.
  - 「MD3E の日付と時刻のピッカー、カルーセル。」 — not checked — Unverified.

#### `35-platform-x11.md`: The X11 backend

- Subject: the X11 backend from the window to DRI3/Present, input, popups, clipboard, drag and drop, and IME.
- Summary: The document records upstream's build of a fourth backend on `x11rb`, behind the same `Host` seam as Wayland: its own event loop, XI2 input with a 300 ms fractional-scroll guess, a global scale only, override-redirect popups placed by a copy of `xdg_positioner`'s rules with XI2 pointer and keyboard grabs for menus, DRI3 and Present with explicit or implicit sync and presentation timestamps for pacing, INCR and clipboard-manager (re)acquisition, XDND on the wire, and XIM with an on-the-spot preedit. It rejects per-monitor scale, touchpad inertia (X11 has no `axis_stop`), touchscreens, GLX and EGL presentation, D-Bus IME, recovery from a failed XIM, `MULTIPLE` clipboard requests, and `XdndProxy`.
- Code match: mostly matches, as [A4c](audit-capabilities-platform.md#code-against-the-design-documents) found (its table has the test counts and the stale `XdndProxy` doc comment). Checked here, on statements that audit did not cover:
  - Menu grab is XI2 `XIGrabDevice` on the top menu plus `grab_keyboard` (`x11/src/popup.rs:400`–`431`) — Confirmed.
  - The fractional-scroll memory is 300 ms (`SMOOTH_FOR`, `input.rs:10`) and 15 vs 10 px per notch (`input.rs:268`, `:295`) — Confirmed.
  - No touchscreen: the X11 crate has no touch class, the Wayland crate no `wl_touch` — Confirmed (absence).
  - No GLX or EGL presentation code and no D-Bus IME in the crate — Confirmed (absence).
  - `MULTIPLE` is not answered: `selection.rs` answers `TARGETS` (`:306`) and `TIMESTAMP` (`:348`) and asks `SAVE_TARGETS` (`:452`) — Confirmed.
  - XIM has no reconnect path: `ime.rs` makes its client once and has no restart code — Confirmed by reading.
- Limits and open items, from "やらないこと":
  - 「モニターごとのスケール。X11 にはない。」 — still true: one scale per display ([A4c](audit-capabilities-platform.md#clipboard-cursor-and-high-dpi-scale)) — Confirmed.
  - 「タッチパッドの慣性。」 — still true: scroll events are always `ScrollPhase::Single` (`state.rs:993`) — Confirmed.
  - 「クライアント側の装飾。」 — no longer true: `decor.rs` handles a frame of the window's own from `_MOTIF_WM_HINTS` and its resize edges (`x11/src/decor.rs:1`–`2`), and `tests/server.rs:764` checks the frame hint — Confirmed.
  - 「読み上げ。」 — still true: no bridge, see `34` — Confirmed.
  - 「XIM の失敗からの立ち直り。」 — still true: nothing reconnects — Confirmed.

#### `36-multi-window.md`: Several windows, and window decorations

- Subject: windows on their own threads, handles, closing order, modal windows, and decorations.
- Summary: A second window is `open_window` (same thread, signals shared directly) or `spawn_window` (own thread, `Send` views, `shared_signal` to cross), both taking `WindowOptions` and returning a `WindowHandle` that is `Send + Sync + Clone`. Each window has its own `Document` and owner tree, so a signal written in one window runs effects in the other's document. Closing goes through `on_close_request`, where `prevent()` keeps the window open; a modal child stops the parent's input; children close with their parent unless `top_level`. Decorations are `Auto`, `Server` or `Client`, with a `title_bar` widget for the client case and resize edges in the window; the environment table gives each OS its parent, modal, decoration and spawned-thread answer. Position setting and parent-attached child popups are deferred.
- Code match: mostly matches, as [A5](audit-platform-matrix.md#documents-against-the-code-and-the-machine) found (its table has the aged "Windows（今後）" column). Checked here:
  - Every name in the API section exists: `open_window` (`window.rs:966`), `spawn_window` (`:999`), `WindowOptions::{title, size, dialog, modal, top_level}` (`:141`–`175`), `WindowHandle::{is_open, close, request_close, set_title}` (`:260`–`274`), `current_window` (`:1072`), `close_window` (`:1085`), `request_close_window` (`:1090`), `set_window_title` (`:1099`), `on_close_request` (`:1119`), `CloseRequest::prevent` (`:328`), `App::decorations` (`app/src/lib.rs:92`), the `window_*` reads and `start_window_*` calls (`:1137`–`1177`), `shared_signal` (`shared.rs:65`) — Confirmed.
  - `WindowOptions` has no position or parent-attached child (`window.rs:126`–`175`) — Confirmed.
  - `EditorRef` is a `Signal<Option<NodeId>>` with no window id (`view.rs:2128`), so the document's remaining item holds — Confirmed.
  - The X11 server tests check `WM_TRANSIENT_FOR` and `_MOTIF_WM_HINTS` (`x11/tests/server.rs:760`–`764`) — Confirmed. No Wayland toplevel-parent test exists yet (`wayland/tests/` has drag, fullscreen, layer, lock and popup grab only) — Confirmed by absence.
- Limits and open items:
  - 「位置の指定と、親に貼り付く小窓…あとで足す」 — still true — Confirmed.
  - 「手で確かめること…」 — not done, and not done here — Unverified.
  - 「Wayland のプロトコルテスト（2 枚目の toplevel、入力の振り分け）」 — still absent — Confirmed.
  - 「同じスレッドの窓の間のアプリ内 DnD」 — still absent: a typed drag cannot cross a window boundary as a type ([A4c](audit-capabilities-platform.md#drag-and-drop)) — Confirmed.
  - 「`EditorRef` が結び付けたウィンドウを覚えること」 — still absent — Confirmed.

#### `37-wayland-shell.md`: A Wayland shell

- Subject: a toolkit for writing a desktop shell: layer surfaces, outputs, and Linux desktop services.
- Summary: The document designs a shell crate rather than another window backend: `Shell::run` (or `App::run_shell`) requires a compositor with `zwlr_layer_shell_v1`, `open_layer`/`spawn_layer` place a view on a chosen output with anchors, size, exclusive zone, margin, layer and keyboard interactivity, and one UI thread per output is possible through `spawn_ui_thread`, with `Task` for work that takes time. On top of that it collects the services a shell needs — outputs, workspaces (compositor IPC first, then `ext_workspace_v1`), foreign toplevels, idle, clipboard history, thumbnails and capture, media, batteries, network, Bluetooth, notifications, tray, audio, installed applications, child processes, system counters, and a session lock — each behind signals, with D-Bus and per-compositor IPC hidden inside. It refuses X11, live reload of configuration, PAM authentication and output power control, and it says plainly that only ShojiWM was verified: Hyprland, niri and sway were checked against fake servers and the rest of the table rests on public information, with GNOME excluded because it has no layer shell.
- Code match: matches. [A4c](audit-capabilities-platform.md#anti-webui-wayland-shell) covers the service list, the feature names, the 66 tests and 9 ignored live tests, and the 101 `unsafe` occurrences. Checked here:
  - `Shell::run` refuses without a layer shell: `supports_layer_shell` (`wayland/src/lib.rs:464`) is checked first and the error names `zwlr_layer_shell_v1` (`wayland-shell/src/lib.rs:267`–`271`) — Confirmed.
  - `LayerOptions` has `layer`, `anchor`, `width`, `height`, `exclusive`, `margin`, `keyboard`, `output`, `namespace` (`wayland-shell/src/lib.rs:349`–`424`); `spawn_for_each_output` (`output.rs:169`) — Confirmed.
  - Workspace IPC reads `HYPRLAND_INSTANCE_SIGNATURE` (`hyprland.rs:16`), `$NIRI_SOCKET` (`niri.rs:15`), and `$SWAYSOCK` with `I3SOCK` beside it (`sway.rs:23`) — Confirmed.
  - No PAM and no output-power protocol: `rg 'pam|output_power'` finds neither (the only power code is power-profiles-daemon, `power_profiles.rs:1`) — Confirmed. `Task` is core's (`core/src/task.rs:28`) — Confirmed.
- Limits and open items, from "やらないこと", "環境ごとの違い" and "決まっていないこと":
  - 「X11 への対応」 — still absent; the crate is Wayland-only — Confirmed.
  - 「設定ファイルを書き換えたときにその場で入れ替わるライブリロード」 — still absent — Confirmed.
  - 「認証（PAM）、出力の電源（`wlr_output_power`）」 — still absent — Confirmed.
  - 「ShojiWM の列だけを…確かめた」 — unchanged: every other compositor row is unverified ([A4c](audit-capabilities-platform.md#anti-webui-wayland-shell)) — Confirmed as a statement, Unverified as behaviour.
  - 「`spawn_for_each_output` のスレッドが自分の Vulkan デバイスを持つか、共有するか」 — still undecided; nothing in the crate answers it — Unverified.

#### `38-platform-macos.md`: The macOS backend

- Subject: the macOS backend: AppKit windows on the main thread, `IOSurface` presentation, input, IME, popups, clipboard, and multi-window.
- Summary: The document describes a backend that keeps all AppKit work on the main thread while each window's UI and drawing run on the window's own thread, waking the main thread with an empty `NSEvent` as GLFW does. Presentation is three of our own `IOSurface`s with buffer age, `waitUntilScheduled`, and a `CATransaction` around the layer update, falling back to a wgpu `CAMetalLayer` surface; a `CADisplayLink` paces frames, a pump keeps live resize drawing, and a 100 ms wait covers a screen without one. Scale is 1 or 2 only, IME is NSTextInputClient, popups are borderless child windows, clipboard is the general pasteboard plus a find one, and child windows hang off the parent. Sheets, native menus as UI, native controls, and `CADisplayLink` on macOS 13 and older are out.
- Code match: matches, as [A4c](audit-capabilities-platform.md#code-against-the-design-documents) found (scale, clipboard and per-thread windows are its). Checked here:
  - The minimal main menu is there as "イベントループ" describes: Hide, Hide Others, Show All, separator, Quit with ⌘Q, and no Edit menu (`macos/src/app.rs:65`–`92`) — Confirmed.
  - `NO_TICK_WAIT` is 100 ms (`macos/src/frames.rs:23`), and `CADisplayLink` is used (`frames.rs:14`, `view.rs:470`) — Confirmed.
  - No native controls or sheets: `rg 'NSButton|NSSheet|NSPopUpButton'` in the crate finds none — Confirmed (absence).
  - The "やらないこと" entry about native menus is not contradicted: the menu above is the application's, as "イベントループ" itself says — Confirmed.
- Limits and open items, from "まだ確かめていないこと" and "やらないこと":
  - Live resize, real Japanese IME, trackpad scroll and inertia, pinch, ⌘Q, fullscreen, moving between Retina and 1×, `DecorationMode::Client` — not verified; [A4c](audit-capabilities-platform.md#macos-backend) records the same, and nothing here was run — Unverified.
  - Dragging out to Finder, `DragSourceEnded`, `start_move` from a spawned window — not verified — Unverified.
  - `app`'s `text_scaled_up_is_as_sharp_as_bigger_text` failing on macOS (a headless font difference) — not checked here — Unverified.
  - 「シート、ネイティブのメニュー、ネイティブのコントロール」 — still absent — Confirmed.
  - 「macOS 13 以前の `CADisplayLink`」 — the 100 ms wait exists (`frames.rs:23`) — Confirmed.

#### `39-platform-windows.md`: The Windows backend

- Subject: the Windows backend, including start-up cost and how it is measured.
- Summary: The document builds a Win32 backend on the shared seam with per-monitor DPI v2, IMM32 for IME, DirectManipulation for touchpads, popups as reused windows with their own frame and a cloak to hide the first frame, and a DXGI swapchain presented with damage. Its larger part is start-up: wgpu's DX12 enumerates every adapter and wakes the sleeping dGPU, so the fork's `Dx12AdapterFilter::Luids` lets only the `MINIMUM_POWER` adapter through, shaders compile on threads to the core count, a DX12 pipeline cache on disk survives restarts, the GPU is made on another thread before the window shows, and the indirect-draw validation pipeline is dropped. It measures each step down to 0.39–0.46 s to the first frame and compares with Zed. Exclusive fullscreen, GDI and Direct2D, and Windows 7 and 8 are out; real IME and real touchpads are unverified.
- Code match: mostly matches, as [A4c](audit-capabilities-platform.md#code-against-the-design-documents) found (per-monitor DPI, IMM32, DirectManipulation, the cloaked first frame, popup reuse). Checked here:
  - The indirect-draw validation is removed unless `WGPU_VALIDATION_INDIRECT_CALL` is set (`wgpu/src/lib.rs:138`–`145`) — Confirmed.
  - `Gpu::start_early` exists (`wgpu/src/lib.rs:296`) in `early.rs`, as the document names it — Confirmed.
  - The adapter filter is `Luids` of the `EnumAdapterByGpuPreference(MINIMUM_POWER)` head (`wgpu/src/adapters.rs:38`–`49`), set on the descriptor at `wgpu/src/lib.rs:146` — Confirmed.
  - No exclusive fullscreen, GDI or Direct2D drawing (`rg 'SetFullscreenState|D2D|GDI|BitBlt'` in the Windows crate finds none) — Confirmed (absence).
  - The disk shader cache is `shader_cache.rs` on DX12 ([A4b](audit-capabilities-rendering.md#shaders-and-filters)) — confirmed there, not re-read here.
- Limits and open items, from "まだ確かめていないこと" and "やらないこと":
  - Real IME (preedit, candidate window), live resize drawing, `WM_DPICHANGED` across monitors, popup shadows, closing a menu by clicking another app — not verified; [A4c](audit-capabilities-platform.md#windows-backend) says the same — Unverified.
  - Vsync thread behaviour with mixed refresh rates, and real DirectManipulation feel — not verified — Unverified.
  - 「排他的なフルスクリーン。」 — still absent — Confirmed.
  - 「GDI や Direct2D での描画。」 — still absent — Confirmed.
  - 「Windows 7 と 8」 — per-monitor v2 is still required and nothing aged it — Inferred.
  - The document's `cargo check --workspace` on Windows claim is A5's row: it passes from Linux but for the `tree-sitter` crates — Confirmed (the cross-check), Unverified on a Windows host.

#### `40-math.md`: Mathematics (LaTeX)

- Subject: `$…$` and `$$…$$` in Markdown, drawn without JavaScript or a WebView.
- Summary: The document picks RaTeX 0.1.15 (KaTeX's parser and layout in Rust) over the alternatives, after comparing 77 formulas with Chromium's KaTeX: 62 of 74 within 0.03 em, with `\tag`, root indices and `\overbrace` as the known differences. Core gains a `drawing` node (glyphs, rectangles and paths in em units, with a baseline for inline boxes) and `shape_glyphs`; `anti-webui-math` lays a formula out, renumbers equations, resolves `\label`/`\ref`, cuts inline formulas at operators, and caches per thread. `anti-webui-markdown` gets a `math` feature over `ENABLE_MATH`. KaTeX itself, HTML+CSS routes, and MathML for assistive technology are out.
- Code match: matches.
  - `Drawing`/`em_scale` (`core/src/drawing.rs:19`, `:128`), `shape_glyphs` (`core/src/text/mod.rs:425`) — Confirmed.
  - `math` and `Math` with `display`, `leqno`, `first_number`, `references`, `on_reference`, `runs` (`math/src/lib.rs:129`–`196`); `check` (`:64`), `equations` (`:80`) — Confirmed.
  - 19 KaTeX fonts (`math/fonts/`), RaTeX pinned `=0.1.15` (`math/Cargo.toml:16`–`19`), 2,048 kept per thread (`layout.rs:96`); `ENABLE_MATH` (`parse.rs:103`); theme `math`/`math_block`/`math_leqno` (`theme.rs:45`, `:47`, `:50`) — Confirmed.
  - The formula is one node with role `Image` and the source as its name (`math/src/lib.rs:295`); 14 unit tests now against the document's 6 — Confirmed (a drift). The KaTeX comparison and timings — Unverified.
- Limits and open items, from "やっていないこと":
  - 「手での確認。」 — still not done here either — Unverified.
  - 「選択した式の中身を選ぶこと」 — still true: selection is over text nodes, and a formula is one image-like box (`core/src/text/selection.rs`) — Inferred.
  - 「RaTeX と KaTeX の違い…0.08〜0.33 em」 — unchanged; it is RaTeX's layout — Unverified.
  - 「読み上げで式を MathML として渡すこと」 — still absent: only the source is the name (`math/src/lib.rs:295`), and `34`'s bridge does not exist — Confirmed.
  - 「番号を振る環境が、分数や添字の中など…にあるとき」 — still true; RaTeX's placement is kept — Unverified.

#### `41-gpu-canvas.md`: A GPU canvas (`gpu_canvas`)

- Subject: a box a program draws into with wgpu, plus `portable_canvas`, offscreen nodes and `gpu_info`.
- Summary: `gpu_canvas::<T>(props)` is a replaced element whose pixels the program writes: `T` has `Props`, a one-time `new` on the window's device, and `render` into a pass in the window's `Rgba8Unorm` format plus an sRGB view (none on GL). Equal props mean no redraw, and the texture is kept while the node lives, even while `content-visibility` skips it. `portable_canvas` carries "how to draw" in the scene so any renderer, on any thread, makes its own painter. `set_offscreen` with `paint_node` paints one laid-out node alone for use as a texture, and `gpu_info` reports backend, adapter, kind, driver and presentation path per window. MSAA, depth, extra features, other formats and 3D are the program's.
- Code match: mostly matches.
  - Drawing behaviour — own encoder, validation error scope, one texture per canvas, `version`/`mark_shader_of`, corners through the picture pipeline — is [A4b](audit-capabilities-rendering.md#gpu-canvas), not re-derived.
  - `NATURAL` 300×150 (`core/src/canvas.rs:18`), `GpuInit` (`wgpu/src/canvas.rs:38`), `GpuFrame` (`:49`), `gpu_canvas` (`:103`) — Confirmed.
  - `portable_canvas` (`:279`), `portable` (`:137`), `CanvasData::portable`/`set_paint` (`canvas.rs:40`, `:57`), `Portable` (`:64`); the thread test (`app/tests/gpu_canvas.rs:384`) — Confirmed.
  - `GpuInfo` and its three enums are `#[non_exhaustive]` (`window.rs:655`–`730`); `gpu_info` (`:772`), `set_gpu_info` (`ui.rs:543`), per window (`core/tests/windows.rs:250`); `Limits::default()` then `using_resolution` (`wgpu/src/lib.rs:50`–`56`) — Confirmed.
  - `set_offscreen` (`tree.rs:1108`), `paint_node` (`paint.rs:153`), `core/tests/offscreen.rs:35`, `core/tests/images.rs:269` — Confirmed; the 8 pixel tests — Unverified.
- Limits and open items, from "やっていないこと":
  - 「MSAA と depth の用意」 — still true: the program makes them, and a lost device is not recovered ([A4b](audit-capabilities-rendering.md#gpu-canvas)) — Confirmed.
  - 「features を足す口」 — still absent: `App` has no features setter — Confirmed.
  - 「描き先の形式を選ぶ」 — still true: `Rgba8Unorm` only — Confirmed.
  - 「UI 要素の 3D 変形、宣言的な 3D シーン」 — still absent — Confirmed.

## Baseline violations

[ADR-0007](../decisions/0007-upstream-design-documents-across-the-fork-lifecycle.md) settles what happens to these: `docs/design` stays unchanged while the fork tracks upstream, and the fork takes it over at the hard fork.

Not rules but facts a reader of `docs/design` must allow for, kept as the baseline so the fork's own documents do not repeat them:

- 23 statements in this audit, and 13 more in A4b (4), A4c (5), and A5 (4), are stale: the documents are not updated when later work changes what they describe. The most misleading are "not done" lines that are done (`06`, `09`, `17`, `18`, `20`, `30`, `35`) and API sketches with old names (`04`'s `ImeRequest`, `31`'s `gutter(Style)`).
- `00-overview.md`'s crate diagram names 14 of the 21 workspace members.
- `40-math.md` has two `## Markdown` headings (lines 123 and 126).

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| "Not done" lists that are out of date | A requirement or gap analysis asks for work that exists, or skips work it thinks exists | Certain (seven cases) | [Statements that disagree with the code](#statements-that-disagree-with-the-code) |
| Every performance figure is upstream's, from upstream's machines | A fork decision based on a document's number may not hold on other hardware | High | Each document's entry; A9 |
| Design knowledge lives in Japanese prose of 870 KB, with no index from code to section | Agents and English-speaking contributors find the reason for a design slowly or not at all | High | [Method](#method) |
| Behaviour checked by hand only (Wayland shell L1–L4, the M1 known limits, IME on real systems) | A regression there has no test to catch it | Medium | [Open items in `07-roadmap.md`](#open-items-in-07-roadmapmd) |

## Open questions

- Which upstream measurements does the fork rely on? Answered by A9 re-measuring the ones it names, on this machine.
- The claims left Unverified in each entry (Compose comparisons in `19` and `20`, `hayro` blend modes and AES in `26`, real video and HDR in `24`, `merman` fidelity in `28`, KaTeX parity in `40`): each entry names what would settle it.

## Inputs for gap analysis

- [Overview](#overview): the design documents can be trusted for intent and structure; numbers and "not done" lists must be checked against the code before a requirement cites them.
- [Open items in `07-roadmap.md`](#open-items-in-07-roadmapmd): the items still open there are upstream's own backlog and the first candidates for the fork's gap list.
- [Statements that disagree with the code](#statements-that-disagree-with-the-code), with the tables in A4b, A4c, and A5: the full list of stale statements, if the fork decides to send corrections upstream.
