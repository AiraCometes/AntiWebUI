# Audit: wgpu fork

- Date: 2026-11-25
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Auditor: Claude Code, Claude Opus 5.5
- Plan item: [A3](phase-1-audit-plan.md#workstreams)

## Scope

The `[patch.crates-io]` block in the root `Cargo.toml`, which replaces ten wgpu crates with branch `adapter-filter` of `bea4dev/wgpu`: what the fork changes against `v30.0.1`, why, where AntiWebUI uses it, what it costs to keep, and the ways back to wgpu from crates.io. Upstream wgpu issue and pull-request states are as of 2026-11-25.

Left out: the rest of the renderer (A4b) and a run on Windows hardware (A5, A9).

## Method

| Question | Command |
|---|---|
| What the fork changes | `gh api repos/gfx-rs/wgpu/compare/v30.0.1...bea4dev:wgpu:ff143aeeb126dce521b87f47547565c4b6a6da1e` (commits, file list, and the diff with `Accept: application/vnd.github.diff`) |
| How far the fork is behind | the same compare against `v30.0.2`, and `v30.0.1...v30.0.2` |
| Why upstream forked | `Cargo.toml:102`–`116`; `docs/design/39-platform-windows.md` ("使う GPU", "シェーダーのキャッシュ", line 446) |
| Where the fork's API is used | `rg -n 'adapter_filter\|Dx12AdapterFilter\|PipelineCache\|PIPELINE_CACHE' src` |
| Build without the patch | `git archive HEAD` into a scratch folder, the `[patch.crates-io]` block removed, `cargo update -w`, then `cargo check --workspace --all-features` (Linux) and `cargo check -p anti-webui-wgpu --target x86_64-pc-windows-msvc`. The same Windows check with `--locked` in the repository, with the patch |
| Upstream wgpu work on the same problems | `gh api search/issues` on `repo:gfx-rs/wgpu` for "pipeline cache dx12" and "adapter LUID"; then the issues and pull requests found |

## Findings

### What the fork changes

- Status: present
- Evidence: the compare above. Branch head `ff143aeeb126dce521b87f47547565c4b6a6da1e` (2026-11-11), which `Cargo.lock` pins. One commit, "add adapter filter", on top of `v30.0.1`; 11 files, +246 −37.
- Behaviour: two DX12-only features.
  1. Adapter filter. `wgpu-types` adds `Dx12AdapterFilter { All, Luids(Vec<u64>) }` and the field `Dx12BackendOptions::adapter_filter` (default `All`). `wgpu-hal/src/dx12/instance.rs` drops adapters whose LUID is not allowed before `Adapter::expose`, which is where `D3D12CreateDevice` runs. `wgpu/src/lib.rs` re-exports the type.
  2. `PipelineCache` on DX12. `wgpu-hal/src/dx12/mod.rs` turns the empty `PipelineCache` into a map from a 16-byte key to compiled DXBC or DXIL, with a 64 MiB cap and a flat serialisation (key, `u32` length, bytes). `device.rs` looks a shader up before FXC or DXC and stores it after. The key hashes the HLSL key, the compiler name, and the debug flag; the validation key hashes the `wgpu-hal` version. `adapter.rs` turns on `Features::PIPELINE_CACHE`, and `wgpu-core/src/pipeline_cache.rs` uses a fixed adapter key (`[255; 15]`) for DX12.
- Two unit tests come with it: `dx12_adapter_filter` in `wgpu-types` and `pipeline_cache_data_round_trips` in `wgpu-hal`.
- No other backend and no shared code path changes, apart from the re-export list in `wgpu/src/lib.rs` and the changelog. Label: Confirmed
- Label: Confirmed

### Why it exists

- Status: present
- Evidence: `Cargo.toml:102`–`105`; `docs/design/39-platform-windows.md`, "使う GPU" and "シェーダーのキャッシュ"
- Behaviour: upstream measured the md3e `gallery` taking about 2.5 s to its first frame on a hybrid laptop (Radeon 880M and RTX 5070 Laptop), mostly from waking the sleeping discrete GPU (`D3D12CreateDevice` 0.72–2.49 s) and from FXC (80–430 ms a shader). Upstream reports about 0.4 s after both changes, 1.5 s without a cache. Label: Confirmed (as documented upstream); Unverified (not measured here: no Windows device)
- Label: Confirmed

### Where AntiWebUI uses it

- Status: present
- Evidence:
  - `src/anti-webui-wgpu/src/adapters.rs`: `preferred()` returns `Dx12AdapterFilter::Luids` with the first adapter from `IDXGIFactory6::EnumAdapterByGpuPreference(MINIMUM_POWER)` (or `HIGH_PERFORMANCE` when the user chose that for the executable), and `All` when `ANTIWEBUI_ADAPTERS=all` or `WGPU_ADAPTER_NAME` is set.
  - `src/anti-webui-wgpu/src/lib.rs:126`–`155`: `instance_desc` sets `backend_options.dx12.adapter_filter`, under `#[cfg(windows)]`.
  - `src/anti-webui-wgpu/src/shader_cache.rs`: asks for `PIPELINE_CACHE` only when the backend is DX12; reads and writes `%LOCALAPPDATA%\AntiWebUI\shaders\…`. `renderer.rs`, `effects.rs`, and `custom_shader.rs` pass the cache to every pipeline.
- Behaviour: the filter is the only fork API that does not exist in crates.io wgpu. `PipelineCache` and `Features::PIPELINE_CACHE` exist there too, so the cache code compiles everywhere and is skipped where the feature is missing.
- Public surface: `anti-webui-wgpu` re-exports `wgpu` (`lib.rs:43`), and the facade exposes it as `anti_webui::gpu::wgpu` for the GPU canvas. `docs/design/41-gpu-canvas.md` says these types do not match crates.io wgpu. Label: Confirmed
- Label: Confirmed

### Build without the patch

- Status: present (Linux), absent (Windows)
- Evidence: the scratch build in [Method](#method)
- Behaviour:
  - Without the patch, `wgpu = "30"` resolves to crates.io `30.0.2`. `cargo check --workspace --all-features` on Linux finishes with no error.
  - `cargo check -p anti-webui-wgpu --target x86_64-pc-windows-msvc` fails with 6 errors, all from the filter: `Dx12AdapterFilter` not found (`adapters.rs:29`, `33`, `41`, `42`; `lib.rs:130`) and no field `adapter_filter` on `Dx12BackendOptions` (`lib.rs:146`). With the patch and `--locked`, the same check passes.
  - macOS was not checked (no Apple target installed). The fork changes no Metal code, so the same result as Linux is expected. Label: Inferred
- Reading: on Linux and macOS the fork buys nothing, and on Windows only one function and one field depend on it. A fallback that builds on stock wgpu needs only those lines behind a `cfg` or a feature, and loses the cache benefit on DX12 and the dGPU fix. Label: Inferred, from the errors above
- Label: Confirmed

### Maintenance cost

- Status: present
- Evidence: the compares in [Method](#method)
- Behaviour:
  - The patch replaces the whole wgpu family on every platform, not only on Windows. The fork is 1 ahead and 11 behind `v30.0.2` (2026-11-12), so every target misses the 30.0.2 fixes: Metal colour space (two backports), `wgpu-core` id and mapped-buffer fixes, and a naga build fix. Label: Confirmed
  - Each wgpu release needs the one commit rebased, the branch pushed, and `cargo update -p wgpu-hal` (upstream's own procedure, `39-platform-windows.md:446`). The diff touches `dx12/device.rs` `load_shader` and pipeline creation, which upstream changes often; a conflict on each major release is likely. Label: Inferred
  - A git dependency cannot be published: `cargo publish` ignores `[patch]`, so a published `anti-webui-wgpu` would depend on crates.io wgpu and fail to build on Windows (previous section). Label: Inferred, from cargo's documented `[patch]` behaviour and the Windows errors above
  - Apps that add crates.io wgpu next to AntiWebUI get two wgpu copies with types that do not match, unless they copy the patch block. Label: Inferred, from `41-gpu-canvas.md` and cargo's resolution rules
- Quality notes on the fork's code:
  - `CompiledShader::bytes` builds a slice with `unsafe` and has no `SAFETY:` comment. Label: Confirmed
  - The adapter filter calls `GetDesc1().unwrap()` for each adapter, which panics where wgpu's own code would skip the adapter. Label: Confirmed
  - Cache keys use `std::hash::DefaultHasher`, whose algorithm the standard library does not promise to keep across Rust releases. The validation key holds only the `wgpu-hal` version, so a toolchain change can turn every entry into a miss. Label: Inferred, from the `DefaultHasher` documentation
  - The cache never drops an entry: `insert` refuses new shaders once 64 MiB is reached, and `shader_cache.rs` keeps everything it loaded ("it only grows"). An app whose shaders change (development, runtime custom shaders) can fill it with stale entries and then stop caching. Label: Inferred, from `PipelineCache::insert` and `ShaderCache::saved`
- Label: Confirmed (facts), Inferred (as marked)

### Path back to standard wgpu

- Status: partial
- Evidence: gfx-rs/wgpu issues and pull requests on 2026-11-25

| Fork feature | Upstream wgpu state | Label |
|---|---|---|
| Adapter filter by LUID | Issue [#9932](https://github.com/gfx-rs/wgpu/issues/9932) (open) reports the same device-on-every-adapter problem. [#10011](https://github.com/gfx-rs/wgpu/pull/10011) (lazy adapter selection) was closed on 2026-10-13 in favour of an extension API, [#10304](https://github.com/gfx-rs/wgpu/pull/10304) (open; a maintainer wrote on 2026-10-22 "I think we should go with this"). [#10775](https://github.com/gfx-rs/wgpu/pull/10775) (open, base `trunk`) adds an LUID filter in `Dx12AdapterOptions`, passed to `enumerate_adapters` through that extension API, and says it fixes #9932 | Confirmed |
| `PipelineCache` on DX12 | Issue [#8526](https://github.com/gfx-rs/wgpu/issues/8526) "Pipeline cache (DX12 & Metal)" is open; a contributor asked for guidance on the DX12 half on 2026-10-20. No pull request was found | Confirmed |

- Reading: the filter will likely reach wgpu in a new major version, with a different API (an extension argument, not a field of `Dx12BackendOptions`), so `adapters.rs` and `instance_desc` will need to change when moving. The cache has no upstream path yet; the fork's implementation could be offered on #8526, but that is a pull request from this fork's side and needs the maintainer's approval (`AGENTS.md`). Label: Inferred, from the table
- Options, for Phase 3 and 4:
  1. Keep the fork until wgpu ships both features. Cost: the previous section.
  2. Not an option: keeping the patch for Windows only. `[patch]` applies to the whole workspace, not per target. Label: Inferred, from cargo's `[patch]` rules
  3. Move to stock wgpu, put the filter call behind a feature, and accept the DX12 start-up cost until upstream merges #10775; keep the cache code, which is skipped while DX12 lacks `PIPELINE_CACHE`.
  4. Offer the DX12 cache to upstream on #8526.
  - Label: Proposed (settled by a Phase 4 spike and an ADR)

## Baseline violations

- 10 git-source packages in `Cargo.lock` (also in [A2](audit-licenses.md#package-sources)).
- The fork is 11 commits behind the latest wgpu 30 patch release.

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| AntiWebUI cannot be published to crates.io while it needs the fork | High | High, once publishing is a goal | [Maintenance cost](#maintenance-cost) |
| Every platform misses wgpu patch releases until the fork is rebased | Medium | High | 11 commits behind `v30.0.2` |
| The upstream LUID filter lands with a different API, so the move is a code change, not a version bump | Medium | High | [#10775](https://github.com/gfx-rs/wgpu/pull/10775) |
| The DX12 cache fills with stale entries and stops caching | Low | Medium | [Maintenance cost](#maintenance-cost) |
| Start-up claims rest on upstream's measurements on one laptop | Medium | Medium | [Why it exists](#why-it-exists); A9 cannot measure Windows here |

## Open questions

- Does the start-up gain hold on other Windows machines (a desktop with one GPU, an Intel and NVIDIA laptop)? Answered by A9 on Windows hardware, which is not available yet.
- Will #10775 ship in wgpu 31, and with which API? Answered by watching the pull request; a Phase 4 spike can try its branch.
- Does macOS build without the patch? Answered by `rustup target add aarch64-apple-darwin` and the same scratch check (A5 needs that target anyway).

## Inputs for gap analysis

- Distribution: the crates.io publishing blocker, for the Phase 2 requirement on how AntiWebUI is shipped.
- Start-up time on Windows: the dGPU and shader-cache numbers, as a candidate performance requirement once measured.
- Phase 4 spike list: moving to stock wgpu (option 3) and the upstream extension API.
