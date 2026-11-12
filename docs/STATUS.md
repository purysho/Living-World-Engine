# Implementation status — v0.3.0

Feature states follow the specification's ladder: `specified → proof_of_concept → implemented → verified`.
**Verified** means there is an automated test in `crates/lwe-core/tests/acceptance.rs` (or a unit test)
that exercises it. Nothing here has been tested on Windows hardware by a person yet — see the test report.

## Against the roadmap gates

| Gate | Spec outcome | State | Notes |
|---|---|---|---|
| M1 typed event contract | Rust core, event envelope, validators, causal references | **verified** | `lwe.event/1` envelope with hash chain, `event_causes` table, role policy, idempotency, optimistic concurrency. *The v0.2 Python reference was not supplied with the documentation package, so Python↔Rust parity fixtures could not be run; the v0.2 contract's scenarios were re-implemented as tests instead.* |
| M2 branches and time | common-ancestor resolver, protected events, conflict report, approvals | **verified** (subset) | Non-sibling merges via lowest common ancestor; protected assets and facts; owner-only approval bound to a digest of both heads; stale detection. No conflict *graph* visualisation, no multi-approver councils. |
| M3 socioeconomic town | 250 residents, households, grain ledger, shock, 3 orgs | **verified** | Conservation and replay tested over multi-year runs; drought → price/hunger/crime chain tested. Parameters are tuned by hand, not calibrated. |
| M4 Windows studio | Tauri 2 UI, branch lab, inspection, installer | **implemented** | UI exercised end-to-end in Chromium and the Tauri binary launched on Linux/WebKitGTK. Windows installer is produced by CI. **Not yet verified:** a human install/upgrade/restore pass on Windows; accessibility audit. |
| M5 playable slice | one rules profile, sessions, mystery, dialogue | **implemented** | Both rules profiles pinned per campaign; combat, checks, evidence, accusation, knowledge firewall tested. Only a small SRD subset (level 1, four classes, no spell list, no conditions beyond dodge/surprise/exhaustion). |
| M6 model-mediated NPCs | tier routing, memory, provider adapter, AI-GM | **proof_of_concept** | Gateway, tier routing, budget cap, structured replies, firewall and offline fallback are implemented and unit-tested. **No live model call has been made** in development (no API key in the build environment): request shape follows the published Messages API but is untested against the real service. |
| M7 walkable 3D | 3 cameras, navigation | **specified** | Not started. The town map is a 2D node graph. |
| M8 release hardening | signing, backups UI, plugin API, security review | **specified** | Export exists; import/restore UI, installer signing, encrypted exports, hosted multiplayer and mods do not. |

## Against the 60 decisions (grouped)

- **Implemented and tested:** 01 shared universe with branching · 02 per-campaign clocks · 11 auditable GM
  intervention · 12 dual 5e profiles · 22 historical editing (preview, protected, compare by branch) ·
  24 epistemic separation · 26 no off-screen harm to PCs · 28 role-based permissions (owner/GM/player/observer) ·
  29 layered content (campaign level, three categories) · 34/42 river-town slice · 43 persistent identities and
  genealogy · 51–53 event-sourced core, atomic transactions · 55 causal references · 57 seeded independent
  streams · 59 rejection without partial state.
- **Implemented in simplified form:** 03/09/18 hard magic (one conserved potential ledger, two effects) ·
  07/17/44 tiered NPC cognition (traits, bounded salience-ranked memory, beliefs, deterministic policies; LLM
  optional) · 10 socioeconomics (six goods, one market) · 16 configurable AI (one provider) · 23 information
  ecosystem (claims, rumour with distortion, forgetting) · 32/33/50 feuds, schemes, investigation ·
  36 creative environment (GM tools + council) · 40/41 Writers' Brains (procedure + unsourced registry) ·
  45/46 biology/ecology (hunger, health, weather-driven harvest only) · 48 relationships, courtship, marriage.
- **Lore only, not simulated:** 05/08 multiverse and other realities · 19 divinity · 20 deep-time emergence ·
  31 technological evolution · 47 metaphysics · 54 time travel.
- **Not started:** 13/14 3D exploration · 15 hosted server · 27 sync windows · 30 art/music/voice generation ·
  37 hardware tiers · 38/39 mods, marketplace, rights manifests · 49 voice/gesture · 58 formal law DSL ·
  60 blinded benchmark.

## Known limitations worth knowing before you play

- One town. The hinterland and "abroad" are accounts, not places you can visit.
- A severity-85 drought across a whole growing season roughly halves the town within fifteen months in
  testing. That is the model working as written, and it is harsh; use an experiment branch.
- Merges carry explicit interventions and play-caused deaths only. Conversations, theft and reputation from a
  campaign do not flow into canon.
- Offline NPC dialogue is keyword-driven and plain. It never leaks, but it is not witty.
- The loopback HTTP host has no authentication. Do not expose it to a network.
- Event hashes detect tampering; they are not signatures.
