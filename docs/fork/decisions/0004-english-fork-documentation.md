---
status: accepted
date: 2026-11-24
decision-makers: AiraCometes (maintainer)
---

# Write fork documents in English under docs/fork, and keep upstream documents unchanged

## Context and problem statement

Upstream documents are in Japanese, in `docs/design/00-…` to `41-…`, with numbers that upstream allocates. The fork aims at a wider OSS audience and many agents, and the bootstrap brief makes English canonical for new documents.

## Decision drivers

- Controlled Public API and long-term OSS use: English reaches more contributors
- Low merge cost with upstream ([ADR-0002](0002-track-upstream-as-a-soft-fork.md))
- Licence hygiene for third-party templates

## Considered options

- English fork documents under `docs/fork/`, upstream documents untouched
- Translate or extend `docs/design/` in place

## Decision outcome

Chosen option: English fork documents under `docs/fork/`. The rules are in the [documentation policy](../documentation-policy.md).

- `docs/design/` and `README.md` stay as upstream wrote them. Fork documents may cite them by path and section.
- The fork has its own layout (`research/`, `architecture/`, `decisions/`, `performance/`, `quality/`, `templates/`).
- arc42 (CC BY-SA 4.0) guides the architecture section order; its template text is not copied. MADR (`MIT OR CC0-1.0`) is adapted for ADRs.

### Consequences

- Good, because merges from upstream never touch fork documents.
- Bad, because readers who want upstream detail must read Japanese. Audits summarise the parts they rely on in English, with section references.

For `docs/design/` at the hard fork, see [ADR-0007](0007-upstream-design-documents-across-the-fork-lifecycle.md).

### Confirmation

`rumdl` and `lychee` in the pre-commit hook check only fork-owned Markdown. Review rejects edits to `docs/design/` that do not come from upstream.
