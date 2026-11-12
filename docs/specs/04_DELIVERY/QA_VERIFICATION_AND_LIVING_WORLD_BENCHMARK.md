# Quality Assurance, Testing Strategy and Living World Benchmark

> **Document ID:** LWE-QA-001 | **Approved basis:** decisions 35, 57–60 | **Status:** v0.2 facts explicitly separated from future acceptance criteria.

## Proven evidence from v0.2

The existing Python/SQLite prototype reports **23/23 passing unittest tests** in a Linux environment (Python 3.13.5, 2026-11-12). Verified fixture categories: atomic sale, rollback on insufficient funds, immutable event triggers, idempotency, optimistic revision check, fork isolation, blocked warehouse conflict, successful sibling merge, snapshot/materialised corruption detection, basic role guards, 120 transfer conservation, replay. Test report and source logs are included in the original v0.2 archive. **Unproven:** Windows installer, production auth, distributed performance, true causal graph, temporal loops, NPCs/economy/ecology/magic, D&D and 3D. Do not conflate 23 unit tests with system readiness.

## Test pyramid

1. **Static/schema:** lint, typecheck, event schema validation, migration and licence checks.
2. **Unit:** domain reducers, conservation, idempotency, numerical integrators, probabilities, rule tables, authority policy.
3. **Property-based/fuzz:** arbitrary command streams, corrupt imports, branch DAG, time transitions, parser/IPC surfaces, mod manifests.
4. **Contract:** versioned API and law contracts, Python↔Rust parity, AI output JSON validation, D&D SRD compatibility.
5. **Integration:** DB snapshots/replay, branch merge, NPC knowledge projection, cross-domain economy/health/ecology coupling.
6. **Scenario/E2E:** warehouse contradiction, harvest-to-crime chain, two campaign sessions, blackout and reconnection, offline action conflicts.
7. **Stress/soak:** thousands of simulated years at aggregate resolution, 250 NPC town detailed loops, many concurrent proposals, memory/cost growth.
8. **Adversarial/privacy:** prompt injection, secrets leakage, malicious mods, role escalation, corrupted archives, unauthorised approval.
9. **Human/blind UX:** scripted NPC vs model-only vs hybrid world, perception of autonomy, narrative coherence, accessibility and engagement.

## Living World Benchmark dimensions

| Dimension | Quantitative signals | Human/qualitative assessment |
|---|---|---|
| Causal consistency | invalid commits; replay mismatch; missing dependencies | explanation comprehensibility |
| Economic/accounting | conservation error; shortages; volatility under shock | perceived credibility |
| Population/social | demographic balance; relationship continuity | psychological plausibility |
| Ecological | stocks/flows error; climate shock response | habitat realism |
| NPC cognition | goal coherence, privacy leaks, repeated choice diversity | blind believability ratings |
| Narrative | genuine event-based leads, consistency of evidence | meaningful adventure potential |
| Player agency | action acceptance, protected-choice violations | perceived control/fairness |
| Rule fidelity | SRD fixture pass rates; combat replay | tabletop usability |
| Performance/cost | tick/sec; p95 latency; RAM; GPU; $/sim year | responsiveness and friction |
| Security/rights | blocked misuse, unresolved vulnerabilities, bad licences | trust in governance |

## Quantitative proposed gates

**Correctness release blockers:** zero known critical invariant violations in test suite; 100% replay hash match for pinned fixture set; idempotency under retries; documented causal conflict for blocked merges; no unauthorised state-changing operations; valid backup restoration. **Performance initial hypotheses:** 250-person town progresses 1 fictional day without uncontrolled memory growth on a reference laptop; UI query p95 ≤200 ms for non-AI local read; quantify thresholds after hardware benchmark. **Narrative:** a minimum of 30 independent blinded sessions per comparator *as proposed study design*, effect size/confidence reporting, and qualified reviewers; pilot first to size sample. **Cost:** measure total provider tokens and per-tick spending; no unbounded background calls.

## Example formal/prop test properties

```text
For all transfer commands T accepted:
 sum(all account balances for commodity c) changes only by recorded issuance/destruction.
For all branch forks B from (A,r):
 replay(B,0) == replay(A,r) and later A events do not change B's inherited prefix.
For all accepted merge branches M:
 verify(M) passes + source branches remain byte-equivalent to pre-merge state.
For all NPC-visible responses:
 every cited secret originates in an observation accessible to that NPC at query time.
For all time-travel loops accepted:
 solver constraints are satisfiable under pinned law version.
```

## Reproducibility report schema

Each benchmark stores run ID, code revision/commit, world fixture hash, RNG stream seed family, engine schema versions, AI model IDs, provider settings, prompt versions, hardware/OS, duration, cost, metrics, failures, confidence intervals, reviewer blinding method and hashes of saved traces. Privacy/redaction applies. Never claim “real-world realism validated” from property tests alone.

## Failure classification

Severity 0: corruption, security/privacy breach or unbounded financial charge → release block and affected service shutdown. Severity 1: reproducible critical playability/merger error → release block. Severity 2: localised defects with safe workaround → release-manager risk acceptance required. Severity 3: cosmetic or documentation → backlog unless it misleads users on claims. Every critical regression requires root-cause analysis, reproducible fixture and counter-test.
