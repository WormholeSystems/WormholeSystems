//! The single entry point for changing a map.
//!
//! Every mutation is a [`MapCommand`] applied through [`execute`], which opens one
//! transaction, authorizes, applies, and records a `map_events` row before committing. The
//! `apply_*` functions stay `pub(super)` so a mutation without an audit entry is
//! unrepresentable outside this module. An [`Effect`]'s `inverse` is itself a
//! `MapCommand`, so undo is just another execution (see [`super::history`]) and redo is
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
use super::ghost::{AddGhostSystem, ResolveGhostSystem, RestoreGhostSystem};
use super::history;
use super::ignored::{AddIgnoredSystem, ClearIgnoredSystems, RemoveIgnoredSystem};
use super::jumps::{
    AddConnectionJump, RemoveConnectionJump, RestoreConnectionJump, UpdateConnectionJump,
};
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
use super::watchlist::{
    AddWatchlistEntry, RemoveWatchlistEntry, RestoreWatchlistEntry, SetWatchlistPinned,
};
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

/// The command table, one line per way a map can change: the variant (named after the
/// struct it carries), what running it hands back when that is more than nothing, and the
/// function that applies it. Generates [`MapCommand`], its `From` impls, `map_id`, the
/// dispatch in [`MapCommand::apply`] and each struct's [`Command`] impl, so a new command
/// is one line here rather than five places that each have to agree.
///
/// The variant names are a stored format: serde tags them in snake case into every
/// history row (see `tests/maps_history_format.rs`), so renaming one breaks undo of the
/// rows already written.
macro_rules! map_commands {
    ($(
        $variant:ident $(($out:ty))? => $($apply:ident)::+ $(with $actor:ident)?
    ),+ $(,)?) => {
        /// Every way a map can change. Variants prefixed `Restore` are compensating
        /// commands: they exist only as the inverse of a removal and are not routed by the
        /// API.
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "command", rename_all = "snake_case")]
        pub enum MapCommand {
            $($variant($variant)),+
        }

        $(
            impl From<$variant> for MapCommand {
                fn from(cmd: $variant) -> Self {
                    MapCommand::$variant(cmd)
                }
            }

            impl Command for $variant {
                type Output = map_commands!(@output $($out)?);
            }
        )+

        impl MapCommand {
            /// Every variant's name, for the test that keeps a stored sample of each.
            #[doc(hidden)]
            pub const VARIANTS: &[&str] = &[$(stringify!($variant)),+];

            pub(super) fn map_id(&self) -> i64 {
                match self {
                    $(MapCommand::$variant(c) => c.map_id),+
                }
            }

            async fn dispatch(self, tx: &mut Tx<'_>, actor: EventActor) -> Result<Effect> {
                match self {
                    $(MapCommand::$variant(c) => {
                        map_commands!(@apply [$($actor)?] $($apply)::+, tx, c, actor)
                    })+
                }
            }
        }
    };
    (@output) => { () };
    (@output $out:ty) => { $out };
    (@apply [] $apply:path, $tx:ident, $c:ident, $actor:ident) => { $apply($tx, $c).await };
    (@apply [actor] $apply:path, $tx:ident, $c:ident, $actor:ident) => {
        $apply($tx, $c, $actor).await
    };
}

map_commands! {
    AddSystem(MapSolarSystem) => solar_system::apply_add_system,
    AddGhostSystem(MapSolarSystem) => ghost::apply_add_ghost_system,
    ResolveGhostSystem(MapSolarSystem) => ghost::apply_resolve_ghost_system,
    RestoreGhostSystem => ghost::apply_restore_ghost_system,
    RemoveSystem => solar_system::apply_remove_system,
    RemoveSystems(u64) => solar_system::apply_remove_systems,
    RestoreSystems => restore::apply_restore_systems,
    RemoveRestored => restore::apply_remove_restored,
    Sequence => apply_sequence with actor,
    ClearMap(u64) => solar_system::apply_clear_map,
    MoveSystem => solar_system::apply_move_system,
    MoveSystems => solar_system::apply_move_systems,
    SetAlias => solar_system::apply_set_alias,
    SetStatus => solar_system::apply_set_status,
    SetOccupier => solar_system::apply_set_occupier,
    SetNotes => solar_system::apply_set_notes,
    SetHome => solar_system::apply_set_home,
    SetRally => solar_system::apply_set_rally,
    RestoreRallies => solar_system::apply_restore_rallies,
    SetPinned => solar_system::apply_set_pinned,
    AddConnection(MapConnection) => connection::apply_add_connection,
    SetConnectionStatus(MapConnection) => connection::apply_set_connection_status,
    RemoveConnection => connection::apply_remove_connection,
    CleanStaleConnections(u64) => connection::apply_clean_stale,
    AgeConnection => connection::apply_age_connection,
    ExpireConnections(u64) => connection::apply_expire_connections,
    TrackJump => tracking::apply_track_jump,
    AddEveScoutConnections(u64) => eve_scout::apply_add_eve_scout_connections,
    AddSignature(Signature) => signatures::apply_add_signature,
    UpdateSignature(Signature) => signatures::apply_update_signature,
    RemoveSignature(signatures::RemovedSignature) => signatures::apply_remove_signature,
    RemoveSignatures(signatures::BulkRemoveOutcome) => signatures::apply_remove_signatures,
    RestoreSignatures => signatures::apply_restore_signatures,
    PasteSignatures => signatures::apply_paste_signatures,
    LinkSignature(Signature) => signatures::apply_link_signature,
    UnlinkSignature(Signature) => signatures::apply_unlink_signature,
    AddConnectionJump(jumps::ConnectionJump) => jumps::apply_add_jump,
    UpdateConnectionJump(jumps::ConnectionJump) => jumps::apply_update_jump,
    RemoveConnectionJump(u64) => jumps::apply_remove_jump,
    RestoreConnectionJump => jumps::apply_restore_jump,
    AddWatchlistEntry(watchlist::WatchlistEntry) => watchlist::apply_add_entry,
    SetWatchlistPinned(watchlist::WatchlistEntry) => watchlist::apply_set_pinned,
    RemoveWatchlistEntry => watchlist::apply_remove_entry,
    RestoreWatchlistEntry => watchlist::apply_restore_entry,
    AddIgnoredSystem(ignored::IgnoredSystem) => ignored::apply_add,
    RemoveIgnoredSystem => ignored::apply_remove,
    ClearIgnoredSystems(u64) => ignored::apply_clear,
}

/// A command struct, and what running it hands back. Implemented by the table above.
pub trait Command: Into<MapCommand> {
    type Output: FromOutput;

    /// Unwrap what the command's `apply` produced.
    fn take(output: CommandOutput) -> Result<Self::Output> {
        Self::Output::from_output(output)
    }
}

/// Run a command as a character and hand back what it produced, typed.
pub async fn run<C: Command>(pool: &PgPool, actor: Actor, cmd: C) -> Result<C::Output> {
    C::take(execute(pool, actor, cmd.into()).await?)
}

/// As [`run`], for background tasks.
pub(super) async fn run_as<C: Command>(
    pool: &PgPool,
    actor: EventActor,
    cmd: C,
) -> Result<C::Output> {
    C::take(execute_as(pool, actor, cmd.into()).await?)
}

/// What a command hands back to its caller.
#[derive(Debug, Clone)]
pub enum CommandOutput {
    None,
    Count(u64),
    System(Box<MapSolarSystem>),
    Connection(Box<MapConnection>),
    Signature(Box<Signature>),
    Jump(Box<jumps::ConnectionJump>),
    Watchlist(Box<watchlist::WatchlistEntry>),
    Ignored(Box<ignored::IgnoredSystem>),
    Removal(Box<signatures::RemovedSignature>),
    BulkRemoval(Box<signatures::BulkRemoveOutcome>),
}

impl CommandOutput {
    /// The output as the type its command promised. For the places that apply a command
    /// inside another and need its result.
    pub(super) fn take<T: FromOutput>(self) -> Result<T> {
        T::from_output(self)
    }
}

/// A type a [`CommandOutput`] unwraps into. A mismatch is a wiring mistake between a
/// command and its `apply`, never something a caller can act on.
pub trait FromOutput: Sized {
    fn from_output(output: CommandOutput) -> Result<Self>;
}

fn wrong_output(output: CommandOutput) -> MapError {
    MapError::Validation(format!("unexpected command output: {output:?}"))
}

/// One impl per boxed variant.
macro_rules! from_output {
    ($($variant:ident => $ty:ty),+ $(,)?) => {$(
        impl FromOutput for $ty {
            fn from_output(output: CommandOutput) -> Result<Self> {
                match output {
                    CommandOutput::$variant(x) => Ok(*x),
                    other => Err(wrong_output(other)),
                }
            }
        }
    )+};
}

from_output! {
    System => MapSolarSystem,
    Connection => MapConnection,
    Signature => Signature,
    Jump => jumps::ConnectionJump,
    Watchlist => watchlist::WatchlistEntry,
    Ignored => ignored::IgnoredSystem,
    Removal => signatures::RemovedSignature,
    BulkRemoval => signatures::BulkRemoveOutcome,
}

impl FromOutput for u64 {
    fn from_output(output: CommandOutput) -> Result<Self> {
        match output {
            CommandOutput::Count(n) => Ok(n),
            other => Err(wrong_output(other)),
        }
    }
}

/// A command run for its effect alone. Whatever it produced is not wanted.
impl FromOutput for () {
    fn from_output(_: CommandOutput) -> Result<Self> {
        Ok(())
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
    pub events: Vec<crate::realtime::MapEvent>,
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

    pub(super) fn emit(mut self, event: crate::realtime::MapEvent) -> Self {
        self.events.push(event);
        self
    }

    pub(super) fn emit_all(
        mut self,
        events: impl IntoIterator<Item = crate::realtime::MapEvent>,
    ) -> Self {
        self.events.extend(events);
        self
    }
}

impl MapCommand {
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
        Box::pin(self.dispatch(tx, actor)).await.map(|effect| {
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
    let mut tx = MapTx::begin(pool, cmd.map_id(), actor, cmd.required_role()).await?;
    let effect = cmd.apply(&mut tx, actor).await?;
    let output = tx.settle(effect).await?;
    tx.commit().await?;
    Ok(output)
}

/// A write to one map, from the role check to the announcement. [`execute_as`] runs every
/// command through one; the writes that are not commands (settings, share links, imports,
/// observed transits) use it directly, so none of them hand-rolls the order: check the
/// role inside the transaction, so a grant revoked meanwhile cannot slip a write through,
/// and publish only after commit, so nobody refetches a change that then rolls back.
pub(super) struct MapTx {
    tx: Tx<'static>,
    map_id: i64,
    actor: EventActor,
    events: Vec<crate::realtime::MapEvent>,
}

impl MapTx {
    /// Open the transaction and check the actor holds `role` on the map in it. A background
    /// task has no role to check.
    pub(super) async fn begin(
        pool: &PgPool,
        map_id: i64,
        actor: EventActor,
        role: Role,
    ) -> Result<Self> {
        let mut tx = pool.begin().await?;
        if let EventActor::Character(character) = actor {
            require_role_tx(&mut tx, map_id, character.user_id, role).await?;
        }
        Ok(Self::adopt(tx, map_id, actor))
    }

    /// Carry on in a transaction that created the map itself. Its creator owns it, so
    /// there is nothing to check yet.
    pub(super) fn adopt(tx: Tx<'static>, map_id: i64, actor: EventActor) -> Self {
        Self {
            tx,
            map_id,
            actor,
            events: Vec::new(),
        }
    }

    /// Queue an event to publish once the write commits.
    pub(super) fn emit(&mut self, event: crate::realtime::MapEvent) {
        self.events.push(event);
    }

    /// Settle the ghosts the write left behind or now owes, so no write has to remember on
    /// its own. Their events are queued with the rest; the steps that would undo them are
    /// handed back for the caller to fold into its own undo, or to drop when it has none.
    pub(super) async fn reconcile_ghosts(&mut self) -> Result<Vec<MapCommand>> {
        let (undo, events) = ghost::reconcile(&mut self.tx, self.map_id).await?;
        self.events.extend(events);
        Ok(undo)
    }

    /// Write the history row for an effect and queue its events. Hands back its output.
    pub(super) async fn record(&mut self, effect: Effect) -> Result<CommandOutput> {
        history::record(&mut self.tx, self.map_id, self.actor, &effect).await?;
        self.events.extend(effect.events);
        Ok(effect.output)
    }

    /// Reconcile the ghosts and record the effect with them, so undoing the change undoes
    /// its ghosts too. What a command does after applying, and an import after loading.
    pub(super) async fn settle(&mut self, effect: Effect) -> Result<CommandOutput> {
        let ghosts = self.reconcile_ghosts().await?;
        let effect = ghost::with_undo_steps(self.map_id, ghosts, effect);
        self.record(effect).await
    }

    /// Commit, then announce.
    pub(super) async fn commit(self) -> Result<()> {
        self.tx.commit().await?;
        let hub = crate::realtime::hub();
        for event in self.events {
            hub.publish(event);
        }
        Ok(())
    }
}

impl std::ops::Deref for MapTx {
    type Target = Tx<'static>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl std::ops::DerefMut for MapTx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tx
    }
}
