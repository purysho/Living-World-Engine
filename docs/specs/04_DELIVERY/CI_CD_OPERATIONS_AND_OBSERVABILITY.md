# Engineering Operations, CI/CD, Releases and Incident Response

> **Document ID:** LWE-OPS-001 | **Status:** operational plan for future production releases.

## Source-control and review workflow

Main branch protected; conventional commits and linked issues; two-person code review for high-risk contracts/auth where staffing allows; author of change cannot self-approve high-risk policy change in hosted deployment. Required check list: Rust compile/lints/unit/fuzz smoke; TS typecheck/tests; schema migration; reference parity; dependency/security scan; generated SBOM; licence manifest; docs/ADR; end-to-end fixture. For a solo creator, require an independent automated adversarial review and manual written signoff, not false claims of two-person review.

## Build reproducibility

Pin compiler/toolchains, lock dependency versions, record OS/container image, code commit, schema revision, signed package hash and build inputs. Release artifacts: Windows installer, portable backup/export tools, migration utility and verification CLI. CI creates archived test reports and reproducibility manifest. A “green” CI badge requires the actual checks to run and finish successfully.

## Release versioning

Semantic product version plus schema and contract versions; an installer version does not imply event schema backward compatibility. For database migration: verify backup → check free disk → run schema migration within explicit plan → perform reference replay/consistency test → record migration event → resume. Support read-only upgrade preview and documented inability to downgrade when irreversible. Security-critical fixes may require compatibility migration.

## Observability and privacy

Structured local logs (level, service, world/branch hashed IDs, command ID, event type, policy decision, latency, error category), trace/metric correlation using OpenTelemetry-compatible schemas if deployed. Avoid logging prompt content, GM secrets, player consent reasons, API keys or unpublished source excerpts. Telemetry is disabled or opt-in by default for local-first product; hosted operations collect necessary abuse/security records under a clear policy.

**Core alerts:** event replay mismatch; invariant violation; invalid approval; repeated branch merge failures; backlog explosion; scheduler drift; outbox retries; local DB integrity; backup restore failure; AI spend threshold; p95 latency; asset import failures; secrets leak detection.

## Incident lifecycle

Detect → contain affected domain (do not indiscriminately shut entire world when safe isolation exists) → snapshot diagnostics → verify last good checkpoint → root-cause hypothesis → reproduce in isolated branch → create repair/migration proposal → execute verification suite → owner approval if high-risk → commit recovery → user communication and postmortem. Restoration cannot overwrite legitimate user content without prior backup and explicit informed permission.

## Disaster recovery

Define RPO/RTO only after deployment topology chosen; for local single-user worlds support automatic local rotating backups plus user-controlled export. Verify restores routinely. Hosted option needs region backup strategy, encrypted backups, tested credential recovery and chain-of-custody for protected world assets. Backups should include event data, manifests, rule/schema versions, content-addressed assets, relevant provenance and encryption metadata, not plaintext provider secrets.

## Capacity and cost planning

Track events/s, snapshot storage growth, 3D asset size, resident count, cognitive-tier distribution, model token prices, session concurrency and remote egress. Use hard AI budget thresholds and kill switches. Forecast linear/nonlinear scaling from measurements rather than a made-up cloud price estimate. For server simulation, run integration soak test before claiming persistent 24/7 availability.
