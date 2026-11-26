---
status: accepted
date: 2026-11-26
decision-makers: AiraCometes (maintainer)
---

# Leave upstream design documents unchanged while soft-forked, take them over at the hard fork, and keep adopting upstream features after it

## Context and problem statement

A8 found that the upstream design documents are reliable for intent and structure but go stale: 23 statements disagree with the code, besides the 13 that A4b, A4c, and A5 found, and seven "not done" lines describe work that is done ([A8](../research/audit-design-docs.md#statements-that-disagree-with-the-code); Confirmed). [ADR-0004](0004-english-fork-documentation.md) keeps `docs/design/` unchanged, and [ADR-0002](0002-track-upstream-as-a-soft-fork.md) leaves the switch to a hard fork to a later ADR. Neither says what happens to `docs/design/` at the switch, or whether upstream is still followed after it.

## Decision drivers

- Low merge cost while soft-forked ([ADR-0002](0002-track-upstream-as-a-soft-fork.md))
- One canonical place per fact ([documentation policy](../documentation-policy.md))
- Reuse Before Rewrite ([principles](../principles.md)): upstream keeps adding capabilities the fork wants

## Considered options

- (a) Keep `docs/design/` as upstream's record; the fork's requirements live in OpenSpec and its notes in `docs/fork/`
- (b) Correct the stale statements in `docs/design/` in place
- (a) while soft-forked, then (b) at the hard fork

## Decision outcome

Chosen option: (a) while soft-forked, then (b) at the hard fork.

- While the fork tracks upstream, `docs/design/` stays as upstream wrote it, as [ADR-0004](0004-english-fork-documentation.md) says. Stale statements are recorded in the audits' "code against the design documents" tables, and a requirement that relies on a document cites the audit that checked it. The fork's requirements are OpenSpec specs ([ADR-0003](0003-openspec-as-the-single-specification-system.md)).
- At the hard fork, the fork takes `docs/design/` over: it may correct, translate, merge, or replace those documents, starting with the statements the audits list. The ADR that decides the hard fork also amends ADR-0004 for `docs/design/`.
- After the hard fork, the fork keeps following upstream. Upstream changes are reviewed, and features worth having are adopted, without duplicating what the fork already has: an upstream feature that overlaps fork work is merged into the fork's version, not added beside it.

### Consequences

- Good, because merges from upstream stay free of conflicts in `docs/design/` while the fork tracks it.
- Good, because the stale statements are already listed, so the correction work at the hard fork has a starting list.
- Good, because upstream remains a source of features after the hard fork.
- Bad, because until the hard fork, readers must check a document against the audits before trusting a number or a "not done" line.
- Bad, because after the hard fork, every upstream feature needs a review and a manual port instead of a merge.

### Confirmation

While soft-forked: review rejects edits to `docs/design/` that do not come from upstream (ADR-0004). After the hard fork: the hard-fork ADR names the process for reviewing upstream changes, such as a periodic comparison with `upstream/main` and an issue per feature considered, and how a ported feature is checked against existing fork code.

## Pros and cons of the options

### (a) Keep `docs/design/` unchanged

- Good, because it costs nothing at merge time.
- Bad, because the documents keep drifting from the code, and a hard fork would inherit drifted documents with no owner.

### (b) Correct `docs/design/` in place now

- Good, because readers get correct documents.
- Bad, because every correction conflicts with upstream's later edits to the same files, against ADR-0002.

### (a), then (b) at the hard fork

- Good, because each phase gets the option that fits it.
- Bad, because the switch adds one documentation task to the hard fork.

## More information

The hard-fork ADR follows [ADR-0002](0002-track-upstream-as-a-soft-fork.md): at the latest when the master architecture needs changes that cannot coexist with upstream. The lists of stale statements are in [A4b](../research/audit-capabilities-rendering.md#code-against-the-design-documents), [A4c](../research/audit-capabilities-platform.md#code-against-the-design-documents), [A5](../research/audit-platform-matrix.md#documents-against-the-code-and-the-machine), and [A8](../research/audit-design-docs.md#statements-that-disagree-with-the-code).
