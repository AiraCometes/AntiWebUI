# Gap analysis

<!-- Phase 2 output. Save as docs/fork/research/gap-analysis.md. One row per requirement candidate.
     Every row needs evidence from an audit; a row without one stays at "Unverified". -->

- Date: YYYY-MM-DD
- Upstream commit: full SHA
- Audits used: links

## Classification

| Class | Meaning |
|---|---|
| REUSE | Keep the existing implementation as it is |
| EXTEND | Add to the existing implementation |
| OPTIMIZE | Keep the behaviour, improve cost |
| NEW | Nothing exists; build it |
| REPLACE | Replace the existing implementation; needs an ADR with evidence |
| DEFER | Valid, but later than the current roadmap horizon |
| DROP | Not needed, or duplicates something else |

## Requirement candidates

| ID | Candidate | Area | Upstream status | Evidence | Class | Rationale | Upstreamable | Depends on | Phase |
|---|---|---|---|---|---|---|---|---|---|
| G-001 | | | present / partial / absent | path, symbol, doc | | | yes / no / maybe | | |

- Area: rendering, spatial and input, lifecycle and resources, motion, headless UI and accessibility, typography and effects, testing and performance, extensibility.
- Upstreamable: whether the change could be offered to `bea4dev/AntiWebUI` as a pull request (sending one needs maintainer approval).

## REPLACE and DROP decisions

Each REPLACE or DROP links an ADR.

## Summary

Counts per class, and the candidates that block the master architecture.
