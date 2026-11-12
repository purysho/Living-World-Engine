# Third-party material and attribution

## Dungeons & Dragons System Reference Documents

This work includes material taken from the System Reference Document 5.1 ("SRD 5.1") by Wizards of the
Coast LLC and available at https://dnd.wizards.com/resources/systems-reference-document. The SRD 5.1 is
licensed under the Creative Commons Attribution 4.0 International License available at
https://creativecommons.org/licenses/by/4.0/legalcode.

This work includes material from the System Reference Document 5.2.1 ("SRD 5.2.1") by Wizards of the Coast
LLC, available at https://www.dndbeyond.com/srd. The SRD 5.2.1 is licensed under the Creative Commons
Attribution 4.0 International License, available at https://creativecommons.org/licenses/by/4.0/legalcode.

What is used: generic mechanics only — ability modifiers, proficiency bonus, d20 tests with advantage and
disadvantage, attack rolls against Armor Class, critical hits, initiative, surprise, exhaustion, the
potion of healing, and four class chassis (fighter, rogue, wizard, cleric) at level 1. Implemented in
`crates/lwe-core/src/rules.rs` and `play.rs`. No setting material, named characters, monsters' lore or
text passages are reproduced. "Dungeons & Dragons" and "D&D" are trademarks of Wizards of the Coast; this
project is not affiliated with or endorsed by Wizards of the Coast.

The rule implementations were written from the author's knowledge of the SRDs and are covered by unit
tests for the specific behaviours listed above. They have **not** been checked line-by-line against the
published SRD text; treat edition-specific details as needing that review before any public release.

## Original setting

Marrowick, the River Sable, the Granary Compact, the Ferrymen's Lodge, the Reeve's Hall, the Lattice and
all generated people are original to this project. Nothing is taken from any published campaign setting.

## Writers' Brains registry

`.claude/skills/writers-brains/registry.json` lists names supplied by the World Owner as inspirations,
grouped by craft lens. It contains names and lens tags only — no quotations, no excerpts, no claims about
what any person thinks. Nothing in this project is written or endorsed by the people listed.

## Software dependencies

Rust crates (serde, serde_json, rusqlite with bundled SQLite, sha2, hex, ureq, tiny_http, tauri) and npm
packages (react, react-dom, vite, typescript, @tauri-apps/*) are used under their own MIT/Apache-2.0/
public-domain licences; see `Cargo.lock` and `apps/desktop/package-lock.json` for exact versions.
