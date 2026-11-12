# NPC Intelligence, Memory, Psychology and Agent Tiers

> **Document ID:** LWE-NPC-001 | **Approved direction:** 07, 17, 26, 43–44, 48–49, 50 | **Important:** this is an agent architecture, not a validated scientific model of human minds.

## Tier ladder (compute budget, not intelligence)

| Tier | Type | Runtime policy | Persistence |
|---|---|---|---|
| T0 | cohort/uninstantiated | quantitative population and institution simulation | demographic/accounting aggregate |
| T1 | ordinary resident | scripted or probabilistic routines, sparse goals | persistent identity, household, relationships, summary memories |
| T2 | recurring character | planning triggers, selective compact model | belief graph, commitments, needs, memory index |
| T3 | major character | rich memory retrieval, deliberative model, strategy evaluation | private goals, long-term event-linked plans |
| T4 | world-shaping actor | high-capability specialist reasoning + explicit review | strategy trees, organisations, high-impact proposals |

NPCs may change tiers by influence, risk or interaction. Promotion adds detail without fabricating contradictory past facts. Agents not being queried still have physical/household/organisation constraints; “T0 means no LLM” is not “T0 has no effects.” Player-character AI autonomy is a separate explicit permission system, not inherited from NPC tiers.

## Cognitive pipeline

```mermaid
flowchart LR
 A[Physical and social perceptions] --> B[Permission-filtered knowledge]
 B --> C[Beliefs and reconstructed memory]
 C --> D[Needs, habits, norms and personality]
 D --> E[Candidate intents]
 E --> F[Planning and option scoring]
 F --> G[Schema-valid bounded proposal]
 G --> H[World contract/permission validation]
 H --> I[Canonical event if accepted]
 I --> J[Consequence perception and learning]
 J --> C
```

**Truth boundaries:** The canonical event store knows what occurred. `Evidence` captures traces; `Observation` captures what an actor could perceive; `Belief` captures subjective propositions with confidence and source; `Memory` records encoding, salience, recall distortion and decay. A character may be sincerely wrong. Model prompt builders *must not* fetch unrestricted world truth and rely on instruction-only hiding. Each retrieval is role and actor scoped.

## State and decision features

Personality vector is a flexible model, not a universal moral alignment. Parameters may include temperament, curiosity, risk tolerance, impulse control, habits, value hierarchies, attachments, knowledge confidence, stress and fatigue. Include individual differences, exposure, longitudinal change, childhood/family/institution effects and cultural norms. Avoid pseudoscientific equations that claim exact forecasts of complex humans. Several competing policy functions can operate, with outcomes assessed in blind behavioural studies.

**Decision inputs:** observable world and self state; social/moral constraints; available action affordances; personal goals and commitments; resources and time; beliefs and misinformation; emotion and habits; uncertainty/risk; budget for deliberation. **Outputs:** typed proposals with preconditions, intended effect, estimated utility (optional), uncertainties, necessary approvals and expiry. Unresolved high-risk proposals remain pending, not committed.

## Memory architecture

- **Episodic:** timestamped personally observed experiences linked to evidence and provenance.
- **Semantic:** beliefs about general facts, including uncertainty and false theories.
- **Social:** relationships, shared experiences, promises, emotional attachments and tensions.
- **Procedural:** acquired skills, routines, habits, training and strategies.
- **Prospective:** goals, tasks, expected deadlines, contingencies.
- **Autobiographical:** generated summary of self-perceived narrative; not canonical truth.

Retrieval should be context-dependent and subject to misremembering. **Source records remain intact** even if a character reconstructs their memory differently. If memory edits have world consequences, they become versioned knowledge/belief events; the underlying murder or transaction does not change.

## Organisation agents

Factions and institutions have policies, budgets, command hierarchies, shared documents, institutional memory, culture and internal subagents. Organisation goals do not necessarily align with the goals of their leaders. A guild can continue after a leader dies; succession and dissident factions are events. Strategic planning can be reviewed by a “villain-strategist” or other speciality, but any plan's effects go through the command validator.

## Important `.md` agent conventions

Generic agent template contains `role`, `persona_constraints`, `knowledge_scope`, `motivation_structure`, `permitted_tool_capabilities`, `budget_tier`, `memory_selectors`, `forbidden_writes`, `escalation_threshold` and `schema_output`. Custom major-NPC files reference canonical entity IDs; do not copy mutable facts into stale free-text biographies. Authors' craft council files and NPC agents are **different namespaces**. No agent can self-assign broader privileges by editing its Markdown config.

## Social and relationship design

Social graph edges store type, intensity, mutuality, obligations, secrets, history and validity times. Romance is permitted within campaign content boundaries and voluntary character choices. Absent-player characters receive routine-only or explicitly delegated actions. Marriage, betrayal, property sale, sacrifice and equivalent choices require player approval unless that specific class of action was authorised, with clear opt-out and timeout behaviour.

## Evaluation suite

- **Leak test:** secret unknown to merchant never appears in merchant's utterance or action, including after adversarial dialogue.
- **Identity test:** character persists across sessions; changing model provider doesn't change known canonical facts.
- **Promotion test:** ordinary NPC becomes relevant; preserved past relationships and household survive escalation.
- **Memory test:** contradictory witness recollections remain distinct from true event; character can correct belief after new evidence.
- **Agency test:** two agents with different goals choose meaningfully different courses under identical physical constraints.
- **Cost test:** cognitive tier routing improves perceived realism per dollar/token over one-model-for-all baseline.
- **Long-run test:** families, relationships and institutions evolve without unnatural repetitive patterns or unbounded memory growth.
