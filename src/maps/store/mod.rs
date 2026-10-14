//! The SQL that writes the three tables a map is drawn from, one module per table:
//! [`placements`] (`map_solar_systems`), [`connections`] (`map_connections`) and
//! [`signatures`], and the jump log that goes with a connection, [`jumps`]. Every full-row write lives here, next to the struct it writes and the
//! select that snapshots it, so a new column is one struct and the queries beside it rather
//! than a hunt through every command that ever inserts a row. So does any narrower write
//! that more than one command makes; a write only one command makes stays with it.
//!
//! These functions are SQL and nothing else. Who may do it, what it announces and how it
//! is undone belong to the command that calls them (see [`super::command`]).

pub(crate) mod connections;
pub(crate) mod jumps;
pub(crate) mod placements;
pub(crate) mod signatures;
