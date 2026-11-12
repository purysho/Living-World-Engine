# AI Orchestration, Research Corpus and Writers' Brains Council

> **Document ID:** LWE-AI-001 | **Approved basis:** decisions 07, 16, 40–41, 44, 49 | **Status:** design. Starter `.md` agents and author registries exist; no working council service yet.

## Provider-neutral model gateway

Expose `GenerateDialogue`, `DeliberateAction`, `SummariseMemory`, `ResearchSources`, `ReviewProposal`, `NarrateScene`, `GenerateAssetBrief` with schema-defined I/O and policy constraints. Model routing evaluates ability/expected quality, latency, task class, privacy classification, local capability, cost per token, remaining budget and failure state. Provider adapters must isolate API syntax and version pinning. Fable 5.2 is a high-capability option, not a state store, narrator-authority or perpetual background process.

**Workflow:** classify task → validate role/consent → retrieve only actor-allowlisted facts/evidence → set model/budget → request bounded structured result → parse/validate/refuse unsafe tools → optionally run independent critic → simulate proposed action in isolated state → seek approval if threshold → commit through domain command. All model text and tool outputs are *untrusted inputs*. No agent outputs shell commands or SQL to execute against an authoritative world.

## Standard AI job envelope (illustrative)

```json
{
 "job_id":"job_abc", "kind":"npc.choose_action", "world_id":"river",
 "branch_id":"campaign_a", "actor_id":"npc_117", "principal_scope":"npc_self",
 "allowed_sources":["npc_117:memory","town:public_notice","location:visible"],
 "allowed_tools":["propose_move","propose_conversation","request_trade"],
 "world_head":"rev_129", "model_policy":"npc-tier-2-v1",
 "max_usd":0.03, "deadline_ms":4000,
 "safety_profile":"campaign_public", "output_schema":"npc-action/2"
}
```

Budgets and deadlines are **illustrative settings**, not measured production costs. If model fails, apply a bounded deterministic fallback or queue a deferred choice. Timeouts do not authorise fictional time to advance beyond an unresolved decision dependency.

## Writers' Brains council workflow

1. **Route:** topic class (plot craft, hard magic, dialogue, forensic mystery, science fiction, politics, myth, warfare, relationships, visual arts, accessibility and interaction).
2. **Source retrieval:** primary sources (author essays, interviews, lectures, licensed craft excerpts) with author/source/date/rights and quote-span restrictions. Secondary commentary labelled.
3. **Independent interpretation:** source-grounded design principles, explicit uncertainty and limits; do not attribute invented dialogue to named authors.
4. **Competing proposals:** at least two substantively different options; not merely tone rewrites.
5. **Adversarial critic:** challenges consistency, originality, agency, social/physical realism, cost and unintended narrative lock-in.
6. **Simulation compatibility:** proposal transformed into schema-valid candidate and dry-run where testable; human creative opinions are not formal proof.
7. **Decision memo:** ranked proposals, supporting citations, disagreements, unresolved facts, test plan and approval requirement.
8. **Audit:** outcome compared to later playtest/benchmark; review is versioned and does not rewrite historical source attribution.

## Author registry design

Each record: `author_id`, `display_name`, `verified_identity`, `discipline`, `strength_tags`, `time_period`, `rights`, `primary_source_count`, `secondary_source_count`, `last_verified`, `source_profile_version`, `confidence_notes` and `dedup_aliases`. Normalize repeated names (e.g., Neil Gaiman, Brandon Sanderson, Robert Jordan) but **do not merge distinct individuals solely due to name similarity**. Some user entries are fictional characters or unclear names; mark them `unresolved/persona_or_fictional` pending identity verification, not verified human authors. Profiles describe evidenced methods, not “the person would definitely say X”. Characters named as inspirations are evaluated for narrative function, not treated as real authorial advisers.

## RAG and source governance

Store metadata-first index (bibliography, licences, extracts where permitted). Chunk text by semantic section with source URL, location, publication, copyright policy, extraction quality and revision. Retrieval pipeline: metadata filter → hybrid keyword/vector search → source reputation and topical rerank → citation-linked context → quotation limits → independent source-verification pass. Source absence should yield “not enough documented evidence”, not fabricated quotes or false consensus. The system's reference starter includes approximately 206 deduplicated candidate author entries and 7 seeded source-linked records, but **not** a completed or audited library for all authors.

## Evaluation

- Retrieval recall@k and citation precision on a human-annotated author-method benchmark.
- Faithfulness: each named-author claim resolves to the cited passage with proper caveat.
- Blind multi-criteria review of council recommendations against non-council baseline.
- Prompt-injection resistance using malicious documents in retrieval and imported world lore.
- Cost, latency, refusal/fallback frequency, context-size saturation, secrets leakage.
- Conflict of interest and model self-preference tests; independent critic cannot write canon.

## Data handling

Never upload private player material or confidential world data without opt-in and clear data-class mapping. Redact API keys and personally identifying information from research corpora. Prefer public bibliographic metadata rather than storing full copyrighted works by default. Provide a per-request “what was sent to provider” privacy preview where feasible.
