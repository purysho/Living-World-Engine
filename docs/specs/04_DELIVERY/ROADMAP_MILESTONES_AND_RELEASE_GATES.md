# Delivery Roadmap, Engineering Milestones and Release Gates

> **Document ID:** LWE-PLAN-001 | **Status:** dependency-led plan, not a promise of dates, staffing or costs. Frozen Decision 61 prioritises an executable core over further broad interviews.

## Programme strategy

Build a **walking skeleton**: a narrow functioning path across storage, commands, timeline, simulation and play before generalising. Maintain the Python v0.2 engine as a reference while production Rust reducer/storage is constructed. Do not release a “multiverse simulator” based on a compelling but unverified procedural lore demo. Each milestone requires runnable commands, test evidence, release notes and explicit gap statement.

## Milestones

| Gate | Outcome | Minimum deliverables | Exit criteria |
|---|---|---|---|
| **M0 / existing v0.2** | Proof of event core | Python SQLite CLI, sibling branches, limited transactions, 23 tests | documented as prototype only |
| **M1 / v0.3** | Typed canonical event contract | Rust core scaffold, event schemas, validators, causal references, Python parity fixtures | ≥1:1 replay equality on agreed fixture set; malformed events rejected |
| **M2 / v0.4** | General branch/time architecture | common-ancestor resolver, protected events, conflict graph, owner approvals, versioned time | deterministic non-sibling merge cases + rejected paradox/inconsistency tests |
| **M3 / v0.5** | First socioeconomic town | 250 persistent resident identities, households, grain ledger, agricultural shock, 3 orgs, seeded schedule | conservation, replay, plausible price/household response, performance data |
| **M4 / v0.6** | Windows world studio | Tauri 2 UI, branch lab, owner console, maps and event inspection, native installer | Windows installation/upgrade/restore/accessibility smoke tests |
| **M5 / v0.7** | Playable RPG vertical slice | one playable D&D 5e rules profile, 2 campaign sessions, evidence mystery, dialogue | end-to-end playtest; no knowledge leak; combat/time consistency |
| **M6 / v0.8** | Model-mediated NPC autonomy | tier routing, structured memory, local/provider adapters, AI-GM trial | cost/latency/consistency vs baseline and adversarial agent tests |
| **M7 / v0.9** | Walkable 3D + richer systems | one town route/interior, 3 cameras, navigation/collision, character view and media rights | frame/perf/geometry/identity parity and content tests |
| **M8 / 1.0 candidate** | Reliable integrated release | backups, hosted beta optional, bounded plugin API, installer signing, security review | gate review on reliability, player value, rights and published limitations |
| **Future** | multiverse-scale/creator marketplace | planetary, interdimensional, time travel, generative assets, community distribution | separate proofs, legal and commercial validation |

These version numbers are roadmap labels, not declarations that a fully featured 1.0 is straightforward. Rebaseline gates if measurements invalidate architecture assumptions.

## Phase staffing disciplines (roles, not current employees)

Required competencies: simulation architect, Rust/backend systems engineer, gameplay/rules designer, 3D technical artist/renderer engineer, AI/RAG engineer, technical writer, QA automation/performance, UX/accessibility, security/privacy, IP/legal consultant. A solo implementation must sequence these responsibilities rather than assume parallel staffing. Specialist reviews can be external when budget permits.

## Delivery slices

**Slice 1:** create world → issue grain → sell warehouse → fork → destructive event → merge conflict → replay verification. **Slice 2:** real harvest → inventory/market price → household food shortage → risk model → public record/evidence. **Slice 3:** NPC decision based on scoped beliefs → valid purchase or conversation → player investigation. **Slice 4:** map/3D representation of same entities and events. Each slice must use actual typed commands, not a mocked facade that claims simulation.

## Effort/cost controls

No fixed completion date or cost because staffing, target hardware, AI tokens, 3D asset scope and required realism are unresolved. Track per-feature story/engineering points, GPU hours, model tokens, licences and hosting bills; forecast only after measured throughput and prototype integration. Maintain a “scope creep budget”: if an unplanned feature does not improve P0 correctness, vertical-slice user value or benchmarks, move it to the backlog.

## Release board template

For each milestone: `requirements met`, `PRs/commits`, `documentation`, `build artifact`, `test report`, `benchmark hardware/versions`, `security scan`, `rights audit`, `known defects`, `rollback plan`, `owner signoff`, `decision impact`. Gate cannot pass by assertion alone.
