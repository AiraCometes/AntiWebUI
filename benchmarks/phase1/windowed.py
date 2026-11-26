#!/usr/bin/env python3
"""Phase 1 A9: windowed baseline measurements.

Run from the repository root, in a desktop session (not the agent sandbox, which cannot reach the
compositor):

    cargo build --release --locked -p anti-webui --example bench_pacing --example bench_shaders
    python3 benchmarks/phase1/windowed.py --runs 5 > windowed.json

Each run starts an example with ANTIWEBUI_FRAME_STATS, records the launch time on the same clock
the CSV uses (CLOCK_MONOTONIC), samples /proc while the window is idle, and reads the CSV when the
example closes itself. Nothing is sent anywhere; the JSON goes to stdout.

Measured per run:
- startup_ms: launch to the first frame's `presented` time (`end` if the compositor gives none)
- idle_wakeups_per_s: context switches of all threads per second while nothing changes
- idle_presents: frames presented while nothing changes
- rss_idle_kib, rss_peak_kib: VmRSS during the idle window, VmHWM at its end
- frame_ms: latency, start -> presented (or end), of each frame under load, as median, p95, p99

On WSL the default GPU is llvmpipe. To reach the Windows GPU, run with
GALLIUM_DRIVER=d3d12 WGPU_BACKEND=gl (OpenGL on Mesa's D3D12 driver, through /dev/dxg). If the
example cannot find a compositor, check /mnt/wslg/stderr.log for Weston crashes; wsl --shutdown
with the Windows session unlocked restarts WSLg.
"""

import argparse
import csv
import json
import os
import subprocess
import sys
import time
from pathlib import Path

EXAMPLES = Path("target/release/examples")


def ctxt_switches(pid):
    total = 0
    for task in Path(f"/proc/{pid}/task").iterdir():
        try:
            for line in (task / "status").read_text().splitlines():
                if line.startswith(("voluntary_ctxt_switches", "nonvoluntary_ctxt_switches")):
                    total += int(line.split()[1])
        except OSError:
            pass
    return total


def status_kib(pid, key):
    for line in Path(f"/proc/{pid}/status").read_text().splitlines():
        if line.startswith(key + ":"):
            return int(line.split()[1])
    return None


def pct(values, p):
    values = sorted(values)
    if not values:
        return None
    k = (len(values) - 1) * p
    lo, hi = int(k), min(int(k) + 1, len(values) - 1)
    return values[lo] + (values[hi] - values[lo]) * (k - lo)


def run(example, args, load_s, idle_s, csv_path):
    env = dict(os.environ, ANTIWEBUI_FRAME_STATS=str(csv_path))
    launch = time.monotonic_ns()
    proc = subprocess.Popen([str(EXAMPLES / example), *args], env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    # The idle window starts some time after the load ends, and stops before the example closes.
    margin = 1.0
    time.sleep(load_s + margin)
    idle = {}
    if proc.poll() is None and idle_s > 2 * margin:
        t0, c0 = time.monotonic_ns(), ctxt_switches(proc.pid)
        idle["rss_idle_kib"] = status_kib(proc.pid, "VmRSS")
        time.sleep(idle_s - 2 * margin)
        t1, c1 = time.monotonic_ns(), ctxt_switches(proc.pid)
        idle["rss_peak_kib"] = status_kib(proc.pid, "VmHWM")
        idle["idle_wakeups_per_s"] = (c1 - c0) / ((t1 - t0) / 1e9)
        idle["window_ns"] = (t0, t1)
    _, stderr = proc.communicate(timeout=120)
    out = {"example": example, "args": args, "returncode": proc.returncode,
           "summary": stderr.strip()}
    if not csv_path.exists():
        # The example failed before it could close its window; keep what it said and go on.
        out["error"] = "no frame-stats CSV was written"
        print(f"{example} failed (exit {proc.returncode}):\n{stderr.strip()[-2000:]}",
              file=sys.stderr)
        return out
    rows = list(csv.DictReader(open(csv_path)))
    shown = lambda r: int(r["presented"] or r["end"])
    out["frames"] = len(rows)
    if rows:
        out["startup_ms"] = (shown(rows[0]) - launch) / 1e6
        loaded = [r for r in rows if shown(r) - launch < load_s * 1e9]
        frame = [(shown(r) - int(r["start"])) / 1e6 for r in loaded]
        out["frame_ms"] = {"median": pct(frame, 0.5), "p95": pct(frame, 0.95),
                           "p99": pct(frame, 0.99), "n": len(frame)}
    if "window_ns" in idle:
        t0, t1 = idle.pop("window_ns")
        idle["idle_presents"] = sum(1 for r in rows if t0 <= shown(r) <= t1)
        idle["idle_window_s"] = (t1 - t0) / 1e9
    out.update(idle)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--load", type=float, default=6.0, help="seconds of load per run")
    ap.add_argument("--idle", type=float, default=10.0, help="seconds of idle after the load")
    a = ap.parse_args()
    cases = [
        ("bench_pacing", ["--seconds", str(a.load), "--linger", str(a.idle)], a.load, a.idle),
        ("bench_pacing", ["--seconds", str(a.load), "--rows", "2000", "--layers", "40",
                          "--linger", "0"], a.load, 0.0),
        ("bench_shaders", ["--seconds", str(a.load), "--kind", "same", "--count", "1000"],
         a.load, 0.0),
    ]
    results = []
    for example, args, load_s, idle_s in cases:
        for i in range(a.runs):
            path = Path(f"/tmp/a9-{example}-{os.getpid()}-{len(results)}.csv")
            results.append(run(example, args, load_s, idle_s, path))
            path.unlink(missing_ok=True)
            print(f"{example} {' '.join(args)} run {i + 1}/{a.runs} done", file=sys.stderr)
    env = {k: os.environ.get(k) for k in ("XDG_SESSION_TYPE", "WAYLAND_DISPLAY", "DISPLAY")}
    print(json.dumps({"environment": env, "results": results}, indent=1))


if __name__ == "__main__":
    main()
