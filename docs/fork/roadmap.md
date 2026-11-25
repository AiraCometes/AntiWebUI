# Development roadmap

Phase order and status. Moving to the next phase needs the maintainer's approval. The order may change based on audit and validation results. Record such changes here, and in an ADR when they are structural.

| Phase | Name | Status | Exit condition |
|---|---|---|---|
| 0 | Project bootstrap | done (2026-11-25, [#2](https://github.com/AiraCometes/AntiWebUI/pull/2)) | Phase 0 pull request merged |
| 1 | Upstream audit | in progress (approved 2026-11-25) | [Audit plan](research/phase-1-audit-plan.md) exit criteria met |
| 2 | Requirements engineering | not started | Constitution and master requirements accepted |
| 3 | Master architecture | not started | arc42 architecture and core ADRs accepted |
| 4 | Technical validation | not started | Spikes answered; ADRs final |
| 5 | Engineering infrastructure | not started | Baseline specs, fitness functions, CI, and benchmark infrastructure in place |
| 6 | Incremental implementation | not started | Per OpenSpec change |
| 7 | Optimisation and stabilisation | not started | Release criteria met |
| 8 | Independent design kits | not started | Separate repositories |

## Phase 0: project bootstrap

Upstream commit pinning, repository and environment inspection, documentation conventions, templates, agent instructions, a minimal toolchain, and this roadmap. Outputs are listed in the [fork README](README.md).

## Phase 1: upstream audit

A complete codebase and documentation audit, capability inventory, dependency analysis, platform matrix, licence audit, technical debt, existing tests and benchmarks, and baseline measurements. See the [plan](research/phase-1-audit-plan.md).

## Phase 2: requirements engineering

Competitor research, the [gap analysis](templates/gap-analysis.md) (REUSE, EXTEND, OPTIMIZE, NEW, REPLACE, DEFER, DROP), grilling sessions, the project constitution, master requirements, functional and non-functional requirements, and non-goals. The requirement candidates from the bootstrap brief (rendering and GPU, spatial and input, lifecycle and resources, motion, headless UI and accessibility, typography and effects, testing and performance, extensibility) are inputs, not commitments.

## Phase 3: master architecture

arc42 architecture, subsystem boundaries, dependency rules, coordinate systems, ownership and lifecycle, rendering and invalidation, GPU resource management, threading and platform abstraction, testing strategy, and performance budgets. Platform-specific materials such as Windows Mica stay in OS backends. Design kits get generic primitives (drawing, state machines, motion, accessibility, materials), never kit-specific code.

## Phase 4: technical validation

Spikes, proofs of concept, architecture review, performance experiments, cross-platform feasibility, final ADRs, the policy on editing upstream-owned files and on pull requests to upstream, the build-cache spike (Kache with its GHCR remote and cleanup, measured against incremental builds), and the [test oracle and model-checking spikes](research/candidate-test-oracles-and-model-checking.md) (taffy differential layout tests, proptest-state-machine, Quint or Stateright, Loom and Shuttle).

## Phase 5: engineering infrastructure

OpenSpec baseline specifications, fitness functions, quality gates, CI/CD on GitHub Actions for Linux, Windows, and macOS, security checks, the services in the [CI and quality services plan](quality/ci-and-quality-services.md) (Renovate, CodeQL, secret scanning, dependency review, Codecov, SonarQube Cloud, CodeRabbit, Kache), documentation publishing, benchmark and testing infrastructure, upstream sync automation, and implementation epics with their dependency graph.

## Phase 6: incremental implementation

One OpenSpec change at a time:

Explore → Grilling → Proposal → Specs → Design → Review → Tasks → Implementation → Verification → Code review → Archive

A retro follows a session that produced useful lessons.

## Phase 7: optimisation and stabilisation

Competitive benchmarks, CPU and GPU profiling, memory work, native testing on every platform, public API stabilisation, documentation, and release preparation.

## Phase 8: independent design kits

HeroUI Kit and Fluent 2 Kit, each in its own repository, after the framework is stable. The framework never depends on a design kit.
