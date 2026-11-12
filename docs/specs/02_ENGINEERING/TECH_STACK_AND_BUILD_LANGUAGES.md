# Technology Stack, Programming Languages and Engineering Toolchain

> **Document ID:** LWE-STACK-001 | **Status:** candidate stack with formal evaluation gates; preserves agreed Windows local-first direction.

## Selection matrix

| Layer | Preferred candidate | Alternatives | Decision rule |
|---|---|---|---|
| Core language | **Rust stable** | C++, C# | safety, low-level control, deterministic domain code, portable libraries |
| Desktop shell | **Tauri 2** | Qt, Avalonia, Electron | Windows packaging, secure IPC, resource use, accessibility |
| Front-end | **TypeScript strict + React** | Svelte, Vue | team familiarity, component tooling, test ecosystem |
| Local database | **SQLite WAL + migrations** | embedded RocksDB, LMDB | transactional integrity, portability, backup/export, simplicity |
| Hosted storage | **PostgreSQL candidate** | distributed SQL | host writes, operations, migrations and transactional workloads |
| Graphics | **Godot 4 / Bevy proof-of-concept** | Unity, custom native renderer | walkability, 3 camera modes, asset pipeline, deployment cost |
| Simulation | **Rust deterministic modules** | Python model prototypes | replay, performance, formal constraints |
| Research/experiments | **Python 3.11+** | Rust notebooks | datasets, experiment harness, independent oracle |
| Agent definition | **Markdown/YAML + structured JSON schemas** | DSL | human-readable, testable, versioned agents |
| Observability | **structured logs, OpenTelemetry compatible** | bespoke metrics | local opt-in telemetry, reproducible diagnostics |
| API | **versioned commands and events over IPC; HTTPS/WebSocket when hosted** | gRPC | typed and inspectable contracts; no raw SQL APIs |
| Docs | **Markdown + Mermaid + ADRs** | static docs site | source-controlled, diffable, portable |

## Fable 5.2 provider integration

Anthropic's current model is **Claude Fable 5.2**, API ID `claude-fable-5-2` (official docs as checked 2026-11-12). Published specifications are 1M token context, 128K maximum output, $8/M input and $40/M output tokens, subject to provider terms and later changes. Those figures are **vendor statements, not project benchmarks or cost commitments**. Model availability, retention, usage limits, training and commercial terms must be checked at integration time. Do not assume it can keep a world “running” after a client closes. The simulation scheduler must run independently of the model.

**Routing rule example:** Tier 0 population aggregates: no LLM. Tier 1: deterministic finite-state/habit model. Tier 2: lightweight model on exceptional decisions. Tier 3: capable general model with persistent structured memory. Tier 4: Fable 5.2 or other high-tier model for exceptional strategic planning. An agent may escalate if its task is hard but its importance tier is low; the tier is a *budget/priority selector*, not an intelligence score.

## Build and test standards

- Rust: pinned compiler, formatting (`cargo fmt`), lints (`cargo clippy`), unit/property/contract tests, fuzzing high-risk parsers and conflict evaluator.
- Frontend: pinned Node LTS and package lock, lint/typecheck/unit tests, Playwright accessibility/smoke tests, no direct credentials in bundle.
- SQLite: tested schema migrations, explicit transactions, WAL checkpoints, reproducible backup from a consistent snapshot, avoid DB on network shares.
- Python reference: install via `pyproject.toml`, run `python -m unittest discover -s tests -v`, compare JSON fixture outputs to Rust reducers.
- Security: software-bill-of-materials, dependency vulnerability alerts, package-signature validation, no dynamic `eval` for world scripts.
- Release: signed builds/installer when distribution begins; versioned migration tests, rollback plan, reproducible manifest and checksums.

## Suggested monorepo (target)

```text
living-world-engine/
  apps/desktop/                # Tauri/TS desktop shell
  apps/renderer/               # chosen 3D engine client, separate process if needed
  apps/server/                 # optional authoritative host
  crates/world-core/           # immutable domain types, command coordinator
  crates/event-store/          # SQLite adapter, replay, projections
  crates/causality/            # event DAG, temporal constraints, merge
  crates/scheduler/            # discrete events + numeric steppers
  crates/agents-policy/        # capabilities, cognitive routing, tool contracts
  crates/economy/ crates/ecology/ crates/magic/ crates/rules-dnd/
  packages/schemas/           # versioned command/event definitions
  agents/                      # generic and specialised Markdown agent specs
  data/                        # licensed source metadata and fixtures only
  tools/reference-python/      # v0.2 reference and property oracle
  tests/fixtures/ tests/e2e/ tests/load/ tests/security/
  docs/adr/ docs/specs/ docs/reports/
```

## Evaluation before dependencies become irreversible

Three build spikes: (1) Windows Tauri 2 shell with Rust command validation and SQLite backup; (2) 3D prototype of shared NPC positions and all three cameras; (3) 250-resident simulation throughput and deterministic replay. Benchmark cold start, install size, p95 responsiveness, RAM/GPU consumption, crash recovery and developer productivity. Record results as ADR evidence before committing to the graphics/process topology.
