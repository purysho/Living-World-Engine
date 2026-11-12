# Local-First Storage, Hosted Multiplayer and Synchronisation

> **Document ID:** LWE-NET-001 | **Basis:** 01–02, 15, 26–28, 37, 53–55 | **Status:** target design; no internet-ready hosting implemented in v0.2.

## Authority model

**Offline-owned world:** local authoritative DB and single writer. **Hosted world:** remote server becomes canonical authority for permitted campaigns; local clients hold encrypted/redacted caches and tentative offline proposals, not competing authoritative copies of the same head. Migration from local to hosted requires full backup, server import, identity remap checks, owner approval and read-only local snapshot. A server remains optional; local gameplay still works without AI when core simulation/rules exist.

## Sync protocol

Handshake: authenticate user → negotiate world/branch/contract versions → verify permissions and content boundaries → exchange last acknowledged revision/checkpoint hash → stream role-allowed events/projections. Commands carry idempotency key, expected revision, actor capability and optional offline causal dependencies. On conflict the server responds with structured failure and candidate reconciliation, not last-write-wins for canonical inventory/rights/history.

**Connection states:** online verified; offline local-only; disconnected with pending commands; reconciling; read-only due to version mismatch; server authoritative. UI must not show pending commands as committed. Time-dependent approval deadlines must state server-reality time semantics to avoid accidental expiry during suspension.

## Cross-region messaging

Each region can own a set of aggregates, but resources moving between them require a transfer protocol with one owner of record, transfer IDs, stage changes and retry-safe reconciliation. Prefer atomic transaction when on same DB; otherwise durable saga/outbox with compensation and explicit in-transit states. Prevent “money teleports” through duplicated messages. Delayed physical communications become world events and are not equated to API transport latency.

## Session governance

Campaign GM controls active session authority within delegated scope. Real-time client presence does not grant narrative permissions. A player's character can act only when present or when absentee policy authorises it. Multiple GMs can edit their own branches, with protected event and cross-region merges routed to owner's policy. Private world/campaign data never broadcast to all clients merely because it is stored near public projection tables.

## Network security and resilience

TLS, authenticated sessions, server-side authorisation on every action, session expiry, rate limiting, abuse controls, careful WebSocket permission scoping, audit logs, versioned APIs and capacity quotas. Reconnection must be idempotent. DDoS and commercial hosting protections are later operations requirements. Never expose a local SQLite DB file over a network share as a multiplayer solution; SQLite WAL is intended for same-host access.

## Test scenarios

Disconnected two users both buy last item; one accepts, other receives conflict after reconnect. Offline branch action starts before a later approved canonical event; merger identifies temporal overlap. Two-region caravan transfer retries after network failure without duplicate assets. A revoked GM role fails every protected command and loses access to future private event streams. Slow client cannot stall server's authoritative world tick.
