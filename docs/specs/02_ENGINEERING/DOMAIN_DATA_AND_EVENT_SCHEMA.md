# Domain Data Model, Event Store and Interface Contracts

> **Document ID:** LWE-DATA-001 | **Status:** prospective production contracts; v0.2 implementation supports a subset detailed in its own `CORE_CONTRACT_V0_2.md`.

## Aggregate and identity model

| Aggregate / entity | Stable identity and key fields | Invariants |
|---|---|---|
| World | `world_id`, owner, cosmology_version, canonical_head | unique owner-controlled canon per selected branch |
| Reality | `reality_id`, `world_id`, law_contract_id, coordinate scheme | laws pinned and valid for contained events |
| Region | `region_id`, parent reality, geometry/LOD, resources | valid nested geometry and references |
| Branch | `branch_id`, world, parent, fork_revision, head_revision, policy | ancestor immutability; valid causal parent |
| Campaign | `campaign_id`, `branch_id`, party, rules_profile, consent | head/GM policy matched for writes |
| Entity | immutable `entity_id`, type, birth/creation event, canonical scope | no identity recycling; uniqueness |
| Person | entity_id, household, biology, cognition references | subject to genealogy/knowledge access |
| Organisation | entity_id, charter, membership, roles, resources | change via approved events only |
| Asset | entity_id, owner, spatial location, condition | at most one exclusive owner at a state |
| Account | owner, commodity, units, balance | ledger invariant; explicit mint/burn |
| Knowledge claim | claim_id, subject, proposition, observer, evidence, confidence | not automatically canonical fact |
| Evidence | evidence_id, generating event, location, custody chain | immutable origin; mutable custody history |
| Contract | contract_id, version, units, checks, domain | transition must match active version |
| Agent profile | agent_id, capability scope, model policy, budget | never direct storage mutation |

## Three time concepts

`occurred_at`: fictional time coordinate in local reality (may be branch-specific and non-comparable globally). `observed_at`: time in observer's frame when an entity learned about event. `committed_at`: monotonic storage ordering timestamp/revision. For time travel also `traveller_proper_time` and `arrival_event_id`. Never rely on local wall clock to order reality events. Use explicit `happens_before` edges plus branch-local monotonic sequences. Future closed-loop temporal models need special strongly-connected constraint components rather than ordinary DAG-only replay.

## Event envelope (proposed)

```json
{
  "schema": "lwe.event/1",
  "event_id": "evt_01...",
  "world_id": "world_river",
  "branch_id": "campaign_a",
  "aggregate_id": "asset_warehouse_01",
  "aggregate_revision": 4,
  "event_type": "asset.transferred",
  "event_version": 1,
  "occurred_at": {"reality_id": "primary", "calendar_id": "river_era", "tick": 12},
  "committed_at": "2026-11-12T00:00:00Z",
  "causes": ["evt_prior_sale_agreement"],
  "actor": {"entity_id": "npc_mira", "principal_id": "gm_01", "role": "campaign_gm"},
  "origin": {"kind": "human_approved", "proposal_id": "proposal_77"},
  "rule_versions": {"world": "1.0.0", "asset": "1.0.0"},
  "idempotency_key": "purchase-001",
  "payload": {"asset_id": "asset_warehouse_01", "from_owner": "council", "to_owner": "npc_mira"},
  "privacy": {"classification": "gm_private", "campaign_allowlist": ["campaign_a"]},
  "integrity": {"previous_hash": "sha256:...", "record_hash": "sha256:..."}
}
```

This is an **illustrative forward-looking schema**, not accepted input to v0.2. `record_hash` must be computed on canonical serialisation omitting integrity's own hash. Hash chaining provides integrity/tamper detection only under trusted boundaries; signed checkpoints and protected backups are needed against malicious storage administrators.

## Command envelope and decision pipeline

```json
{
 "schema":"lwe.command/1", "command_id":"cmd_...", "world_id":"world_river",
 "branch_id":"campaign_a", "expected_head_revision":24,
 "actor":{"principal_id":"gm_01","capability":"asset.sell"},
 "intent":"asset.sell", "parameters":{"asset_id":"warehouse","price_milli":5000,"currency":"coin"},
 "read_set":{"asset:warehouse":4,"account:npc_mira:coin":8},
 "approval_id":"approval_123", "dry_run":false
}
```

Execution: authenticate principal (hosted) → enforce scope and consent → validate schema and contracts → check branch/head/read-set revisions → derive proposed events → invoke domain reducer on candidate state → verify invariants and causal constraints → atomic append+projection+outbox → ack event IDs. Rejected commands emit **diagnostic** records, not fabricated canonical events. Approval is scoped to proposal digest, relevant branch heads, expiration and principal; mutate any input and approval becomes stale.

## Minimal relational schema (design sketch)

```sql
CREATE TABLE worlds (world_id TEXT PRIMARY KEY, owner_id TEXT NOT NULL, settings_json TEXT NOT NULL);
CREATE TABLE branches (branch_id TEXT PRIMARY KEY, world_id TEXT NOT NULL REFERENCES worlds(world_id), parent_id TEXT REFERENCES branches(branch_id), fork_seq INTEGER NOT NULL, head_seq INTEGER NOT NULL DEFAULT 0);
CREATE TABLE events (event_id TEXT PRIMARY KEY, branch_id TEXT NOT NULL REFERENCES branches(branch_id), seq INTEGER NOT NULL, schema_version INTEGER NOT NULL, event_type TEXT NOT NULL, occurred_json TEXT NOT NULL, committed_at TEXT NOT NULL, actor_json TEXT NOT NULL, payload_json TEXT NOT NULL, record_hash TEXT NOT NULL, UNIQUE(branch_id,seq));
CREATE TABLE event_causes (event_id TEXT NOT NULL REFERENCES events(event_id), cause_event_id TEXT NOT NULL REFERENCES events(event_id), relation TEXT NOT NULL, PRIMARY KEY(event_id,cause_event_id,relation));
CREATE TABLE idempotency (principal_id TEXT NOT NULL, command_id TEXT NOT NULL, canonical_request_hash TEXT NOT NULL, response_json TEXT NOT NULL, PRIMARY KEY(principal_id,command_id));
CREATE TABLE snapshots (branch_id TEXT NOT NULL REFERENCES branches(branch_id), through_seq INTEGER NOT NULL, state_hash TEXT NOT NULL, blob_ref TEXT NOT NULL, engine_version TEXT NOT NULL, PRIMARY KEY(branch_id,through_seq));
CREATE INDEX idx_events_branch_seq ON events(branch_id,seq);
CREATE INDEX idx_causes_source ON event_causes(cause_event_id);
```

Real migrations must add world-consistency checks, projections, protected-event policy, auth, outbox, checkpoints and immutability controls. This sketch is **not a drop-in production migration**.

## Domain vocabulary and contracts

A **fact** is a world claim accepted as canon on a branch. A **belief** is an individual's (possibly wrong) proposition. An **observation** is a causally delivered perception. An **inference** derives a belief from evidence. A **proposal** is uncommitted candidate content/action. An **event** is an accepted state transition. A **projection** is a query-friendly derivation of events. An **asset** is a persistent uniquely identified item or property; fungible resources use ledgers. A **snapshot** is a checkpoint derived from validated replay. A **world law** is an explicit machine-readable contract scoped to a reality and version.

## Migration, indexing and retention

Migrations specify from/to schema, replay compatibility, irreversible content transforms and backup verification. Never mutate historical event meanings silently; upcasters convert old schemas at read/replay time or a signed migration creates explicit replacement provenance. Index hot branch/head queries, geographic/entity time, evidence custody and known causal edges. Store large asset blobs content-addressed outside DB with manifest references and quotas. Retention policies must not remove events necessary for verified reconstruction or legal/audit obligations.
