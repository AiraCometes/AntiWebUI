---
status: accepted
date: 2026-11-24
decision-makers: AiraCometes (maintainer)
---

# Use OpenSpec as the single specification system

## Context and problem statement

Requirements and change plans need one home. The bootstrap brief names OpenSpec as the core and forbids running GitHub Spec Kit alongside it. It also names extension candidates: `intent-driven` (ADRs), `epic-driven` (epics and phases), and `spec-gen` (specs generated from code). Phase 0 itself has to be done somehow before any spec exists.

## Decision drivers

- One source of truth for behaviour
- Minimal Abstraction: no schema customisation before it is needed
- Bounded Complexity in the process itself

## Considered options

- OpenSpec with the built-in `spec-driven` schema, extensions deferred
- OpenSpec with the `intent-driven` and `epic-driven` schemas from the start
- GitHub Spec Kit

## Decision outcome

Chosen option: OpenSpec with the built-in `spec-driven` schema.

- `openspec/` is initialised in Phase 0 with no specs and no changes.
- Phase 0 is done on a branch and reviewed as a pull request, not as an OpenSpec change. Its outputs are the process documents themselves.
- The first OpenSpec changes are the Phase 5 baseline specifications. Later fork work follows the cycle in the [roadmap](../roadmap.md#phase-6-incremental-implementation).
- Change names follow `<issue number>-<kebab-case summary>` when the change comes from a GitHub issue.
- Extensions are deferred:
  - `epic-driven` is reconsidered when Phase 5 builds implementation epics.
  - `intent-driven` is covered by [ADR-0001](0001-record-decisions-as-madr.md).
  - `spec-gen` was renamed to OpenLore (`clay-good/OpenLore`, MIT). It is considered for the Phase 1 capability inventory in a spike, if reading the code by hand proves too slow.

### Consequences

- Good, because there is one spec format and one CLI.
- Good, because the decision about extensions is made with real changes in hand.
- Bad, because epics and phases are tracked in the roadmap document until an epic schema is chosen.

### Confirmation

`mise run spec:validate` (`openspec validate --all --strict`) runs in the pre-commit hook for `openspec/**`.

## More information

The OpenSpec CLI is pinned at 2.0.0 to match the maintainer's user-level schema additions. 2.1.1 exists (2026-11-19); upgrading is a separate change.
