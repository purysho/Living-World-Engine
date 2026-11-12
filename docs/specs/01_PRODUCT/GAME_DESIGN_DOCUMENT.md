# Game Design Document (GDD)

> **Document ID:** LWE-GDD-001 | **Genre:** persistent simulation platform / solo and multiplayer TTRPG / systems-driven sandbox | **Rating:** campaign-specific within owner boundaries | **State:** approved direction, proposed mechanics.

## Game fantasy and core loops

A player inhabits a world that existed before their arrival and continues independently afterward. Their character is neither automatically central nor irrelevant: informed actions produce consequences within a causally coherent society. A GM can run authored adventures without suppressing emergence; the AI can serve as GM without controlling the world's objective truth.

**Short loop (seconds–minutes):** observe/perceive → choose/socialise/move/act → D&D resolution or validated real-world action → event commit → feedback and perception update. **Medium loop (hours–weeks):** travel → gather knowledge → build relationships → acquire resources → investigate and negotiate → adapt to faction reactions. **Long loop (years–generations):** careers → households → institutions → technological progress → geopolitics → legacy across campaigns.

## Game modes

- **Human-GM:** GM controls narrative presentation, authorises canon changes, manages pacing and player consent; deterministic rules adjudication assisted by engine.
- **AI-GM:** structured tool access, read-scope from role/campaign/character; narration from filtered state; no direct database writes; robust fallback when model unavailable.
- **Collaborative-GM:** human delegates NPC voicing, encounters, inventories or world event previews to AI; all tool writes require capability checks.
- **Observatory/world studio:** no player-avatar requirement, run experiments and inspect long-term emergent society.
- **Multiplayer:** optional hosted authoritative world; sessions may share a branch or diverge as controlled working branches.

## Rules and combat

Rules profiles are pinned to **2014 SRD 5.1** or **2024 SRD 5.2.1**, plus a versioned homebrew overlay. Use only rights-permitted rule text, properly attributed. Abilities, proficiency, actions, saving throws, hit points, conditions, spell effects, rest, inventory, character advancement and encounter rules are **edition-specific**, not approximated from memory. A domain translator maps D&D hit point/condition outcomes to deeper health/environment simulation with explicit conflict policy. Rule tables must be data-driven and verified against licensed SRD fixtures.

Outside combat, simulated time advances at context-specific rates. In combat, rounds and initiative use tabletop ordering; speculative pre-declarations can be collected simultaneously but resolution obeys the chosen profile. No real-world wait during deliberation implies fictional time passing unless an explicit real-time mode later exists.

## Exploration

Macro map → region → settlement → interior/dungeon → tactical encounter. Each view references stable entity IDs and positions. Isometric, first-person and third-person have shared collision/navigation state and common interactable IDs. Generated visual assets cannot introduce doors, treasure or roads absent from the canonical physical model. Fog-of-war is a knowledge view, not deletion of geometry. Travel includes distance, terrain, weather, provisions, fatigue, transport and communication delay; all numerically approximated with stated fidelity.

## Adventures and mystery

No universal quest-generator may directly assert a crime happened to provide a hook. The narrative indexer scans committed events for opportunities (crime, disaster, unusual discovery, interpersonal conflict, natural phenomena). A candidate adventure contains trigger IDs, involved entities, character knowledge, evidence records, unresolved state, expiry conditions and possible player-facing leads. Authored stories enter through a GM-approved branch and must satisfy core invariants.

**Investigation fairness:** objective crime facts precede evidence; evidence has origin, degradation, access history and detectability; NPCs may perceive/lie/misremember; red herrings must arise from a plausible cause. A mystery may be insoluble, but the system must not assert a deductively unique solution absent sufficient evidence.

## Social, romance, horror and consent

Dynamic social relationships distinguish trust, respect, fondness, obligation, attraction, fear and familiarity. Partners, family, affection, marriages and feuds emerge through voluntary NPC choices and socioeconomic/cultural constraints. Player romantic agency is never assumed for an absent player without explicit permissions. Violence and horror can be present in canonical events while visual/narrative intensity is filtered by universe, campaign and individual boundaries. NPC consent and power asymmetries should be modelled as social realities, not gamified as frictionless “affection points.”

## Progression, failure and consequence

Progression includes D&D level progression (when rules profile dictates), reputation, languages, economic position, knowledge, institutional influence, family ties and political standing. Fail-forward is a **presentation choice**, not an authorisation to rewrite outcomes: failure can open a different opportunity naturally, but the world may not guarantee that outcome. Permanent death or failed negotiations can remain final according to metaphysical and campaign rules. Unresolved NPC goals continue during authorised clock advancement.

## First vertical slice: River Trading Town

**Scale:** 1 detailed town, 100–300 persistent residents, 3 competing organisations, household inventories and grain supply, 2 campaign branches, 1 mystery/evidence chain, 1 walkable interior + streets at first, 1 playable combat fixture, 1 GM timeline view. **Proposed thematic seed, not approved canon:** harvest shock puts pressure on transport, creates inventory conflict, fuels a theft and two parties investigate different economic incentives. The seed must be generated/validated rather than smuggled in as an absolute lore fact.

## Anti-patterns

- “NPC autonomy” that consists of periodic fictional diary prose with no committed effects.
- Converting every conflict into a cinematic set piece regardless of player interests.
- Infinite chat retaining unsanitised GM secrets.
- Illusory branching where one player action retroactively changes unrelated campaigns.
- Artificially scripted drama disguised as simulation-driven events.
- Mechanics silently switching edition after a campaign starts.
