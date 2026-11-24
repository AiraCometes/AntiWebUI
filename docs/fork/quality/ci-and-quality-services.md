# CI and quality services plan

- Date: 2026-11-25
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Status: the maintainer chose the services (2026-11-25). Phase 4 measures the build cache, and Phase 5 sets up everything here. Nothing in this file is configured yet.

Maintainer direction (2026-11-25):

- Documents follow Docs as Code: written, managed, and published like code. The rules are in the [documentation policy](../documentation-policy.md#docs-as-code).
- GitHub Actions CI builds and tests on Linux, Windows, and macOS runners. The runners have no GPU, so GPU work uses a software rasteriser such as Mesa llvmpipe.
- Mend Renovate updates dependencies instead of Dependabot. It groups related crates into one pull request and merges it automatically after CI passes.
- CodeQL, secret scanning, and dependency review are on.
- Codecov shows coverage. SonarQube Cloud tracks complexity and technical debt.
- CodeRabbit reviews pull requests, in addition to the agent's diff review.
- Kache caches builds locally and in CI, with `ghcr.io` as its OCI remote. Old remote entries are removed by a scheduled cleanup, because Kache never deletes them.

Tool versions and adoption phases are in the [tool adoption matrix](tool-adoption-matrix.md). This file holds the design and the open questions.

## CI matrix and GPU tests

### Facts

| Item | Finding | Label |
|---|---|---|
| Upstream backend order | Windows tries DX12, then the primary backends, then GL; other platforms try the primary backends, then GL | Confirmed (`src/anti-webui-wgpu/src/lib.rs`, `attempts`) |
| Upstream GPU tests without an adapter | They print `skipping: …` and pass | Confirmed ([inspection](../research/phase-0-inspection.md#code-size-tests-and-benchmarks)) |
| How wgpu runs its own GPU tests on GitHub runners | Linux: Vulkan on lavapipe (Mesa) and GL on llvmpipe. Windows: DX12 on WARP and GL on llvmpipe (Mesa). macOS: Metal on the runner's device, `macos-14` | Confirmed (`gfx-rs/wgpu` `README.md` "Testing" and `.github/workflows/ci.yml`, job `gpu-test`) |
| Mesa builds for Windows CI | `gfx-rs/ci-build` provides the Mesa build that wgpu and rerun use on Windows | Confirmed (`rerun-io/rerun#8620`) |
| macOS runner GPU | Other projects report an "Apple Paravirtual device" with Metal on arm64 runners | Unverified: print the adapter in the first macOS CI job |

### Plan

| Runner | Backend in CI | Software device | Why |
|---|---|---|---|
| Linux (`ubuntu-24.04`) | Vulkan | Mesa lavapipe (llvmpipe for Vulkan) | The primary backend on Linux, as in wgpu's CI |
| Windows (`windows-2025`) | DX12, and Vulkan | WARP for DX12; Mesa lavapipe from `gfx-rs/ci-build` for Vulkan | Upstream picks DX12 first on Windows, so WARP tests the path users run; lavapipe gives a second, Mesa-based check |
| macOS (arm64) | Metal | The runner's paravirtual device (no Mesa) | Mesa has no Metal driver; Vulkan on macOS needs MoltenVK, which upstream does not use |

Rules (Proposed):

- A GPU job fails when a test skips for lack of an adapter. Until the Phase 4 decision on editing upstream-owned files, CI can do this without touching upstream code: fail the job when the test output contains `skipping:`. Label: Inferred from the skip message above.
- Each GPU job prints the adapter name and backend, so a green job shows what it ran on.
- Pixel results differ between software rasterisers and real GPUs. Golden-image tests record which device made each reference image, and the tolerance is set per device in Phase 5.
- Linux window tests need a display server. Run them under a headless Wayland compositor (for example Weston with its headless backend) and Xvfb for X11. Which tests need a window is a Phase 1 audit item (workstream A6).

## Dependency updates: Renovate

Plan (Proposed):

- Install the Mend Renovate GitHub app on `AiraCometes/AntiWebUI` only, with a `renovate.json5` in the repository.
- Group related packages: the wgpu family (`wgpu`, `wgpu-core`, `wgpu-hal`, `naga`), the zbus family, `windows` and `windows-sys`, `objc2` crates, GitHub Actions, and mise tools.
- Automerge after CI passes for patch and minor updates; major updates wait for the maintainer. Use GitHub's native auto-merge, so the branch protection's required checks decide.
- Set `minimumReleaseAge` (for example 3 days), so a release that is pulled soon after publication never lands.
- Keep Dependabot alerts on, because Renovate's `vulnerabilityAlerts` reads them. Dependabot version updates and security update pull requests stay off.

Open questions:

- `Cargo.toml` and `Cargo.lock` are upstream-owned ([ADR-0002](../decisions/0002-track-upstream-as-a-soft-fork.md)). A Renovate pull request edits them, and the next upstream merge then conflicts. Until the Phase 4 decision on editing upstream-owned files, Renovate updates only fork-owned files: GitHub Actions, `mise.toml`, and fork crates added later. Label: Inferred.
- The wgpu crates come from `bea4dev/wgpu` by git branch, so Renovate cannot update them by version. Label: Confirmed (`Cargo.toml` `[patch.crates-io]`).

## Security: CodeQL, secret scanning, dependency review

| Service | Facts | Plan (Proposed) |
|---|---|---|
| CodeQL | Rust is generally available since 2025-10-14, in default and advanced setup; nightly features are not supported (Confirmed: GitHub changelog 2025-10-14, CodeQL supported-languages page) | Advanced setup in a workflow, languages `rust` and `actions`, on pull requests and weekly |
| Secret scanning and push protection | Repository setting; no workflow needed | Turn on now or in Phase 5; it costs nothing to run |
| Dependency review | `actions/dependency-review-action` reads the dependency graph | Fail a pull request that adds a dependency with a known vulnerability or a licence outside the allow list. It checks only the pull request's changes; cargo-deny checks the whole graph, so both are kept |

## Coverage and code quality: Codecov and SonarQube Cloud

| Service | Facts | Plan (Proposed) |
|---|---|---|
| Codecov | wgpu uploads an LCOV report from cargo-llvm-cov with `codecov/codecov-action` (Confirmed: `gfx-rs/wgpu` `ci.yml`) | Upload coverage from every OS job, flagged per OS, so one platform's untested code shows. Pull request comments show the coverage change |
| SonarQube Cloud | Analyses Rust on all plans, including Free; imports LCOV or Cobertura coverage and Clippy JSON; reports cognitive and cyclomatic complexity and duplication. Clippy and coverage import need CI-based analysis, not automatic analysis (Confirmed: Sonar documentation, "Rust" and "External analyzer reports") | CI-based analysis on Linux with the Clippy JSON and the LCOV report. The quality gate applies to new code only, so upstream's existing code does not block pull requests |

SonarQube Cloud covers complexity and duplication, so rust-code-analysis and jscpd are no longer planned ([matrix](tool-adoption-matrix.md)).

## Pull request review: CodeRabbit

Plan (Proposed):

- Install the CodeRabbit GitHub app on the fork only, with a `.coderabbit.yaml`. Its review instructions point to `AGENTS.md` and the documentation policy, and its comments are in English.
- Order: the agent's diff review, then CodeRabbit. Neither replaces the maintainer's approval, except for the Renovate automerge rules above.
- Skip Renovate pull requests, or review them lightly, so automerge does not wait on review comments.
- Free use for public repositories: Unverified; check the current plan when installing.

## Build cache: Kache with GHCR

### Facts

| Item | Finding | Label |
|---|---|---|
| Release | `kunobi-ninja/kache` v2.3.0, 2026-11-21, Apache-2.0 | Confirmed (GitHub API) |
| Remotes | S3, GCS, a shared filesystem, or an OCI registry such as GHCR (`[cache.remote] type = "oci"`, `repository = "ghcr.io/<owner>/<name>"`) | Confirmed (Kache docs, "OCI registry setup") |
| CI credentials | `KACHE_OCI_TOKEN`, or `KACHE_OCI_USERNAME` with `KACHE_OCI_PASSWORD`; leave them unset, not empty, in jobs without the secret | Confirmed (same page) |
| Who writes | On GitHub Actions, a push to a protected branch writes; other jobs read only. Scheduled and merge-queue jobs write only when listed in `KACHE_GITHUB_WRITES`. Pull request jobs on an OCI remote are always read-only | Confirmed (Kache docs, "CI setup") |
| Deletion | Kache never deletes tags. Every cache object is its own tag, so all `kache-v2-` and `kache-v1-` tags must expire at the same age | Confirmed (Kache docs, "OCI registry setup") |
| Listing cost | A registry cannot list by prefix; each listing reads the whole tag list, 100 tags per request | Confirmed (same page) |
| Platforms | Linux, macOS, and Windows; Rust executables and build scripts are cached by default on Linux and macOS only. `kache-action` sets `cache-executables` to `false` by default | Confirmed (`README.md`, Kache docs "CI setup") |
| GHCR retention | GHCR has no retention setting; old versions are deleted through the REST API or a workflow. A deleted version can be restored for 30 days. A public package version with more than 5,000 downloads cannot be deleted through the API | Inferred from two secondary sources; check against GitHub's "Deleting and restoring a package" page in Phase 4 |
| `actions/delete-package-versions` | Deletes at most 100 versions per run, selects by count, not by age, and matches `ignore-versions` against the digest, not the tag | Confirmed (action `README.md`) |

### Plan (Proposed)

- Local: pin Kache in `mise.toml` and run `kache init`; set a disk budget for the local store.
- CI: `kunobi-ninja/kache-action` with a pinned version, the OCI remote `ghcr.io/airacometes/antiwebui-kache` (a dedicated package), and `cache-executables: true` where it helps. Only pushes to `main` and a scheduled warm-up job write.
- Cleanup: a scheduled workflow that lists the package's versions through the REST API and deletes every version older than N days, regardless of tag. A script is needed because the action selects by count and caps each run at 100.
- The Phase 4 build-cache spike measures cold and warm builds on all three runners, the hit rate, the tag count after a week, and listing time, and picks N.

Open questions:

- The 5,000-download limit: hot cache entries, such as a popular dependency, could pass it and become undeletable through the API. Options: keep the package private (storage then counts against the account's quota), or move to a new package name every period and delete the old package. Settle in the Phase 4 spike.
- A public package exposes compiled outputs. They are built from public sources, so this is not a secret leak, but nobody outside CI may write to it. Label: Inferred.

## What the maintainer sets up outside the repository

These are account and repository settings, done once in Phase 5 (secret scanning can be earlier):

- Install the Renovate, CodeRabbit, and Codecov GitHub apps on the fork only.
- Create the SonarQube Cloud organisation and project, and add `SONAR_TOKEN` as a secret.
- Add `CODECOV_TOKEN` as a secret, unless tokenless upload works for this public repository.
- Turn on secret scanning, push protection, Dependabot alerts, and the dependency graph.
- Protect `main` with the required checks, so Renovate automerge waits for CI.
- Give the workflow `packages: write` for the Kache package, and admin access for the cleanup workflow.
