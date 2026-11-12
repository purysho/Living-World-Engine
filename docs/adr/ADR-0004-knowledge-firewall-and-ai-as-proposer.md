# ADR-0004 — Knowledge firewall in code; AI is a proposer with an offline twin
Status: Accepted provisionally under Decision 61
Date: 2026-11-12
Applicable decisions: 07, 16, 24, 44
Context/problem: R-02 (fake emergence from narrative LLMs), R-06 (prompt injection, leaks), R-11 (cost runaway).
Decision and rationale:
1. Objective truth (`Crime.culprit`, `Claim.truth`) is never read when building an NPC context. Context comes from that NPC's own knowledge projection, memories and relationships.
2. A model's reply is untrusted input. It is parsed as `npc-reply/1`; disclosed claim ids are intersected with what the NPC knows; the reducer independently rejects any `play.talked` event that discloses an unknown claim (`knowledge_leak`). Deliberation returns one action from a closed list and is dry-run through the real reducer before it is shown; applying it is a separate, logged GM command.
3. The daily simulation never calls a model. Everything a model can do has a deterministic offline twin built from the same context, so the app is complete without a key and replay never depends on a provider.
4. Tier selects the model (a budget choice, user-configurable). A hard monthly cap is checked before every call; failures are surfaced, logged and never replaced by invented output.
5. Agent `.md` profiles are persona text. Scope and permissions come from tier in world state, so editing a profile cannot widen what an NPC knows or may do.
Privacy: with AI on, the filtered NPC context and the player's line are sent to the provider. There is no per-request preview screen yet.
Unresolved: no live-model evaluation has been run (leak rate, cost per session, believability); local model adapters are not implemented.
Proof: `at011_013_…`, `council_and_agents_work_offline`, `ai::tests`.
