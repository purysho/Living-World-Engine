# User Journeys, UX Principles and Interfaces

> **Document ID:** LWE-UX-001 | **Status:** proposed UX specification; approved capabilities derive from decisions 06, 13–15, 24, 28–30 and 36.

## Information architecture

**Global navigation:** Worlds / Campaigns / World Studio / Simulation Observatory / Chronology & Merge Lab / NPC & Factions / Rules / AI & Writers' Brains / Assets / Community / Settings & Administration. Keep player-facing mode separate from GM and Owner privileges. Avoid an omniscient player map even if Owner can see one.

### Workspace 1 — World Library

World cards show name, version, owner, source/import rights, active branch, health state, last snapshot, database size and compatibility warnings. Actions: New World, Import, Clone, Export, Verify, Repair Preview, Archive. New World wizard collects minimal initial conditions, cosmology preset, generation mode, seed and offline/provider choice. No forced interview before basic use.

### Workspace 2 — Campaign Hub

Campaign record has branch identity, party/roles, current local time, rules edition, GM mode, consent profile, schedule and pending synchronisation. Session launcher must verify branch head and player scope, then establish an append-only session log. Session summary distinguishes observed facts, rumours and GM-private truths.

### Workspace 3 — World Studio

Multiple synchronized panes: map/terrain, timeline, population/economics charts, entity inspector, factions/relationship graph, climate/biomes, magic contract editor and natural-language proposal console. User edits write to a **proposal branch first**, show causal impact and validation, then allow authorised commit. Tooltips expose units, assumptions, confidence and effect range.

### Workspace 4 — Play and exploration

Mode switch: narrative text / tactical 2D/isometric / walkable first person / third person. The 3D client is a view onto world-state entities, not independent source of truth. Physical interactions use collision, pathing and validated actions. Conversation is freeform but supports accessibility controls, captions, replay, and a “what my character knows” journal. Inactive player's autonomous action policy visible and revocable.

### Workspace 5 — Historical Merge Lab

Dual timeline lanes with branch head, parentage and time-zone context. Conflicts are colour-coded by type (resource, ownership, knowledge, chronology, permission, protected fact, unresolved temporal loop). Affected graph view lists upstream/downstream nodes and confidence of dependency analysis. GM may simulate resolutions in new branch, inspect invariants, approve, reject or defer. Never offer a one-click “force merge” that bypasses immutable contracts.

### Workspace 6 — Observatory and test lab

Live views of population, output, trade, event throughput, confidence ranges, costs, model routing and simulations paused due to faults. Experiments are explicit branch jobs: input seed, variable perturbation, run limit, output metrics, comparison. Graphs label observed vs generated vs assumed values. “Healthy” badge requires actual measured verification; no faux dashboards.

## Example workflows

**World Owner creates town:** create local world → set planet/region/law constraints → choose guided settlement → review region proposal → run constraint checks → accept initial canon → generate 250 identity stubs → seed household/account balances → establish initial branch and archive snapshot.

**GM prepares mystery:** open events and unresolved consequences → view candidate crime with objective fact graph → check evidence availability and witness knowledge → approve presentation hooks (not rewrite truth) → publish player handouts with field-level redaction → start session.

**Player speaks to merchant:** reach valid physical proximity → request conversation → retrieve only merchant-visible beliefs/perceptions/current affect → route to model under budget → parse structured intent and utterance → validate no tool-privilege abuse or lore contradiction → render speech/gesture → commit any legitimate promises, transfer or reputation changes as separate events.

**Co-GMs merge:** sign in → submit branch head → dependency evaluation → inspect conflicts and protected events → collaborative review if policy requires → create approved merge branch → notify relevant campaigns only when communication/knowledge rules permit.

## Interaction design rules

- Default to a powerful but uncluttered novice view; expert telemetry behind toggles.
- Dangerous actions require dry-run preview, owner-sensitive permission, explicit confirmation and backup point.
- Every AI action visibly classified as **suggestion**, **queued approval**, **rejected**, or **committed**.
- User can inspect “Why did this happen?” explaining mechanical causal chain and known uncertainty, not pretending to expose authentic model thought processes.
- Design for keyboard-only controls, captions, transcripts, high contrast, adjustable type, colour-independent statuses and reduced motion.
- Secrets always permission-filtered in backend. Hiding data in UI alone is insufficient.
