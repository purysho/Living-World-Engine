# Product Requirements Document (PRD)

> **Document ID:** LWE-PRD-001 | **Status:** proposed detailed requirements mapping frozen decisions | **Priority convention:** P0 correctness/security; P1 first vertical slice; P2 subsequent core capability; P3 long-horizon expansion.

## Users, contexts and core jobs

| Persona | Main workflows | Data visibility / authority | Primary failure to avoid |
|---|---|---|---|
| World Owner | create laws, seed worlds, view canon, manage GM approvals | full owner-controlled access | silent canon corruption |
| Human GM | prepare session, run encounter, inspect secrets, reconcile campaigns | scoped GM rights | leaked secrets or conflicting outcomes |
| Player | create character, explore, speak, fight, investigate | character-/campaign-permitted | loss of player agency or spoilers |
| AI-GM service | propose narration, rules applications, NPC actions | task-scoped filtered context | hallucinated facts and unauthorised writes |
| Regional co-GM | run territory/faction, propose shared-world changes | delegated territory/role | privilege escalation |
| Mod/content creator | define modular content, test, publish | explicit capability grants | malicious plugin, licence violation |
| QA/researcher | run seeded scenarios, compare model policies | isolated fixtures and redacted outputs | unverifiable benchmarks |

## Functional requirements (traceable identifiers)

### World studio and state (LWE-WLD)

- **WLD-001 P0:** Create, inspect, back up and restore locally owned world packages with immutable world IDs and versioned schema.
- **WLD-002 P0:** Create branch from any permitted revision; source branch stays unchanged when target branch advances.
- **WLD-003 P0:** Record append-only committed events with stable ID, schema version, actor, source, branch, causality, data classification and hashes.
- **WLD-004 P0:** Propose canonical merge; show overlap/conflict graph; authorisation gate; non-destructive acceptance; explicit refusal/correction on conflicts.
- **WLD-005 P0:** Maintain materialised state and snapshots, verify with replay; automatic safe recovery and tamper evidence.
- **WLD-006 P1:** Edit entities, assets, relationships, geography and historical timelines with dry-run impact previews.
- **WLD-007 P1:** Support three authorship modes: emergence, guided constraints and direct intervention; keep provenance distinct.
- **WLD-008 P2:** Progressively generate settlements, continents, planets and alternate realities with locked established facts.

### Simulated societies and minds (LWE-SIM)

- **SIM-001 P1:** Inventory/resource accounting, costs, production/consumption, wages and deterministic transactions for initial town.
- **SIM-002 P1:** Persist ~250 sample residents, households, genealogy, affiliations and basic routines using tiered processing.
- **SIM-003 P1:** Simulate periodic population, production and ecological change with reproducible seed streams, versioned rates and explicit assumptions.
- **SIM-004 P2:** NPC belief/knowledge separation, reconstructive memory, social development, conversation and strategic deliberation.
- **SIM-005 P2:** Dynamic factions and organisations, migration, political competition, market shocks and independent events.
- **SIM-006 P3:** Interplanetary evolution, cross-reality economies, sophisticated physiology, full strategic warfare and technological evolution.
- **SIM-007 P0:** No module bypasses invariant enforcement or canonical permission checks, regardless of simulation tier.

### Roleplay, rules and presentation (LWE-PLAY)

- **PLAY-001 P1:** Human GM and AI-GM session records; player knowledge boundaries; GM-only truth view.
- **PLAY-002 P1:** Rules profile per campaign for 2014 SRD 5.1 and 2024 SRD 5.2.1; homebrew versioned separately.
- **PLAY-003 P1:** Turn-based initiative integrated with world time; deterministic RNG, explicit dice audit and encounter resolution.
- **PLAY-004 P1:** Persistent evidence and at least one unscripted investigable event in the river town.
- **PLAY-005 P2:** Walkable location with isometric, first- and third-person modes over shared geometry/collision/navigation.
- **PLAY-006 P2:** Text + voice NPC conversation with player-visible knowledge filter and content boundaries.
- **PLAY-007 P2:** Absent-player autonomy profile; only delegated actions allowed; significant decisions queued.
- **PLAY-008 P3:** Interstellar traversal, time travel, cosmic horror, romance and mature themes according to per-campaign rules.

### Creation, AI and community (LWE-CRE)

- **CRE-001 P1:** Structured AI proposal API, constrained tools, validation, explicit approve/apply stage.
- **CRE-002 P2:** Model routing policies across Fable 5.2, other hosted providers and optional local models; budgets and fallback.
- **CRE-003 P2:** Evidence-grounded Writers' Brains retrieval, source records, critical evaluations and GM approval.
- **CRE-004 P2:** Asset provenance and versioning; consistent NPC portraits, environmental representations and audio.
- **CRE-005 P3:** Generative 3D/audio pipeline, plugin SDK, community publishing and commercial rights support.

### Governance, security and operations (LWE-OPS)

- **OPS-001 P0:** Code-enforced RBAC/ABAC, world/branch scope, protected event approvals, tamper-evident audit.
- **OPS-002 P0:** Secure offline save and restore; backup manifest + integrity checks + migration and downgrade policy.
- **OPS-003 P0:** No secrets/API keys in world snapshots, prompts, logs or content exports.
- **OPS-004 P1:** Player, campaign and universe content boundaries; collaborative sessions enforce highest applicable restriction.
- **OPS-005 P2:** Optional authoritative server for remote multiplayer, with authenticated API and controlled sync.
- **OPS-006 P2:** Creator licensing, asset provenance, import scanning and untrusted plugin isolation.

## Nonfunctional requirements (proposed acceptance targets; not measured)

| NFR | Proposed initial target / test | Status |
|---|---|---|
| Data correctness | Zero accepted double-spend, negative-resource, stale-revision or unauthorised events in fault tests | proposed gate |
| Replay | 100% state hash match on reference fixtures after rebuild with pinned engine/version/seed | prototype limited pass |
| Durability | Unclean shutdown recovery test shows no partially committed transaction | not yet demonstrated |
| Local command responsiveness | p95 ≤200 ms for non-AI single-town state query on reference PC | target, hardware TBD |
| Merge explanations | 100% tested conflicts show affected entity, events and violated rule | target |
| Offline core | create/fork/play deterministic core without network; online AI fails visibly, not silently | future target |
| Accessibility | keyboard-first, focus order, scalable text, captions; WCAG 2.2 AA guidance for applicable UI | target |
| Security | pre-release review against relevant OWASP ASVS 5.0 controls + desktop/IPC threat model | target |
| Backup | round-trip import/export fixture equivalence, checksums and migration safety | target |
| Scalability | tiered profiles with 250 resident starting town; no claim of universal scale until benchmarked | target |
| Localisation | UI language separable from diegetic/in-world languages; Unicode and RTL readiness | target |
| Cost | per-provider token/cost accounting and hard optional user budgets | target |

## Prioritisation and anti-scope rules

P0 correctness blocks release. P1 creates an integrated playable slice. P2 and P3 do not block the P1 milestone unless needed for a truthful claim. Any feature must include evidence of completion: implementation, tests, UI flow, known limitations, security assessment and a requirements trace. “AI generated plausible text” is not accepted as evidence that a physical/economic system is simulated.

## Release acceptance scenarios

1. **Double claim:** two branches spend the same resource; compatible alone, conflict on merge, no loss of either branch.
2. **Knowledge firewall:** two NPCs with different knowledge receive filtered prompts; neither discloses unauthorized event details in tests.
3. **Timestamp semantics:** event creation/commit time differs from fictional occurrence and observation; queries return the right historical view.
4. **Disaster chain:** crop failure changes aggregate supply then prices, households, and crime risk according to fixed quantified parameters; changing one seed preserves reproducible replays.
5. **Content boundary:** high-intensity narration is downgraded in lower-rated campaign, without changing underlying canonical events.
6. **Offline recovery:** lose network during AI proposal; authoritative transaction is neither partially applied nor silently fabricated.
