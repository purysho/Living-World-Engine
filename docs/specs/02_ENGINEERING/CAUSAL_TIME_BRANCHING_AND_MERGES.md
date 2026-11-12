# Causality, Temporal Mechanics, Branching and Historical Reconciliation

> **Document ID:** LWE-CAUSAL-001 | **Approved basis:** decisions 01–02, 22, 25, 27, 52, 54–55, 57–59.

## Fundamental distinction

Three graphs coexist and are not substitutes: **(1) branch ancestry** defines shared history and forks, **(2) event causal dependency graph** describes which events can influence others, and **(3) temporal constraint structure** defines local clocks, conversions and exceptional loops. A business event at fictional tick 10 may be committed after an event at fictional tick 12 because the former was imported by approved historical branching. Its validity is decided by causality, branch scope and invariant contracts—not database insertion timestamp alone.

## Branch semantics

- A branch has an immutable fork reference to `(parent_branch_id,parent_head_revision,ancestor_state_hash)`.
- Each branch stores only its local delta where possible; reads compose ancestral events up to its frozen fork revision.
- A merge proposal references source branch heads and candidate target parent; stale proposal invalidated by a source-head change or policy update.
- Accepting a merge creates a **new branch or explicit approved event series**; neither original branch is rewritten.
- “Canonical” is a pointer/reference controlled by world governance. It must not imply deleting alternative histories.
- Protected events are constraints, not magical guarantees. If no compatible history meets them, the merge fails with a proof/diagnostic of unsatisfiable constraints where feasible.
- A merging algorithm cannot be complete for arbitrary semantic human history; conservative false-positive conflicts are preferable to silent canon damage.

## Event ordering and vectorial causality

Classify event pairs: `happens_before`, `after`, `concurrent/unordered`, `inconsistent_constraints`, and `unknown`. Within a reality obey its local clock policies. Between realities maintain explicit transfer/event links; never infer a universal clock from comparable numeral values. Use deterministic tie-break ordering only to make state transitions reproducible for otherwise independent commutative operations. For operations that do not commute, execute conflict analysis before choosing an order.

**Time travel policy:** local timelines and branch identity are first-class. Backward travel may generate a new branch or satisfy a closed loop according to reality-specific law. For loops, identify strongly connected components and solve consistency as a constraint satisfaction problem; **do not** simply insert cyclic edges into an algorithm expecting a DAG. If no consistent solution exists, reject the proposed traversal. This subsystem is P3; tick-only v0.2 is not evidence of implementation.

## Causal dependency types

- **Direct read/write:** resource balance, asset owner, position, health, consent, law version, protected fact.
- **Structured domain:** `causes`, `enables`, `prevents`, `transmits_information`, `produces_resource`, `changes_institution`, `creates_evidence`.
- **Probabilistic influence:** rate parameter affecting subsequent stochastic process; retain model version and sensitivity assumptions.
- **Epistemic:** observation, testimony, translation and belief update; don't classify beliefs as objective facts.
- **Speculative consequence:** narrative/AI suggested influence *not* accepted as known causal link until validated or approved.

A causal edge has source ID, target ID, type, domain, confidence/justification, branch scope, validation status and model version. Runtime impact analysis first traverses explicit dependencies, then applies domain-wide invalidation rules (e.g., climate affects agriculture over a region) and conservative “unknown impact” markers. It should never promise the entire knock-on-effects closure has been proved.

## Reconciliation algorithm (target)

1. Freeze source heads and the common ancestor revision; collect approved permissions and protected constraints.
2. Verify each source branch independently, including snapshots, event signatures/checkpoints and domain contracts.
3. Partition deltas by aggregate/write-set and domain dependencies; compute candidate causal ordering.
4. Detect direct conflicts: missing asset, double-spend, exclusive owner, overlapping physical occupation, timeline inconsistency, duplicate unique identity, invalid rules version, secrecy/consent conflicts.
5. Run relevant domain validators for indirect effects; expand the read/write set if ambiguous.
6. For compatible commuting events, build a candidate replay branch and compute projections. For conflicts, produce specific resolution proposals with all assumptions declared.
7. Dry-run and verify proposed target; require policy-specific approvals bound to a digest of source heads and event set.
8. Commit new branch + provenance event, emit safe notifications, preserve originals and diagnostic record.

## Detailed example

At tick 10, Branch B destroys warehouse W. At tick 12, Branch A's merchant purchases W. `asset.destroyed` invalidates W's usable state and `asset.sold` requires `active=true`. The prospective merged branch has no legal state at tick 12 under standard rules. The engine blocks. Possible GM actions include creating a separately justified reconstruction before the sale, moving a valid original sale to before destruction with consent, or keeping branches separate. It must **not** invent an impostor warehouse in the data to satisfy the player's planned story.

## Selective replay and snapshots

Snapshot contains projected canonical state, event-tail hash, law/module versions, seed checkpoint and relevant parent hash. Reconstruct from the nearest trusted snapshot earlier than affected dependencies, invalidate downstream projections and re-run reducers. Conservatively expand to full branch/domain replay when dependency coverage is incomplete. Compare hashes and invariant reports before approval. Adaptive snapshot interval chosen from write frequency, replay cost, storage budget and recovery objective; this is not yet present in v0.2.

## Time and causality invariants to test

- No branch reads parent events after fork revision unless explicit merge.
- An event cannot cause an effect in a logically inaccessible temporal region without permitted travel/communication.
- Two exclusive owner transitions at the same valid state cannot both succeed.
- Replay of pinned event stream + versions + random inputs yields identical canonical projections.
- No cross-branch knowledge leakage without an actual world event transferring information.
- No loop acceptance without a satisfiable time/identity/energy constraint set.
- Repair/correction never mutates previous canonical events in place.
