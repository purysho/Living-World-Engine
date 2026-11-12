# Living World Engine — notes for Claude Code

- Read `docs/STATUS.md` before claiming a feature exists; `docs/specs/00_GOVERNANCE/DECISIONS_01_60_FROZEN.md` is the frozen baseline.
- All world changes go through `lwe_core::reducer::apply` via `Engine::commit`. Never add another write path.
- The simulation (`crates/lwe-core/src/sim`) must stay deterministic: integers only, randomness only from `rng::Rng::new(seed, label, a, b)`, iterate `BTreeMap`s. A change that alters outcomes breaks replay of existing worlds — say so.
- Player-visible reads are filtered in `api.rs`/`knowledge.rs`; never rely on the UI to hide GM truth.
- Checks: `cargo test -p lwe-core` and `cd apps/desktop && npm run build`. Run the app with `cargo run --release -p lwe-server` after building the UI.
- Design decisions: use the `writers-brains` skill. NPC voices: `.claude/agents/npc-*.md` with context from the `npc.context` op.
