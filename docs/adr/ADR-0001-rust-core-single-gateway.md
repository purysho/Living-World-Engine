# ADR-0001 — Rust core behind one JSON gateway; Tauri 2 shell; SQLite
Status: Accepted provisionally under Decision 61 (owner may revise)
Date: 2026-11-12
Applicable decisions: 13, 15, 51, 52, 53
Context/problem: The specification recommends Rust + Tauri 2 + SQLite (ADR candidates 001–004) but leaves them unadopted. The v0.2 Python reference was not included in the package handed over, so extending it was not possible.
Options considered: (a) extend Python v0.2; (b) Rust core + Tauri; (c) Electron/TypeScript end to end.
Decision and rationale: (b). All authority lives in `lwe-core`. It exposes exactly one entry point, `App::dispatch(json) -> json`. The Tauri shell registers a single IPC command that forwards to it; `lwe-server` forwards HTTP to it. Because the two transports share every line of logic, the UI can be driven in a browser by automated tests and the result holds for the desktop app. JSON envelopes (not a binary encoding) keep events inspectable.
Security, privacy and rights impact: The webview has `core:default` capability only — no filesystem, shell or HTTP plugins. Role filtering is done in `dispatch`, never in the UI. The HTTP host binds to loopback, rejects cross-origin posts, and is unauthenticated: it is a development/headless tool, not the multiplayer server of Decision 15.
Performance: measured in docs/reports/TEST_REPORT_v0.3.md.
Data/schema and replay compatibility: n/a (first schema).
Proof/benchmark needed: Windows install smoke test by a person; cold-start and memory on a reference laptop.
Rollback: the gateway contract is transport-neutral; replacing Tauri would not touch the core.
Unresolved assumptions: PostgreSQL for hosted mode, renderer choice and schema encoding remain open (spec ADR-002/003/004).
