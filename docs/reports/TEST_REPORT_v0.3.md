# Test report — v0.3.0

**Executed:** 2026-11-12 · **Environment:** Linux x86-64 container, 4 vCPU Intel Xeon 2.1 GHz, rustc 1.97.0, Node 22 · **Windows:** not run by hand; CI runs the core tests and builds the installer on `windows-latest` (see the *build* workflow for the result on the current commit).

## Automated tests — 29 / 29 passed

```bash
cargo test -p lwe-core
```

| Suite | Tests | Covers |
|---|---:|---|
| Unit (`src/`) | 9 | seeded RNG reproducibility; ability/proficiency maths; 2014 vs 2024 exhaustion and potion rules; dice audit replay; PC creation; tier→model routing; tolerant JSON extraction; registry and lens routing |
| Acceptance (`tests/acceptance.rs`) | 20 | below |

| Spec ID | Test | Result |
|---|---|---|
| — | genesis: ~250 residents, 3 organisations, 3 generations, tiers, conserved ledger | pass |
| — | same seed → identical state hash; different seed → different | pass |
| AT-001/002/003 | atomic sale; rejection leaves head and hash unchanged and logs a diagnostic; idempotent retry; key reuse conflict; stale head | pass |
| — | fork isolation incl. nested forks; full-lineage verify | pass |
| AT-004 | bought in A on day 12, destroyed in B on day 10 → merge blocked, destroying event named, sources untouched | pass |
| AT-005/007 | compatible non-sibling merge → new branch; owner-only approval; stale proposal; canon pointer | pass |
| AT-006 | UPDATE/DELETE on events refused; edited payload and tampered snapshot detected | pass |
| AT-008 | protected asset cannot be destroyed; protected fact cannot be overwritten | pass |
| — | role policy matrix (player, observer, GM, unknown role); forged chronicle event refused | pass |
| AT-010 | drought vs calm on the same seed: lower growth index, higher grain price, hunger and crime; conservation; verify; rebuild in a fresh database gives the identical hash | pass |
| — | two calm years: population holds, memory and history bounded, state < 4 MB | pass |
| AT-011/013 | claim absent from a non-witness's context; no GM truth in any context; forged disclosure rejected; player views carry no culprit or GM events; mistaken sighting stays false | pass |
| AT-012 | conversation commits memory and regard; absent NPC unreachable; T1→T3 keeps identity, family, memories | pass |
| AT-014 | profiles pinned per campaign; edition cannot change after play begins; same seed, same roll | pass |
| — | combat to completion with dice log; non-lethal survivor levels up and becomes enemy/nemesis; no off-screen harm to the PC over 60 days | pass |
| AT-019 | cost law 4·level²; non-casters refused; no casting from an empty field; no overdraft | pass |
| AT-018 | veil vs allowed: different presentation, identical canon | pass |
| — | a killing in play becomes a murder case and merges into canon as a death | pass |
| — | council and agent profiles offline; no key → visible `ai_no_key`; key never written to the database | pass |
| — | export covers the full lineage; every incident traces to its tick | pass |

Not covered by any test: AT-009 (cross-region transfer), AT-015 (cameras), AT-016 (network drop), AT-017 (malicious retrieved document — retrieval is not implemented), AT-020 (time loops).

## End-to-end UI run

A scripted Chromium session against `lwe-server` + the built UI: generate world → advance a year → chronicle and causal view → inspect a person → factions → cases → start a campaign → add a character → walk to the market → two conversations → skill check → brawl to completion → map → weather shock → verify → merge analysis → council → AI settings → bible → switch to player view → library verify. **0 page errors, 0 console errors, 0 error banners.** The Tauri release binary was also launched under a virtual display on Linux and rendered the library with engine version read over IPC. This script lives outside the repository and is not yet part of CI.

## Measurements (release build, this container)

| Operation | Time |
|---|---:|
| Advance 360 days (360 ticks + ~70 incidents, one transaction) | 1.84 s |
| `overview` | 9 ms |
| `people.list` (263 people) | 47 ms |
| `events.list` | 15 ms |
| Full replay verification of 1 year | 2.28 s |

State is about 1.2 MB of JSON after two years; a year adds roughly 3 MB to the database (WAL, before checkpoint). Desktop binary 18 MB; server binary 8.7 MB. These are single runs on shared hardware, not benchmarks.

## Observed model behaviour (seed 42)

- Calm years: grain 60–260 % of normal with a pre-harvest squeeze in year two; hunger near zero; a handful of crimes a year.
- Severity-85 drought for 330 days: growth index collapses, grain hits the price ceiling (12×), peak average hunger 61/100, 460 crimes, population 266 → 117 in 15 months.

## Not verified

Windows install, upgrade and uninstall by a person · any call to the real Anthropic API · accessibility (keyboard order, screen reader, contrast) · unclean-shutdown recovery · long soak (10+ years) · behaviour with several worlds of several years each in one database · SRD rule text conformance.
