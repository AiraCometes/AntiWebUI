# Benchmark: subject

<!-- Save as docs/fork/performance/<yyyy-mm-dd>-<subject>.md. A performance claim is Confirmed only with a
     report like this. Raw data goes beside the report or in a linked artifact. -->

- Date: YYYY-MM-DD
- Commit: full SHA of this repository (and of each compared framework)
- Harness: benchmark name and its path under benchmarks/ or the crate

## Question

What is measured and why. Link the requirement or budget it checks.

## Environment

| Item | Value |
|---|---|
| OS and version | |
| Display server / compositor | |
| CPU, GPU, driver | |
| Display resolution, refresh rate, scale factor | |
| Power settings | |
| Rust toolchain, profile, flags, features | |

## Method

Workload, warm-up, iterations, and how outliers are handled. For framework comparisons: the same visual output, features, resolution, DPI, GPU, OS, and build settings for each framework, and any difference that remains.

## Results

| Metric | Unit | Median | P95 | P99 | N |
|---|---|---|---|---|---|

Metrics come from the list in `docs/fork/principles.md` (startup, binary size, memory, GPU memory, idle wakeups, frame time, input latency, allocations).

## Comparison with baseline

The previous report or upstream baseline, and the change in percent.

## Threats to validity

Noise sources, missing platforms, and what the numbers do not show.

## Reproduce

The exact commands.
