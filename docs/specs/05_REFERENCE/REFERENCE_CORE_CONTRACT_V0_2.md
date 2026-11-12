# Executable milestone 0.2: supported invariants and boundaries

## Execution model

The repository contains a **dependency-free Python/SQLite reference core**. It is a tested feasibility slice for the eventual Rust-backed Tauri Windows application, not a replacement for the agreed production target. No Fable or other model access is needed for the initial test. All AI/NPC proposals remain outside the authoritative commit path.

### State and event model

- A world has a root branch and zero or more child branches.
- An immutable event row has an ID, branch-local sequence, fictional tick, type, typed JSON payload, actor, provenance metadata, previous hash and event hash.
- A branch inherits all parent events up to a recorded fork-local sequence. Its own events are numbered from 1.
- Materialised state includes entity identity, assets/ownership, stock balances, recorded facts and a clock.
- Event hashes use canonical JSON SHA-256. They are **integrity checks, not cryptographic signatures or protection against administrators who can rewrite the entire database**.
- SQLite transactions use `BEGIN IMMEDIATE`, WAL, foreign keys and idempotency keys. Immutable event-table triggers reject ordinary updates or deletes.
- Branch local clocks advance monotonically via events. A tick is an abstract unit, not an implemented calendar or relativistic time.
- Snapshots are recorded every five local events and at branch creation. The verification procedure reconstructs all state from events and compares stored snapshots and materialised projections.

### Supported actions

| Event | Guard |
|---|---|
| `entity.created` | Unique identity, restricted entity kind, existing asset owner |
| `resource.issued` | Positive amount, explicit reason, existing recipient |
| `resource.transferred` | Atomic debit/credit; source balance sufficient; commodity conserved |
| `asset.sold` | Active asset, verified owner, sufficient buyer funds, atomic ownership/currency transfer |
| `asset.destroyed` | Existing active asset; no repeated destruction |
| `fact.recorded` | Unique fact ID |
| `clock.advanced` | Forward-only timeline |

`quantity_milli` means **one thousandth of the declared commodity unit**, not a currency conversion or market valuation. Resources can enter a world only through explicit issuance events. This prototype does not calculate production, prices, inflation, biology, or currency exchange rates.

### Conflicting campaign histories

A merge proposal can currently compare **two sibling branches forked at the same parent revision**. Candidate events are sorted deterministically by fictional tick, clock-change priority, branch ID and branch-local event sequence; transitions are replayed against the common ancestral state. The first incompatible event is rejected, and relevant previously touched keys and events are reported. This is a **narrow causal conflict detector**, not a general semantic dependency graph.

A compatible proposal may be GM-approved to create **a new branch** replaying the combined events, with links to the original event IDs. Original branches are unchanged. Proposals become stale if either source branch changes. Blocking conflicting proposals is intentional; the GM must resolve the scenario in a new explicit branch rather than receive a fabricated fix.

### Authority boundaries

- Proof-of-concept method checks prohibit low-privilege roles from minting resources or creating identities.
- Merge approval requires `world_owner` in the Python API.
- **No authentication, secure RPC, collaborative RBAC, multiplayer server or permission-filtered NPC access exists in v0.2.** Do not expose the database or CLI over an untrusted network.

### Deterministic replay guarantee (limited)

Replay of *committed records* is deterministic for this fixed implementation. Future changes to transition code will need explicit versioning, migrations and reference vectors before replay compatibility can be claimed. Repeated LLM calls are not part of replay.

### Key limitations

- Sibling-only merges; no complete arbitrary DAG merge or temporal loop simulation.
- Tick-only local chronology; no relativistic clocks or cross-reality causal calculus.
- No general semantic causal graph, event repair or selective replay optimiser.
- No physically simulated world, deep NPC cognition, D&D rules, 3D renderer or Windows installer yet.
- All economic and social events in this prototype are explicit API/CLI inputs.
- One local SQLite writer at a time; no geographically distributed transaction coordinator.
- Snapshots are periodic, not yet adaptive.
- Module contracts are encoded as Python validators; no formal proof of their correctness has been performed.

## Follow-on milestones

1. **0.3: event schema and causal graph.** Typed versioned events, entity-relationship projections, explicit causal dependency references, unit/property tests and a security review.
2. **0.4: general branch merging and temporal graph.** Common-ancestor discovery, protected events, approval workflows, merge planners and conflict graph visualisation.
3. **0.5: town simulation modules.** Inventory, real production/consumption, agents with deterministic routines, socioeconomic metrics and reproducible stochastic streams.
4. **0.6: GUI prototype.** Tauri 2 + Rust core or interoperable command layer, world timelines, branch diff and approvals. Test Windows installers.
5. **0.7+: agents and gameplay.** Model-provider routing, NPC cognitive tiers, filtered context, dice/rules integration, and eventually walkable spaces.

Advance versions based on passing concrete tests rather than arbitrary schedules.
