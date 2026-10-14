//! Map import and export, wire-compatible with the legacy `wormholesystems-map-export`
//! format (version 1), so files move between the two applications in both directions.
//!
//! The file is a JSON envelope with six independently selectable sections. Cross
//! references inside it are portable by construction: EVE solar system ids, wormhole code
//! names ("K162"), signature type names, and entity ids for access rows. Signatures point
//! at connections by array index within the same file.
//!
//! Legacy speaks a slightly different vocabulary for the life-cycle enums (mass `fresh`
//! vs `stable`, ship sizes vs hole sizes, lifetime `healthy` vs `stable`); the `Wire*`
//! enums in [`format`] are that vocabulary, with conversions. What this app cannot
//! represent is skipped rather than refused: a ghost placement is not exported, a
//! signature without a scanner id is not imported.

mod export;
mod format;
mod import;

pub use export::*;
pub use format::*;
pub use import::*;
