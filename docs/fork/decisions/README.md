# Decision log

Each row is one MADR file in this folder ([ADR-0001](0001-record-decisions-as-madr.md)). To add one, copy the [template](../templates/adr.md), take the next number, and add a row here.

| ADR | Title | Status | Date |
|---|---|---|---|
| [0001](0001-record-decisions-as-madr.md) | Record architecture decisions as MADR files in docs/fork/decisions | accepted | 2026-11-24 |
| [0002](0002-track-upstream-as-a-soft-fork.md) | Track upstream as a soft fork, pinned to an audited baseline | accepted | 2026-11-24 |
| [0003](0003-openspec-as-the-single-specification-system.md) | Use OpenSpec as the single specification system | accepted | 2026-11-24 |
| [0004](0004-english-fork-documentation.md) | Write fork documents in English under docs/fork, and keep upstream documents unchanged | accepted | 2026-11-24 |
| [0005](0005-minimal-phase-0-toolchain.md) | Start with a documentation-only toolchain in Phase 0 | accepted | 2026-11-24 |
| [0006](0006-repository-as-the-source-of-truth-for-sessions.md) | Keep agent instructions and session state in the repository | accepted | 2026-11-24 |

## Decisions awaiting confirmation

Smaller decisions made in conversation, which are not ADRs yet. Each one becomes an ADR or a constitution clause when it is confirmed.

| Topic | Current position | Source | Settles in |
|---|---|---|---|
| CI platform | All CI runs on GitHub Actions; the repository is public, so usage cost is not a constraint. Builds and tests run on Linux, Windows, and macOS runners, with software GPU devices (Mesa lavapipe or llvmpipe, WARP) ([plan](../quality/ci-and-quality-services.md#ci-matrix-and-gpu-tests)) | Maintainer, 2026-11-24 and 2026-11-25 conversations | Phase 5 |
| Docs as Code | Documents are written, reviewed, checked, and published like code; CI publishes the site and rustdoc to GitHub Pages ([policy](../documentation-policy.md#docs-as-code)) | Maintainer, 2026-11-25 conversation | Phase 5 (site generator) |
| Dependency updates | Mend Renovate, not Dependabot: grouped pull requests, automerge after CI. Fork-owned files only until the Phase 4 decision on upstream-owned files ([plan](../quality/ci-and-quality-services.md#dependency-updates-renovate)) | Maintainer, 2026-11-25 conversation | Phase 5 |
| Security and quality services | CodeQL, secret scanning, dependency review, Codecov, SonarQube Cloud, and CodeRabbit ([plan](../quality/ci-and-quality-services.md)) | Maintainer, 2026-11-25 conversation | Phase 5 |
| Build cache | Kache locally and in CI, with `ghcr.io` as the OCI remote and a scheduled cleanup of old tags ([plan](../quality/ci-and-quality-services.md#build-cache-kache-with-ghcr)) | Maintainer, 2026-11-25 conversation | Phase 4 spike, then ADR |
| Pull requests to upstream | Whether to send any at all is open; if sent, each one needs the maintainer's approval | Maintainer, 2026-11-24 and 2026-11-25 conversations | Phase 4 |
| Editing upstream-owned files | Whether fork work may edit upstream-owned files (for example a `cfg(loom)` sync facade in `anti-webui-core`), and how to keep tracking upstream if it does. Until then, fork work uses fork-owned paths and copies | Maintainer, 2026-11-25 conversation | Phase 4 |
| Officially supported platforms | Windows, macOS, and Linux as equals (bootstrap brief); the order of manual verification is open | Bootstrap brief | Phase 2 |
| taffy | Used only as a differential-testing oracle, never as the layout engine | Maintainer, 2026-11-25 conversation | Phase 4 spike, then ADR |
| State-transition and concurrency testing | Quint Connect is not used. Candidates: proptest-state-machine, Quint + ITF or Stateright, Loom, Shuttle ([evaluation](../research/candidate-test-oracles-and-model-checking.md)) | Maintainer, 2026-11-25 conversation | Phase 4 spikes, then ADR |
| Upstream wgpu fork | Kept as-is until Phase 1 audits it | Phase 0 inspection | Phase 1 audit, then ADR |
