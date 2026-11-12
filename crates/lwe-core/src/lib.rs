//! Living World Engine core.
//!
//! Authority flows one way: command → policy → reducer → invariants → atomic
//! append to the event log → projections. UIs, renderers, AI models and mods
//! only ever submit commands or read permission-filtered views.

pub mod agents;
pub mod ai;
pub mod api;
pub mod council;
pub mod knowledge;
pub mod merge;
pub mod model;
pub mod play;
pub mod reducer;
pub mod rng;
pub mod rules;
pub mod sim;
pub mod store;
pub mod worldgen;

/// Version of the deterministic simulation rules. Bump when a change would alter replay.
pub const ENGINE_VERSION: &str = "town/1";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
