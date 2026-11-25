# Audit: platform matrix

- Date: 2026-11-26
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Auditor: Pi, DeepSeek V4.1 Flash (draft); cross-checks, review, and corrections by Claude Code, Claude Opus 5.5
- Plan item: [A5](phase-1-audit-plan.md#workstreams)

## Scope

For each OS and display server: what builds, what runs, what has been tested, and how the tests get by without a session. The crates are `anti-webui-app` (backend choice, target gates, test entry points), `anti-webui-wayland`, `anti-webui-x11`, `anti-webui-wayland-shell`, `anti-webui-macos`, `anti-webui-windows`, and `anti-webui-wgpu`'s backend order; the inputs are the manifests, `rust-toolchain.toml`, and the tests under `src/anti-webui-app/tests`, `src/anti-webui-wayland/tests`, `src/anti-webui-x11/tests`, and `src/anti-webui-wayland-shell/tests`.

The upstream documents read as the yardstick are `docs/design/06-platform-wayland.md`, `35-platform-x11.md`, `36-multi-window.md`, `37-wayland-shell.md`, `38-platform-macos.md`, `39-platform-windows.md`, `07-roadmap.md`, and `README.md`.

Left out: what each backend can do ([A4c](audit-capabilities-platform.md)); test counts, failures, and coverage beyond the numbers the matrix cites ([A6](audit-tests.md)); crate layers and the dependency graph ([A1](audit-dependencies.md)); measurements (A9); lints (A7). No window was opened on any platform for this file. Builds were type-checks only (`cargo check`): nothing was linked for macOS or Windows.

## Method

Reading order: the plan's [Limits of this machine](phase-1-audit-plan.md#limits-of-this-machine) and [Exit criteria](phase-1-audit-plan.md#exit-criteria); then A4c and A6; then the eight documents above; then `Cargo.toml` at the root and in each backend crate, `rust-toolchain.toml`, `runner.rs`, `present.rs`, and `src/anti-webui-wgpu/src/lib.rs`; then the test entry points and their `#![cfg]` lines.

| Question | Command |
|---|---|
| Branch and base | `git status --short`, `git log --oneline -3` |
| Installed Rust targets | `rustup target list --installed` |
| Target-gated dependencies | `rg -n "target\.'cfg" src/*/Cargo.toml` |
| Test binaries without a harness | `rg -n '\[\[test\]\]\|harness' src/*/Cargo.toml` |
| Backend chosen at run time | `rg -n 'fn choose_backend\|ANTIWEBUI_BACKEND\|WAYLAND_DISPLAY\|can_connect' src/anti-webui-app/src/runner.rs src/anti-webui-wayland/src/lib.rs` |
| The gate in each backend | `rg -n '^#!\[cfg' src/anti-webui-*/src/lib.rs` |
| The app's gates | `rg -n 'cfg\(unix\)\|cfg\(not\(unix\)\)\|cfg\(target_os = "linux"\)\|cfg\(windows\)\|cfg\(target_os = "macos"\)' src/anti-webui-app/src/*.rs src/anti-webui-core/src/platform.rs src/anti-webui-video/src/*.rs` |
| GPU backend order and override | `rg -n 'Backends::\|WGPU_BACKEND' src/anti-webui-wgpu/src/*.rs` |
| Every environment variable the app reads | `rg -n 'ANTIWEBUI_[A-Z_]+\|WGPU_[A-Z_]+\|VK_[A-Z_]+' src --glob '*.rs'` |
| Test files per OS | `rg -n '#!\[cfg' src/anti-webui-{wayland,x11,wayland-shell,app}/tests/*.rs` |
| How platform tests run headless | read `src/anti-webui-wayland/tests/*_protocol.rs`, `src/anti-webui-x11/tests/server.rs`, `src/anti-webui-app/tests/{macos,macos_spawn,windows}.rs` |
| Test counts per backend | `rg -c '#\[test\]' src/anti-webui-{wayland,x11,wayland-shell}/src src/anti-webui-{wayland,x11,wayland-shell}/tests` |
| What upstream says it tested on | `rg -n 'ShojiWM\|XWayland\|Xorg\|M1 Pro\|Windows 11\|macOS 26\|Radeon\|RTX' docs/design/*.md` |
| What this machine has | `ls /dev/dri`, `ls /usr/share/vulkan/icd.d`, `ldconfig -p \| rg -i 'xkbcommon\|libgbm\|libvulkan'`, `command -v Xvfb ffmpeg wayland-info`, `ls /tmp/.X11-unix`, and a Python `socket.AF_UNIX` connect to `$XDG_RUNTIME_DIR/wayland-0` |

This machine, as found (all Confirmed by the commands above unless marked):

- Toolchain 1.99.0 from `rust-toolchain.toml`. Installed targets: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` (listed in `rust-toolchain.toml`), and `aarch64-apple-darwin`, added for this audit with `rustup target add` and not listed in `rust-toolchain.toml`.
- WSL2, Linux 6.18, inside the agent sandbox. `WAYLAND_DISPLAY` and `DISPLAY` are unset in this shell, while `$XDG_RUNTIME_DIR/wayland-0` is a symlink to `/mnt/wslg/runtime-dir/wayland-0` and an X socket exists at `/tmp/.X11-unix/X0`: WSLg is serving both, but this shell is not pointed at either.
- Connecting to that Wayland socket from the sandbox fails with `Operation not permitted` (Python `AF_UNIX` connect). Whether it connects from the maintainer's terminal is Unverified; what would check it is running an example outside the sandbox.
- No `/dev/dri`. Eight Vulkan ICDs are installed (`asahi`, `gfxstream`, `intel`, `intel_hasvk`, `lvp`, `nouveau`, `radeon`, `virtio`); only `lvp` (lavapipe) works without a device node, so a headless render here is software (Inferred from the missing device node and A6's adapter note).
- `Xvfb`, `ffmpeg`, and `wayland-info` are not installed, so the X11 server tests skip and video tests skip-pass ([A6, failures on this machine](audit-tests.md#failures-on-this-machine)).
- `libxkbcommon-x11.so` and `libgbm.so.1` are installed now. A4c recorded `-lxkbcommon-x11` as missing when it ran; A6's `cargo nextest list` built and linked the X11 test binaries and listed their 22 tests, so the link succeeds today (Confirmed).

Build and cross-checks:

```sh
cargo check --workspace --all-targets --locked --all-features            # Linux, 83 s
rustup target add aarch64-apple-darwin
cargo check --workspace --all-targets --locked --keep-going --target x86_64-pc-windows-msvc   # 39 s
cargo check --workspace --all-targets --locked --keep-going --target aarch64-apple-darwin     # 71 s
cargo tree --locked -i tree-sitter --target all -e normal,build --prefix none
cargo check -p anti-webui --no-default-features --lib --locked
cargo check -p anti-webui --no-default-features --all-targets --locked
```

Results (Confirmed by those runs):

- Linux with `--all-features`: passes, with the warnings recorded in [A7](audit-baseline-quality.md#clippy-and-rustc-warnings).
- Windows and macOS targets: every workspace crate type-checks except `anti-webui-code` and `anti-webui-markdown`. Those two depend on 15 `tree-sitter` crates whose build scripts compile C with cc-rs. For Windows, cc-rs finds no `lib.exe` (MSVC's archiver); for macOS, it passes `-arch arm64 -mmacosx-version-min=11.0` to the host `cc`, which rejects them. No other crate depends on the two, and the facade crates enable them only through the non-default `markdown` and `code` features (`src/anti-webui-internal/Cargo.toml:19`, `:21`).
- No backend feature: the `anti-webui` library type-checks, with dead-code warnings for the backend macros in `anti-webui-app`. The `svg` example fails with `cannot find function parse in module svg` (`src/anti-webui/examples/svg.rs:228`), because it uses the `svg` feature and has no `required-features`.

## Findings

### The matrix

Rows are OS or display server; `wayland-shell` and its feature groups get their own rows because they build and run under different conditions from the plain Wayland backend.

| Platform | Backend crate | GPU API (from the code) | Builds on this machine | Runs on this machine | Tests that exist | Upstream says tested on | Untested per upstream |
|---|---|---|---|---|---|---|---|
| Linux, Wayland | `anti-webui-wayland` | Vulkan: dmabufs from GBM, `VK_EXT_image_drm_format_modifier`; wgpu surface fallback (Vulkan, then GL) | Yes: `--all-features` check; A6 built and ran its tests | No: the compositor is unreachable from the sandbox and there is no `/dev/dri`, so the dmabuf path would fall back to a surface on lavapipe | 10: 5 unit, 5 protocol tests against a mock compositor over a Unix socket | ShojiWM only | `07`, “M1 の既知の制限”; GNOME, KDE, and other compositors |
| Linux, X11 | `anti-webui-x11` | Vulkan through DRI3 and Present; wgpu surface fallback (Vulkan, then GL) | Yes: `--all-features` check; A6 linked its tests | No: no `DISPLAY`, no Xvfb, so the 9 server tests skip | 22: 13 unit, 9 against `Xvfb :93` with XTEST | XWayland 24.1 under ShojiWM | Plain Xorg: implicit sync, popups without a compositor |
| Linux, layer surfaces and session lock | `anti-webui-wayland` (`Role::Layer`, `Role::Lock`) | as Wayland | Yes, as Wayland | No: needs a compositor with `zwlr_layer_shell_v1` | inside the Wayland protocol tests (layer, lock, popup grab) | ShojiWM, nested, with `wayland-info` | A second toplevel and input routing have no protocol test |
| Linux, desktop-shell services | `anti-webui-wayland-shell` | as Wayland | Yes: `--all-features` check | No: `Shell::run` refuses without a compositor and layer shell | 64 with `--all-features`, 10 with default features, 9 `#[ignore]`d | ShojiWM only, for the layer and lock columns | Hyprland, niri, Sway, KDE, COSMIC, river; GNOME is excluded by design |
| Linux, compositor IPC | `anti-webui-wayland-shell` (`hyprland`, `niri`, `sway`) | — (no drawing) | Yes: `--all-features` check | No: no compositor, but the fake servers run anywhere they can bind | 3, one per compositor, against a fake IPC server | None: fake servers only | Every real compositor's IPC |
| Linux, D-Bus services and audio | `anti-webui-wayland-shell` (`network`, `battery`, `power-profiles`, `bluetooth`, `media`, `notifications`, `tray`, `session`, `audio`) | — (no drawing) | Yes: `--all-features` check | Partly: the fake-bus tests need sockets, which the sandbox refuses; some live tests read this machine's services | 12 with fake services, 4 live in 3 files (D-Bus, audio, capture), plus apps, process, system | This machine's services, by hand | Real desktops' services, PipeWire restarts |
| macOS | `anti-webui-macos` | Metal: `IOSurface` frames, `CAMetalLayer` fallback | Type-checks for `aarch64-apple-darwin`; not linked | No: type-checks only | 10 unit, plus 20 and 10 scenarios in two harness-free programs | M1 Pro, macOS 26, built-in Retina screen | [38's list](audit-capabilities-platform.md#macos-backend), kept in `38`, "まだ確かめていないこと" |
| Windows | `anti-webui-windows` | DX12; then the primary backends (Vulkan), then GL | Type-checks for `x86_64-pc-windows-msvc`; not linked | No: the target compiles, nothing can run | 10 unit, plus 23 scenarios in one harness-free program | Windows 11 at 175 %, Radeon 880M and RTX 5070 Laptop | `39`, "まだ確かめていないこと"; OLE DnD (W4) and TSF (W6) are not built |
| Any, headless | `anti-webui-app` (`headless`) | wgpu, software here | Yes: A6 built and ran it | Yes: pixel tests drew on lavapipe (A6 printed no skip) | 8 files in `app` and 5 in `md3e` skip-pass without an adapter | The authors' macOS and Windows machines (Metal, DX12); nothing recorded for a Linux GPU | A real Linux GPU, and software against hardware pixel results |

Notes, one per row:

1. **Linux, Wayland.** `#![cfg(target_os = "linux")]` (`wayland/lib.rs:3`) and Linux-only dependencies (`wayland/Cargo.toml:9`) make the crate empty elsewhere. The GPU column is Confirmed by `06`, "dmabuf presenter", "フォールバック" and by the attempt order `PRIMARY` then `GL` (`wgpu/lib.rs:198`). Its tests are `wayland/tests/*_protocol.rs`, five files, each binding a `ListeningSocket` (for example `layer_protocol.rs:332`) — a Unix socket, which the sandbox refuses (A6). Label: Confirmed, except the adapter identity (Inferred).
2. **Linux, X11.** Same gate (`x11/lib.rs:3`); `x11rb` with `dri3`, `present`, and `xinput` (`x11/Cargo.toml:9`). The server tests start `Xvfb :93` themselves (`tests/server.rs:24`, `:33`) and print "Xvfb isn't installed; skipping" when it is not (`:68`), and they do not present because Xvfb has no DRI3 (same file, lines 1–2). XWayland 24.1 and the 1.8 scale are in `35`, "確かめ方". Label: Confirmed.
3. **Layer surfaces and session lock.** `Role::Layer` and `Role::Lock` are core types ([A4c](audit-capabilities-platform.md#multi-window)); the protocols are bound in `wayland/lib.rs:769` and `:757`. Upstream's real-compositor check was ShojiWM, with `wayland-info`, on 2026-11-07 (`37`, "環境ごとの違い", "確かめた範囲"; `06:105` and `:131` for the lock-screen bugs found there). That a second toplevel and input routing have no test is `36`, "まだのもの". Label: Confirmed; Unverified for any compositor but ShojiWM.
4. **Desktop-shell services.** Linux-only (`wayland-shell/lib.rs:21`); `Shell::run` refuses twice, once without a compositor and once without `zwlr_layer_shell_v1` (`lib.rs:261`–`271`). Feature groups: `hyprland`, `niri`, `sway` pull `serde_json`; the eight D-Bus features pull `zbus` through the private `dbus` feature; `audio` pulls `libloading`; `apps`, `process`, and `system` pull nothing (`wayland-shell/Cargo.toml`). `37`, "環境ごとの違い" says only the ShojiWM column was checked. Label: Confirmed; Unverified for the other compositors.
5. **Compositor IPC.** One test each: `tests/hyprland_ipc.rs`, `niri_ipc.rs`, `sway_ipc.rs`, each against a fake server. `37`, "実装の状態" calls the IPC done and waiting for a manual check. Label: Confirmed (tests exist), Unverified (real compositors).
6. **D-Bus services and audio.** 12 tests run against a private `dbus-daemon` with fake services (the `*_bus.rs` files); `dbus_live.rs`, `audio_live.rs`, and `capture_live.rs` read this machine and are `#[ignore]`d or need features ([A6](audit-tests.md#test-inventory)). Label: Confirmed.
7. **macOS.** `#![cfg(target_os = "macos")]` (`macos/lib.rs:3`) and macOS-only dependencies (`macos/Cargo.toml:9`); Metal is chosen explicitly for `IOSurface` (`wgpu/iosurface.rs:34`). Upstream's evidence is `38:312`, "確かめたこと（2026-11-07、M1 Pro、macOS 26、内蔵の Retina 画面）"; the two programs are `app/tests/macos.rs` and `macos_spawn.rs`, both `fn main() {}` off macOS (`macos.rs:6`, `macos_spawn.rs:5`) and declared `harness = false` (`app/Cargo.toml:44`–`54`). Label: Confirmed for the code and declarations; run-time behaviour is documented upstream and Unverified here.
8. **Windows.** `#![cfg(windows)]` (`windows/lib.rs:3`); `windows` crate features for DirectManipulation, IME, Dxgi, and Com (`windows/Cargo.toml:9`). DX12 first is Confirmed by `wgpu/lib.rs:190`–`195` and by `39`, "提示" and "バックエンドの選び方" (a Vulkan swapchain on a window is opaque, so popup shadows go black). Upstream's evidence is `39:399`, "確かめたこと（2026-11-08、Windows 11、175% の画面）". Label: Confirmed for the code; run-time behaviour documented upstream and Unverified here.
9. **Headless.** `anti_webui_app::headless::render_rgba` makes `Gpu::headless()` (`headless.rs:29`; `wgpu/lib.rs:325`). A6 recorded no skip message and used lavapipe. Label: Confirmed (A6's run); the adapter identity is Inferred.

The `Builds on this machine` column comes from the cross-checks in [Method](#method); "type-checks" means `cargo check` passed, which proves nothing about linking or running. Everything else in the table is covered by the notes above.

### Linux, Wayland

- Status: present; builds only on Linux; runs only under a compositor.
- Evidence: the gate and the dependencies above; at run time `choose_backend` takes Wayland when `WAYLAND_DISPLAY` is set and `anti_webui_wayland::can_connect()` succeeds (`runner.rs:487`–`489`), and again when `WAYLAND_DISPLAY` is unset at all, because the default socket needs no variable (`:495`–`498`).
- Behaviour: the crate binds `wl_compositor`, `xdg_wm_base`, and the optional protocols listed in `06`, "使用プロトコル"; the daemon-free part is [A4c](audit-capabilities-platform.md#wayland-backend). What matters for the matrix: it needs a live compositor for anything beyond the mock-compositor tests, and on a machine without `/dev/dri` the presenter falls back to a wgpu surface (`06`, "フォールバック").
- Quality notes: 10 tests, and the five protocol tests assert request order against a small `wayland-server` compositor rather than a real one; they fail in the agent sandbox on `bind_absolute` (A6) and pass in a normal WSL terminal (A6's maintainer run). Line coverage 4.2 % on Linux (A6).
- Label: Confirmed.

### Linux, X11

- Status: present; builds only on Linux; runs only against an X server.
- Evidence: the gate, `DISPLAY` is not probed before it is chosen ([A4c](audit-capabilities-platform.md#the-platform-abstraction)), and `server_main_device` opens a second connection and asks DRI3 (`x11/present.rs:74`–`86`), which is how the app limits the Vulkan drivers.
- Behaviour: an X socket exists at `/tmp/.X11-unix/X0` (WSLg's, Inferred from `/mnt/wslg/.X11-unix/X0`), but the shell has no `DISPLAY`, so nothing selects X11, and there is no Xvfb to run the server tests against.
- Quality notes: the only automated coverage is Xvfb plus XTEST, which cannot present a frame (no DRI3), so the DRI3/Present path has no automated test at all; the manual XWayland check in `35`, "確かめ方", is the only evidence that path works. Line coverage 11.7 % (A6). The test binary links against a system `libxkbcommon-x11` that no manifest names — see [Documents against the code and the machine](#documents-against-the-code-and-the-machine).
- Label: Confirmed.

### Linux, layer surfaces and session lock

- Status: present; the same crate and the same compositor requirements as Wayland, plus `zwlr_layer_shell_v1` for layers and `ext_session_lock_v1` for locks.
- Evidence: binds at `wayland/lib.rs:769` and `:757`; `supports_layer_shell()` at `wayland/lib.rs:464` is what a shell checks before it starts.
- Behaviour: layer surfaces are how a shell draws a bar or a wallpaper; the lock role takes one surface per output and must not commit before the first `configure` ack, a rule found the hard way on ShojiWM (`37`, "実装の状態", and `06:105`).
- Quality notes: covered by the Wayland protocol tests (`layer_protocol.rs`, `lock_protocol.rs`), which run on any machine that can bind a socket; no test covers a compositor that lacks the global, because upstream has only met ShojiWM.
- Label: Confirmed; Unverified for other compositors.

### Linux, desktop-shell services

- Status: present; Linux-only, and a separate product surface from the window backends.
- Evidence: `Shell`, `open_layer`, `spawn_layer`, and the service modules in `anti-webui-wayland-shell`; the feature list in its manifest.
- Behaviour: pulls its own Wayland connection per service family, prefers `HYPRLAND_INSTANCE_SIGNATURE`, `NIRI_SOCKET`, or `SWAYSOCK` for workspaces, then `ext_workspace_v1`, and fails on GNOME ([A4c](audit-capabilities-platform.md#anti-webui-wayland-shell)).
- Quality notes: 64 tests with `--all-features` and 10 without, 9 `#[ignore]`d; 11 of the bus and IPC tests failed in the agent sandbox for lack of sockets and a session bus, and A6's maintainer run passed them outside it. That run is the only evidence for the D-Bus features.
- Label: Confirmed for the tests; Unverified for every real service except those A6 ran.

### macOS

- Status: present; type-checks from this machine, cannot be linked or run here.
- Evidence: the gate and the manifest; `cargo check --target aarch64-apple-darwin` passes for `anti-webui-macos` and `anti-webui-app`'s `macos` and `macos_spawn` programs ([Method](#method)). This is the first compile check of the crate in the fork; A4c had no Apple target ([A4c, Method](audit-capabilities-platform.md#method)).
- Behaviour: AppKit on the main thread; a spawned window keeps its `NSWindow` there and moves drawing to its own thread ([A4c](audit-capabilities-platform.md#macos-backend)); `App::run` must be on the main thread (`Session::connect`, `macos/lib.rs:512`).
- Quality notes: the two harness-free programs are the real coverage and they need a window server and a logged-in session; off macOS they compile to an empty `main`, so a Linux CI job proves nothing about them. Upstream's untested list (`38`, "まだ確かめていないこと") is long and includes live resize, real Japanese input, trackpads, and dragging out.
- Label: Confirmed for the code paths; the whole run-time story is documented upstream and Unverified.

### Windows

- Status: present; the whole workspace but `anti-webui-code` and `anti-webui-markdown` type-checks here, but nothing runs.
- Evidence: the gate and the manifest; `x86_64-pc-windows-msvc` is in `rust-toolchain.toml`, which is why A4c and A7 could build for it (handoff, "Verification").
- Behaviour: DX12 first with a DirectComposition visual so popups have alpha, then Vulkan, then GL; the adapter is limited to the low-power one unless `ANTIWEBUI_ADAPTERS=all` or `WGPU_ADAPTER_NAME` is set ([A4c](audit-capabilities-platform.md#windows-backend); `wgpu/lib.rs:185`, `:190`–`195`).
- Quality notes: 23 scenarios in `app/tests/windows.rs` run against a real window; off Windows the file is `fn main() {}` (`windows.rs:6`) and `harness = false`. A4c and this audit type-checked the test binary for Windows, but nobody has run it since the fork, and `39` records that the 23 passed upstream on 2026-11-08.
- Label: Confirmed for the code; Unverified for the run.

### How the app chooses a backend

On Linux, `choose_backend` (`runner.rs:475`) runs in this order:

1. `ANTIWEBUI_BACKEND=wayland` or `x11` returns that backend, if its feature is on (`runner.rs:478`–`481`); any other non-empty value is an error naming the variable (`:482`–`484`).
2. Wayland, if `WAYLAND_DISPLAY` is set **and** `can_connect()` succeeds (`:487`–`489`); `can_connect` is a real `Connection::connect_to_env()` (`wayland/lib.rs:459`), not a variable check.
3. X11, if `DISPLAY` is set (`:491`–`493`). `DISPLAY` is not probed: a stale value fails later, in `Session::connect`.
4. Wayland again, if `WAYLAND_DISPLAY` is unset, because the default socket needs no variable (`:495`–`498`).
5. Otherwise `found neither a Wayland compositor nor an X server` (`:500`).

macOS and Windows accept only their own name and report `ANTIWEBUI_BACKEND=<value> isn't built in` for anything else (`runner.rs:449`–`458`, `:462`–`470`). An OS that is none of the three fails earlier: `App::run` returns `no platform backend is available for this OS yet` (`lib.rs:144`–`148`), and `run_shell` returns `a shell needs Wayland` off Linux (`lib.rs:118`–`129`). Before the UI thread starts, `run_here` asks the chosen display server for its GPU and restricts the Vulkan loader to that driver (`runner.rs:1079`–`1090`, `limit_vulkan_drivers` at `:504`).

Variables that change which platform path runs, with the code that reads them:

| Variable | Effect | Where |
|---|---|---|
| `ANTIWEBUI_BACKEND` | Picks Wayland or X11 on Linux; only the native name is accepted on macOS and Windows | `runner.rs:450`, `:463`, `:476` |
| `WAYLAND_DISPLAY` | Makes Wayland a candidate only when set before the `can_connect` test; absence means the default socket | `runner.rs:488`, `:496` |
| `DISPLAY` | Makes X11 a candidate | `runner.rs:492` |
| `WGPU_BACKEND` | Turns off the built-in backend order and the GPU-first path on Windows | `wgpu/lib.rs:185`, `early.rs:14` |
| `ANTIWEBUI_PRESENT=surface` | Forces the wgpu surface over dmabufs or `IOSurface` | `present.rs:96`; `wgpu/lib.rs:443` |
| `ANTIWEBUI_ALLOCATOR=vulkan` | Skips GBM allocation on Wayland | `wgpu/dmabuf.rs:280` |
| `ANTIWEBUI_SYNC=implicit` | Forces implicit sync on Wayland | `present/dmabuf.rs:622` |
| `ANTIWEBUI_VULKAN_DRIVERS=all` | Turns off the driver filter | `wgpu/drivers.rs:94` |
| `ANTIWEBUI_ADAPTERS=all`, `WGPU_ADAPTER_NAME` | Turns off the adapter filter on Windows | `wgpu/adapters.rs:30`–`31` |
| `ANTIWEBUI_SCALE` | Overrides X11's one global scale | `x11/settings.rs:91` |
| `ANTIWEBUI_SHADER_CACHE=off` | Turns off the DX12 shader cache | `wgpu/shader_cache.rs:24` |

Label: Confirmed, from the code paths named in each row.

### Features and cfg gates

| Crate | Feature or cfg | What it gates |
|---|---|---|
| `anti-webui`, `anti-webui-internal` | `wayland`, `x11`, `macos`, `windows`, all on by default | Which backend crates `anti-webui-app` links; on another OS the matching dependency is target-gated, so `--all-features` there adds nothing |
| `anti-webui-app` | `[target.'cfg(target_os = "linux")'.dependencies]` | Wayland and X11 as optional dependencies (`app/Cargo.toml:35`) |
| `anti-webui-app` | `[target.'cfg(windows)'.dependencies]`, `[target.'cfg(target_os = "macos")'.dependencies]` | The Windows and macOS backends, optional (`:59`, `:64`) |
| `anti-webui-app` | `[[test]] macos`, `macos_spawn`, `windows` with `harness = false` | Test programs that need the OS's own window and event loop (`:44`–`54`) |
| `anti-webui-app` | `handler_for!` and `platform_for!` per backend, each behind `cfg(all(target_os, feature))` | The four seams the `Runner` uses (`runner.rs:424`–`434`, `:728`–`740`) |
| `anti-webui-app` | `src/present/{dmabuf,iosurface,no_dmabuf}.rs` | Own buffers on Linux and macOS; a value-less type elsewhere (`present.rs:11`–`21`) |
| `anti-webui-app` | `#[cfg(unix)]` in `pacing.rs` | The GPU-fence (sync file) type; value-less elsewhere (`pacing.rs:10`–`13`) |
| `anti-webui-core` | `#[cfg(unix)]` / `#[cfg(not(unix))]` in `platform.rs` | `BorrowedFd`-based dmabuf types, or same-named types that can only be zero-sized (`platform.rs:10`–`30`) |
| `anti-webui-core` | `cfg(all(target_os = "linux", target_env = "gnu"))` | `libc` for the image worker (`Cargo.toml:30`; `images/worker.rs:165`) |
| `anti-webui-wayland`, `-x11`, `-wayland-shell` | `#![cfg(target_os = "linux")]` | Whole crate: these are empty off Linux (`lib.rs:3`, `:3`, `:21`) |
| `anti-webui-macos` | `#![cfg(target_os = "macos")]` | Whole crate (`lib.rs:3`) |
| `anti-webui-windows` | `#![cfg(windows)]` | Whole crate (`lib.rs:3`) |
| `anti-webui-wayland` | `wp_fractional_scale_v1`, `zwp_linux_dmabuf_v1`, `wp_linux_drm_syncobj_manager_v1` | Not cfgs: optional protocols decided at connect time, so one binary covers both cases |
| `anti-webui-wayland-shell` | `hyprland`, `niri`, `sway`, `dbus` family, `audio`, `apps`, `process`, `system` | Services and their dependencies; the D-Bus family is one private feature |
| `anti-webui-video` | `[target.'cfg(windows)'.dependencies]` (WASAPI), `#[cfg(unix)]` in `player.rs` | Sound on Windows; dmabuf export only on Unix (`Cargo.toml:15`; `player.rs:8`) |
| `anti-webui-wgpu` | target tables for `windows`, `linux`, `macos` | The DXGI swapchain, dmabuf/`ash`, and `IOSurface` paths (`Cargo.toml:20`, `:30`, `:35`) |

Label: Confirmed, each from the manifest or source line named.

### Documents against the code and the machine

| Where | The document says | The code or this machine says | Label |
|---|---|---|---|
| `35-platform-x11.md`, "確かめ方" | 15 unit tests and 8 server tests | 13 in `src/` and 9 in `tests/server.rs` | Confirmed (already in [A4c](audit-capabilities-platform.md#code-against-the-design-documents); counts drifted) |
| `36-multi-window.md:267` | The environment table's column for Windows is "Windows（今後）" | The Windows backend and its multi-window work landed 2026-11-08 (`39`, "段階", W2); the table predates it | Confirmed (an aged statement; no behaviour follows) |
| `39-platform-windows.md`, "バックエンドの選び方" | `cargo check --workspace` passes on Windows | Cross-checked from Linux, the workspace passes except `anti-webui-code` and `anti-webui-markdown`, whose `tree-sitter` build scripts need MSVC's `lib.exe`; on a Windows host with MSVC the claim is plausible but unchecked | Confirmed (the cross-check); Unverified on a Windows host |
| `35-platform-x11.md`, "確かめ方" | Nothing about a system library for the test binary | The test binary needs `libxkbcommon-x11`, which no manifest declares; A4c found it missing, and it is installed and links now (A6) | Confirmed |

## Baseline violations

On Linux everything builds, with all features. The baseline for this workstream is what does not build in a configuration, and what has never been built or run at all:

- `anti-webui-macos` and `anti-webui-windows` have only been type-checked from Linux, never linked; no CI job builds them yet.
- Cross-checks for Windows and macOS fail in `anti-webui-code` and `anti-webui-markdown`, because their `tree-sitter` C parsers need a C toolchain for the target.
- `cargo check -p anti-webui --no-default-features --all-targets` fails in the `svg` example, which lacks `required-features = ["svg"]`.
- The macOS and Windows end-to-end programs (`app/tests/macos.rs`, `macos_spawn.rs`, `windows.rs`) have not been run since the fork; their last recorded run is upstream's, on 2026-11-08.
- The X11 DRI3/Present path has no automated test: Xvfb has no DRI3, so the server tests stop before presenting (`x11/tests/server.rs:1`–`2`).
- The X11 test binary depends on `libxkbcommon-x11` from the system, which nothing in the repository declares or installs.
- In the agent sandbox, 5 Wayland protocol tests and 11 wayland-shell bus and IPC tests fail for lack of Unix sockets, so an agent cannot verify a platform change end to end ([A6](audit-tests.md#failures-on-this-machine)).
- Every Wayland claim rests on ShojiWM; every macOS claim on one M1 Pro; every Windows claim on one laptop at 175 %. No GNOME, KDE, plain Xorg, Intel GPU, or 1× display has been exercised.
- With no backend feature the library builds, and `choose_backend` on Linux can only return `found neither a Wayland compositor nor an X server` (`runner.rs:500`); a backend-less build is a type-check target, not a working app (Inferred from the cfg arms at `runner.rs:478`–`498`).

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| One machine per OS is the whole evidence base (one M1 Pro, one Windows laptop, one ShojiWM session), and this fork has run none of them | A compositor, GPU, or OS-version regression is invisible until a user reports it | High | [The matrix](#the-matrix); `38:312`, `39:399`, `36:285` |
| No macOS or Windows CI job, and the Apple target is not in `rust-toolchain.toml` | The two crates can rot silently; a type-check here catches API breaks but not link or run failures | Certain today | [Method](#method); `rust-toolchain.toml` |
| Cross-checks need a C toolchain for `tree-sitter` | `anti-webui-code` and `anti-webui-markdown` cannot be checked for Windows or macOS from Linux | Certain today | [Method](#method) |
| The X11 test binary needs a system library (`libxkbcommon-x11`) that the repository never declares | The same tree links on one machine and not on another: A4c could not link and this audit finds the library installed | Medium | `ldconfig -p`; [Baseline violations](#baseline-violations) |
| Platform tests need Unix sockets and, for X11, a display server | Agents cannot verify platform changes; only the maintainer's terminal can | Certain in the sandbox | [A6](audit-tests.md#failures-on-this-machine); `wayland/tests/layer_protocol.rs:332` |
| Behaviour that differs per compositor (layer shell, lock, decorations, workspaces) is tested only against ShojiWM or fakes | A GNOME or KDE user sees failures that no test predicts | High for a shell product | [Desktop-shell services](#linux-desktop-shell-services); `37`, "環境ごとの違い" |
| `--all-features` on Linux compiles nothing extra for macOS and Windows, because those dependencies are target-gated | A green Linux `--all-features` check reads as cross-platform confidence it does not give | Medium | [Features and cfg gates](#features-and-cfg-gates); `app/Cargo.toml:35`, `:59`, `:64` |
| The GPU path per OS is chosen by fallback chains (`PRIMARY` then `GL`; DX12 then Vulkan then GL) that no test pins | A driver change can move a user to a slower backend without a failing test | Medium | `wgpu/lib.rs:185`–`198`; [Windows](#windows) |
| Headless rendering is the only path verifiable in an agent session, and it runs on lavapipe | Pixel and platform bugs that need a real session stay unseen | High | [The matrix](#the-matrix), last row |

## Open questions

- Which real macOS and Windows devices are available for Phase 1 exit and later phases? Owned by the maintainer. Answered by naming one device per OS with its OS version and display scale, then running `cargo test -p anti-webui-app --test macos` (and `macos_spawn`) and `--test windows` there, recording the adapter and scale. Today both are Unverified.
- Does WSLg's Wayland compositor work from the maintainer's terminal? Answered by running `showcase` there with `ANTIWEBUI_PERF_OVERLAY=1` and reading the presenter line. The socket exists and the sandbox refuses it, so whether the compositor works at all is Unverified.
- Should `aarch64-apple-darwin` join `x86_64-pc-windows-msvc` in `rust-toolchain.toml`, so every session can type-check macOS? Proposed; the maintainer decides, since it changes a fork-owned toolchain file.
- Can `anti-webui-code` and `anti-webui-markdown` be cross-checked with a target C toolchain (for example cargo-xwin for MSVC, or zig as the C compiler)? Answered by trying one in a later phase; it is a tool adoption decision ([tool matrix](../quality/tool-adoption-matrix.md)).
- Does plain Xorg behave as `35` predicts for implicit sync and popups without a compositor? Answered by an Xorg session on another VT (amdgpu, glamor), as `35`, "確かめ方", "素の Xorg" plans.
- Do GNOME's client-side decorations and KDE's server-side decorations produce working title bars? Answered by running `multi_window` there, which `36`, "確かめ方", lists as manual work.
- Which compositor, other than ShojiWM, has layer surfaces, session lock, and workspaces working? Answered by running `shell_bar` with `--features shell-all` on Hyprland, niri, and Sway; doc 37 says this has not been done.

## Inputs for gap analysis

- [The matrix](#the-matrix): the (OS, display server) pairs a requirement can name today are Wayland and X11 on Linux, macOS, and Windows; layer surfaces and session lock exist only on Wayland with `zwlr_layer_shell_v1`; desktop-shell services exist only on Linux/Wayland. A requirement that names GNOME, plain Xorg, or a non-wlroots compositor is new work or a new test plan, not a setting.
- [How the app chooses a backend](#how-the-app-chooses-a-backend): on Linux the run-time path is decided by `ANTIWEBUI_BACKEND`, `WAYLAND_DISPLAY`, and a real connect attempt. A requirement about "the app on Linux" must say which of the two paths it means, because they differ in scale, cursors, IME, and drag and drop ([A4c](audit-capabilities-platform.md#clipboard-cursor-and-high-dpi-scale)).
- [Features and cfg gates](#features-and-cfg-gates): a new platform costs a new crate with `#![cfg(...)]`, target-gated dependencies, and three additions in `anti-webui-app` (`handler_for!`, `platform_for!`, `choose_backend`) plus a harness-free test program, because a window must be made on the OS's own thread.
- [Baseline violations](#baseline-violations): any requirement whose acceptance needs a window must name either a human on a real device or a CI display server; today neither exists, and the agent sandbox can only render headlessly.
- [Documents against the code and the machine](#documents-against-the-code-and-the-machine): A8 should take this table as given rather than re-derive it.
