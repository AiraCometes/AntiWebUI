# Benchmark: upstream baseline

- Date: 2026-11-26
- Commit: `4a642397f2d5ed3a262d654fa36ceca5136b2b8a` (this repository); `src/`, `Cargo.toml`, and `Cargo.lock` are identical to upstream `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Harness: `cargo build --timings`, cargo-bloat, the headless examples `bench_changes` and `bench_drag` (`src/anti-webui/examples/`), and [`benchmarks/phase1/windowed.py`](../../../benchmarks/phase1/windowed.py) for the windowed metrics
- Plan item: [A9](../research/phase-1-audit-plan.md#workstreams)

## Question

What upstream costs today on this machine, so that fork changes can be compared with it: build time cold and warm, example binary size, frame cost without a window, and, from a desktop session, startup time, idle wakeups and presents, resident memory, and frame time. The metrics are the ones in [principles](../principles.md#measured-metrics). No budget exists yet; this report is the reference the budgets will be set against.

## Environment

| Item | Value |
|---|---|
| OS and version | Ubuntu 26.04.1 LTS on WSL2, Linux 6.18.58.1-microsoft-standard-WSL2 |
| Display server / compositor | None for the measurements in this report: they ran in the agent sandbox, which cannot reach WSLg ([A5](../research/audit-platform-matrix.md#method)) |
| CPU, GPU, driver | Intel Core i5-11300H, 4 cores and 8 threads, 7 GB of memory visible to WSL2; no `/dev/dri`, so the GPU is llvmpipe through Mesa 26.0.8 (`llvmpipe (LLVM 21.1.8, 256 bits)`, printed by the benches) |
| Display resolution, refresh rate, scale factor | Not applicable (headless; `bench_drag` renders at 2340×1616) |
| Power settings | Unrecorded (laptop, WSL2 host power plan) |
| Rust toolchain, profile, flags, features | 1.99.0 (`rust-toolchain.toml`). Workspace has no `[profile]`; the maintainer's `~/.cargo/config.toml` sets `[build] jobs = 4` and `[profile.dev] debug = "line-tables-only"`, which every number here includes. Release is Cargo's default (opt-level 3, no LTO, 16 codegen units). Default features. |

## Method

**Builds.** A fresh target directory (`CARGO_TARGET_DIR=target/a9-cold`) with the compiler wrapper turned off (`RUSTC_WRAPPER=`, `HOST_CC=cc`, `HOST_CXX=c++`, `CC_KNOWN_WRAPPER_CUSTOM=`), so neither Kache nor an earlier `target/` gave cache hits. The registry and git sources were already downloaded, so no time is network time. Each command ran once, in order: cold dev build of `showcase`; a no-op rebuild; after `touch src/anti-webui-core/src/lib.rs`; after touching the example; cold release build of three examples; release after touching core. Wall time from `date +%s.%N` around each command; unit times from the `--timings` reports.

**Binary size.** `ls -l` of the release examples, then `strip -o` on a copy. Per-crate `.text` from `cargo bloat --release --crates`. cargo-bloat rebuilds the example with its own settings (the file grew from 27.1 MB to 31.3 MB), so its percentages are used and its file size is not.

**Headless frame cost.** `bench_changes` (hover, caret blink, text change, scroll, and inertia in documents of 2,000 and 20,000 rows) and `bench_drag` (drag sessions), release build, three runs each in a row. Both print their own per-phase averages or p50/p99 over many iterations; this report gives the median of the three runs. Peak RSS and wall time from `/usr/bin/time`.

**Windowed metrics.** Not measured here; see [Pending: windowed measurements](#pending-windowed-measurements).

## Results

### Build time

| Build | Unit | Value | N | Label |
|---|---|---:|---:|---|
| Cold dev, `showcase` (268 units, 220 s of unit time) | s | 58.1 | 1 | Confirmed |
| No-op dev rebuild | s | 0.68 | 1 | Confirmed |
| Dev after touching `anti-webui-core/src/lib.rs` (9 units) | s | 3.06 | 1 | Confirmed |
| Dev after touching `examples/showcase.rs` | s | 0.82 | 1 | Confirmed |
| Cold release, `showcase` + `widgets` + `multi_window` (272 units, 572 s of unit time) | s | 149.1 | 1 | Confirmed |
| Release after touching `anti-webui-core/src/lib.rs` | s | 30.1 | 1 | Confirmed |
| Cold `target/` size after both cold builds | GB | 2.7 | 1 | Confirmed |

Slowest units, cold: dev `x11rb-protocol` 14.5 s, `anti-webui-core` 14.2 s, `naga` 12.6 s, `ash` 11.7 s, `wgpu-core` 10.0 s; release `anti-webui-core` 49.0 s, `naga` 40.0 s, `wgpu-core` 36.7 s, `x11rb-protocol` 34.0 s, `image` 23.2 s.

### Binary size

| Example (release) | Unit | File | Stripped | Label |
|---|---|---:|---:|---|
| `showcase` | MB | 27.1 | 21.0 | Confirmed |
| `widgets` | MB | 29.1 | 22.3 | Confirmed |
| `multi_window` | MB | 26.9 | 20.8 | Confirmed |
| `showcase`, dev | MB | 133.8 | — | Confirmed |

`showcase` sections: `.text` 12.9 MB, `.rodata` 4.6 MB, `.eh_frame` 1.4 MB. Largest crates by `.text` share: `naga` 12.6%, `anti_webui_core` 10.8%, `std` 8.9%, `wgpu_core` 8.7%, `usvg` 4.7%, `skrifa` 3.9%, `wgpu_hal` 3.7%, `harfrust` 3.2%, `anti_webui_wgpu` 3.2%, `anti_webui_wayland` 2.5%, `anti_webui_x11` 2.1% (Confirmed by cargo-bloat, which calls its own numbers estimates). Both Linux backends are linked into every Linux binary.

### Frame cost without a window (llvmpipe)

`bench_changes`, total time of one change, median of 3 runs (each run is itself an average):

| Change | Unit | 2,000 rows | 20,000 rows | Label |
|---|---|---:|---:|---|
| Hover | ms | 2.12 | 2.01 | Confirmed |
| Caret blink | ms | 1.01 | 1.07 | Confirmed |
| Text change | ms | 2.20 | 18.46 | Confirmed |
| Scroll | ms | 5.94 | 5.94 | Confirmed |
| Inertia | ms | 5.57 | 5.60 | Confirmed |

At 20,000 rows a text change spends about 16.7 ms of its 18.5 ms in layout and positioning (first run's breakdown): layout grows with the document while the other phases do not. Scroll and inertia are 92% GPU time, which here is llvmpipe on the CPU. The first run's hover at 2,000 rows was 3.38 ms against 2.10–2.12 ms later, a warm-up effect.

`bench_drag`, median of 3 runs of each case's p50 and p99:

| Case | UI p50 | UI p99 | GPU p50 | GPU p99 | Unit | Label |
|---|---:|---:|---:|---:|---|---|
| Carry 100 selected paragraphs | 0.63 | 0.81 | 4.11 | 11.49 | ms | Confirmed |
| Carry 5,000 selected paragraphs | 0.72 | 0.89 | 4.10 | 11.15 | ms | Confirmed |
| Drag a card over 600 | 0.37 | 0.72 | 0.76 | 1.34 | ms | Confirmed |
| `sortable_list` of 500 | 0.15 | 0.40 | 2.41 | 4.22 | ms | Confirmed |

Starting a drag of 6.6 MB of selected text took 11.5 ms; of 131 KB, 1.6 ms (first run; Confirmed).

| Process | Unit | Peak RSS | Wall | N | Label |
|---|---|---:|---:|---:|---|
| `bench_changes` | MiB | 414 | 6.7 s | 3 | Confirmed |
| `bench_drag` | MiB | 518 | 13.9 s | 3 | Confirmed |

These RSS figures include llvmpipe's own memory and the benches' large documents; they are not an app's idle memory.

### Pending: windowed measurements

Startup time, idle wakeups, idle presents, idle RSS, and frame time on a compositor need a desktop session, which the agent sandbox does not have ([A5](../research/audit-platform-matrix.md#linux-wayland)). [`benchmarks/phase1/windowed.py`](../../../benchmarks/phase1/windowed.py) measures them; the maintainer runs it from a terminal on this machine (WSLg) and, where available, on a native Linux session:

```sh
cargo build --release --locked -p anti-webui --example bench_pacing --example bench_shaders
python3 benchmarks/phase1/windowed.py --runs 5 > a9-windowed.json
```

It records, per run: launch to first presented frame (on `CLOCK_MONOTONIC`, the clock `ANTIWEBUI_FRAME_STATS` uses); context switches per second of all threads and frames presented during 8 s with nothing changing (`bench_pacing --linger`); VmRSS then and VmHWM at its end; and start-to-presented frame time under `bench_pacing --rows 2000 --layers 40` and `bench_shaders --kind same --count 1000`. Until those results are added here, these metrics are Unverified.

## Comparison with baseline

None: this is the first report. Upstream's own measurements in `docs/design` are from other machines (Radeon 880M, RTX 5070 Laptop, M1 Pro) and are not comparable ([A8](../research/audit-design-docs.md)).

## Threats to validity

- One run per build measurement; build times on WSL2 vary with the host's load and file cache. The no-op and touch rebuilds depend on what was just built.
- `jobs = 4` on 8 hardware threads, from the maintainer's user config, halves the parallelism CI would use.
- Every GPU number is llvmpipe on the CPU, so GPU time competes with the UI thread for cores. It says nothing about a real GPU.
- The benches report averages or p50/p99 computed inside the program; this report takes the median of three such summaries, not percentiles over raw samples.
- Release binaries carry the default profile; a fork profile with LTO, `codegen-units = 1`, or `panic = "abort"` would change size and speed.
- Nothing here was measured on macOS or Windows.

## Reproduce

```sh
# Builds (fresh target, no wrapper)
export CARGO_TARGET_DIR=target/a9-cold RUSTC_WRAPPER= HOST_CC=cc HOST_CXX=c++ CC_KNOWN_WRAPPER_CUSTOM=
cargo build --locked -p anti-webui --example showcase --timings
cargo build --locked -p anti-webui --example showcase
touch src/anti-webui-core/src/lib.rs && cargo build --locked -p anti-webui --example showcase --timings
touch src/anti-webui/examples/showcase.rs && cargo build --locked -p anti-webui --example showcase
cargo build --locked --release -p anti-webui --example showcase --example widgets --example multi_window --timings
touch src/anti-webui-core/src/lib.rs && cargo build --locked --release -p anti-webui --example showcase
# Sizes
strip -o showcase.stripped $CARGO_TARGET_DIR/release/examples/showcase
size -A $CARGO_TARGET_DIR/release/examples/showcase
cargo bloat --locked --release -p anti-webui --example showcase --crates -n 20
# Headless frame cost
cargo build --locked --release -p anti-webui --example bench_changes --example bench_drag
/usr/bin/time -f '%e s %M KiB' $CARGO_TARGET_DIR/release/examples/bench_changes
/usr/bin/time -f '%e s %M KiB' $CARGO_TARGET_DIR/release/examples/bench_drag
rm -rf target/a9-cold
```
