# Phase 0 repository inspection

- Date: 2026-11-24
- Upstream baseline: `6908e3119db4ecf7080f57de59870e083f2cbc98` (`bea4dev/AntiWebUI`, `main`, 2026-11-19 "improve prev PR")
- Scope: the state of the fork before any fork-specific work. A complete audit is Phase 1 ([plan](phase-1-audit-plan.md)).

Every statement is tagged with an evidence label from the [documentation policy](../documentation-policy.md#evidence-labels).
Commands were run on WSL2 (Linux 6.18, x86_64) inside the agent sandbox.

## Git state

| Item | Finding | Label |
|---|---|---|
| Remotes | `origin` = `https://github.com/AiraCometes/AntiWebUI` (public, GitHub fork of `bea4dev/AntiWebUI`); `upstream` = `https://github.com/bea4dev/AntiWebUI` | Confirmed (`git remote -v`, `gh repo view`) |
| Branches | Only `main`; `main` = `origin/main` = `upstream/main` = `6908e31`; 0 commits ahead, 0 behind | Confirmed (`git rev-list --left-right --count main...upstream/main`) |
| History | 98 commits, first on 2026-11-02; 90 by `bea4dev`, the rest from 4 external contributors | Confirmed (`git log`) |
| Upstream activity | 10 open PRs, all from one external contributor (#9 to #18); merged external PRs get a follow-up "improve prev PR" commit by the author | Confirmed (`gh pr list -R bea4dev/AntiWebUI`) |
| AI authorship upstream | 21 upstream commits carry a `Co-Authored-By: Claude` trailer | Confirmed (`git log --format='%(trailers…)'`) |
| Upstream contribution rules | No `CONTRIBUTING`, code of conduct, issue template, or PR template | Confirmed (GitHub community profile API) |
| Fork settings | `allow_auto_merge: false`; merge, squash, and rebase merges all allowed | Confirmed (`gh api repos/AiraCometes/AntiWebUI`, earlier session the same day) |

Inference: upstream changes quickly (98 commits in 23 days). Any change to an upstream-owned file is a likely merge conflict, so fork work must keep those edits small ([ADR-0002](../decisions/0002-track-upstream-as-a-soft-fork.md)).

## Cargo workspace and toolchain

| Item | Finding | Label |
|---|---|---|
| Workspace | 21 member crates under `src/`, `resolver = "2"`, `edition = "2024"`, version `0.1.0` | Confirmed (`Cargo.toml`) |
| MSRV | No `rust-version` in any manifest; no `rust-toolchain(.toml)` | Confirmed (`rg rust-version`) |
| Local toolchain | rustc 1.99.0 and cargo 1.99.0 through mise; rustup also has 1.100.0 installed | Confirmed (`rustc -V`, `rustup show`) |
| Build | `cargo check --workspace --all-targets --locked` passes in 1 min 33 s (warm `target/`); output has many warnings, such as `float_literal_f32_fallback` | Confirmed (exit 0) |
| Formatting | `cargo fmt --all --check` reports 30 `Diff in` hunks on upstream code | Confirmed |
| wgpu | `[patch.crates-io]` replaces `wgpu`, `naga`, and 8 related crates with `bea4dev/wgpu` branch `adapter-filter`, locked at `ff143aeeb126dce521b87f47547565c4b6a6da1e` | Confirmed (`Cargo.toml`, `Cargo.lock`) |
| wgpu fork contents | One commit ("add adapter filter", 2026-11-11) on top of `v30.0.1`: DX12 `Dx12BackendOptions::adapter_filter` and pipeline-cache changes in `wgpu-hal/src/dx12/*`, `wgpu-types`, and `wgpu-core/src/pipeline_cache.rs` | Confirmed (`gh api …/compare/v30.0.1...ff143ae`) |
| wgpu upstream status | No PR from `bea4dev` in `gfx-rs/wgpu`; `gfx-rs/wgpu` has released `v30.0.2` (2026-11-12) | Confirmed (`gh search prs`, releases API) |

Rationale recorded upstream: the patch keeps a sleeping dGPU asleep at startup and keeps compiled shaders between launches on Windows. Source: the comment above `[patch.crates-io]` in `Cargo.toml` and `docs/design/39-platform-windows.md`. Label: Confirmed. Whether standard wgpu can replace the patch is Unverified and is a Phase 1 item.

## Code size, tests, and benchmarks

Rust lines are counted with `git ls-files <crate> | rg '\.rs$' | xargs cat | wc -l`. The count includes comments and blank lines. The files columns count tracked files under `tests/` and `examples/`.

| Crate | Rust lines | Test files | Example files |
|---|---:|---:|---:|
| `anti-webui-core` | 50,318 | 33 | 0 |
| `anti-webui-md3e` | 24,305 | 15 | 15 |
| `anti-webui-pdf` | 19,608 | 4 | 4 |
| `anti-webui-wayland-shell` | 15,978 | 18 | 0 |
| `anti-webui-app` | 13,624 | 27 | 7 |
| `anti-webui-wgpu` | 11,485 | 0 | 0 |
| `anti-webui-widgets` | 9,385 | 10 | 0 |
| `anti-webui-symbols-outlined` | 8,401 | 1 | 0 |
| `anti-webui-wayland` | 7,891 | 5 | 0 |
| `anti-webui` (facade) | 6,710 | 0 | 27 |
| `anti-webui-x11` | 6,326 | 1 | 0 |
| `anti-webui-markdown` | 6,097 | 5 | 3 |
| `anti-webui-video` | 5,049 | 11 | 0 |
| `anti-webui-macos` | 4,339 | 0 | 0 |
| `anti-webui-windows` | 4,106 | 0 | 0 |
| `anti-webui-code` | 3,347 | 1 | 0 |
| `anti-webui-math` | 2,278 | 1 | 1 |
| `anti-webui-style` | 2,261 | 0 | 0 |
| `anti-webui-macros` | 75 | 0 | 0 |
| `anti-webui-internal` | 55 | 0 | 0 |
| `anti-webui-dylib` | 8 | 0 | 0 |

- `#[test]` attributes: 1,693. Label: Confirmed (`rg -c '#\[test\]' src`).
- Benchmarks: no `benches/` directory and no `criterion` or `divan` dependency. Label: Confirmed.
- `unsafe` matches: 660. The count includes comments and identifiers, so it is not a count of unsafe blocks. Label: Confirmed as a text match; the real count is a Phase 1 item.
- GPU-dependent render tests print `skipping: …` and pass when no adapter is found, for example in `src/anti-webui-app/tests/render.rs`. Label: Confirmed. Consequence: a CI runner without a GPU or lavapipe passes these tests without running them. Label: Inferred.

## Development tools, CI, and configuration

| Item | Finding | Label |
|---|---|---|
| CI | No `.github/`, no `.woodpecker/` | Confirmed |
| Lint and format config | No `rustfmt.toml`, `clippy.toml`, `deny.toml`, `_typos.toml` | Confirmed |
| Task runners | No `justfile`, `Makefile`, `mise.toml`, `xtask` | Confirmed |
| Editor config | `.idea/` and `.vscode/` are not tracked; only `.gitignore` (`/target`, `__pycache__/`) | Confirmed (`git ls-files`) |
| `.claude/` | Local only; excluded through `.git/info/exclude` | Confirmed (`git check-ignore -v`) |
| Maintainer's user-level hk steps | `hk check --all` also runs the maintainer's user-level `ast-grep` rules, which report 17 errors in upstream code, for example `#[ignore = "reads this machine's tray"]` in `src/anti-webui-wayland-shell/tests/tray_bus.rs:241`. Pre-commit runs on staged files only, so fork-only commits are not affected | Confirmed (`hk check --all`); a baseline item for Phase 1 (A6, A7) |

Tool availability on this machine (2026-11-24) is recorded in the [tool adoption matrix](../quality/tool-adoption-matrix.md).

## Agent instructions and skills

- The repository had no `AGENTS.md`, `CLAUDE.md`, or skills. Label: Confirmed.
- User-level skills already installed: `grilling`, `handoff`, `retro`, `writing-for-agents`, `codebase-design`, `tdd`, `diagnosing-bugs`, and the OpenSpec skills. Label: Confirmed (`~/.claude/skills`, `~/.agents/skills`).
- `mattpocock/skills` at `abeeaf95660ba9e8b7c753eb38f441ca707a8823` (MIT) contains all eight candidate skills. Details are in the [tool adoption matrix](../quality/tool-adoption-matrix.md#agent-skills). Label: Confirmed.

## Documentation

- Upstream documents: `README.md` (English) and `docs/design/00-overview.md` to `41-gpu-canvas.md` plus `README.md`, 10,528 lines in total, written in Japanese. Each topic has one numbered file. `07-roadmap.md` tracks milestones M1 to M3. Label: Confirmed.
- These files stay as written ([documentation policy](../documentation-policy.md)).

## License and tracking constraints

| Item | Finding | Label |
|---|---|---|
| Workspace license | `MIT OR Apache-2.0` (`[workspace.package]`); `LICENSE-MIT` and `LICENSE-APACHE` at the root | Confirmed |
| Exception | `anti-webui-symbols-outlined` is `Apache-2.0` only, to match the bundled Material Symbols font | Confirmed (`Cargo.toml`, `README.md`) |
| Video | FFmpeg is loaded at run time; no FFmpeg code is included (`docs/design/24-video.md`) | Confirmed as documented; not re-verified |
| GitHub license detection | GitHub reports `Apache-2.0` for the upstream repository | Confirmed (API); it does not change the dual license |
| Workspace metadata | `authors = ["bea4dev"]`, `repository` points to upstream | Confirmed |

Constraints for tracking upstream (Inferred from the findings above):

1. `Cargo.toml` (members, `[workspace.dependencies]`, `[patch.crates-io]`) and `Cargo.lock` change often upstream, so every fork edit to them is a conflict point.
2. `docs/design/NN-*.md` numbers are allocated by upstream. Fork documents live under `docs/fork/` so they never take an upstream number.
3. Running `rustfmt` on upstream files would rewrite 30 hunks and conflict with upstream. Formatting checks apply only to fork-owned files until the policy changes.
4. Fork contributions keep the dual license. New fork files carry no separate license header.
5. Documentation templates must respect third-party licenses: MADR is `MIT OR CC0-1.0`; the arc42 template is CC BY-SA 4.0 ([ADR-0004](../decisions/0004-english-fork-documentation.md)).
