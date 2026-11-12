# Risk Register, Assumptions, Dependencies and Open Decisions

> **Document ID:** LWE-RISK-001 | **Status:** programme risk view. Probability/impact rankings are qualitative planning estimates, not measured likelihoods.

| Risk ID | Risk | Probability | Impact | Mitigation / evidence gate | Owner role |
|---|---|---|---|---|---|
| R-01 | World ambitions exceed feasible compute | High | Critical | strict LOD, budgets, single-town vertical slice, benchmark per tier | simulation lead |
| R-02 | Fake emergence from narrative LLM | High | Critical | typed events, deterministic authority, blind model-only comparison | AI + QA |
| R-03 | Causal dependency omissions corrupt selective replay | High | Critical | conservative invalidation, full replay fallback, causal audit suites | core engineer |
| R-04 | Temporal loops make event graph unsound | High | High | separate solver semantics, defer exotic mechanics | temporal architect |
| R-05 | Cross-branch exploit double-spends resource | Med | Critical | atomic validation and property-based merge fuzzing | storage lead |
| R-06 | AI prompt injection leaks secrets or executes tools | High | Critical | capability isolation, filtered retrieval, adversarial tests | security lead |
| R-07 | Content/legal licensing violations | Med | Critical | source provenance, counsel review, import policy | legal/rights |
| R-08 | Renderer architecture too heavy for laptop | High | High | Godot/Bevy spikes and non-3D fallback | graphics lead |
| R-09 | Full memory/psychology feels repetitive | High | High | blind testing, constrained causal memory, simpler baselines | NPC lead |
| R-10 | Foreign-source/author corpus falsely attributes principles | High | Med | primary-sourced claims, strict quotes, no invented endorsements | research lead |
| R-11 | Cost runaway for autonomous LLM calls | High | High | per-task budgets, no-LMM default, rate limits | AI lead |
| R-12 | Historical data migration loses world continuity | Med | Critical | versioned reducers, snapshots, test restores | persistence lead |
| R-13 | Multiplayer authority creates sync inconsistency | High | Critical | server-owned canon, tentative offline actions, conflict protocol | server lead |
| R-14 | Security of plugins/marketplace | High | Critical | capability manifest, sandbox, signatures, release gate | security lead |
| R-15 | Product inaccessible due to UI complexity | High | Med | progressive disclosure, user tests, WCAG target | UX lead |
| R-16 | Unrealistic science despite valid math | High | High | specialist review, empirical calibration, label assumptions | domain QA |
| R-17 | v0.2 prototype mistaken for production software | High | High | explicit status matrix, CI/Windows verification | product lead |
| R-18 | Feature prioritisation paralysis | High | High | Decision 61 freeze, milestones and blocker-only questions | product owner |

## Assumptions currently in force

- Initial operating platform is Windows desktop; optional service not required for basic local mode.
- Python v0.2 is an executable reference, not production core.
- Rust/Tauri/SQLite production direction is proposed and aligned to earlier planning, but final renderer/back-end choices require spikes.
- SRD 5.1/5.2.1 provide a rights-permitted subset of the target D&D mechanics; complete published books are not assumed redistributable.
- 250 town residents are a design target, not measured capacity.
- Fable 5.2 is available as an external model choice at time of research; future price and terms may change.
- The world is fictional and original; no borrowed D&D settings are automatically canonical.
- The multiverse is an expansion goal; it does not need to be instantiated in detail for the first release.

## Open *implementation* decisions (not new interviews unless blocking)

1. **Renderer:** Godot vs Bevy/native vs other after actual Windows prototype and workflow comparison.
2. **Server authority:** Rust service, storage database, identity management and sync format.
3. **Schema encoding:** JSON vs binary event transport; commit to one canonical checksum representation.
4. **Cryptographic model:** signed checkpoints and export encryption/key recovery policy.
5. **Exact law DSL:** typed invariant language and proof system boundary (SMT? refinement types? property-based first).
6. **Time-loop solver:** represent strongly connected constraint components after general causal DAG proves robust.
7. **Physical magic model:** resource units and conversion equations, conservation/exception rules and scope.
8. **NPC behaviour framework:** benchmark alternatives rather than claim validated full psychology in advance.
9. **D&D licensing:** exact public release attributions, trademarks, homebrew package separation.
10. **Commercial distribution:** software licence choice, creator terms, pricing/hosting costs and legal review.
11. **Hardware reference:** choose several benchmark tiers and collect actual CPU/RAM/GPU data.
12. **World data rights:** owner/export/share/deletion/backup policies for published shared worlds.

## Decision escalation protocol

If an open question blocks M1, propose a one-page ADR with alternatives, objective acceptance tests, cost/security impact, migration and fallback; engineering may choose a reversible implementation when consistent with the baseline. Ask the World Owner only when rights, gameplay authority or immutable baseline principles would change.
