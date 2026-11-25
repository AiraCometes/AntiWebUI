---
status: accepted
date: 2026-11-24
decision-makers: AiraCometes (maintainer)
---

# Record architecture decisions as MADR files in docs/fork/decisions

## Context and problem statement

The project runs across many AI sessions and agents, so a decision that lives only in a conversation is lost. Decisions need one durable place that any agent can find from `AGENTS.md`. The bootstrap brief names MADR for decision records, and `intent-driven` as a candidate extension.

## Decision drivers

- Documentation as Code; repository as the source of truth
- Minimal Abstraction: no extra tooling to read or write a decision
- A format both humans and agents already know

## Considered options

- MADR 4.0.0 files in `docs/fork/decisions/`
- The `intent-driven` OpenSpec schema, which writes ADRs from each change into a top-level `adr/` folder
- Decisions kept inside OpenSpec `design.md` only

## Decision outcome

Chosen option: MADR 4.0.0 files in `docs/fork/decisions/`, numbered `NNNN-<title>.md`, with an index in `docs/fork/decisions/README.md`. Template: [adr](../templates/adr.md).

The `intent-driven` schema stays a candidate. Its ADR folder (`adr/`) does not match this layout, and its compatibility with OpenSpec 2.x is Unverified ([tool adoption matrix](../quality/tool-adoption-matrix.md#specification-and-documentation-workflow)).

### Consequences

- Good, because a decision is one file with a stable number and link.
- Good, because MADR is `MIT OR CC0-1.0`, so adapting it adds no licence obligation.
- Bad, because OpenSpec changes and ADRs must be linked by hand.

### Confirmation

Reviewers check that a structural change links an ADR. A link check (`mise run docs:links`) keeps references valid. An automated "ADR required" check is a Phase 5 candidate.

## More information

Revisit when the first OpenSpec changes run (Phase 5–6). If ADRs drift from changes, evaluate `intent-driven` in a spike.
