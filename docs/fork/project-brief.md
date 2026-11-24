# Project brief

- Status: provisional (Phase 0). Phase 2 replaces the goals with accepted requirements.

## What this is

A fork of [`bea4dev/AntiWebUI`](https://github.com/bea4dev/AntiWebUI), a Rust UI framework with its own wgpu renderer, integer physical-pixel layout, SolidJS-like signals, and native backends for Wayland, X11, macOS, and Windows. The fork aims to grow it into an independent, fast, lightweight, low-memory native GUI framework. It keeps upstream's technical assets and builds on them, instead of starting over.

## Goals

- Windows, macOS, and Linux are equal, officially supported platforms.
- No WebView, Chromium, DOM, or JavaScript runtime.
- Faster, lighter on CPU, and smaller in memory than GPUI, egui, Iced, Slint, Xilem, and Makepad, all at once. Each claim is proven by reproducible benchmarks, not assumed.
- Physical-pixel precision under fractional scaling.
- Advanced 2D canvas, custom GPU drawing, physics-based motion, and modern UI.
- Unused features cost nothing at run time.
- Long-term OSS: tracking upstream, a stable public API, and bounded dependencies, code size, crate count, and maintenance load.

## Scope now

The framework itself. Design kits (HeroUI Kit, Fluent 2 Kit) come after the framework stabilises, each in its own repository. The framework never depends on them. Generic building blocks the kits need (drawing primitives, state machines, motion, accessibility, materials) are considered in the architecture from the start.

## How work is organised

- Principles: [principles](principles.md) (provisional until the Phase 2 constitution).
- Phases: [roadmap](roadmap.md).
- Decisions: [decision log](decisions/README.md).
- Specifications: OpenSpec in `openspec/` ([ADR-0003](decisions/0003-openspec-as-the-single-specification-system.md)).
- Relationship to upstream: soft fork pinned to an audited baseline ([ADR-0002](decisions/0002-track-upstream-as-a-soft-fork.md)).

## Non-goals for now

- Implementing new GUI features before the audit, requirements, and architecture phases.
- Refactoring upstream code, or fixing every existing upstream lint at once.
- Claiming performance without measurements.
