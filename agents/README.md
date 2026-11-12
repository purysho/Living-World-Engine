# NPC agent profiles

Each `.md` file is an agent specification: YAML front-matter (tier, knowledge scope, permitted tools,
forbidden writes, escalation rule) plus the persona instructions used as the system prompt when a
model voices that NPC.

- `generic/` — importance tiers T1–T4. Most residents share `t1-resident`; the engine promotes people
  automatically as they matter more (office, crime, feud, repeated contact with the party).
- `custom/` — hand-written profiles for story-critical characters. They reference a canonical entity
  by role, never by copied facts: everything mutable (name, age, memories, grievances) is injected at
  runtime from world state, filtered to what that character actually knows.

The same files are compiled into the app (and can be edited per installation under **AI & Agents**),
and mirrored as Claude Code subagents in `.claude/agents/` for running NPCs from a terminal session.

Hard rules enforced in code, not by prompt:
1. An agent can never write world state. Its output is a proposal that the reducer validates.
2. Context is assembled from the NPC's own knowledge projection; GM-only truth is never in the prompt.
3. Disclosed claim IDs are checked against what the NPC knows; anything else is rejected (`knowledge_leak`).
4. No agent can widen its own scope by editing its profile: scopes are read by the engine from tier, not from the model.
