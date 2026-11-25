# Documentation policy

This policy covers documents the fork creates. Upstream documents (`README.md`, `docs/design/`) stay as written, in their original language ([ADR-0004](decisions/0004-english-fork-documentation.md)).

## Docs as Code

Documents are written, managed, and published the same way as code (maintainer direction, 2026-11-25).

- Write: plain text in this repository (Markdown, OpenSpec artifacts, rustdoc, text diagrams), next to the code they describe.
- Change: through branches and pull requests, reviewed like code: the agent's diff review, CodeRabbit, and the maintainer ([CI and quality services](quality/ci-and-quality-services.md#pull-request-review-coderabbit)).
- Check: by machines at every gate. hk runs the fast checks on commit; CI runs the same mise tasks on every pull request, rustdoc and doc tests with the build, and the online link check on a schedule (Phase 5).
- Publish: CI builds the documentation site and rustdoc from `main` and publishes them to GitHub Pages. The site generator is chosen in Phase 5; mdBook is the first candidate ([tool adoption matrix](quality/tool-adoption-matrix.md)).
- Generate, do not copy: API references, metrics, and dependency graphs are generated from the source by tools, never pasted by hand.

## Language

- English is canonical for technical documents, OpenSpec artifacts, architecture documents, ADRs, `AGENTS.md`, `CLAUDE.md`, source comments, rustdoc, commit messages, issues, and pull requests.
- Conversation with the maintainer may be in Japanese. Anything decided in conversation is written down here in English before it counts.

## Format

- GitHub Flavored Markdown, UTF-8, LF line endings.
- File names are kebab-case. ADRs start with a four-digit number (`0007-…`).
- Headings use sentence case: capitalise only the first word and proper nouns.
- One topic per file. A document links to the single source of truth instead of repeating it.
- `mise run docs:lint`, `docs:links`, and `spell` check these rules ([tool adoption matrix](quality/tool-adoption-matrix.md)).

## Evidence labels

Every technical claim carries one label. A claim without a label counts as Unverified.

| Label | Meaning | Required support |
|---|---|---|
| Confirmed | Checked during the work | A path and symbol, a commit SHA, a command and its result, or a primary-source URL |
| Inferred | Follows from confirmed facts | The facts it follows from |
| Proposed | A recommendation that still needs a decision | The decision record or OpenSpec change that will settle it |
| Unverified | Believed but not checked | What would check it |

Performance claims are Confirmed only with a reproducible [benchmark report](templates/benchmark-report.md).

## Document types and where they live

| Type | Location | Template |
|---|---|---|
| Entry point and navigation | `docs/fork/README.md` | — |
| Project brief, principles, roadmap | `docs/fork/*.md` | — |
| Project constitution (Phase 2) | `docs/fork/constitution.md` | [project-constitution](templates/project-constitution.md) |
| Research: audits, competitor analysis, gap analysis | `docs/fork/research/` | [technical-audit](templates/technical-audit.md), [competitor-analysis](templates/competitor-analysis.md), [gap-analysis](templates/gap-analysis.md) |
| Technical spikes (Phase 4) | `docs/fork/research/spikes/` | [technical-spike](templates/technical-spike.md) |
| Architecture (Phase 3) | `docs/fork/architecture/`, following the arc42 section order | — (see below) |
| Decisions | `docs/fork/decisions/NNNN-*.md` | [adr](templates/adr.md) (MADR 4.0.0) |
| Performance requirements and results | `docs/fork/performance/` | [benchmark-report](templates/benchmark-report.md) |
| Quality policies and tool choices | `docs/fork/quality/` | — |
| Requirements and changes | `openspec/specs/`, `openspec/changes/` | OpenSpec schema |
| Session handoff | `docs/fork/handoff.md` | [handoff](templates/handoff.md) |

Folders are created when their first document is written. Empty placeholder folders are not committed.

## Single source of truth

| Information | Canonical place |
|---|---|
| Behaviour the code must have (requirements, scenarios) | `openspec/specs/` |
| Work in progress for one change | `openspec/changes/<change>/` |
| Why a structural choice was made | `docs/fork/decisions/` |
| How the system is built (structure, views, cross-cutting concepts) | `docs/fork/architecture/` |
| Phase order and status | [roadmap](roadmap.md) |
| Current session state | [handoff](handoff.md) |
| Tool versions | `mise.toml` (the matrix explains, `mise.toml` pins) |

When two documents disagree, the canonical place wins and the other document is fixed.

## Third-party templates

- MADR 4.0.0 is `MIT OR CC0-1.0`. The ADR template adapts it and names the source.
- arc42 is CC BY-SA 4.0. Architecture documents follow its section order and cite it, but do not copy its template text, so ShareAlike does not apply to fork files. Copying the text needs a decision first.
- C4 diagrams are text (for example Structurizr DSL or Mermaid) kept beside the architecture document. The tool is chosen in Phase 3.
