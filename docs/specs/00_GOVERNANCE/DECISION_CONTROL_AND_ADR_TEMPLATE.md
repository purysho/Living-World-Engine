# Decision Control, Requirements Traceability and ADR Procedure

> **Document ID:** LWE-ADR-001 | **Authority:** historical user approvals in `DECISIONS_01_60_FROZEN.md` and post-baseline Decision 61. This document governs *new* technical proposals; it does not rewrite earlier choices.

## Source-of-truth hierarchy

1. Explicit owner decisions 01–61, including later supersessions (e.g., Decision 07 ultimately D), as recorded in frozen source.
2. Later owner-approved amendments with date and impact analysis.
3. Versioned machine-readable law contracts, domain schema and accepted ADRs implementing those decisions.
4. Approved development issues and task-specific acceptance tests.
5. Candidate technical architecture in this dossier, which remains provisional until adopted.
6. AI-generated suggestions, research notes and examples, which are non-authoritative.

## ADR template

```markdown
# ADR-XXXX — Decision title
Status: Proposed / Accepted / Superseded / Rejected
Date: YYYY-MM-DD
Owner/approver:
Applicable decisions: [nn,nn]
Context/problem:
Constraints and invariants:
Options considered:
Decision and rationale:
Security, privacy and rights impact:
Performance, operating-cost and maintainability impact:
Data/schema and replay compatibility:
Proof/benchmark needed:
Rollout, feature flag, migration and rollback:
Unresolved assumptions:
Supersedes / superseded by:
```

## Requirements trace record

`decision_id` → `requirement_id` → `architecture doc/contract` → `implementation path/commit` → `test IDs` → `benchmark/run evidence` → `release version` → `known limitations`. No implemented claim can rely solely on the existence of an ADR or copied prompt. If test evidence is missing, state `specified`, not `verified`.

## Change control

Proposed alteration to world metaphysics, ownership, player autonomy, canon consistency or licensing is a product change and requires owner approval. Adding an index to improve query time is an engineering implementation change, provided no externally visible semantics change. Database/event schema changes need migration plan, fixture replay checks, invariant proof/test, and rollback or read-only backward compatibility. Avoid informal “approved” in AI output without a recorded authorised actor approval event.
