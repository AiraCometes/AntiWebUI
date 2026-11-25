# Tool adoption matrix

Status as of 2026-11-24; Phase 1 audit tools pinned on 2026-11-25. `mise.toml` pins the versions; this file records why each tool is or is not used. Every tool here is development-only and never becomes a runtime dependency of a product crate.

Version and licence data come from the GitHub releases API and the crates.io API on 2026-11-24 (Confirmed). "Local" is what this machine has: `pinned` means `mise.toml` pins it, `available` means installed but not pinned for this repository, and `—` means not installed. The platform column is Unverified unless marked; Phase 5 confirms platforms per CI runner.

Adoption phases follow the [quality gates](#quality-gate-tiers): commit, pull request, critical change, and phase completion.

## Adopted in Phase 0

| Tool | Purpose | Version | Licence | Local | Gate |
|---|---|---|---|---|---|
| mise | Tool pins and task runner (all tasks go here) | 2026.11.14 latest; 2026.11.10 local | MIT | available | — |
| hk | Git hooks | 2.13.0 pinned (2.14.0 latest; 2.13.0 matches the user-level hk config) | MIT | pinned | commit |
| OpenSpec | Specs and changes | 2.0.0 pinned (2.1.1 latest) | MIT | pinned | commit |
| typos | Spelling | 1.52.2 | Apache-2.0 | pinned | commit |
| rumdl | Markdown lint | 0.3.5 | MIT | pinned | commit |
| taplo | TOML format check | 0.10.0 | MIT | pinned | commit |
| lychee | Link check (offline at commit, online on demand) | 0.24.2 | Apache-2.0 | pinned | commit |

## Planned, with entry conditions

| Tool | Purpose | Version | Licence | Local | Overlap and notes | Adopt in | Gate |
|---|---|---|---|---|---|---|---|
| rustup + `rust-toolchain.toml` | Pin the toolchain | stable 1.99 local | MIT/Apache-2.0 | available | Upstream has no pin or MSRV. Choose the pin after the Phase 1 baseline, because a pin changes builds for every contributor | 1 | — |
| rustfmt | Formatting | with toolchain | MIT/Apache-2.0 | available | Upstream fails `--check` in 30 hunks; apply to fork-owned code only | 1 (record), 5 (gate) | commit |
| Clippy | Lints | with toolchain | MIT/Apache-2.0 | available | Record the upstream warning baseline first | 1, 5 | pull request |
| rust-analyzer | IDE | — | MIT/Apache-2.0 | not on the active toolchain | Editor setup, not a gate | when needed | — |
| cargo xtask | Rust-specific automation behind mise tasks | — | — | — | A new workspace member edits upstream `Cargo.toml`; add it with the first Rust-only task | 5 | — |
| cargo metadata | Dependency graph source for fitness functions | with cargo | — | available | Reads the graph for architecture tests | 5 | pull request |
| cargo-modules | Module graph, cycle detection | 0.28.0 | MPL-2.0 | pinned | Phase 1 dependency analysis | 1 | phase completion |
| cargo-deny | Licences, advisories, bans, sources | 0.22.0 pinned | Apache-2.0 | pinned | Licence audit in Phase 1; `deny.toml` needs a `verify-commits accept` | 1, 5 | pull request |
| cargo-shear | Unused dependencies | 1.14.1 | MIT | available | Chosen over cargo-machete; use one, not both | 5 | pull request |
| cargo-machete | Unused dependencies | 0.10.1 | MIT | available | Overlaps cargo-shear | dropped | — |
| cargo-hack | Feature matrix | 0.6.46 | Apache-2.0 | — | Many features (`wayland`, `x11`, `macos`, `windows`, `svg`, `video`, …) | 5 | critical change |
| rust-code-analysis | Complexity metrics | 0.0.25 (2023-01-13) | MPL-2.0 | — | Superseded by SonarQube Cloud | dropped | — |
| jscpd | Duplicate code | 5.4.3 | MIT | — | Superseded by SonarQube Cloud | dropped | — |
| cargo-semver-checks | Public API compatibility | 0.51.0 | Apache-2.0 | — | Meaningful once a public API is declared | 7 | pull request |
| cargo-nextest | Test runner | 0.9.152 | Apache-2.0 | pinned | Check GPU tests that pass by skipping (see the [inspection](../research/phase-0-inspection.md#code-size-tests-and-benchmarks)) | 1, 5 | pull request |
| cargo-llvm-cov | Coverage | 0.9.2 pinned | Apache-2.0 | pinned | Baseline in Phase 1 | 1 | phase completion |
| cargo-mutants | Mutation testing | 27.1.0 | MIT | available | Changed files only on critical changes | 5 | critical change |
| proptest | Property tests (dev-dependency) | 1.12.0 | MIT OR Apache-2.0 | — | dev-dependency only | 6 | — |
| insta | Snapshot tests (dev-dependency) | 1.49.0 | Apache-2.0 | — | dev-dependency only | 6 | — |
| Kani | Model checking | 0.68.0 | MIT OR Apache-2.0 | — | Linux and macOS only (Unverified); target unsafe hot spots | 5 | phase completion |
| Miri | UB detection | nightly component | MIT/Apache-2.0 | — | Needs nightly; GPU and FFI code cannot run under it | 5 | phase completion |
| AddressSanitizer | Memory errors | nightly `-Zsanitizer` | — | — | Needs nightly | 5 | phase completion |
| AccessKit | Platform accessibility bridge (product dependency candidate, not a dev tool) | 0.25.2 | MIT OR Apache-2.0 | — | Upstream has its own access API (`docs/design/34-accessibility.md`); integration is a Phase 2 requirement question | 2 (decide) | — |
| kittest | AccessKit-based UI testing | 0.5.0 | MIT OR Apache-2.0 | — | `egui_kittest` 0.37.0 is the egui adapter; needs AccessKit first | 5 | pull request |
| Criterion | Micro-benchmarks | 0.8.2 | Apache-2.0 OR MIT | — | Choose one of Criterion and Divan in a Phase 4 spike | 4 | — |
| Divan | Micro-benchmarks | 0.1.21 (2025-04-10) | MIT OR Apache-2.0 | — | Last release is old | 4 | — |
| Tracy | Frame profiler | 0.15.0 | BSD-3-Clause | — | Feature-gated client crate only | 4 | — |
| samply | Sampling profiler | 0.13.1 | MIT OR Apache-2.0 | pinned | Local profiling | 1 | — |
| RenderDoc | GPU frame capture | 1.47 | MIT | — | No macOS support (Unverified) | 4 | — |
| cargo-bloat | Binary size breakdown | 0.12.1 (2024-05-10) | MIT | pinned | Baseline binary size in Phase 1 | 1 | phase completion |
| cargo-binstall | Installs prebuilt crate binaries for mise `cargo:` tools | 1.28.0 (2026-11-21) | GPL-3.0-only | pinned | Development-only installer, never linked into a crate. cargo-modules has no prebuilt binary and builds from source (about 3.5 min) | 1 | — |
| Allocation counter | Zero hot-path allocation checks | `allocation-counter` 0.8.1, `stats_alloc` 0.1.10, `dhat` 0.3.3, or a small in-repo global allocator | MIT/Apache-2.0 | — | All three crates are old; a test-only allocator may be simpler | 5 (decide) | pull request |
| Vale | Prose style | 3.28.1 | MIT | — | Add only if rumdl and review miss real problems | 5 (evaluate) | commit |
| mdBook | Docs site generator (Docs as Code) | 0.5.4 | MPL-2.0 | — | First candidate; choose the generator in Phase 5 ([documentation policy](../documentation-policy.md#docs-as-code)) | 5 | pull request |
| GitHub Pages | Hosts the docs site and rustdoc | — | — | — | Built and published by CI from `main` | 5 | — |
| GitHub Actions | CI on Linux, Windows, and macOS runners | — | — | — | All CI goes here ([plan](ci-and-quality-services.md#ci-matrix-and-gpu-tests)) | 5 | pull request |
| Mesa lavapipe and llvmpipe | Software Vulkan and GL devices for GPU tests | Linux packages; `gfx-rs/ci-build` on Windows | MIT | — | Runners have no GPU | 5 | pull request |
| WARP | Software DX12 device on Windows runners | ships with Windows | — | — | Tests upstream's default DX12 path | 5 | pull request |
| Mend Renovate | Grouped dependency updates with automerge after CI | GitHub app | AGPL-3.0 (self-hosted CLI; the app is a hosted service) | — | Replaces Dependabot version updates; fork-owned files only until Phase 4 ([plan](ci-and-quality-services.md#dependency-updates-renovate)) | 5 | pull request |
| CodeQL | Code scanning for Rust and Actions | GitHub service | — | — | Rust GA since 2025-10-14 | 5 | pull request |
| Secret scanning and push protection | Leaked-secret detection | GitHub setting | — | — | No workflow needed; can be turned on before Phase 5 | 5 | commit (push) |
| Dependency review | Blocks pull requests that add vulnerable or disallowed dependencies | `actions/dependency-review-action` | MIT | — | Pull request diff only; cargo-deny checks the whole graph | 5 | pull request |
| Codecov | Coverage reports and pull request comments | GitHub app and action | — | — | Uploads the cargo-llvm-cov LCOV report from each OS | 5 | pull request |
| SonarQube Cloud | Complexity, duplication, and technical debt | Free plan | — | — | Imports Clippy JSON and LCOV; gate on new code only | 5 | pull request |
| CodeRabbit | Automated pull request review | GitHub app | — | — | Runs after the agent's diff review; does not replace the maintainer's approval | 5 | pull request |
| actionlint | Workflow lint | 1.7.12 | MIT | available | With the first workflow | 5 | commit |
| zizmor | Workflow security | 1.33.1 | MIT | available | With the first workflow | 5 | commit |
| pinact | Pin actions by SHA | 5.0.2 | MIT | available | With the first workflow | 5 | commit |
| Dependabot | Dependency updates | — | — | — | Replaced by Renovate; only Dependabot alerts stay on, as Renovate's vulnerability source | dropped (alerts kept) | — |
| Kache | Build cache, local and CI, with `ghcr.io` as its OCI remote | 2.3.0 | Apache-2.0 | — | Chosen by the maintainer; the Phase 4 spike measures it and sets the GHCR cleanup age ([plan](ci-and-quality-services.md#build-cache-kache-with-ghcr)). Rust executables cached by default on Linux and macOS only | 4 (spike), 5 | — |
| sccache | Build cache | 0.20.0 latest; 0.19.0 local | Apache-2.0 | available | Fallback only if the Kache spike fails | 4 (spike) | — |
| taffy | Differential layout oracle and Chrome-generated fixtures (dev-dependency only, never the layout engine) | 0.16.0 | MIT | — | ±1 px per edge, because of edge snapping versus integer-first rounding ([evaluation](../research/candidate-test-oracles-and-model-checking.md#part-1-taffy-as-a-differential-layout-oracle)) | 1 (count fixtures), 4 (spike), 5 | pull request |
| proptest-state-machine | Single-thread state machines against a reference model (focus, gestures, undo/redo, history, scheduler logic) | 0.9.0 | MIT OR Apache-2.0 | — | Default for single-thread state machines | 4 (spike), 5 | pull request |
| Quint + ITF | Design-level invariants; ITF traces (`quint run --mbt --out-itf`) replayed by an in-repo driver | Quint 0.34.0 (npm) | Apache-2.0 | — | Needs Node.js (JVM for Apalache); choose one of Quint and Stateright ([evaluation](../research/candidate-test-oracles-and-model-checking.md#how-they-fit-together)) | 4 (spike) | critical change |
| `itf` crate | ITF JSON decoding | 0.4.0 (2025-05-28) | Apache-2.0 | — | Stale; an in-repo decoder for the needed subset is the fallback | 4 (spike) | — |
| Stateright | Model checking with models written in Rust | 0.31.0 (2025-07-27) | MIT | — | Alternative to Quint; no commits since 2025-07-27 | 4 (spike) | critical change |
| Loom | Exhaustive interleaving and memory-model checks for small cross-thread primitives | 0.7.3 | MIT | — | Needs a `cfg(loom)` sync facade; whether upstream-owned files get it is a Phase 4 decision | 4 (spike) | critical change |
| Shuttle | Randomised concurrency and async testing for larger flows | 0.9.10 | Apache-2.0 | — | Same facade as Loom; split: Loom for primitives, Shuttle for flows and async | 4 (spike) | critical change |
| Quint Connect | Trace replay against a Rust driver | 0.1.2 (2026-05-25) | Apache-2.0 | — | No longer updated; replaced by Quint + ITF with an in-repo driver | dropped | — |
| Competitor benchmark suite | Same workloads across frameworks | — | — | — | Lives in `benchmarks/` | 4, 7 | phase completion |

## Specification and documentation workflow

| Item | Status | Evidence | Decision |
|---|---|---|---|
| OpenSpec `spec-driven` schema | Adopted | `openspec/config.yaml` | [ADR-0003](../decisions/0003-openspec-as-the-single-specification-system.md) |
| `intent-driven` schema | Candidate | `intent-driven-dev/openspec-schemas`, MIT, last commit 2026-08-16, no releases; writes ADRs to `adr/`; written against OpenSpec 1.6 | Covered by MADR ([ADR-0001](../decisions/0001-record-decisions-as-madr.md)) |
| `epic-driven` schema | Candidate | `IvanFeric/openspec-epic-driven`, MIT, last commit 2026-10-16, no releases; targets OpenSpec 1.10; epic children use `spec-driven` | Reconsider in Phase 5 |
| `intent-driven` + `epic-driven` together | Not compatible within one change: epic children must be `spec-driven` | READMEs of both repositories | — |
| spec-gen | Renamed to OpenLore (`clay-good/OpenLore`, MIT, v3.8.0, 2026-11-22; supports Rust and OpenSpec) | GitHub redirect from `clay-good/spec-gen` | Phase 1 spike candidate |
| GitHub Spec Kit | Not used | Bootstrap brief | [ADR-0003](../decisions/0003-openspec-as-the-single-specification-system.md) |
| just | Not used | Bootstrap brief; tasks live in mise | — |

## Agent skills

Source: `mattpocock/skills` at `abeeaf95660ba9e8b7c753eb38f441ca707a8823` (MIT). Install them per user (`claude plugin install mattpocock-skills@claude-plugins-official`, or `npx skills@latest add mattpocock/skills`), not vendored into this repository.

| Skill | Path upstream | Installed for this user | Use | Notes |
|---|---|---|---|---|
| grilling | `skills/productivity/grilling` | yes | Initial design and major decisions | — |
| grill-with-docs | `skills/engineering/grill-with-docs` | no | Grilling that also writes ADRs and a glossary | Calls `grilling` and `domain-modeling` |
| domain-modeling | `skills/engineering/domain-modeling` | no | Glossary and ADRs | Defaults to `docs/adr/` and `GLOSSARY.md`. `setup-matt-pocock-skills` can point it at `docs/fork/decisions/`; run that setup before the first use |
| retro | `skills/engineering/retro` | yes | End of sessions with lessons | Prefer turning a lesson into an automated check |
| improve-codebase-architecture | `skills/engineering/improve-codebase-architecture` | no | Structural audit at phase completion | Uses `codebase-design` (installed) |
| code-review | `skills/engineering/code-review` | no | Review against standards and spec | Needs `setup-matt-pocock-skills` for the issue tracker |
| writing-for-agents | `skills/productivity/writing-for-agents` | yes | Editing `AGENTS.md`, `CLAUDE.md`, skills | — |
| handoff | `skills/productivity/handoff` | yes | Session handoff | Writes to `$TMPDIR` by design; this project writes the result to `docs/fork/handoff.md` ([ADR-0006](../decisions/0006-repository-as-the-source-of-truth-for-sessions.md)) |

Task-breakdown and spec skills that overlap OpenSpec (`to-spec`, `to-tickets`, `implement-spec`) are not adopted.

## Quality gate tiers

| Tier | Runs | Contents (target) | Now |
|---|---|---|---|
| Commit | hk pre-commit, seconds | Format, lint, and spelling on changed fork-owned files; OpenSpec validation | Markdown, links (offline), spelling, TOML, OpenSpec |
| Pull request | CI on Linux, Windows, and macOS, minutes | Build and tests (GPU tests on software devices), Clippy, dependency rules, fitness functions, CodeQL, dependency review, Codecov, SonarQube Cloud, agent review, CodeRabbit | none (Phase 5) |
| Critical change | CI on label or path, tens of minutes | Feature matrix, mutation testing on changed code, safety checks | none (Phase 5) |
| Phase completion | Manual or scheduled | Kani, ASan, broad mutation testing, competitor benchmarks, real-device checks | none |
