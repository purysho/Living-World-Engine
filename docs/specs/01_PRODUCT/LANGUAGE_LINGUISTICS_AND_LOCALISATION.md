# Language Strategy: Programming, In-World Linguistics and Localisation

> **Document ID:** LWE-LANG-001 | **Scope:** distinguishes software implementation languages from fictional spoken languages and UX localisations.

## Software language standards

**Production recommendation (not a new approved decision):** Rust stable for authoritative state/simulation/contract validation and local IPC, TypeScript (strict mode) for Tauri 2 desktop UI and client orchestration, SQL for local SQLite migrations, Python 3.11+ for research, offline experiment tools and the existing proof-of-concept, JSON Schema/Protocol Buffers (evaluate) for formal interface contracts, Markdown/MDX and Mermaid for docs. Renderer-dependent language to be selected after a benchmark (Godot: GDScript/C#; native Rust renderer: Rust/WGSL). **Avoid** building duplicate authority in Python and Rust indefinitely; Python must become an executable reference oracle or be retired after conformance.

### Repository-wide coding rules

- Rust edition pinned in `rust-toolchain.toml`; clippy warnings reviewed; public APIs documented; `unsafe` forbidden by default except audited isolated crates.
- TypeScript `strict: true`; runtime validation at trust boundaries; never treat TS static types as JSON/IPC validation.
- SQL migrations numbered and irreversible semantics documented; use explicit foreign keys and immutable event constraints.
- Units are types or tagged values (e.g., `quantity_milli`, `metres`, `joules`, fictional magical potential units), never implicit bare floats for invariants.
- Public API schemas versioned; unknown fields rejected for security-sensitive commands or explicitly ignored under documented forward-compatibility contracts.
- All stored text UTF-8 and Unicode normalised where identity comparison requires it; preserve original user orthography separately.
- All time representation includes fictional realm, local coordinate, causal ordering metadata and commit time; a JavaScript `Date` is not sufficient.

## In-world language model

A **Language** entity defines writing systems, phonology/phonotactics, morphology, syntax, semantics, register, social distribution, intelligibility matrix, loanwords, historical lineage and dialect territories. A **SpeakerProficiency** record is character-specific and multidimensional (reading, listening, speaking, writing, specialist register). A **Transmission** record notes original language, translation chain, uncertainty and available medium. “Language exists” does not imply every NPC can speak it.

### Evolution and contact

- Historical language changes operate at population level, influenced by isolation, migration, conquest, prestige and education.
- Local languages are instantiated as structured grammars and lexica when a campaign needs dialogue or translation. Keep invented phrases canonical once used; do not regenerate new grammars each scene.
- Communicative misunderstandings may produce consequential NPC beliefs and events, but translation models do not get unrestricted access to hidden source meaning.
- The engine can render player-readable glosses as accessibility overlays while still tracking the character's actual comprehension.
- A divine or magical universal translator has defined range, units, cost, limitations and knowledge consequences in the magic contract.

## Internationalisation and accessibility

UI localisation uses keyed messages with pluralisation and grammatical context; separate language packs from fictional lexicon. Support Unicode, non-Latin scripts, keyboard input methods, bidirectional text/layout readiness, voice locale preferences, subtitles, transcript export, and culturally aware date/number/units presentation. **UI localisation never changes canonical world data.** Store stable IDs instead of localised display names; keep in-world chronology independent of the operating system locale.

## Dialogue generation quality gates

A dialogue event includes speaker ID, listener IDs, selected language/register, utterance text, speech performance metadata, source knowledge references, consent classification, per-player render permissions and any proposed consequential actions. Test continuity of greetings, honorifics, misinformation, translations, remembered promises and character-specific speech. Voice cloning of real people requires demonstrable rights and consent.
