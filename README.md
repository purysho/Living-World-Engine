# Living World Engine

A local-first Windows app for running tabletop campaigns in a world that keeps living when nobody is looking.
The world is an **append-only, hash-chained history**; every campaign is a **branch** of it, so several
campaigns can run in the same world at once and be merged back under the World Owner's approval. A
deterministic simulation — not an AI narrator — decides what is true. AI models voice characters and
advise; they can only *propose*.

**Version 0.3.0 — first playable vertical slice.** One river trading town (Marrowick), ~250 persistent
residents. Read [docs/STATUS.md](docs/STATUS.md) for exactly what is and is not implemented.

## What you can do today

| | |
|---|---|
| **Create worlds** | Seeded generation of a town with three generations of genealogy, ~70 households, three rival organisations, a hinterland cohort, a ledger economy and a structured world bible. Same seed → same world. |
| **Run concurrent campaigns** | Fork any timeline into a campaign with its own clock, party, pinned rules edition (D&D 5e 2014 SRD 5.1 or 2024 SRD 5.2.1), GM mode and content settings. |
| **Watch it live** | Observatory: prices, hunger, population, crime, weather and river, organisation treasuries and policies, the magic potential field, ledger integrity. Advance a day, week, month or year. |
| **A real economy** | Households produce, earn, eat and trade daily; prices move on supply and demand; the Compact arbitrages against downstream prices when barges can run; taxes, wages, strikes, hoarding, relief. Every unit of coin and grain is conserved. Droughts cause failed harvests → scarcity → hunger → theft. |
| **People who think at different depths** | T0 cohort → T1 residents → T2 recurring → T3 major → T4 world-shaping. Tier is a compute budget; people are promoted as they matter. Each tier has an editable `.md` agent profile; story-critical characters get custom ones. |
| **Alliances, enemies, nemeses** | Grievances accumulate into rivalry, enmity and sworn nemeses who slander, sabotage, denounce, assault or bid for office. Organisations ally, feud, audit and embargo. Opponents who survive the party remember it, grow, and come looking. |
| **Mysteries that are actually true** | Crimes have a real culprit, physical evidence with custody, and witnesses who may be mistaken or lying. The watch investigates on evidence and testimony and can gaol the wrong person. Players see only what their characters have learned. |
| **Play** | Walk the town, talk to anyone present (answers come only from what that person knows), search scenes, roll skill checks, trade, steal, cast under a conserved hard-magic law, fight turn-based combat with an auditable dice log. |
| **Merge Lab** | Carry a campaign's consequences into canon. Impossible combinations are blocked with the conflicting events named. No force-merge. |
| **GM tools** | Weather shocks, property sales and destruction, recorded and protected facts, planted grievances — all typed, validated, logged events, with dry-run preview. "Why did this happen?" on any event. |
| **Writers' Brains** | A decision council: route to craft lenses → competing proposals → adversarial critique → simulation test → memo. In the app and as a Claude Code skill. |

## Install (Windows)

Every push builds an installer in GitHub Actions: open the latest **build** run for this branch →
**Artifacts** → `LivingWorldEngine-windows-installer` → run `Living World Engine_0.3.0_x64-setup.exe`.
It installs per-user (no admin) and needs the WebView2 runtime that ships with Windows 10/11.
Worlds are stored in `%APPDATA%\com.livingworldengine.desktop\worlds.db`.

The installer is **not code-signed** yet, so SmartScreen will warn; choose *More info → Run anyway*.

## AI (optional)

The app is fully usable offline. To have Claude voice NPCs, narrate and write council memos:
**AI & Agents → Claude (Anthropic API)**, paste an API key, set a monthly budget. Routing defaults:
T1–T2 and narration → `claude-haiku-5-5`, T3 → `claude-opus-5-5`, T4 → `claude-fable-5-2`,
council → `claude-opus-5-5` (all changeable). The key is kept in the Windows Credential Manager, never in
a world file. If a request fails or the budget is spent, you are told, and the offline voice answers.

## Build from source

```bash
cargo test -p lwe-core                 # 29 tests incl. the acceptance scenarios
cd apps/desktop && npm ci && npm run build
cargo run --release -p lwe-server      # http://127.0.0.1:4747  (browser version, same engine)
cd apps/desktop && npx tauri build     # desktop app + installer (on Windows: NSIS .exe)
```

Requirements: Rust stable, Node 22. Windows needs the MSVC build tools; Linux needs the WebKitGTK 4.1 dev packages.

## How it is built

```
apps/desktop/            React + TypeScript UI, Tauri 2 shell (one IPC command)
crates/lwe-core/         the engine: event store, reducers, simulation, rules, knowledge, AI gateway, merge
crates/lwe-server/       loopback HTTP host for the same gateway (dev, tests, headless)
agents/                  NPC agent profiles (.md): generic tiers + custom characters
.claude/agents/          the same tiers as Claude Code subagents
.claude/skills/writers-brains/   the council skill + author registry
docs/specs/              the v1.0 specification package (60 frozen decisions) this implements
docs/adr/                architecture decisions taken during implementation
docs/reports/            test report with measured numbers
```

One rule holds everywhere: **command → role policy → reducer → invariants → atomic append → projection.**
Nothing else can change the world. See [docs/adr](docs/adr) for why.

## Rights

No software licence has been chosen for this repository yet (an open owner decision in the specification);
until one is added, all rights are reserved by the owner. Rules mechanics come from the D&D System
Reference Documents under CC BY 4.0 — see [docs/LICENSES.md](docs/LICENSES.md). The setting is original.
