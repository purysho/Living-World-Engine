# Project Charter and Product Strategy

> **Document ID:** LWE-CHARTER-001 | **Version:** 1.0 | **Classification:** project working specification | **State:** proposed implementation of approved direction | **Baseline:** LWE-FOUNDATION-1.0 (decisions 01–60); Decision 61 authorises implementation | **Date:** 2026-11-12

## Executive intent

**Living World Engine (LWE)** is a local-first, Windows-oriented persistent fictional-universe simulation and tabletop roleplaying platform. It is intended to support original high-fidelity multiversal settings, multiple concurrent D&D-compatible campaigns, independent NPC societies, scientific and magical worldbuilding, GM-controlled canonical history, and optional playable 3D environments. Its differentiator is not an AI narrator. It is an *authoritative, inspectable, independently evolving world state* that both humans and AI can act upon through constrained interfaces.

**Product thesis:** A fictional world becomes meaningfully reusable when its history and causality remain consistent across long periods, independent actors, multiple campaigns and contradictory proposals. Narrative quality, simulation fidelity, accessibility and economics must be measured separately. Fictional realism means internal coherence and a defensible approximation of chosen physical/social processes, not the literal reproduction of entire real universes.

**Primary audiences:** (1) World Owner or simulation designer; (2) human GM; (3) player in human- or AI-GM campaign; (4) collaborative regional GM; (5) mod, simulation and asset creator; (6) researcher/tester assessing emergence. **Initial audience:** one expert hobbyist creator operating locally on Windows. Public commercial product fit is not established.

**Jobs to be done:** Design and revise worlds without losing established history; operate solo/multiplayer campaigns across controlled branches; talk to and observe autonomous NPCs; inspect economic, ecological, geopolitical, magical and familial consequences; test whether outcomes were causal rather than invented; optionally experience places through text, map and walkable 3D; publish original content under controlled rights.

## North-star product principles

1. **Simulation is authoritative:** model/GM narrative proposals never directly mutate state.
2. **Canon is branch-qualified:** every fact has scope, provenance, temporal validity and authorised origin.
3. **Epistemic separation:** objective truth, witnessed evidence, character beliefs and player-visible narration are not interchangeable.
4. **Transparency without ruining discovery:** GMs may inspect full provenance; players receive their permitted view.
5. **Causality before spectacle:** no image, dialogue or quest overwrites established physics, ownership or chronology.
6. **Reproducible where defined:** committed transitions, inputs, versions and random streams are replayable; LLM outputs are not presumed deterministic.
7. **Complexity earns its cost:** model scale, realism modules and high-cost reasoning must outperform simpler baselines on measured criteria.
8. **Human authority and consent:** protected player decisions, campaign content boundaries and GM authorisation are enforced by code.
9. **Local ownership and interoperability:** portable world exports, provider-independent AI interfaces, secure optional hosting.
10. **Creative freedom with provenance:** emergent, guided and authored content remain distinguishable and editable through approved branches.

## Product scope versus delivery scope

| Layer | Target product | Current verified status (v0.2) | Near-term release gate |
|---|---|---|---|
| Canon/history | Persistent causal world with multiverse branches | Python/SQLite event core and narrow sibling merges | Rust parity, typed events, causal graph |
| AI intelligence | Tiered NPC cognition and provider routing | Design templates only | sandboxed proposal API and access-filtered retrieval |
| Simulations | Socioeconomics, biology, ecology, magic, geopolitics | Not implemented | deterministic town grain/inventory/household vertical slice |
| Play | D&D 2014/2024, human/AI GM, solo/multiplayer | Not implemented | rules-core conformance and session authority |
| Interactive 3D | Isometric/first/third-person walkable places | Not implemented | single test interior and navigation integration |
| Creative tools | Dialogue, maps, timeline, programming workshop | Command-line prototype only | native desktop timeline/branch explorer |
| Marketplace | Mods/assets, licensing and commercial distribution | Not implemented | import/export + signed manifests before any store |
| Evaluation | multidimensional blinded benchmark | 23 Python tests, limited demo | formal conformance, load and threat test suites |

## Scope hierarchy

- **Foundation / non-negotiable:** correctness of canonical transitions, branch isolation, approvals, snapshots, replay, deterministic seeded simulation, data privacy, recoverability.
- **Vertical slice / demonstrable:** one river trading town, 100–300 instantiated residents, households and inventories, three organisations, two campaigns, an emergent investigation, simple walkable areas, playable D&D episode.
- **Expansion / aspirational:** planetary evolution, multiversal cosmology, interstellar civilisations, hundreds of thousands of NPCs, autonomous media generation, global creator marketplace. These are architectural destinations, not near-term production promises.
- **Explicit non-goals for v0.3:** large-scale virtual world, entire 5e content catalogue, realtime multiplayer hosting, unrestricted third-party mods, mathematically complete human psychology, automatic geological billion-year fidelity.

## Governance and decision rights

World Owner owns the canonical acceptance policy and baseline. Engineering changes use an Architecture Decision Record (ADR) with context, alternatives, trade-offs, migration and rollback. Product scope changes cite a frozen decision or a new explicit amendment. Technical recommendations in this dossier **do not** count as user-approved decisions. Conflict-of-requirement cases are flagged and not guessed away. Each release publishes features, known limitations, measured tests, security assessment and open risks.

## Commercial/product hypotheses, not assertions

Potential products include a premium world studio, multiplayer hosting, licensed creator extensions and paid cloud processing. None are validated. No revenue projections, market share or sales-ready claims are accepted without research. Any hosted service requires incident response, privacy/legal review, billing controls, support plans and sustainable inference-cost analysis.

## Definition of a successful first release

A user can create a world, create two campaign forks, move time independently, perform at least one ownership transaction and one destructive event, inspect a conflict explanation, approve a compatible merge, reconstruct state from event history and export/restore all data. Windows GUI and multiplayer are later gates; the current CLI proof is not a public release.
