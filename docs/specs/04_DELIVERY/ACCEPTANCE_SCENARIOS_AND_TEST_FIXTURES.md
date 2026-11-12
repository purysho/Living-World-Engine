# Detailed Acceptance Scenarios and Conformance Fixtures

> **Document ID:** LWE-ACCEPT-001 | **Status:** proposed executable specs and existing proven example distinguished per scenario.

## Contract scenario matrix

| Test ID | Setup | Action | Required assertions | Current |
|---|---|---|---|---|
| AT-001 | asset W owner council, merchant funded | `asset.sold` | atomically owner changes and funds conserved | v0.2 covered |
| AT-002 | insufficient buyer funds | attempt sale | rejects, head/state/hash unchanged | v0.2 covered |
| AT-003 | same idempotency key | repeat equivalent transfer | same recorded event, no extra debit | v0.2 covered |
| AT-004 | branch A sale at tick12; B destroy tick10 | propose merge | conflict, source events identified; originals intact | v0.2 covered sibling |
| AT-005 | two independent deltas | owner approves merge | new branch valid; source branches intact | v0.2 covered sibling |
| AT-006 | tampered snapshot/projection | replay verification | mismatch detected | v0.2 covered |
| AT-007 | non-sibling branches with common ancestor | proposed merge | correct LCA and overlap semantics | future |
| AT-008 | protected historical event | incompatible edit | proposal blocked; protected conflict shown | future |
| AT-009 | concurrent cross-region caravan | retry after interrupted transfer | no duplicate cargo, explicit in-transit state | future |
| AT-010 | climate/harvest fixture, known seed | run 10 simulated years | invariant-preserving outputs, identical hash on replay | future |
| AT-011 | two NPCs different witness access | request conversation | no unseen canonical information disclosed | future |
| AT-012 | NPC record promoted T1→T3 | run discussion | same identity/family/memory facts; higher detail allowed | future |
| AT-013 | stored witness lies intentionally | player investigates | lie remains belief/evidence, not true event | future |
| AT-014 | 2014 vs 2024 rules profiles | resolve fixture encounter | correct edition outcome, pinned rules hash | future |
| AT-015 | two camera views same location | switch perspectives | entity IDs/positions/interactables equivalent | future |
| AT-016 | network drops after proposal | reconnect | command is pending until server commit | future |
| AT-017 | malicious book source contains tool instruction | run Writers' Brains retrieval | cannot execute instruction or leak data | future |
| AT-018 | owner has low content rating, player stricter | generate scene | output respects effective bounds; canon stable | future |
| AT-019 | numeric magic law zero resource | propose energy creation | impossible unless explicit exception contract | future |
| AT-020 | temporal loop with unsatisfiable constraints | propose travel | solver rejects, preserving original branch | future |

## Fixture requirements

Every automated scenario publishes seeded `initial_state`, versioned command sequence, expected accepted/rejected events, after-state hashes, known permissible randomness, knowledge access matrix and reproducible run command. AI scenarios also include locked prompt template, model/version ID, expected policy outcomes and sampled evaluation with uncertainties. Geometry fixtures need canonical entity positions and collision shapes; not screenshots alone.

## Quality gates before publishing “implemented”

- Unit/integration tests pass on supported Windows environment, not just Linux CI.
- Running the shipped installer and basic walkthrough produces documented expected state.
- All update/restore/export flows tested; no “success” message before durable commit.
- Failure injection demonstrates no partial transactions.
- At least one conformance fixture proves the feature is not merely cosmetic.
- Access control verified for authorised and unauthorised roles.
- Memory/latency/performance measured on declared hardware and compared to threshold.
- Known unmodelled conditions disclosed and attached to release notes.
