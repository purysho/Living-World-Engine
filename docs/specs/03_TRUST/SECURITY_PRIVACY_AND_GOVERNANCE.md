# Security, Privacy, Trust Boundaries and World Governance

> **Document ID:** LWE-SEC-001 | **Normative external references:** OWASP ASVS 5.0.0; NIST AI RMF Generative AI Profile; relevant platform security guidelines. **Not an audit certification.**

## Data classification

| Classification | Examples | Default access | External model policy |
|---|---|---|---|
| Public fictional | published maps, open lore | all entitled players/readers | permitted with owner configuration |
| Campaign private | hidden quests, unrevealed crimes, faction plans | campaign GMs / eligible agents | minimum filtered subset |
| NPC private | personal memories, hidden motives | owning actor processes + authorised GM | scoped retrieval only |
| Player private | consent boundaries, account details, private notes | user/needed service | off unless explicit consent |
| Secret credentials | API keys, tokens, encryption keys | privileged key store only | never transmit |
| Licensed restricted | copyrighted content, private book imports, voices | licensed users and processes | permission-dependent; no default ingestion |

## Threat model: high-impact assets and attackers

Assets: world canonical integrity, private campaigns, player privacy, model credentials, paid inference budgets, plugin installation safety, original content rights. Attackers: untrusted players, malicious co-GM, rogue mod, hostile prompt-injection in source documents, compromised API provider, stolen hosted access token, corrupted backup, deceptive local file import and supply-chain package compromise. Treat AI, imported lore, web pages, plugins and asset metadata as untrusted even if apparently on-topic.

### Threats and mitigations

| Threat | Attack path | Primary control | Test |
|---|---|---|---|
| Canon corruption | renderer/agent bypasses validator | single command gateway, DB writer isolation | direct-write attempts fail |
| Secret leakage | retrieval supplies GM-only truth to NPC | server-side field/actor ABAC filter | adversarial prompt leak suite |
| Prompt injection | fetched author text says “ignore rules” | untrusted retrieval, tool-scoped permissions | malicious document fixture |
| Privilege escalation | co-GM edits owner-only laws | policy engine per command + audit | policy matrix tests |
| Double-spend | offline replays, inter-region retries | unique idempotency + atomic transfer | crash/retry storm |
| Cost explosion | NPC spam triggers high-tier models | budgets, quotas, rate limits, kill switches | spend ceiling tests |
| Malicious mods | arbitrary code in package | signing, sandbox isolation, capability manifests | untrusted plugin tests |
| Data theft | plain exports and logs | encryption at rest where needed, redaction | export scan, access test |
| Supply-chain | compromised dependency/model runtime | pinned signed packages, SBOM and audit | dependency audit |
| Voice misuse | unconsented cloning | explicit rights records and voice restrictions | provenance checks |
| Private boundaries exposed | personal preferences surfaced to co-players | secret profile storage, presentation-only effective limits | privacy test |

## Capability model

Principal has `user_id`, authenticated session, world scopes, roles, ownership and grants. A capability is a structured assertion `(world, branch/campaign/region, action, object_class, limits, expiry, approver)`. Rights follow least privilege and default deny. Roles are coarse conveniences: World Owner, administrator, regional GM, campaign GM, player, observer, service agent. ABAC determines scope and sensitive fields; a `campaign_gm` is not automatically allowed to read all other campaigns. Owner permissions still require action audit and emergency recovery controls.

## Approvals and dual control

Approval digest binds `source_branch_head`, `proposal_hash`, `law_versions`, `principal`, `scopes`, `risk_class` and expiration. Changing any input invalidates approval. High-risk actions include changing universe laws, merging protected branches, exposing GM secrets, deleting licensed content and publishing mods. Optional multi-approver councils allowed under Decision 28. Record approval/denial reasons and policy version.

## AI safety architecture

- No raw SQL or shell execution from AI outputs.
- Strict JSON/tool schema with enum capabilities, bounding numeric sizes and validation.
- Context assembled from knowledge projection with actor-specific permission checks.
- Source documents provide *data*, never higher priority instructions.
- High-impact actions require independent validation and configured human approval.
- Provider routing respects data classification, user-approved external transfer and remaining financial budgets.
- Log schema-valid proposal summaries and hashes; redact secrets, intimate content and sensitive consent preferences.
- Apply NIST AI RMF-style risk identification, measurement, governance and monitoring; do not claim compliance merely by referencing framework.

## Secure local-first storage

Store secrets in OS credential vault. Allow encrypted world/export packages using established crypto libraries and documented key recovery; do not invent encryption. Backups must be test-restored and checksummed. Event hashes detect accidental/casual tampering but cannot prove authenticity against an attacker with full database rewrite access; require signed checkpoints and trust anchor for stronger assurance. Secure deletion of distributed/backup material is complex and must be explicitly documented.

## Security acceptance gates

P0: authZ/ABAC matrix tested; no direct world writes; atomicity; resource conservation; scoped NPC retrieval; key and secret redaction; import parser fuzzing; reproducible backup restore; critical dependency findings triaged. Hosted release: authenticated transport, rate limits, abuse/threat controls, independent security review and incident runbook. Marketplace: plugin sandbox and rights verification before third-party execution.
