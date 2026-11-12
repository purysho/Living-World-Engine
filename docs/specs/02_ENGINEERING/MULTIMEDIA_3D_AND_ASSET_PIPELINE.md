# 3D Exploration, Art, Music, Voice and Asset Pipeline

> **Document ID:** LWE-MEDIA-001 | **Basis:** decisions 13–14, 30, 36–37 | **Status:** target capabilities, renderer selection pending an ADR and benchmark.

## Design intent

Visual/audio presentation is derived from world-state truth. The same named location and NPC must be represented consistently across tactical maps, narrative portraits, first-person and third-person cameras. Text images are not geometry; generated geometry does not automatically create legal paths or interactable assets. Architectural, clothing, art and musical styles should reflect culture, climate, technology, history, materials and economics where the simulation has supporting data.

## Geometry and world model

`WorldSpatialEntity`: stable entity ID, geometry reference, physical units, local coordinate reference frame, collider, navmesh contribution, material/interaction properties, location hierarchy and instance state. `WalkableLocation`: static geometry, procedural seed, navigation mesh, collision and occlusion volumes, enter/exit transitions, visibility policy and LOD configuration. `NPCInstance`: canonical actor ID, current position/pose, equipment, animation state, voice profile and visibility.

**Three camera perspectives:** isometric orthographic/near-perspective tactical, first-person eye-level with reachable interactions, third-person follow/orbit. Switching camera must not teleport, despawn or duplicate objects. Tactical movement and combat grids, if displayed, are overlays/adapters; physics coordinates remain canonical. Accessibility alternative for motion sickness required (reduced bob/motion and teleport navigation where compatible).

## Asset pipeline stages

1. **Request:** approved creative brief attached to world entity / culture / era / required art rights.
2. **Reference:** materials, region palette, production technique, construction resources, species anatomy, historical precedent.
3. **Generation/import:** licensed manual, procedural or AI output. Never imply copyrighted reference assets grant redistribution rights.
4. **Review:** aesthetic, style, rights/consent, cultural plausibility, adult content controls and model consistency.
5. **Technical validation:** 3D topology, poly budget, collider correctness, UV/textures, textures under max size, animation rig, navmesh connectivity, fallback and asset naming.
6. **Bind:** stable entity ID, asset version, content hash, origin and rights manifest.
7. **Deliver:** local cache, incremental downloads, memory/LOD profile, optional offline fallback.
8. **Rollback:** if content becomes invalid or licensing changes, revoke publishing and maintain historical manifests without unlawful redistribution.

## Speech and music

NPC voices should be optional, not required for gameplay. Voice profile includes permitted language/dialect, speech model, timbre traits, pace, emotion tags, consent/rights and accessibility captions. Avoid cloning an identifiable real person's voice without clear authorisation. Music generation is separate from diegetic world history: style descriptors derive from region's instrument materials, scales/modes, cultural tradition and technological era. Save tracks and stems with copyright/provenance and use non-generated silence/ambience as a safe fallback. Not all societies need Western tonal conventions.

## Renderer choice proof

Compare Godot 4 versus Bevy/native Rust approach against: Windows deployment/integration with Tauri; 3 view modes; procedural scenes; instanced NPC crowds; pathfinding and collision; dialogue UI; asset import; accessibility; scripting/mod safety; RAM/GPU on reference laptop; IPC attack surface and build reproducibility. No binding decision until tested. Native 3D as a child process implies hardened local IPC and crash isolation; sharing a window or embedding technology must be tested, not assumed.

## MVP graphics limits

One river quay and market route; one tavern interior; a handful of real interactable NPCs; no open-world seamless streaming initially. 3D assets curated or procedural with deterministic IDs. No promise of instant on-demand walkable cities from text prompts. Establish import/build time, frame rate and physical collision conformance tests before generating thousands of assets.
