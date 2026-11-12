# Research notes (2026-11-12)

What was looked at before building, and what was taken from it.

**The specification package** (`docs/specs/`, 60 frozen decisions + Decision 61). Authoritative. Its handoff
protocol sets the order: authoritative state first, simulation second, UI third, AI last and only as a proposer.

**Reference repositories supplied**

| Source | What it is | Used |
|---|---|---|
| flamableconcrete/awesome-dnd | CC0 link list. Points to Open5e (SRD API), Azgaar's map generator, Kanka, and an open Claude DM skill. | Confirmed the SRD is the rights-safe rules source. No code taken. |
| opendnd (org) | MIT generators: `genetica` (heritable traits), `personae`, `dominia`, `nomina`, `dynastia`. Mostly unmaintained since 2019–22. | Ideas only: trait inheritance by parental blend + noise; family-first generation. Reimplemented in Rust. |
| gitUmaru/dnd | Personal Obsidian vault, CC BY-NC-ND. | Nothing (licence forbids derivatives). |
| MrLeebo/dnd-5e | MIT Gatsby SRD viewer. | Nothing needed. |
| react-dnd | A drag-and-drop UI library — unrelated to Dungeons & Dragons. | Not used. |
| BeaGiudici/DnD-Books | Collection of published rulebooks, no licence. | **Deliberately not used**: likely copyrighted books. |

**brains.ceo.** Unreachable from the build sandbox (DNS blocked). From search results: each "brain" is one
person's public talks, essays and posts made searchable, and every answer cites its source. That is the
standard adopted for the Writers' Brains skill: lenses with candidate voices, a strict no-unsourced-attribution
rule, and a `sources` list per entry that starts empty and is filled by retrieval. The site itself was not read.

**Established techniques adopted** (from general knowledge, not fetched in this session)
- *Event sourcing with snapshots* for auditable, branchable history.
- *Generative-agent memory* (observation stream scored by recency, importance, relevance; bounded) for NPC
  memory — here with integer salience and decay, and no model in the loop.
- *Nemesis-style rivalry* (opponents who remember the player, survive, are promoted and return) generalised to
  NPC↔NPC feuds through an accumulating grievance model.
- *Tiered level-of-detail cognition*: cohort accounts → routine individuals → deliberative individuals.
- *Tauri 2 + GitHub Actions on `windows-latest`* for an NSIS installer without a Windows machine.

**Claude API.** Rust has no official SDK, so the gateway uses the Messages API over HTTPS (`anthropic-version:
2026-10-21`, `output_config.effort`, server-side refusal fallback for Opus/Fable). Not exercised against the
live service.
