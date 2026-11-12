# ADR-0002 — The simulation step is a deterministic reducer; incidents are derived chronicle events
Status: Accepted provisionally under Decision 61
Date: 2026-11-12
Applicable decisions: 52, 56, 57, 59
Context/problem: A town of 250 produces thousands of micro-transactions a day. Storing each as an event would be faithful but heavy; storing only "a day passed" would make replay depend on code that might change.
Decision and rationale: One `sim.tick` event advances one day. Its reducer is a pure integer function of prior state and seeded streams keyed by `(world seed, process, day, entity)` — no floating point, no wall clock, no model calls. Every event row records the **state hash after it**; `world.verify` replays from genesis and compares each one, so any drift in the rules is detected rather than silently accepted. Notable outcomes of a tick (a theft, a death, an arrest, a feud) are written as `chronicle.*` rows caused by that tick: they are the readable history and the causal graph, and they are no-ops on replay.
Consequences: Weather is keyed by day only, so sibling branches share a sky and differ only by what people did — which makes "same world, one change" comparisons meaningful. Changing simulation rules is a breaking change: bump `ENGINE_VERSION` and keep old reducers if old worlds must still verify. That versioned-reducer mechanism is **not built yet**; today a rules change invalidates verification of existing worlds.
Invariants enforced at commit: commodity conservation residual = 0 for every commodity; no negative account except the external world; magical potential conserved.
Proof: `at010_…replays_bit_for_bit`, `same_seed_same_world…`, `two_calm_years…`.
