# Audit: platform capabilities

- Date: 2026-11-25
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Auditor: Pi, DeepSeek V4.1 Flash (draft); reviewed by Claude Code, Claude Opus 5.5
- Plan item: [A4c](phase-1-audit-plan.md#workstreams)

## Scope

The four OS backends and what sits between them and core: `src/anti-webui-wayland` (5,058 lines of source and 2,833 of tests), `src/anti-webui-x11` (5,551 and 775), `src/anti-webui-macos` (4,339), `src/anti-webui-windows` (4,106), the platform-facing half of `src/anti-webui-app` (`runner.rs` 1,106 lines, `host.rs` 305, `lib.rs` 168), and core's `platform.rs` (429), `window.rs` (1,179), `access.rs` (552), `drag.rs` (1,775), and `layer.rs` (379, the `zwlr_layer_shell_v1` API). `src/anti-webui-wayland-shell` (13,169 and 2,809) is covered briefly as a toolkit for desktop shells.

The upstream documents read as the yardstick are `docs/design/06-platform-wayland.md`, `23-drag-and-drop.md`, `27-fullscreen.md`, `34-accessibility.md`, `35-platform-x11.md`, `36-multi-window.md`, `37-wayland-shell.md`, `38-platform-macos.md`, and `39-platform-windows.md`.

Left out: core's input, focus, selection, and IME state machines ([A4a](audit-capabilities-core.md)); the renderer, dmabuf and `IOSurface` allocation, presentation buffers, damage, and frame pacing ([A4b](audit-capabilities-rendering.md)); the wgpu fork ([A3](audit-wgpu-fork.md)); crate layers and target-gated dependencies ([A1](audit-dependencies.md)); what each OS was tested on ([A5](phase-1-audit-plan.md#workstreams)); the full `unsafe`, Clippy, and formatting baseline ([A7](phase-1-audit-plan.md#workstreams)); and the document-by-document summary (A8). Windows and macOS were compile-checked only, so statements about their run-time behaviour are what the code and the upstream documents say, not what was observed.

## Method

Reading order: the design documents above, then core's `platform.rs`, `window.rs`, `access.rs`, `drag.rs`, and `layer.rs`; then `anti-webui-app`'s `runner.rs` and `host.rs`; then each backend from `lib.rs` and `state.rs` outward; then the tests and the shell crate.

| Question | Command |
|---|---|
| Branch and base | `git status --short`, `git log --oneline -3` |
| Sizes | `wc -l src/anti-webui-<backend>/src/*.rs src/anti-webui-core/src/{platform,window,access,drag,layer}.rs src/anti-webui-app/src/{runner,host,lib}.rs` |
| Where a symbol is | `rg -n '^(pub )?(fn|struct|enum|impl|trait|const|static|type)' <file>` and `rg -n '<symbol>' <crate>/src` |
| Sections of a design document | `rg -n '^#{1,3} ' docs/design/<file>.md` |
| A document's wording | `sed -n '<range>p' docs/design/<file>.md` |
| Protocols and APIs per backend | `rg -n 'protocols::|z?wp_|wlr_|xdg_|ext_' src/anti-webui-wayland/src/lib.rs`; `rg -n 'xinput|dri3|present|xfixes|shape|randr|sync::|xkb|xim' src/anti-webui-x11/src`; `rg -n 'NS[A-Z]|IOSurface|CAMetalLayer' src/anti-webui-macos/src`; `rg -n 'Win32_|WM_|DirectManipulation' src/anti-webui-windows/src` |
| Feature-table checks | `rg -n 'fn set_(cursor|ime|clipboard|fullscreen)|fn supports_drag|fn start_drag|fn request_paste' src/anti-webui-<backend>/src` |
| Scale per backend | `rg -n 'Scale\\(|Scale::from_f64|dpi_of|Xft/DPI|fractional|backingScaleFactor|GetDpiForWindow' src/anti-webui-<backend>/src` |
| Installed targets | `rustup target list --installed` |
| Linux compile check | `cargo check --locked -p anti-webui-wayland -p anti-webui-x11 -p anti-webui-wayland-shell --all-features` |
| Windows compile checks | `cargo check --locked -p anti-webui-windows --target x86_64-pc-windows-msvc`; `cargo check --locked -p anti-webui-app --target x86_64-pc-windows-msvc` |
| Extra compile checks | `cargo check --locked -p anti-webui-wayland -p anti-webui-x11`; `cargo check --locked -p anti-webui-app --all-features`; `cargo check --locked -p anti-webui-app --target x86_64-pc-windows-msvc --tests` |
| `#[test]` per file | `rg -c '#\[test\]' src/anti-webui-<backend>/src/*.rs src/anti-webui-<backend>/tests/*.rs` |
| Tests that run | `cargo test --locked -p anti-webui-wayland -- --list`; `cargo test --locked -p anti-webui-wayland-shell --all-features -- --list` |
| Tests that would not link | `cargo test --locked -p anti-webui-x11 -- --list` |
| End-to-end scenario counts | `rg -c 'check\\(' src/anti-webui-app/tests/{macos,macos_spawn,windows}.rs`, then read where a scenario is recorded without `check` |
| `unsafe` per backend | `rg -o '\\bunsafe\\b' src/<crate>/src \\| wc -l`, then a Python script over the same files counting `SAFETY` on the line or the three lines above (kept only in `$TMPDIR`) |
| Fork checks | `mise run check` |

Compile checks, all run on WSL2 Linux x86_64 with `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-msvc` installed:

| Check | Result | Warning lines |
|---|---|---|
| Wayland, X11, wayland-shell, `--all-features` | pass | 163: 162 `falling back to f32` from `anti-webui-widgets`, 1 summary line; the three audited crates themselves emit none |
| `anti-webui-windows`, Windows target | pass | 0 |
| `anti-webui-app`, Windows target | pass | 0 |
| Wayland and X11 only (extra) | pass | 0 |
| `anti-webui-app --all-features` on Linux (extra) | pass | 0 |
| `anti-webui-app` with `--tests`, Windows target (extra) | pass | 158, all in tests and in `anti-webui-pdf`; none in `anti-webui-windows` |
| macOS | not compile-checked here: the Apple target is not installed | — |

Label: Confirmed; every row was run in this checkout.

Test counts. Wayland: 10 tests from `--list` (5 protocol tests in `tests/` and 5 unit tests). Wayland shell: 66 listed with `--all-features`, 9 of them `#[ignore]` (they read this machine or add a null sink). X11: 13 unit tests in `src/` and 9 in `tests/server.rs` by `#[test]`, but `--list` fails to link here: `rust-lld: error: unable to find library -lxkbcommon-x11`, which this machine lacks. macOS: 10 unit tests by `#[test]`, and two harness-free programs with 20 and 10 scenarios (`tests/macos.rs`, `tests/macos_spawn.rs`; the tenth of the second is recorded without `check`). Windows: 10 unit tests by `#[test]`, and a harness-free program with 23 scenarios (`tests/windows.rs`). Core's platform-facing tests are `tests/windows.rs` 11, `tests/fullscreen.rs` 18, `tests/popups.rs` 19, `tests/accessibility.rs` 7, `tests/drag.rs` 53, and `tests/text_drag.rs` 42, counted by `#[test]` (by `--list` for the last three). Label: Confirmed by the commands in [Method](#method); the X11 count is source-level because its list does not link here.

`unsafe`, for the numbers A7 will expand: Wayland 1 occurrence, 1 with `// SAFETY:` within three lines; X11 none; macOS 126, 17 documented; Windows 122, 116 documented; wayland-shell 101, 52 documented. Of macOS's 126, 61 are `#[unsafe(method(...))]` or `#[unsafe(method_id(...))]` attributes and 12 are `unsafe impl` of Objective-C protocols, so the keyword count overstates the number of hand-written unsafe blocks (Inferred from the per-form counts: 46 `unsafe {`, 61 `unsafe(method`, 12 `unsafe impl`, 1 `unsafe extern`).

"No GUI program was run" applies to this file: every run-time statement below is either from the code or from the upstream documents, which are marked as documented upstream and Unverified here.

## Findings

### Checklist

| Capability from the [plan](phase-1-audit-plan.md#capability-checklist) | Status | Where |
|---|---|---|
| Accessibility API | partial: core's tree is present, no OS bridge | [Accessibility API](#accessibility-api) |
| Multi-window | present, with limits on position and typed cross-window drags | [Multi-window](#multi-window) |
| OS backends | present for Wayland, X11, macOS, and Windows | [Wayland backend](#wayland-backend), [X11 backend](#x11-backend), [macOS backend](#macos-backend), [Windows backend](#windows-backend) |
| Fractional-scaling handling, as each backend delivers it | present; Wayland and X11 express `n/120`, macOS is 1× or 2×, Windows follows per-monitor DPI | [Clipboard, cursor, and high-DPI scale](#clipboard-cursor-and-high-dpi-scale) |

Labels: Confirmed, from the rows below. Core's own handling of the scale is [A4a](audit-capabilities-core.md#layout-engine).

### The platform abstraction

- Status: present
- Evidence: core `platform.rs` defines `PlatformRequest` (22 variants, line 125) and the data the backends and the presenter hand each other (`ImeState` 43, `PopupKind` 63, `PopupParams` 93, `place_popup` 178, `SurfaceId` 259, `DmabufFeedback` 270, `SyncPoints` 350, `Presentation` 359). `event.rs` defines `WindowEvent` (22 variants). `anti-webui-app` has `trait Host` (52 methods, `host.rs:20`) implemented for each backend's `Ctx` by the `host_for!` macro, and `trait Platform` (14 methods, `runner.rs:589`) implemented by each backend's `Session` through `platform_for!` (`runner.rs:636`–`724`). Backends implement `WaylandHandler`, `X11Handler`, `MacHandler`, and `WinHandler` (`runner.rs:423`–`434`); `Runner` is the handler for all four through `handler_for!`.
- Behaviour, the seam: core asks and the platform answers. `PlatformRequest` goes out with `Document::request` and is taken once per event (`Ui::take_requests`); `WindowEvent` comes in and is handed to `Ui::handle_event`. The seam has no return values and no futures: an answer, such as `Paste`, `PopupConfigured`, `Fullscreen`, or `DropData`, arrives later as an event. The backend keeps its own state (`State` in Wayland and X11, `Shared` plus `State` in macOS and Windows) and the `Runner` keeps `Ui`, `Presenter`, and `Pacing` ([A4b](audit-capabilities-rendering.md)).
- Behaviour, choosing a backend on Linux: `choose_backend` (`runner.rs:475`) reads `ANTIWEBUI_BACKEND` first (`wayland` or `x11`; another value is an error); otherwise Wayland if `WAYLAND_DISPLAY` is set and `anti_webui_wayland::can_connect()` succeeds (`lib.rs:459`, a real connect); otherwise X11 if `DISPLAY` is set; otherwise Wayland again if `WAYLAND_DISPLAY` is absent, because the default socket needs no variable; otherwise the error "found neither a Wayland compositor nor an X server". X11's `DISPLAY` is not probed before choosing, so a dead `DISPLAY` fails at `Session::connect`. macOS and Windows accept only their own name (`runner.rs:449`, `462`). Before it starts the UI thread, `run_here` asks the display server for its GPU (Wayland's `compositor_main_device`, X11's `server_main_device`) and restricts the Vulkan loader to that device's driver (`runner.rs:1079`, `limit_vulkan_drivers` at 504; [A4b](audit-capabilities-rendering.md) covers the consequence).
- Behaviour, threads: one event loop per UI thread, not per window. `Thread::run` (`runner.rs:851`) calls `session.dispatch`, `run_roots`, `take_requests`, then `session.wait`; a `Session` owns a list of windows and each window owns a `Runner`. On Linux, `spawn_window` starts a thread (`runner.rs:946`–`963`) because every backend declares `OWN_THREADS = true` (`wayland/lib.rs:526`, `x11/lib.rs:494`, `macos/lib.rs:508`, `windows/lib.rs:470`). Wayland shares one connection and gives each window its own event queue (`wayland/lib.rs:500`); X11 opens one connection per window (`x11/state.rs:117`); Windows requires a window on the thread that made it (doc 39, "方針"); macOS requires `App::run` on the main thread (`macos/lib.rs:511`) and moves only drawing and UI work of a spawned window to its thread (`macos/remote.rs`).
- Quality notes: the seam is duplicated four times in the `host_for!` and `platform_for!` macros, which is mechanical but keeps each backend independent; `#[deny(unconditional_recursion)]` guards the forwarding methods. Linux-only concepts (`BorrowedFd`, dmabuf, timeline sync) are no-op types on Windows (`platform.rs:20`–`30`), which doc 39 describes. `Host` mixes window, presentation, popup, and drag methods, so a new backend must answer 52 methods even if it has no dmabufs; only the `IOSurface` and spare-popup methods are `#[cfg]`-gated, so the dmabuf half exists everywhere and is answered with empty bodies off Linux (`windows/lib.rs:257`–`370`).
- Label: Confirmed

### Wayland backend

- Status: present
- Evidence: `wayland-client` and `wayland-protocols` spoken directly, with `xkbcommon` for keys (`Cargo.toml`). Bound globals: `xdg_wm_base` (`lib.rs:750`), `ext_session_lock_manager_v1` (757), `zwlr_layer_shell_v1` (769), `wp_viewporter` (828), `wp_fractional_scale_manager_v1` (838), `zxdg_decoration_manager_v1` (849), `xdg_wm_dialog_v1` (870), `wp_cursor_shape_manager_v1` (877), `zwp_text_input_manager_v3` (879), `zwp_pointer_gestures_v1` (883), `zwp_primary_selection_device_manager_v1` (887), `zwp_linux_dmabuf_v1` v4–5 (894), `wp_linux_drm_syncobj_manager_v1` (900), `wp_presentation` (903), `wl_subcompositor` (832), `wl_data_device_manager` v1–3 (888). Design document: `06-platform-wayland.md`, sections "使用プロトコル", "スケールとサイズ", "イベントループ", "スクロール", "ポップアップ", "キーボード", "クリップボード", "IME", "dmabuf presenter" and its subsections.

| Feature | Status | Evidence |
|---|---|---|
| Toplevel windows, resize, maximize, fullscreen, move | present | `lib.rs:578`–`626`, `decor.rs`; protocol test `fullscreen_protocol.rs` |
| Decorations (server or client) | present | `decor.rs:415` handles `zxdg_toplevel_decoration_v1.configure`; `lib.rs:865` falls back to `Client` |
| Layer surfaces (`zwlr_layer_shell_v1`) | present | `lib.rs:769`, `layer.rs` (344 lines); protocol test `layer_protocol.rs` |
| Lock surfaces (`ext_session_lock_v1`) | present | `lock.rs`; protocol test `lock_protocol.rs` |
| Fractional scale | present: `wp_fractional_scale_v1` `n/120` with `wp_viewporter`; integer `wl_surface.set_buffer_scale` without it | `state.rs:725`–`735`, `408`–`431` |
| Cursor shape | partial: `wp_cursor_shape_v1` only; without it `apply_cursor` returns before setting anything (`state.rs:445`), except `CursorIcon::None`, which hides through `wl_pointer.set_cursor` | `state.rs:433`–`463` |
| IME, `zwp_text_input_v3` | present | `state.rs:466`–`516`, `1184`–`1234` |
| Clipboard and primary selection | present, copies and pastes read or written on a thread (`state.rs:584`, `613`) | `state.rs:518`, `550` |
| Drag and drop | present when `wl_data_device_manager` is v3 or newer | `drag.rs:17`, `94`, `102`; protocol test `drag_protocol.rs` |
| Presentation time | present | `lib.rs:903`, `state.rs`; [A4b](audit-capabilities-rendering.md#frame-pacing) |
| dmabuf buffers and explicit sync | present, with a wgpu-surface fallback | `lib.rs:894`, `900`; [A4b](audit-capabilities-rendering.md#damage-tracking) |
| Touchpad scroll, pinch, held fingers | present | `state.rs:990`, `1033`; unit test `fingers_begin_move_and_lift` (`state.rs:1455`) |
| Popups: `xdg_popup` for menus and tooltips, `wl_subsurface` for bubbles and for lock surfaces | present | `popup.rs:154`–`260`; protocol test `popup_grab_protocol.rs` |
| Touch (`wl_touch`) | absent | no `wl_touch` in the crate; `23-drag-and-drop.md` lists it as not done |
| Mouse cursor theme without `wp_cursor_shape_v1` | absent | see the cursor row |

- Behaviour, sizes and scale: `update_size` computes `round(logical × n / 120)` (R3), sets `wp_viewport.set_destination(logical)`, and otherwise sets the integer buffer scale (`state.rs:408`–`431`); `Resized { size, scale }` is emitted once per change. The compositor's `preferred_scale` arrives from `wp_fractional_scale_v1` (`state.rs:734`), and `xdg_toplevel.configure`'s states decide fullscreen (`state.rs:676`); a zero-size configure after leaving fullscreen restores the previous size (protocol test `the_window_goes_there_and_comes_back`).
- Behaviour, the loop: one connection, each window's `EventQueue`, a wake pipe, timers from the handler's `deadline`, and `wl_surface.frame` callbacks; a popup's wait is cut at `POPUP_FRAME_WAIT = 100 ms` (`state.rs:79`, `390`); key repeat is done by the client from `repeat_info` (`state.rs:598`); nothing changed commits nothing. This is the loop described in 06's "イベントループ", and it is split between `Session::wait_until` (`lib.rs:647`) and the app's `Runner`.
- Quality notes: 10 tests, 5 of them protocol tests against a `wayland-server` mock compositor (`tests/drag_protocol.rs`, `fullscreen_protocol.rs`, `layer_protocol.rs`, `lock_protocol.rs`, `popup_grab_protocol.rs`); one `unsafe` (the keymap fd, `keyboard.rs:36`–`38`). The protocol tests assert the requests and their order, not a real compositor's answers. No wlroots or KWin-specific quirks are encoded. `supports_drag` is false when the compositor offers only `wl_data_device_manager` v1 or v2, in which case drags stay in the window (doc 23, "第 3 層", bullet 「対応しないプラットフォーム」). Without `wp_cursor_shape_v1` the pointer keeps the compositor's shape, which is a limit the protocol table's "任意" does not spell out (Inferred from `apply_cursor` returning early).
- Label: Confirmed, except the user-visible consequence of the missing cursor fallback (Inferred) and everything about a real compositor (Unverified here)

### X11 backend

- Status: present
- Evidence: `x11rb` with the `dri3`, `present`, `randr`, `shape`, `sync`, `xfixes`, `xinput`, and `xkb` features, `xkbcommon` with `x11`, and the `xim` crate (`Cargo.toml`). One `XCBConnection` per window (`state.rs:117`). Design document: `35-platform-x11.md`, sections "方針", "バックエンドの選び方", "app の作り替え", "使う拡張", "ウィンドウ", "スケールとサイズ", "イベントループ", "入力", "カーソル", "提示（DRI3 と Present）", "ポップアップ", "クリップボード", "ドラッグ＆ドロップ", "IME", "フルスクリーン", and "やらないこと".

| Feature | Status | Evidence |
|---|---|---|
| Windows and window properties | present | `state.rs:117`–`360`, `atoms.rs` |
| Server-side decorations; client-side resize strips | present | `decor.rs:63`; doc 36, "CSD のリサイズの縁" |
| Scale | partial: one global scale from XSETTINGS `Xft/DPI`, else `Xft.dpi`, else 96, overridable with `ANTIWEBUI_SCALE` | `state.rs:545`–`550`, `settings.rs:85`–`91` |
| Keymap, modifiers, repeat | present | `keyboard.rs`, `state.rs:436` (`DETECTABLE_AUTO_REPEAT`) |
| Pointer, scroll, pinch | present; XInput 2.2 for scroll, 2.4 for pinch | `input.rs`, `state.rs:367`–`391`, `872` |
| Cursor | present, Xcursor theme read in Rust | `cursor.rs:9`–`40` |
| Clipboard and primary, INCR, `SAVE_TARGETS` | present | `selection.rs:1`–`60`, `412`, `452` |
| Drag and drop, XDND 5 both ways | present | `xdnd.rs:355`, `359`; server tests below |
| IME, XIM on-the-spot | present | `ime.rs:179`–`275`, `state.rs:670`, `728` |
| Presentation: DRI3 pixmaps and Present, explicit sync if available | present | `present.rs:16`–`68`; [A4b](audit-capabilities-rendering.md#damage-tracking) |
| Popups: override-redirect windows, menus grabbed | present | `popup.rs:152`–`457` |
| Fullscreen | present, EWMH `_NET_WM_STATE_FULLSCREEN`, plus `_NET_WM_BYPASS_COMPOSITOR` | `state.rs:612`, `1029` |
| Layer surfaces and lock surfaces | absent: opening one returns "layer surfaces need Wayland" / "lock screens need Wayland" | `lib.rs:530`–`535` |
| Per-monitor scale, touchpad inertia, touchscreen | absent by design | doc 35, "やらないこと" |
| Accessibility (AT-SPI) | absent | see [Accessibility API](#accessibility-api) |

- Behaviour, scale and sizes: the scale becomes `n = round(dpi / 96 × 120)` (`settings.rs:85`), the first window is created at `round(logical × n / 120)`, `ConfigureNotify` is passed through as physical `Resized`, and a change in XSETTINGS or `RESOURCE_MANAGER` emits `Resized` with the new scale without resizing the window (`state.rs:350`, `545`–`570`). All windows of the display share one scale.
- Behaviour, input: XI2 raw values in 16.16 fixed point become `PointF`; event coordinates in 16.16 and valuator deltas in 32.32 (`input.rs:12`–`16`) become logical positions and notches or pixels by the per-device accumulator, which remembers a device as smooth for 300 ms after a fractional value (`input.rs:10`); no inertia is added, because X11 has no `axis_stop` equivalent; pinch comes from `XI_GesturePinch*` when the server offers 2.4 (`state.rs:872`). Keys come from core `KeyPress`/`KeyRelease` so that XIM can see them first: `ime.filter` (`state.rs:670`) and `ime.forward` (`state.rs:728`).
- Behaviour, drag and drop: incoming `XdndEnter`, `XdndPosition`, `XdndLeave`, and `XdndDrop` become the core drag events; outgoing drags grab the pointer, walk the tree with `TranslateCoordinates` to an `XdndAware` window, and answer `XdndSelection` from the `DragOffer` (`xdnd.rs:157`–`700`). The drop icon is an override-redirect ARGB window. `35`'s "ドラッグ＆ドロップ" describes the same flow; a drop onto a proxy window is explicitly out of scope.
- Quality notes: 13 unit tests in `src/` and 9 server tests in `tests/server.rs`. The server tests start `Xvfb :93`, drive it with XTEST, and skip when Xvfb is missing; they do not draw (no DRI3 there). they cover scale, keys and repeat, pointer and wheel, a clipboard larger than one request through INCR, menu placement and dismissal, `WM_DELETE_WINDOW`, an outside drop, an outgoing drag, and a second window above the first. There is no `unsafe` in the crate. The `xdnd_version` doc comment says it goes "through `XdndProxy`" (`xdnd.rs:672`), but the body reads only `XdndAware` (`:679`), and `35` lists `XdndProxy` under "やらないこと"; the comment is ahead of the code (Confirmed).
- Label: Confirmed; run-time behaviour on a real Xorg or XWayland session is documented upstream and Unverified here

### macOS backend

- Status: present
- Evidence: AppKit through `objc2`, `objc2-app-kit`, `objc2-quartz-core` (`NSView`, `CAMetalLayer`, `CADisplayLink`), `objc2-io-surface`, and `dispatch2` (`Cargo.toml`). One `NSWindow` with a custom `AntiWebUIView`, flipped, backed by a `CAMetalLayer` (`view.rs:676`) with a `CALayer` above it for the `IOSurface` frame (`frames.rs:368`). Design document: `38-platform-macos.md`, sections "方針", "ウィンドウ", "スケールとサイズ", "イベントループ", "提示", "入力", "IME", "カーソル", "ポップアップ", "クリップボード", "複数ウィンドウ", "ドラッグ＆ドロップ", "確かめたこと", "やらないこと".

| Feature | Status | Evidence |
|---|---|---|
| `NSWindow` and `NSView`, move, minimize, zoom, close | present | `state.rs:63`–`170`, `285`–`320`; `windowShouldClose` in `view.rs` |
| Decorations: server, or client with a full-size content view | present | `state.rs:140`–`156`; doc 38, "ウィンドウ" |
| Scale | present but integer only: `backingScaleFactor` (1 or 2), `Scale::from_f64` | `view.rs:683`, `691`–`707` |
| IME through `NSTextInputClient` | present | `view.rs:536`–`660` |
| Clipboard | present for the general pasteboard; primary is absent and ignored | `state.rs:245`–`265` |
| Drag and drop both ways | present, `NSDraggingDestination` and `NSDraggingSource` on the view; drag image from the `IOSurface` | `view.rs:483`, `dnd.rs:151`, `221`, `252`, `361` |
| Popups as borderless child windows | present | `popup.rs:116`, `140` |
| Fullscreen | present, `toggleFullScreen`, `windowDidEnter/ExitFullScreen` | `state.rs:275`, `view.rs:888`–`896` |
| Multiple windows, a spawned window drawn on its own thread | present, windows stay on the main thread and the rest moves | `lib.rs:508`, `remote.rs`; doc 38, "複数ウィンドウ" |
| Accessibility (`NSAccessibility`) | absent | no such protocol in the crate |
| Native menu bar beyond Quit/Hide; sheets; native controls | absent by design | doc 38, "やらないこと" |
| Spawned window with a wgpu surface fallback | absent: `raw_handles` is `None` for those | doc 38, "app の作り替え" |

- Behaviour, the loop: the app calls `Session::dispatch` and `Session::wait_until`, which use `nextEventMatchingMask` and `sendEvent:` instead of handing control to `NSApp.run()`; Other threads wake the main thread with an empty `ApplicationDefined` event (`app.rs:100`–`130`) or a `CFRunLoop` source (`remote.rs`). A live resize pumps the loop from `setFrameSize:` and `CADisplayLink` so the content follows the frame (`view.rs:708`, doc 38, "イベントループ"). Frames come from a `CADisplayLink` (`displayLinkWithTarget:selector:`) on the window's screen and are presented through `IOSurface`; the fallback is a `CAMetalLayer` ([A4b](audit-capabilities-rendering.md#frame-pacing)).
- Behaviour, keys and text: AppKit events map as doc 38's table says; `Modifiers::primary()` is Command and `word()` is Option, which core and the widgets use instead of Ctrl (doc 38, "macOS のキーの決まり"). IME collects `setMarkedText:` and `insertText:` during one `keyDown:` and emits either `Ime` or `Key`, never both for the same key (`view.rs:536`–`660`). `purpose` is not consulted, so a password field still takes part in IME (`rg 'ImePurpose|purpose' src/anti-webui-macos/src` finds nothing).
- Quality notes: 10 unit tests (`keyboard.rs` 4, `input.rs` 4, `dnd.rs` 2). The two harness-free programs are the real coverage (`tests/macos.rs` 20 scenarios, `tests/macos_spawn.rs` 10); they post synthetic `NSEvent`s to the app's own queue, which doc 38 says needs no accessibility permission. 126 `unsafe` occurrences, 17 with `// SAFETY:` within three lines; most are `#[unsafe(method(...))]` attributes and 12 `unsafe impl`s of Objective-C protocols, so the count is structural rather than 126 hand-written blocks (Inferred). `App::run` must be called on the main thread; otherwise `Session::connect` returns "AppKit runs on the main thread only: call App::run there" (`lib.rs:511`).
- Label: Confirmed for the code paths; run-time behaviour is documented upstream (M1 Pro, macOS 26, 2026-11-07) and Unverified here

### Windows backend

- Status: present
- Evidence: the `windows` crate (0.62) for Win32, DirectComposition through wgpu's DX12 surface ([A4b](audit-capabilities-rendering.md)), and `windows-core` for `#[implement]` (`Cargo.toml`). Design document: `39-platform-windows.md`, sections "方針", "ウィンドウ", "スタック", "スケールとサイズ", "イベントループ", "起動", "提示", "入力", "タッチパッド", "キーボード", "IME", "カーソル", "ポップアップ", "クリップボード", "複数ウィンドウ", "ドラッグ＆ドロップ", "確かめたこと", "やらないこと".

| Feature | Status | Evidence |
|---|---|---|
| `HWND` window, `WS_EX_NOREDIRECTIONBITMAP`, `WM_NCCALCSIZE` for client decorations | present | `state.rs`, `wndproc.rs`; doc 39, "ウィンドウ" |
| Per-monitor DPI v2 and `WM_DPICHANGED` | present | `wndproc.rs:45`, `141`–`150`, `194`; `state.rs:181` |
| Stack: 8 MiB fiber for `App::run` | present | `stack.rs:25`, `77`; doc 39, "スタック" |
| IME, IMM32 (composition string, attributes, candidate window) | present; TSF and reconversion are future | `ime.rs:95`–`225`, `wndproc.rs:538` |
| Clipboard `CF_UNICODETEXT` with retries | present; no primary | `clipboard.rs:21`–`85` |
| Cursor | present, `WM_SETCURSOR` | `wndproc.rs:164`–`173` |
| Touchpad scroll, inertia, pinch through DirectManipulation | present, with a `WM_MOUSEWHEEL` fallback | `manipulation.rs:1`–`100`, `200`; doc 39, "タッチパッド" |
| VBlank thread and `WaitForVBlank` | present | `vblank.rs:133`, `220` |
| Fullscreen: borderless on the window's monitor | present | `state.rs:375`–`430` |
| Popups as `WS_POPUP` windows, reused and pre-created | present | `state.rs:489`, `528`, `555` |
| Drag and drop to other programs | absent: `supports_drag()` returns false and `start_drag` does nothing | `lib.rs:347`–`357`; doc 39, "ドラッグ＆ドロップ" and W4 |
| Accessibility (UI Automation) | absent | no UIA code |
| Exclusive fullscreen, GDI/Direct2D drawing, Windows 7 and 8 | absent by design | doc 39, "やらないこと" |
| Per-window threads | present: windows belong to the thread that made them | `lib.rs:470`; doc 39, "方針" |

- Behaviour, the loop: the window procedure puts `WindowEvent`s into a `Shared` box and never touches `State` (`wndproc.rs`; doc 39, "イベントループ"); `Session::wait_until` sleeps in `MsgWaitForMultipleObjectsEx` on a wake event object and the next deadline, and a modal loop during a live resize is pumped from `WM_SIZE` and `WM_TIMER` (`wndproc.rs`, doc 39, "イベントループ", bullet 「モーダルなループ」). `TranslateMessage` creates `WM_CHAR`, which is taken in the same `WM_KEYDOWN` handling for `text`; `VK_PROCESSKEY` is not passed to core (`wndproc.rs:621`), and repeat comes from bit 30 (`wndproc.rs:638`).
- Behaviour, presentation: `anti-webui-wgpu`'s DXGI swapchain or wgpu's surface on a DirectComposition visual; the DWM dirty rectangles and buffer age are [A4b](audit-capabilities-rendering.md#damage-tracking)'s. The window is cloaked until the first frame and shown after the next vblank (doc 39, "提示", bullet 「最初のフレームまで隠す」).
- Quality notes: 10 unit tests (`manipulation.rs` 4, `keyboard.rs` 3, `stack.rs` 2, `ime.rs` 1) and a 23-scenario program (`tests/windows.rs`), which the document says passes on Windows 11 at 175% (2026-11-08). 122 `unsafe` occurrences, 116 documented within three lines; of the six without, four are `unsafe fn` or `unsafe extern` items and two are calls (`testing.rs:100`, `lib.rs:663`). The IME is IMM32, not TSF; doc 39's comparison table says so, and `rg 'ITf|TextServices' src/anti-webui-windows/src` finds nothing (Confirmed).
- Label: Confirmed for the code paths; run-time behaviour is documented upstream and Unverified here

### Multi-window

- Status: present, with limits
- Evidence: `window.rs`: `WindowId` (24), `WindowOptions` (82), `Role` (106, `Toplevel`, `Layer(LayerSpec)`, `Lock(connector)`), `Command` (201), `Info` (351), `Parked` (370), `activate` (476), `within` (510), `start_ui_thread` (797), `start_root` (850), `run_roots` (904), `open_window` (966), `spawn_window` (999), `spawn_ui_thread` (1046). App: `Thread` (`runner.rs:744`), `spawn` (946), `start_thread` (963), `run_on` (1021). Design document: `36-multi-window.md`, sections "API", "スレッドの選び方", "中の作り：core", "中の作り：app", "中の作り：バックエンド", "窓の飾り", "環境ごとの違い", "実装の状態".
- Behaviour: `open_window` queues a window for the current thread and returns a handle at once; `spawn_window` queues a thread of its own; `spawn_ui_thread` starts a thread with no window. Same-thread windows share the reactive runtime, fonts, settings, and the `ui_channel`; a spawned thread gets a copy of the settings and the fonts registered so far, and shares signals only through `shared_signal` or `ui_channel`. Each window keeps its own `Document`, timers, frame callbacks, accessibility watchers, and scale and fullscreen signals; unused windows are parked and swapped in with `mem::swap` on switching (`Parked::swap`, `activate`). A window closes with the one that opened it unless `top_level`; a `dialog` or `modal` window is also told to the platform as a child (`Spec::of`, `runner.rs:528`). The first window of `App::run` ends the program.
- Behaviour, popups versus windows: a popup is a `PopupId` surface of a window, not a `WindowId`. It has no `Ui` of its own; its content is a `NodeKind::Popup` subtree of the window's document, laid out and painted by core (doc 09), and the `Runner` presents the popup's `Scene` to its own surface (`runner.rs:314`). Backends map `PopupKind::Menu`, `Tooltip`, and `Bubble` onto `xdg_popup` (menus and tooltips) and `wl_subsurface` (bubbles) on Wayland, override-redirect windows on X11 (a child window for a bubble) and `WS_POPUP` windows on Windows, and borderless child `NSWindow`s on macOS; the placement is `place_popup` on X11, macOS, and Windows, and an `xdg_positioner` on Wayland.
- Quality notes: 11 tests in core's `tests/windows.rs` cover a signal written in one window changing the other window's document, timers and cleanups in the window that made them, a prevented close request, a modal window blocking its parent, a handle sending and closing, a `NodeRef` from another window's handler, the frame and state following the platform, each window saying what draws it, a window-less thread closing what it opened, a window following its content size, and a window-less thread animating on its own frames. Every backend has an `OWN_THREADS = true` path; Wayland's connection sharing has no protocol test for a second toplevel (doc 36 lists that under "まだのもの"). There is no API to set a window's position, and no way to attach an arbitrary small window to another as a child; the parent relationships are the dialog or modal one and the `Bubble` popup. Doc 36's "位置の指定と、親に貼り付く小窓" are listed as later work.
- Label: Confirmed

### Accessibility API

- Status: partial: core's accessibility tree and actions exist; no OS accessibility service is wired, and no bridge crate is in the workspace.
- Evidence: `access.rs` (552 lines): `Role` (16, 39 variants), `Checked` (94), `Live` (109), `Access` (132), `AccessNode` (228), `AccessTree` (254), `AccessAction` (277, five), `snapshot` (297), `perform` (304), `watch` (357), `tell_watchers` (371), `build` (390), `make` (458), `Made::Through` (449) for boxes that say nothing. `view.rs:868` gives `.access(f)`; `ui.rs:299` tells watchers at the end of an update and `ui.rs:303`–`343` performs the actions. Tests: `tests/accessibility.rs` 7. Design document: `34-accessibility.md`, sections "方針", "書き方", "自分で決まるもの", "橋渡しのための API", "標準の部品と MD3E", "やっていないこと". `07-roadmap.md:57` lists AccessKit as future work.
- Behaviour: the tree is made only when `snapshot` is called or a watcher exists; a node that has no role, no label, and no focusability is left out and its children go to the parent, as Chromium does. The role is taken from `.access`, or worked out for text, edits (single or multi-line), editors, images with a label, and named-from-content roles. The label otherwise comes from the content. Bounds are logical pixels, converted from `window_rect`. `watch` calls back at once and then only when the tree changed; colour-only repaints do not rebuild it. `perform` queues `Focus`, `Click`, `Increment`/`Decrement` (sent as the Up and Down keys) or `ScrollIntoView` for the next update. There is no such bridge in the workspace: `rg -n -i 'accesskit|atspi|at-spi|NSAccessibility|UIAutomation|IRawElementProvider|accessib' src` finds only `anti-webui-core/src/access.rs` and its test, and `Cargo.lock` has no `accesskit` or `atspi` (zbus comes from the shell crate's D-Bus services).
- Quality notes: 7 tests cover the omitted box, name from content, label and states, hidden subtrees, text input's value, watchers hearing of changes only, focus and press, and increment as the Up key. The widgets attach roles to buttons, toggles, switches, checkboxes, radios, sliders, tabs, menus, selects, lists, trees, tooltips, dialogs, toasts, and progress bars (`rg -c '\\.access\\(' src/anti-webui-widgets/src`); MD3E inherits them through the widgets' motion layer (`34-accessibility.md`, "標準の部品と MD3E"). `AccessTree::get` is a linear scan, and the whole tree is rebuilt and compared on every change while someone watches. No OS-level test can exist yet, and there is no `SetValue` action and no caret or selection range in the tree, as doc 34's "やっていないこと" says.
- Label: Confirmed

### Drag and drop

- Status: partial: in-window and OS drags work on Wayland, X11, and macOS; Windows has in-window drags only; a typed drag does not cross windows.
- Evidence: core `drag.rs`: `Mode` (505, `Inside`, `Platform`, `External`), `DragSession` (522), `ExternalDrag` (93), `set_platform_drag` (707), `start_drag` (844), `gesture_released` (797), `cancel_gesture` (810); the mode is `Platform` when the platform supports drags and the item list is non-empty, otherwise `Inside` (`drag.rs:1015`–`1052`). App: `set_platform_drag(ctx.supports_drag())` in `Runner::set_waker` (`runner.rs:110`). Backends: Wayland `drag.rs` (358 lines), X11 `xdnd.rs` (800), macOS `dnd.rs` (488), Windows `supports_drag() == false` (`lib.rs:347`). Design document: `23-drag-and-drop.md`, sections "第 1 層", "第 2 層", "第 3 層", "選択した文字のドラッグ", "並べ替え", "core に足すもの", "既存の部品の置き換え", "測ったもの", "確認していないこと".

| Where the drag goes | Wayland | X11 | macOS | Windows |
|---|---|---|---|---|
| Inside one window | present, core | present, core | present, core | present, core |
| To another window of the same program | through the OS as `ExternalDrag`, not as the typed value | through XDND, likewise | through AppKit, likewise | absent with OS DnD |
| To another program | present when the data device manager is v3+ | present, XDND 5 | present, `NSDraggingSource` | absent |
| From another program | present | present, XDND 5 | present, `NSDraggingDestination` | absent |

Label: Confirmed, from the file and tests named in the subsection.

- Behaviour: a drag whose payload is only a Rust type never leaves the window: the release is hit-tested in the same `Document`, so another window cannot receive it as that type. With external payloads, the platform takes over and a drop into another window arrives as `ExternalDrag` and the offered MIME formats; doc 36 records the same state ("同じスレッドの窓の間のアプリ内 DnD（今は他のアプリとの DnD と同じ道を通る）"). Escape cancels a `Platform` drag through `PlatformRequest::CancelDrag`, and core asks for `CloseDragIcon` after the source says it ended. On Wayland the icon is a `wl_surface`; on X11 an override-redirect ARGB window; on macOS an `NSImage` made from the freshly drawn `IOSurface` (`dnd.rs:274`).
- Quality notes: core has `tests/drag.rs` 53 and `tests/text_drag.rs` 42, and `tests/drag.rs` in `anti-webui-app` adds 6 pixel tests. Wayland has `drag_protocol.rs`; X11 has the two server tests for drops in and out; macOS's 20-scenario program includes a drop and a file drop; Windows has none, and doc 39 puts OLE DnD at W4. The X11 `supports_drag` is `xi_opcode.is_some()` (`xdnd.rs:355`), because an outgoing drag grabs the pointer through `xinput_xi_grab_device` (`xdnd.rs:380`–`397`), so a server without XInput 2 gets no OS drags even though XDND itself would work. The document's test counts are behind the code: it says 48 and 41, the code has 53 and 42 (Confirmed; see [Code against the design documents](#code-against-the-design-documents)).
- Label: Confirmed; cross-application behaviour on real desktops is documented upstream and Unverified here

### Fullscreen

- Status: present, for the window and for one element inside it
- Evidence: `27-fullscreen.md`, sections "API", "要素のフルスクリーンの決まり", "描画と配置", "切り替えの順番", "Wayland", "スキャンアウト", "後回しにするもの". Core: `NodeRef::request_fullscreen` (`view.rs:87`), `is_fullscreen` (95), `window_fullscreen` (388), `set_window_fullscreen` (394), `exit_fullscreen` (399), `Document::request_fullscreen` (`tree.rs:752`), `exit_fullscreen` (762), `window_fullscreen_changed` (777). Backends: Wayland `set_fullscreen` (`lib.rs:209`) and `configure` states (`state.rs:676`); X11 `_NET_WM_STATE` (`state.rs:612`) and `_NET_WM_BYPASS_COMPOSITOR` (1029); macOS `toggleFullScreen` (`state.rs:275`) and the delegate callbacks (`view.rs:888`); Windows borderless fullscreen with a saved `WINDOWPLACEMENT` (`state.rs:375`).
- Behaviour, the element: request puts the element in the window's fullscreen state, the window is asked to go fullscreen too unless it already is, `WindowEvent::Fullscreen` and `Resized` arrive together, and only then does the element take over the tree, layout, positioning, painting, and hit-testing; leaving reverses it. Escape exits, focus moves in and back, and Tab and focus cannot leave while it lasts. If the compositor or window manager refuses, no `Fullscreen` event comes and the request stays until the next `exit_fullscreen` or `request_fullscreen`, exactly as doc 27's "切り替えの順番" says.
- Behaviour, the window: each backend only toggles the window; the element logic is shared. Wayland's `xdg_toplevel.set_fullscreen(NULL)` asks the compositor to choose the output, and a `0 × 0` configure after leaving restores the previous size (protocol test `fullscreen_protocol.rs`). X11 sends `_NET_WM_STATE_ADD/REMOVE` and follows `PropertyNotify`; while fullscreen it sets `_NET_WM_BYPASS_COMPOSITOR = 1`. macOS enters another space with `toggleFullScreen:`; Windows removes the frame to `WS_POPUP` over the monitor and restores the previous style and placement. No backend uses exclusive fullscreen.
- Behaviour, scanout: `DmabufFeedback::scanout_modifiers` (`platform.rs:309`) takes the first main-device tranche only when it is marked `scanout`, and `wanted_modifiers` (`present/dmabuf.rs:120`) offers those first if the GPU can render to any, else all modifiers. This is the change doc 27's "スキャンアウト" describes: its "今は全 tranche の修飾子をまとめてドライバに選ばせている" is the state before the change, the next bullet is the change, and the paragraph after it records the result on ShojiWM, so code and document agree (Confirmed, reviewer check). Whether a compositor actually scans out is not visible from here; doc 27 says the same ("コンポジタが実際に直接画面に出したかどうかは、こちらからは確かめられていない").
- Quality notes: core's `tests/fullscreen.rs` has 18 tests; Wayland's protocol test covers starting fullscreen, entering, and leaving with a `0 × 0` configure. There is no X11, macOS, or Windows test for fullscreen specifically beyond their end-to-end scenarios (Windows's 23 include maximize and fullscreen and the return). The element fullscreen state is entirely core, so a backend that ignores `SetFullscreen` leaves the element unshown and the request pending (doc 27: 「コンポジタが断ると何も起きず、要求は残る」).
- Label: Confirmed

### IME per backend

- Status: present on all four; what each can tell the input method differs.
- Evidence: core's model is `ImeState` (`platform.rs:43`: `surrounding`, `cursor`, `anchor`, `cursor_rect` in window physical pixels, `multiline`, `purpose`) sent as `PlatformRequest::SetIme(Option<ImeState>)`; core sends it only when it changes and after layout settles (`ui.rs:2326`). Backends: Wayland `state.rs:466`–`516` and `1184`–`1234`; X11 `ime.rs` (281 lines) and `state.rs:633`; macOS `view.rs:536`–`660`; Windows `ime.rs` (227 lines) and `wndproc.rs:538`. Design documents: `06` "IME", `35` "IME", `38` "IME", `39` "IME", and `04-text.md` for the model ([A4a](audit-capabilities-core.md#selection-editing-and-ime)).

| What the model offers | Wayland (`zwp_text_input_v3`) | X11 (XIM) | macOS (`NSTextInputClient`) | Windows (IMM32) |
|---|---|---|---|---|
| Inline preedit | yes, `preedit_string` | yes, `preedit_draw` callbacks | yes, `setMarkedText:` | yes, `GCS_COMPSTR` with attributes |
| Commit | `commit_string` | `commit` | `insertText:` | `GCS_RESULTSTR` |
| Delete around the caret | `delete_surrounding_text` | no | no (core deletes) | no |
| Surrounding text | yes, `set_surrounding_text` | no (doc 35: "周りの文字（surrounding text）がない") | no (`attributedSubstringForProposedRange:` returns nil) | no (reconversion is future) |
| Caret rectangle for candidates | `set_cursor_rectangle`, surface-local logical | `SpotLocation`, window physical | `firstRectForCharacterRange:`, screen | `ImmSetCandidateWindow` + `ImmSetCompositionWindow` |
| Content purpose (password, number, URL) | `set_content_type` with hints; password marked sensitive | only password: the IC is unfocused | not used | not used |
| Stands down when nothing editable has focus | `disable` + `commit` | `unset_focus` | `ImeState == None` stops `interpretKeyEvents:` | context detached, composition cancelled |

- Behaviour: Wayland is the only backend that passes surrounding text and a content purpose; X11 passes on-the-spot preedit and the caret position, and disables the input context for a password because XIM cannot say why; macOS passes marked text and keeps the marked and selected ranges in UTF-16, and Windows passes composition strings and their clause attributes and puts the candidate window at the caret. Core owns the order of applying an `ImeUpdate` (remove old preedit, delete, insert commit, set new preedit), so the backends only report.
- Quality notes: the X11 IME module has no unit tests; Windows has one (`the_target_clause_is_where_the_attributes_say`); macOS's IME is exercised by the end-to-end program (compose, commit, Delete, candidate rectangle) and not by a unit test; Wayland's protocol tests do not drive text input. `purpose` is ignored by macOS and Windows (`rg 'ImePurpose|purpose' src/anti-webui-{macos,windows}/src` finds nothing), so a password field can still be composed into there; the masking itself is core's ([A4a](audit-capabilities-core.md)). Real input methods (fcitx5, IBus, Microsoft IME, macOS Japanese input) were not run here; doc 35 says fcitx5 and IBus both serve XIM, and doc 39 says the Windows IME is unverified with a real one.
- Label: Confirmed

### Clipboard, cursor, and high-DPI scale

- Status: present on all four; the units and the scale model differ.
- Evidence: `platform.rs` `ClipboardKind`, `CursorIcon` (re-exported from the style crate, 12 variants including `None` and `Copy`), `ImePurpose`; `CursorIcon` at `src/anti-webui-style/src/types.rs:386`. Backends: Wayland clipboard in `state.rs:518`–`611`, cursor in `state.rs:433`–`463`, scale in `state.rs:408`–`431` and `725`–`756`; X11 `selection.rs`, `cursor.rs`, `settings.rs`, `state.rs:545`; macOS `state.rs:245`, `view.rs:146`–`180` and `683`; Windows `clipboard.rs`, `wndproc.rs:164`–`173` and `194`.

| Concern | Wayland | X11 | macOS | Windows |
|---|---|---|---|---|
| Clipboard | `wl_data_device`, plus `zwp_primary_selection_v1`; reads and writes on a thread | CLIPBOARD and PRIMARY selections, INCR, `SAVE_TARGETS`; all in the event loop | `NSPasteboard.general`, strings only; no primary | `CF_UNICODETEXT`, open retried; no primary |
| Cursor | `wp_cursor_shape_v1` only; no theme fallback | Xcursor theme, CSS name then older names | `NSCursor` mapping; `None` hides and unhides | `IDC_*`; `None` sets a null cursor |
| Scale source | `wp_fractional_scale_v1` preferred scale; integer `preferred_buffer_scale` fallback | one global scale: XSETTINGS `Xft/DPI`, else `Xft.dpi`, else 96; `ANTIWEBUI_SCALE` overrides | `backingScaleFactor`, 1 or 2 | `GetDpiForWindow`, per monitor, `Scale(dpi × 120 / 96)` |
| Scale granularity | `n/120`, from 1 upward | `n/120` from DPI, but the same for every window | integer only; a scaled display is drawn at 2× and shrunk by the system | `n` = 120, 150, 180, 240, 210 at 100%, 125%, 150%, 200%, 175%; exact in `n/120` |
| Change at run time | `preferred_scale`, then `Resized` | XSETTINGS or `RESOURCE_MANAGER` change, then `Resized` at the same window size | `viewDidChangeBackingProperties`, then `Resized` | `WM_DPICHANGED`, `SetWindowPos` to the suggested rectangle, then `Resized` |
| Physical buffer | `round(logical × n / 120)` with viewporter destination | the window's physical size from `ConfigureNotify` | `round(bounds × factor)` | the client area from `WM_SIZE` |
| Popup scale | `PopupParams.buffer` and `logical`, own viewport | `params.buffer` and `logical`; physical windows | `contentsScale` on the popup's layer | per-monitor DPI of the popup's window |

- Behaviour: core's integer model is the same everywhere ([A4a](audit-capabilities-core.md#layout-engine)); what differs is where the scale comes from and what the platform can express. Wayland and X11 can hand core a fractional `n/120`; macOS cannot (1 or 2); Windows can, but only per monitor and only in DPI steps. X11's single scale is the largest functional difference: moving a window to a monitor with another DPI does not rescale it in this backend, which doc 35 says is by design ("X11 にはウィンドウごとのスケールがない"). Wayland's cursor without `wp_cursor_shape_v1` is the other user-visible gap: `apply_cursor` returns before doing anything when `cursor_device` is `None` (`state.rs:445`), so only `CursorIcon::None` has an effect (Inferred from the code path).
- Quality notes: X11 has unit tests for XSETTINGS parsing, DPI to scale, cursor names, and Latin-1 (`settings.rs` 2, `cursor.rs` 1, `selection.rs` 1); macOS has 4 for input (scroll directions and phases) and 2 for the pasteboard MIME mapping; Windows has 0 for the clipboard and cursor, and 4 for the DirectManipulation transform-to-gesture mapping; Wayland has 4 unit tests in `state.rs`, one of which (`fingers_begin_move_and_lift`) is about scroll phases, not scale. The scale paths themselves have only the X11 server test (`the_window_opens_at_the_scale_of_the_dpi`); Wayland's fractional scale, macOS's 1×/2× switch, and Windows's 175% are covered by the end-to-end programs or not at all.
- Label: Confirmed; "Inferred" on the missing Wayland cursor fallback's visible effect

### anti-webui-wayland-shell

- Status: present
- Evidence: `lib.rs` (614 lines): `Shell` (222), `LayerOptions` (294), `LayerHandle` (468), `open_layer` (541), `spawn_layer` (549), `run` (261, refuses without a compositor or `zwlr_layer_shell_v1`), `prelude` (128); `output.rs`, `workspaces.rs`, `toplevels.rs`, `idle.rs`, `clipboard.rs`, `capture.rs`, `lock.rs`, `apps.rs`, `process.rs`, `system.rs`, `audio.rs`, `dbus.rs`, and per-compositor `hyprland.rs`, `niri.rs`, `sway.rs`. `Cargo.toml` features: always-on Wayland, workspace, toplevel, idle, clipboard, capture, and lock support; `hyprland`, `niri`, `sway` (serde_json); `network`, `battery`, `power-profiles`, `bluetooth`, `media`, `notifications`, `tray`, `session` (all through the internal `dbus` feature); `audio` (PipeWire loaded with `libloading`); `apps`, `process`, `system`. Design document: `37-wayland-shell.md`, sections "API", "レイヤーサーフェス", "ディスプレイ", "シェル用の機能", "一覧", "Wayland の機能と接続", "汎用ワークスペース", "Hyprland、niri、sway", "ウィンドウ一覧、アイドル、クリップボード", "画面ロック", "クレートの形", "環境ごとの違い", "やらないこと", "実装の状態".
- Behaviour: it is a toolkit for a desktop shell, not another window backend. A shell opens layer surfaces (`open_layer`, `spawn_layer`, one UI thread per output if wanted) on the Wayland backend, and gets signals for outputs, workspaces, toplevels, idle, clipboard history, thumbnails, media, batteries, network, Bluetooth, notifications, tray, audio, installed applications, child processes, and system counters. Workspaces prefer a compositor's IPC when `HYPRLAND_INSTANCE_SIGNATURE`, `NIRI_SOCKET`, or `SWAYSOCK` is set, then `ext_workspace_v1`, else are unavailable; toplevels use `wlr_foreign_toplevel_management` or `ext_foreign_toplevel_list`; capture uses `ext_image_copy_capture_v1` or Hyprland's export protocol. `Shell::run` requires a Wayland compositor with `zwlr_layer_shell_v1` and therefore fails on GNOME; X11 has no layer surfaces (`x11/lib.rs:530`), and `App::run_shell` (`runner.rs:1048`) runs on Wayland only.
- Quality notes: 66 tests with `--all-features`, 9 `#[ignore]`d live tests; the non-ignored ones use a private `dbus-daemon` with fake services (`*_bus.rs`), a fake IPC server (`hyprland_ipc.rs`, `niri_ipc.rs`, `sway_ipc.rs`), temporary `.desktop` files, a real child process, and this machine's `/proc` and `/sys`. The layer and lock protocols have Wayland protocol tests (`layer_protocol.rs`, `lock_protocol.rs`). Doc 37 says only ShojiWM was verified on a real compositor (2026-11-07), and that Hyprland, niri, and sway were checked against fake servers, not the real ones; the other compositor rows rest on public information. 101 `unsafe` occurrences, 52 documented: 37 are in `audio.rs` (the PipeWire C callbacks), and the rest are `libc` calls and `unsafe extern` items in `apps.rs`, `session.rs`, `time.rs`, `capture.rs`, and `system.rs` (Confirmed by their locations).
- Label: Confirmed for the structure and tests; Unverified for every compositor except ShojiWM

### Code against the design documents

Each item was read twice, in both the code and the whole document, before being listed.

| Where | The document says | The code does | Label |
|---|---|---|---|
| `35-platform-x11.md`, "確かめ方": 「単体テスト（サーバーなし、15 件）」 and 「X サーバーを使うテスト（`tests/server.rs`、8 件）」 | 15 and 8 | 13 `#[test]` in `src/` and 9 in `tests/server.rs` | Confirmed (the counts have drifted; no behaviour follows) |
| `23-drag-and-drop.md`, "測ったもの": 「`anti-webui-core/tests/drag.rs`: 合成イベントでの動作（48 件）」 and 「`text_drag.rs`: …（41 件）」 | 48 and 41 | 53 and 42 from `cargo test --list`; [A4a](audit-capabilities-core.md#state-machines) counted the same | Confirmed (ageing counts) |
| `35-platform-x11.md`, "やらないこと": 「`XdndProxy`。デスクトップのアイコンなど、代理のウィンドウで受けるものには落とせない。」 | XdndProxy is not done | No proxy handling exists, but `xdnd_version`'s doc comment says "going through `XdndProxy` if it has one" (`xdnd.rs:672`), while its body reads only `XdndAware` (`:679`) | Confirmed (comment ahead of code; document correct) |
| `27-fullscreen.md`, "後回しにするもの": 「X11、Windows、macOS（バックエンド自体がまだ）」 | No backends then | All four backends exist and each implements `set_fullscreen` (Wayland `lib.rs:209`, X11 `state.rs:612`, macOS `state.rs:275`, Windows `state.rs:375`) | Confirmed (the statement was about that time) |

What matched, having been compared and not listed above: doc 06's protocol table against the binds (including `wp_cursor_shape_v1` and `wp_pointer_gestures_v1` as optional, and dmabuf v4), its 100 ms popup wait, `AXIS_MOTION_GAIN`, and the `Held` phase from `zwp_pointer_gesture_hold_v1`; doc 35's extension table against the X11 crate's `x11rb` features and its XIM-on-the-spot, INCR, and `SAVE_TARGETS` behaviour; doc 38's scale (1 or 2), clipboard, and per-thread windows; doc 39's per-monitor DPI, IMM32, DirectManipulation, cloaked first frame, and popup reuse; doc 36's thread table, close ordering, modal blocking, and decoration rules; doc 37's services and feature names. Label: Confirmed.

## Baseline violations

These are what upstream is today; fork changes must not add to them.

- No OS accessibility bridge on any platform: core's tree has no consumer outside tests (`34-accessibility.md`, "やっていないこと"; `07-roadmap.md:57`).
- Windows has no OS drag and drop (`windows/lib.rs:347`) and uses IMM32 rather than TSF (`39-platform-windows.md`, W4 and W6).
- Wayland sets no cursor at all without `wp_cursor_shape_v1` (`state.rs:445`), and never loads an Xcursor theme as a fallback.
- macOS and Windows ignore `ImePurpose` (`rg 'ImePurpose|purpose'` finds nothing in either crate), so a password field still composes there; X11 and Wayland handle it.
- X11's `supports_drag` depends on XInput 2 rather than on XDND (`xdnd.rs:355`), so an XI2-less server loses OS drags it could otherwise do.
- macOS: 126 `unsafe` occurrences, 17 with `// SAFETY:` within three lines; the rest are mostly `#[unsafe(method(...))]` and `unsafe impl` forms A7 will classify.
- Nine live tests in `anti-webui-wayland-shell` are `#[ignore]`d and read the machine; a normal run reports success without them.
- `anti-webui-app`'s tests cannot link on this machine without `libxkbcommon-x11`, so the X11 test list is only count-by-source here.
- The design documents' test counts have drifted (two places above), and `27`'s "後回しにするもの" still lists the X11, Windows, and macOS backends as missing.

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| Four hand-written protocol/API backends, each with its own event loop, input translation, popup placement, and DnD | A mistake is invisible to the other three and often only shows on a real compositor, WM, or OS; the surface is about 19,000 source lines | Certain | [Method](#method); [Wayland backend](#wayland-backend), [X11 backend](#x11-backend), [macOS backend](#macos-backend), [Windows backend](#windows-backend) |
| The only macOS run-time checks are two harness-free programs on one machine (M1 Pro, macOS 26) | Regressions on Intel, 1× screens, or macOS 13 are unmeasured; the doc itself lists live resize, real IME, trackpad, and drags out as still manual | High | `38-platform-macos.md`, "確かめたこと" and "まだ確かめていないこと"; [Method](#method) |
| Windows' real IME and real touchpad are unverified; the 23 scenarios cannot compose IME text for real | A wrong candidate position or a stuck composition reaches users first | High | `39-platform-windows.md`, "まだ確かめていないこと"; [IME per backend](#ime-per-backend) |
| X11 has one global scale and no per-monitor handling | Mixed-DPI setups show the wrong size on one monitor; doc 35 accepts it as an X11 limit | Medium in practice | `35-platform-x11.md`, "やらないこと"; `settings.rs:85` |
| A typed drag cannot leave its window as a type | Kanban or tree moves across two windows need `ExternalDrag` plus MIME handling, or copying between documents by hand | Medium | [Drag and drop](#drag-and-drop); `drag.rs:1015`–`1052`; `36-multi-window.md`, "実装の状態" |
| No accessibility bridge, and the tree is rebuilt and compared while anyone watches | Any Phase 2 requirement about screen readers starts with a new crate and probably AccessKit, not with a setting | High if accessibility is in scope | [Accessibility API](#accessibility-api); `34-accessibility.md`, "やっていないこと" |
| Wayland without `wp_cursor_shape_v1` has no cursor | On a compositor without it, every `SetCursor` is silently ignored, which is hard to diagnose | Low (most compositors have it) | `state.rs:438`–`463` |
| The macOS main thread does all AppKit input and window work | A busy main thread delays all input, even for windows drawn on their own threads (doc 38 says drawing continues) | Medium | `38-platform-macos.md`, "複数ウィンドウ"; [The platform abstraction](#the-platform-abstraction) |
| The `Host` seam carries 52 methods, many Linux-presentation-specific, with empty bodies elsewhere | Each new OS pays for a large trait, and empty methods hide "not supported" from core | Certain | `host.rs:20`–`115`; `windows/lib.rs:347`–`357` |
| `anti-webui-wayland-shell` needs compositor protocols or IPC; GNOME is excluded | A shell requirement can rule out GNOME, and every compositor row except ShojiWM is unverified | Medium for a shell product | [anti-webui-wayland-shell](#anti-webui-wayland-shell); `37-wayland-shell.md`, "環境ごとの違い" |
| Documentation statements age quickly (counts, "not yet" lists, a stale XdndProxy comment) | An audit or a reader trusts a number or a limit that is no longer true | Certain (five examples) | [Code against the design documents](#code-against-the-design-documents) |

## Open questions

- Do the Wayland and X11 paths work on a real session, not only against mock and Xvfb servers? Answered by A5 running the showcase, widgets, and multi-window examples on ShojiWM, GNOME, KDE, plain Xorg, and XWayland, with `ANTIWEBUI_DEBUG_DAMAGE=1` and `ANTIWEBUI_PERF_OVERLAY=1`.
- Does `wp_fractional_scale_v1` plus `wp_viewporter` produce the same pixel sizes on Mutter and KWin as on ShojiWM? Answered by the same A5 run, comparing window screenshots at 1.25, 1.5, and 1.8.
- Does a real IME (fcitx5, IBus, Microsoft IME, macOS Japanese input) show the preedit inline and the candidates at the caret on each backend? Answered by manual tests on each OS; the code paths are [IME per backend](#ime-per-backend).
- Can `perform`'s `Click` and `Increment` be driven from a real screen reader once a bridge exists, including on a node that is scrolled out of view? Answered by a Phase 4 spike with `accesskit_unix` and the 7 existing tests.
- Is an OS round trip through `ExternalDrag` enough for dragging between two windows of one program, or is an in-process cross-window mode needed? Answered by a spike that drags a typed card from one `open_window` to another and records what the target sees.
- How does X11's implicit-sync path behave under plain Xorg with a compositor, and do popups keep their shape without one? Answered by A5 on an Xorg session; doc 35 says this is untested.
- Is the XIM limit worth lifting by talking to fcitx5 and IBus over D-Bus, which would also carry surrounding text and input purpose on X11? Answered by a Phase 4 spike against both, measured against the existing XIM path; doc 35 lists this as future work.
- Which compositors does `anti-webui-wayland-shell` actually work on beyond ShojiWM? Answered by running `shell_bar` on Hyprland, niri, and sway with `--features shell-all`; doc 37 says this has not been done.

## Inputs for gap analysis

- [Checklist](#checklist): the OS backends and multi-window are present; accessibility is the one checklist item that is only half there (core API, no OS service); fractional scaling is expressed differently on each backend, so a requirement must say which platform model it means.
- [The platform abstraction](#the-platform-abstraction): the seam is `PlatformRequest` (22) and `WindowEvent` (22), with `Host` at 52 methods and `Platform` at 14. A requirement that adds a platform feature costs one request, one event, and up to four backend implementations.
- [Wayland backend](#wayland-backend), [X11 backend](#x11-backend), [macOS backend](#macos-backend), [Windows backend](#windows-backend): the feature tables are the per-OS inventory. Windows is the weakest (no OS DnD, IMM32, real IME unverified); Wayland is the richest (layer and lock surfaces, fractional scale, explicit sync).
- [Accessibility API](#accessibility-api): any requirement about screen readers is a new bridge crate, not a port; the core tree and its 7 tests are the starting point, and `SetValue` and text ranges do not exist.
- [Multi-window](#multi-window): windows exist, positions and parent-attached child windows do not, and a typed drag cannot cross a window boundary. A requirement naming any of those is new work.
- [Drag and drop](#drag-and-drop): OS DnD is present on three platforms and absent on Windows; the in-window path is fully covered by core tests.
- [Fullscreen](#fullscreen): window and element fullscreen both exist and are shared across backends; scanout preference is implemented but not observable from here.
- [IME per backend](#ime-per-backend): only Wayland carries surrounding text and purpose; X11 carries no surrounding text and turns the input context off for passwords, and macOS and Windows ignore purpose. A requirement about password fields or reconversion must name its target OSes.
- [Clipboard, cursor, and high-DPI scale](#clipboard-cursor-and-high-dpi-scale): primary selection exists only on Wayland and X11; X11 is one-scale-per-display; macOS is integer-scale only.
- [Code against the design documents](#code-against-the-design-documents): five statements are stale; A8 should cite this table rather than re-derive it.
