//! The single entry point for changing a map.
//!
//! Every mutation is a [`MapCommand`] applied through [`execute`], which opens one
//! transaction, authorizes, applies, and records a `map_events` row before committing. The
//! `apply_*` functions stay `pub(super)` so a mutation without an audit entry is
//! unrepresentable outside this module. An [`Effect`]'s `inverse` is itself a
//! `MapCommand`, so undo is just another execution (see [`super::events_log`]) and redo is
//! undoing the undo row.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::access::require_role_tx;
use super::connection::{
    AddConnection, AgeConnection, CleanStaleConnections, ExpireConnections, RemoveConnection,
    SetConnectionStatus,
};
use super::error::{MapError, Result};
use super::eve_scout::AddEveScoutConnections;
use super::events_log;
use super::ghost::{AddGhostSystem, ResolveGhostSystem, RestoreGhostSystem};
use super::ignored::{AddIgnoredSystem, ClearIgnoredSystems, RemoveIgnoredSystem};
use super::jumps::{AddConnectionJump, RemoveConnectionJump, UpdateConnectionJump};
use super::restore::{RemoveRestored, RestoreSystems};
use super::signatures::{
    AddSignature, LinkSignature, PasteSignatures, RemoveSignature, RemoveSignatures,
    RestoreSignatures, UnlinkSignature, UpdateSignature,
};
use super::solar_system::{
    AddSystem, ClearMap, MoveSystem, MoveSystems, RemoveSystem, RemoveSystems, RestoreRallies,
    SetAlias, SetHome, SetNotes, SetOccupier, SetPinned, SetRally, SetStatus,
};
use super::tracking::TrackJump;
use super::watchlist::{AddWatchlistEntry, RemoveWatchlistEntry, SetWatchlistPinned};
use super::{
    Actor, MapConnection, MapSolarSystem, Role, Signature, connection, eve_scout, ghost, ignored,
    jumps, restore, signatures, solar_system, tracking, watchlist,
};

/// An open transaction, threaded through every `apply_*` so the write and its audit
/// row commit together.
pub(super) type Tx<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

/// Who applied a command. Background tasks record without a character and are never
/// undoable.
#[derive(Debug, Clone, Copy)]
pub enum EventActor {
    Character(Actor),
    System,
}

impl EventActor {
    pub(super) fn character_id(self) -> Option<i64> {
        match self {
            EventActor::Character(actor) => Some(actor.character_id),
            EventActor::System => None,
        }
    }
}

/// Several commands applied as one. Only produced as an inverse, so a command whose undo
/// takes several steps still occupies one entry in the history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence {
    pub map_id: i64,
    pub steps: Vec<MapCommand>,
}

/// Every way a map can change. Variants prefixed `Restore` are compensating commands:
/// they exist only as the inverse of a removal and are not routed by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum MapCommand {
    AddSystem(AddSystem),
    AddGhostSystem(AddGhostSystem),
    ResolveGhostSystem(ResolveGhostSystem),
    RestoreGhostSystem(RestoreGhostSystem),
    RemoveSystem(RemoveSystem),
    RemoveSystems(RemoveSystems),
    RestoreSystems(RestoreSystems),
    RemoveRestored(RemoveRestored),
    Sequence(Sequence),
    ClearMap(ClearMap),
    MoveSystem(MoveSystem),
    MoveSystems(MoveSystems),
    SetAlias(SetAlias),
    SetStatus(SetStatus),
    SetOccupier(SetOccupier),
    SetNotes(SetNotes),
    SetHome(SetHome),
    SetRally(SetRally),
    RestoreRallies(RestoreRallies),
    SetPinned(SetPinned),
    AddConnection(AddConnection),
    SetConnectionStatus(SetConnectionStatus),
    RemoveConnection(RemoveConnection),
    CleanStaleConnections(CleanStaleConnections),
    AgeConnection(AgeConnection),
    ExpireConnections(ExpireConnections),
    TrackJump(TrackJump),
    AddEveScoutConnections(AddEveScoutConnections),
    AddSignature(AddSignature),
    UpdateSignature(UpdateSignature),
    RemoveSignature(RemoveSignature),
    RemoveSignatures(RemoveSignatures),
    RestoreSignatures(RestoreSignatures),
    PasteSignatures(PasteSignatures),
    LinkSignature(LinkSignature),
    UnlinkSignature(UnlinkSignature),
    AddConnectionJump(AddConnectionJump),
    UpdateConnectionJump(UpdateConnectionJump),
    RemoveConnectionJump(RemoveConnectionJump),
    AddWatchlistEntry(AddWatchlistEntry),
    SetWatchlistPinned(SetWatchlistPinned),
    RemoveWatchlistEntry(RemoveWatchlistEntry),
    AddIgnoredSystem(AddIgnoredSystem),
    RemoveIgnoredSystem(RemoveIgnoredSystem),
    ClearIgnoredSystems(ClearIgnoredSystems),
}

/// What a command hands back to its caller.
#[derive(Debug, Clone)]
pub enum CommandOutput {
    None,
    Count(u64),
    System(Box<MapSolarSystem>),
    Connection(Box<MapConnection>),
    Signature(Box<Signature>),
    Jump(Box<super::jumps::ConnectionJump>),
    Watchlist(Box<super::watchlist::WatchlistEntry>),
    Ignored(Box<super::ignored::IgnoredSystem>),
    Removal(Box<super::signatures::RemovedSignature>),
    BulkRemoval(Box<super::signatures::BulkRemoveOutcome>),
}

impl CommandOutput {
    /// A mismatch is a wiring mistake between a command and its `apply`, never something a
    /// caller can act on.
    fn wrong(self) -> MapError {
        MapError::Validation(format!("unexpected command output: {self:?}"))
    }

    pub(super) fn system(self) -> Result<MapSolarSystem> {
        match self {
            CommandOutput::System(x) => Ok(*x),
            other => Err(other.wrong()),
        }
    }

    pub(super) fn connection(self) -> Result<MapConnection> {
        match self {
            CommandOutput::Connection(x) => Ok(*x),
            other => Err(other.wrong()),
        }
    }

    pub(super) fn signature(self) -> Result<Signature> {
        match self {
            CommandOutput::Signature(x) => Ok(*x),
            other => Err(other.wrong()),
        }
    }

    pub(super) fn jump(self) -> Result<super::jumps::ConnectionJump> {
        match self {
            CommandOutput::Jump(x) => Ok(*x),
            other => Err(other.wrong()),
        }
    }

    pub(super) fn watchlist(self) -> Result<super::watchlist::WatchlistEntry> {
        match self {
            CommandOutput::Watchlist(x) => Ok(*x),
            other => Err(other.wrong()),
        }
    }

    pub(super) fn ignored(self) -> Result<super::ignored::IgnoredSystem> {
        match self {
            CommandOutput::Ignored(x) => Ok(*x),
            other => Err(other.wrong()),
        }
    }

    pub(super) fn removal(self) -> Result<super::signatures::RemovedSignature> {
        match self {
            CommandOutput::Removal(x) => Ok(*x),
            other => Err(other.wrong()),
        }
    }

    pub(super) fn bulk_removal(self) -> Result<super::signatures::BulkRemoveOutcome> {
        match self {
            CommandOutput::BulkRemoval(x) => Ok(*x),
            other => Err(other.wrong()),
        }
    }

    pub(super) fn count(self) -> Result<u64> {
        match self {
            CommandOutput::Count(n) => Ok(n),
            other => Err(other.wrong()),
        }
    }
}

/// The audit record a command produces, plus its return value.
pub(super) struct Effect {
    /// Dotted group kind, e.g. `systems.added`.
    pub kind: &'static str,
    /// Human sentence without the actor, e.g. `added Jita`.
    pub label: String,
    pub entries: i64,
    /// `None` marks the change as not undoable.
    pub inverse: Option<MapCommand>,
    pub output: CommandOutput,
    /// What open clients should refetch. Each `apply_*` declares its own, and [`execute_as`]
    /// publishes them after commit: a handler can no longer forget one, and a command
    /// applied from the background or as an undo step announces itself the same way.
    pub events: Vec<super::MapEvent>,
}

impl Effect {
    pub(super) fn new(kind: &'static str, label: impl Into<String>, output: CommandOutput) -> Self {
        Self {
            kind,
            label: label.into(),
            entries: 1,
            inverse: None,
            output,
            events: Vec::new(),
        }
    }

    pub(super) fn entries(mut self, entries: i64) -> Self {
        self.entries = entries;
        self
    }

    pub(super) fn undo_with(mut self, inverse: MapCommand) -> Self {
        self.inverse = Some(inverse);
        self
    }

    pub(super) fn emit(mut self, event: super::MapEvent) -> Self {
        self.events.push(event);
        self
    }

    pub(super) fn emit_all(mut self, events: impl IntoIterator<Item = super::MapEvent>) -> Self {
        self.events.extend(events);
        self
    }
}

impl MapCommand {
    pub(super) fn map_id(&self) -> i64 {
        match self {
            MapCommand::AddSystem(c) => c.map_id,
            MapCommand::AddGhostSystem(c) => c.map_id,
            MapCommand::ResolveGhostSystem(c) => c.map_id,
            MapCommand::RestoreGhostSystem(c) => c.map_id,
            MapCommand::RemoveSystem(c) => c.map_id,
            MapCommand::RemoveSystems(c) => c.map_id,
            MapCommand::RestoreSystems(c) => c.map_id,
            MapCommand::RemoveRestored(c) => c.map_id,
            MapCommand::Sequence(c) => c.map_id,
            MapCommand::ClearMap(c) => c.map_id,
            MapCommand::MoveSystem(c) => c.map_id,
            MapCommand::MoveSystems(c) => c.map_id,
            MapCommand::SetAlias(c) => c.map_id,
            MapCommand::SetStatus(c) => c.map_id,
            MapCommand::SetOccupier(c) => c.map_id,
            MapCommand::SetNotes(c) => c.map_id,
            MapCommand::SetHome(c) => c.map_id,
            MapCommand::SetRally(c) => c.map_id,
            MapCommand::RestoreRallies(c) => c.map_id,
            MapCommand::SetPinned(c) => c.map_id,
            MapCommand::AddConnection(c) => c.map_id,
            MapCommand::SetConnectionStatus(c) => c.map_id,
            MapCommand::RemoveConnection(c) => c.map_id,
            MapCommand::CleanStaleConnections(c) => c.map_id,
            MapCommand::AgeConnection(c) => c.map_id,
            MapCommand::ExpireConnections(c) => c.map_id,
            MapCommand::TrackJump(c) => c.map_id,
            MapCommand::AddEveScoutConnections(c) => c.map_id,
            MapCommand::AddSignature(c) => c.map_id,
            MapCommand::UpdateSignature(c) => c.map_id,
            MapCommand::RemoveSignature(c) => c.map_id,
            MapCommand::RemoveSignatures(c) => c.map_id,
            MapCommand::RestoreSignatures(c) => c.map_id,
            MapCommand::PasteSignatures(c) => c.map_id,
            MapCommand::LinkSignature(c) => c.map_id,
            MapCommand::UnlinkSignature(c) => c.map_id,
            MapCommand::AddConnectionJump(c) => c.map_id,
            MapCommand::UpdateConnectionJump(c) => c.map_id,
            MapCommand::RemoveConnectionJump(c) => c.map_id,
            MapCommand::AddWatchlistEntry(c) => c.map_id,
            MapCommand::SetWatchlistPinned(c) => c.map_id,
            MapCommand::RemoveWatchlistEntry(c) => c.map_id,
            MapCommand::AddIgnoredSystem(c) => c.map_id,
            MapCommand::RemoveIgnoredSystem(c) => c.map_id,
            MapCommand::ClearIgnoredSystems(c) => c.map_id,
        }
    }

    /// The ceiling lives here so a new command cannot forget its check. Editing the graph
    /// is Member+; the ignore list changes where everyone's routes and jumps go, so it is
    /// a manager's call.
    pub(super) fn required_role(&self) -> Role {
        match self {
            MapCommand::AddIgnoredSystem(_)
            | MapCommand::RemoveIgnoredSystem(_)
            | MapCommand::ClearIgnoredSystems(_) => Role::Manager,
            _ => Role::Member,
        }
    }

    pub(super) async fn apply(self, tx: &mut Tx<'_>, actor: EventActor) -> Result<Effect> {
        match self {
            MapCommand::AddSystem(c) => solar_system::apply_add_system(tx, c).await,
            MapCommand::AddGhostSystem(c) => ghost::apply_add_ghost_system(tx, c).await,
            MapCommand::ResolveGhostSystem(c) => ghost::apply_resolve_ghost_system(tx, c).await,
            MapCommand::RestoreGhostSystem(c) => ghost::apply_restore_ghost_system(tx, c).await,
            MapCommand::RemoveSystem(c) => solar_system::apply_remove_system(tx, c).await,
            MapCommand::RemoveSystems(c) => solar_system::apply_remove_systems(tx, c).await,
            MapCommand::RestoreSystems(c) => restore::apply_restore_systems(tx, c).await,
            MapCommand::RemoveRestored(c) => restore::apply_remove_restored(tx, c).await,
            MapCommand::Sequence(c) => apply_sequence(tx, c, actor).await,
            MapCommand::ClearMap(c) => solar_system::apply_clear_map(tx, c).await,
            MapCommand::MoveSystem(c) => solar_system::apply_move_system(tx, c).await,
            MapCommand::MoveSystems(c) => solar_system::apply_move_systems(tx, c).await,
            MapCommand::SetAlias(c) => solar_system::apply_set_alias(tx, c).await,
            MapCommand::SetStatus(c) => solar_system::apply_set_status(tx, c).await,
            MapCommand::SetOccupier(c) => solar_system::apply_set_occupier(tx, c).await,
            MapCommand::SetNotes(c) => solar_system::apply_set_notes(tx, c).await,
            MapCommand::SetHome(c) => solar_system::apply_set_home(tx, c).await,
            MapCommand::SetRally(c) => solar_system::apply_set_rally(tx, c).await,
            MapCommand::RestoreRallies(c) => solar_system::apply_restore_rallies(tx, c).await,
            MapCommand::SetPinned(c) => solar_system::apply_set_pinned(tx, c).await,
            MapCommand::AddConnection(c) => connection::apply_add_connection(tx, c).await,
            MapCommand::SetConnectionStatus(c) => {
                connection::apply_set_connection_status(tx, c).await
            }
            MapCommand::RemoveConnection(c) => connection::apply_remove_connection(tx, c).await,
            MapCommand::CleanStaleConnections(c) => connection::apply_clean_stale(tx, c).await,
            MapCommand::AgeConnection(c) => connection::apply_age_connection(tx, c).await,
            MapCommand::ExpireConnections(c) => connection::apply_expire_connections(tx, c).await,
            MapCommand::TrackJump(c) => tracking::apply_track_jump(tx, c).await,
            MapCommand::AddEveScoutConnections(c) => {
                eve_scout::apply_add_eve_scout_connections(tx, c).await
            }
            MapCommand::AddSignature(c) => signatures::apply_add_signature(tx, c).await,
            MapCommand::UpdateSignature(c) => signatures::apply_update_signature(tx, c).await,
            MapCommand::RemoveSignature(c) => signatures::apply_remove_signature(tx, c).await,
            MapCommand::RemoveSignatures(c) => signatures::apply_remove_signatures(tx, c).await,
            MapCommand::RestoreSignatures(c) => signatures::apply_restore_signatures(tx, c).await,
            MapCommand::PasteSignatures(c) => signatures::apply_paste_signatures(tx, c).await,
            MapCommand::LinkSignature(c) => signatures::apply_link_signature(tx, c).await,
            MapCommand::UnlinkSignature(c) => signatures::apply_unlink_signature(tx, c).await,
            MapCommand::AddConnectionJump(c) => jumps::apply_add_jump(tx, c).await,
            MapCommand::UpdateConnectionJump(c) => jumps::apply_update_jump(tx, c).await,
            MapCommand::RemoveConnectionJump(c) => jumps::apply_remove_jump(tx, c).await,
            MapCommand::AddWatchlistEntry(c) => watchlist::apply_add_entry(tx, c).await,
            MapCommand::SetWatchlistPinned(c) => watchlist::apply_set_pinned(tx, c).await,
            MapCommand::RemoveWatchlistEntry(c) => watchlist::apply_remove_entry(tx, c).await,
            MapCommand::AddIgnoredSystem(c) => ignored::apply_add(tx, c).await,
            MapCommand::RemoveIgnoredSystem(c) => ignored::apply_remove(tx, c).await,
            MapCommand::ClearIgnoredSystems(c) => ignored::apply_clear(tx, c).await,
        }
        .map(|effect| {
            // Background writers never offer an undo.
            if matches!(actor, EventActor::System) {
                Effect {
                    inverse: None,
                    ..effect
                }
            } else {
                effect
            }
        })
    }
}

/// Apply each step in order, and hand back the reversed inverses as one command, so
/// undoing walks the steps backwards.
async fn apply_sequence(tx: &mut Tx<'_>, cmd: Sequence, actor: EventActor) -> Result<Effect> {
    let map_id = cmd.map_id;
    let mut inverses = Vec::new();
    let mut entries = 0;
    let mut events = Vec::new();
    for step in cmd.steps {
        // Boxed because this is `apply` calling itself: the future would otherwise be
        // infinitely sized.
        let effect = Box::pin(step.apply(tx, actor)).await?;
        entries += effect.entries;
        events.extend(effect.events);
        if let Some(inverse) = effect.inverse {
            inverses.push(inverse);
        }
    }
    inverses.reverse();
    Ok(
        Effect::new("sequence", "several changes", CommandOutput::None)
            .entries(entries.max(1))
            .undo_with(MapCommand::Sequence(Sequence {
                map_id,
                steps: inverses,
            }))
            .emit_all(events),
    )
}

/// Authorize, mutate, and record, all in one transaction. Recording also advances the
/// map's history cursor onto the new step, so a change made after an undo branches off
/// where the map is sitting rather than off the newest row.
pub async fn execute(pool: &PgPool, actor: Actor, cmd: MapCommand) -> Result<CommandOutput> {
    execute_as(pool, EventActor::Character(actor), cmd).await
}

/// As [`execute`], for background tasks, whose changes are recorded for the audit trail
/// but never become undoable steps.
pub(super) async fn execute_as(
    pool: &PgPool,
    actor: EventActor,
    cmd: MapCommand,
) -> Result<CommandOutput> {
    let map_id = cmd.map_id();
    let mut tx = pool.begin().await?;
    if let EventActor::Character(character) = actor {
        require_role_tx(&mut tx, map_id, character.user_id, cmd.required_role()).await?;
    }
    let effect = cmd.apply(&mut tx, actor).await?;
    // Whatever the command was, the ghosts it left behind or now owes are settled here, so
    // no write has to remember on its own. Undo of a command undoes its ghosts with it.
    let (ghosts, ghost_events) = ghost::reconcile(&mut tx, map_id).await?;
    let effect = ghost::with_undo_steps(map_id, ghosts, effect).emit_all(ghost_events);
    events_log::record(&mut tx, map_id, actor, &effect).await?;
    tx.commit().await?;
    // Published after commit, so nobody refetches a change that then rolls back.
    let hub = super::hub();
    for event in effect.events {
        hub.publish(event);
    }
    Ok(effect.output)
}
