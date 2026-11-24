# Audit: subject

<!-- Save as docs/fork/research/audit-<subject>.md. One subsystem or concern per file.
     Label every claim (Confirmed / Inferred / Proposed / Unverified), see docs/fork/documentation-policy.md. -->

- Date: YYYY-MM-DD
- Upstream commit: full SHA audited
- Auditor: agent and model, or person
- Plan item: link to the row in docs/fork/research/phase-1-audit-plan.md

## Scope

Crates, modules, and documents covered, and what is deliberately left out.

## Method

Commands, tools, and reading order used, so the audit can be repeated.

## Findings

### Capability or concern

- Status: present | partial | absent
- Evidence: paths, symbols, tests, upstream design doc sections
- Behaviour: what it does today, with limits
- Quality notes: tests, unsafe, allocation, complexity, platform coverage
- Label: Confirmed | Inferred | Unverified

## Baseline violations

Rule breaks that exist upstream today (for example lints, formatting, dependency rules). Record them as the baseline; fork changes must not make them worse.

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|

## Open questions

Each with what would answer it.

## Inputs for gap analysis

Pointers into docs/fork/research/gap-analysis.md that this audit supports.
