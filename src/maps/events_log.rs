//! The map's history tree: reading it, and moving the map through it.
//!
//! Rows are written by [`command::execute`](super::command::execute) inside the same
//! transaction as the change they describe, so the log can never drift from the state.
//! Steps form a tree (each points at the step that was current when it was applied) and the
//! map holds a cursor onto it, so undo and redo move rather than append: undoing and then
//! making a new change branches instead of destroying the old step, and the abandoned
//! branch stays reachable through [`goto`].
//!
//! Each step stores both directions (`inverse` undoes it, `forward` re-applies it), each
//! produced by applying the other. That is what keeps restored rows on their original ids
//! however often you walk back and forth.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::access::{require_role, require_role_tx};
use super::command::{EventActor, MapCommand, Tx};
use super::error::{MapError, Result};
use super::{Actor, Role};

/// How long an entry stays in the history (and so how far back undo can reach).
pub const RETENTION_DAYS: i32 = 7;
/// How many entries the history endpoint returns.
const HISTORY_LIMIT: i64 = 50;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapEventEntry {
    pub id: i64,
    pub map_id: i64,
    pub character_id: Option<i64>,
    /// Actor name, or `None` for background tasks.
    pub character_name: Option<String>,
    pub kind: String,
    pub label: String,
    pub entries_count: i32,
    /// The step this one happened on top of. `None` for a root, or once retention has
    /// dropped the ancestor it pointed at.
    pub parent_id: Option<i64>,
    /// Whether this row is a step in the tree. Audit-only rows (background writers) are
    /// shown in the history but cannot be jumped to.
    pub is_step: bool,
    /// Whether this step is currently in effect, i.e. on the path from a root to the head.
    pub applied: bool,
    pub created_at: DateTime<Utc>,
}

/// The history as the status bar needs it: the entries plus where the map is sitting.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapHistory {
    pub entries: Vec<MapEventEntry>,
    /// The step the map is on. `None` means every step has been undone.
    pub head_event_id: Option<i64>,
    /// The step undo would move to (`None` when already at a root and nothing is applied).
    pub undo_target: Option<i64>,
    /// The step redo would move to.
    pub redo_target: Option<i64>,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Write the journal row for an applied command, and advance the cursor onto it. Called
/// only from [`command::execute`](super::command::execute), inside its transaction. A
/// command with no inverse is recorded for the audit trail but never becomes a step:
/// leaving it as the head would strand undo behind something it cannot reverse.
pub(super) async fn record(
    tx: &mut Tx<'_>,
    map_id: i64,
    actor: EventActor,
    effect: &super::command::Effect,
) -> Result<()> {
    let is_step = matches!(actor, EventActor::Character(_)) && effect.inverse.is_some();
    let inverse = match &effect.inverse {
        Some(cmd) => Some(to_json(cmd)?),
        None => None,
    };
    // Audit rows get a parent too, so the history shows them where they happened. Nothing
    // ever descends from one, so they stay leaves.
    let parent_id = head_of(tx, map_id).await?;

    let id = sqlx::query_scalar!(
        "insert into map_events
             (map_id, character_id, kind, label, entries_count, inverse, parent_id, is_step)
         values ($1, $2, $3, $4, $5, $6, $7, $8)
         returning id",
        map_id,
        actor.character_id(),
        effect.kind,
        effect.label,
        effect.entries as i32,
        inverse,
        parent_id,
        is_step,
    )
    .fetch_one(&mut **tx)
    .await?;

    if is_step {
        set_head(tx, map_id, Some(id)).await?;
    }
    Ok(())
}

fn to_json(cmd: &MapCommand) -> Result<serde_json::Value> {
    serde_json::to_value(cmd)
        .map_err(|e| MapError::Validation(format!("could not serialize a history step: {e}")))
}

fn from_json(value: serde_json::Value) -> Result<MapCommand> {
    serde_json::from_value(value)
        .map_err(|e| MapError::Validation(format!("corrupt history step: {e}")))
}

async fn head_of(tx: &mut Tx<'_>, map_id: i64) -> Result<Option<i64>> {
    let head = sqlx::query_scalar!("select head_event_id from maps where id = $1", map_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(MapError::NotFound)?;
    Ok(head)
}

async fn set_head(tx: &mut Tx<'_>, map_id: i64, head: Option<i64>) -> Result<()> {
    sqlx::query!(
        "update maps set head_event_id = $1 where id = $2",
        head,
        map_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// A step and the two commands that move through it.
struct Step {
    id: i64,
    parent_id: Option<i64>,
    inverse: Option<serde_json::Value>,
    forward: Option<serde_json::Value>,
}

async fn fetch_step(tx: &mut Tx<'_>, map_id: i64, id: i64) -> Result<Step> {
    let row = sqlx::query!(
        "select id, parent_id, inverse, forward from map_events
         where id = $1 and map_id = $2 and is_step",
        id,
        map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;
    Ok(Step {
        id: row.id,
        parent_id: row.parent_id,
        inverse: row.inverse,
        forward: row.forward,
    })
}

/// The chain from `node` up to its root, nearest first. Empty when `node` is `None`.
///
/// With a `floor`, the walk stops below that id. A parent is always older than its child,
/// so ids only fall along the chain and nothing past the floor can turn up again: the
/// history listing uses this to walk only as far as the oldest entry it shows.
async fn ancestors<'e, E>(
    executor: E,
    map_id: i64,
    node: Option<i64>,
    floor: Option<i64>,
) -> Result<Vec<i64>>
where
    E: sqlx::PgExecutor<'e>,
{
    let Some(node) = node else {
        return Ok(Vec::new());
    };
    let ids = sqlx::query_scalar!(
        r#"with recursive chain as (
               select id, parent_id, 0 as depth
               from map_events where id = $1 and map_id = $2 and is_step
               union all
               select e.id, e.parent_id, chain.depth + 1
               from map_events e
               join chain on e.id = chain.parent_id
               where e.map_id = $2 and e.is_step
                 and ($3::bigint is null or e.id >= $3)
           )
           select id as "id!" from chain order by depth"#,
        node,
        map_id,
        floor,
    )
    .fetch_all(executor)
    .await?;
    Ok(ids)
}

/// The child of `node` that redo moves to: the most recently applied one. Undoing and then
/// making a new change therefore redoes into the *new* branch, and the abandoned branch is
/// reached deliberately through [`goto`] instead of by accident.
async fn newest_child<'e, E>(executor: E, map_id: i64, node: Option<i64>) -> Result<Option<i64>>
where
    E: sqlx::PgExecutor<'e>,
{
    let id = sqlx::query_scalar!(
        "select id from map_events
         where map_id = $1 and is_step and parent_id is not distinct from $2
         order by id desc limit 1",
        map_id,
        node,
    )
    .fetch_optional(executor)
    .await?;
    Ok(id)
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct GotoMapEvent {
    pub map_id: i64,
    /// The step to move the map onto. `null` rewinds to before the first step, which is a
    /// real destination rather than an omitted field, so it is nullable and not optional.
    #[serde(default)]
    pub event_id: Option<i64>,
}

/// Move the map to `target`, undoing back to the nearest common ancestor and then redoing
/// down to it. Undo, redo and jumping into an abandoned branch are all this one operation.
async fn goto_tx(
    tx: &mut Tx<'_>,
    actor: Actor,
    map_id: i64,
    target: Option<i64>,
    head: Option<i64>,
) -> Result<()> {
    if head == target {
        return Err(MapError::Conflict("the map is already there".into()));
    }
    if let Some(target) = target {
        // Fails when the step is not on this map, or is an audit row.
        fetch_step(tx, map_id, target).await?;
    }

    let from = ancestors(&mut **tx, map_id, head, None).await?;
    let to = ancestors(&mut **tx, map_id, target, None).await?;
    let common = from.iter().find(|id| to.contains(id)).copied();

    let rewind: Vec<i64> = from
        .iter()
        .take_while(|id| Some(**id) != common)
        .copied()
        .collect();
    for id in rewind {
        let step = fetch_step(tx, map_id, id).await?;
        let inverse = step
            .inverse
            .ok_or_else(|| MapError::Conflict("that change cannot be undone".into()))?;
        // Undoing produces exactly the command that re-applies the step.
        let produced = step_through(tx, actor, inverse).await?;
        sqlx::query!(
            "update map_events set forward = $1 where id = $2",
            produced,
            step.id,
        )
        .execute(&mut **tx)
        .await?;
    }

    let mut replay: Vec<i64> = to
        .iter()
        .take_while(|id| Some(**id) != common)
        .copied()
        .collect();
    replay.reverse();
    for id in replay {
        let step = fetch_step(tx, map_id, id).await?;
        let forward = step
            .forward
            .ok_or_else(|| MapError::Conflict("that change cannot be redone".into()))?;
        let produced = step_through(tx, actor, forward).await?;
        sqlx::query!(
            "update map_events set inverse = $1 where id = $2",
            produced,
            step.id,
        )
        .execute(&mut **tx)
        .await?;
    }

    set_head(tx, map_id, target).await
}

/// Apply one stored direction and hand back the command that reverses it.
async fn step_through(
    tx: &mut Tx<'_>,
    actor: Actor,
    command: serde_json::Value,
) -> Result<Option<serde_json::Value>> {
    let effect = from_json(command)?
        .apply(tx, EventActor::Character(actor))
        .await?;
    match &effect.inverse {
        Some(c) => Ok(Some(to_json(c)?)),
        None => Ok(None),
    }
}

/// Move the map onto an arbitrary step, which is how an abandoned branch is re-entered.
/// Member+.
pub async fn goto(pool: &PgPool, actor: Actor, cmd: GotoMapEvent) -> Result<()> {
    let mut tx = pool.begin().await?;
    require_role_tx(&mut tx, cmd.map_id, actor.user_id, Role::Member).await?;
    let head = head_of(&mut tx, cmd.map_id).await?;
    goto_tx(&mut tx, actor, cmd.map_id, cmd.event_id, head).await?;
    tx.commit().await?;
    super::hub().publish(super::events::MapEvent::HistoryChanged { map_id: cmd.map_id });
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapIdBody {
    pub map_id: i64,
}

/// Step back to the head's parent. Member+.
pub async fn undo(pool: &PgPool, actor: Actor, cmd: MapIdBody) -> Result<()> {
    let mut tx = pool.begin().await?;
    require_role_tx(&mut tx, cmd.map_id, actor.user_id, Role::Member).await?;
    let head = head_of(&mut tx, cmd.map_id).await?;
    let Some(head_id) = head else {
        return Err(MapError::Conflict("there is nothing to undo".into()));
    };
    let parent = fetch_step(&mut tx, cmd.map_id, head_id).await?.parent_id;
    goto_tx(&mut tx, actor, cmd.map_id, parent, head).await?;
    tx.commit().await?;
    super::hub().publish(super::events::MapEvent::HistoryChanged { map_id: cmd.map_id });
    Ok(())
}

/// Step forward onto the head's most recent child. Member+.
pub async fn redo(pool: &PgPool, actor: Actor, cmd: MapIdBody) -> Result<()> {
    let mut tx = pool.begin().await?;
    require_role_tx(&mut tx, cmd.map_id, actor.user_id, Role::Member).await?;
    let head = head_of(&mut tx, cmd.map_id).await?;
    let Some(child) = newest_child(&mut *tx, cmd.map_id, head).await? else {
        return Err(MapError::Conflict("there is nothing to redo".into()));
    };
    goto_tx(&mut tx, actor, cmd.map_id, Some(child), head).await?;
    tx.commit().await?;
    super::hub().publish(super::events::MapEvent::HistoryChanged { map_id: cmd.map_id });
    Ok(())
}

/// The map's history and where it currently sits, newest first. Viewer+.
pub async fn list_history(pool: &PgPool, actor: Actor, map_id: i64) -> Result<MapHistory> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;

    let head = sqlx::query_scalar!("select head_event_id from maps where id = $1", map_id)
        .fetch_optional(pool)
        .await?
        .ok_or(MapError::NotFound)?;

    let rows = sqlx::query!(
        r#"select e.id, e.map_id, e.character_id, c.name as "character_name?",
                  e.kind, e.label, e.entries_count, e.parent_id, e.is_step, e.created_at
           from map_events e
           left join characters c on c.id = e.character_id
           where e.map_id = $1
           order by e.id desc
           limit $2"#,
        map_id,
        HISTORY_LIMIT,
    )
    .fetch_all(pool)
    .await?;

    // Everything on the path from a root to the head is in effect; everything else is an
    // undone branch. Only the part of the path the page shows is walked: on a busy map the
    // whole chain is a week of steps, and only the newest few are on screen.
    let floor = rows.iter().map(|r| r.id).min();
    let applied: std::collections::HashSet<i64> = ancestors(pool, map_id, head, floor)
        .await?
        .into_iter()
        .collect();

    let entries: Vec<MapEventEntry> = rows
        .into_iter()
        .map(|r| MapEventEntry {
            applied: r.is_step && applied.contains(&r.id),
            id: r.id,
            map_id: r.map_id,
            character_id: r.character_id,
            character_name: r.character_name,
            kind: r.kind,
            label: r.label,
            entries_count: r.entries_count,
            parent_id: r.parent_id,
            is_step: r.is_step,
            created_at: r.created_at,
        })
        .collect();

    // Undo targets the head's parent, which is `None` at a root: that is still a real move
    // (it rewinds the first step), so `can_undo` follows the head, not the target.
    let undo_target =
        head.and_then(|h| entries.iter().find(|e| e.id == h).and_then(|e| e.parent_id));
    let redo_target = newest_child(pool, map_id, head).await?;

    Ok(MapHistory {
        entries,
        head_event_id: head,
        undo_target,
        redo_target,
        can_undo: head.is_some(),
        can_redo: redo_target.is_some(),
    })
}

/// Drop entries past the retention window. The head is never dropped, because the cursor
/// has to keep pointing at a real step. Old ancestors are fair game: `parent_id` nulls out,
/// the oldest survivor becomes a root, and undo stops at the retention boundary.
pub async fn purge(pool: &PgPool) -> Result<u64> {
    let deleted = sqlx::query!(
        "delete from map_events e
         where e.created_at < now() - make_interval(days => $1)
           and e.id is distinct from (select m.head_event_id from maps m where m.id = e.map_id)",
        RETENTION_DAYS,
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(deleted)
}

pub fn start_purge(pool: PgPool) {
    crate::background::spawn_every(
        "map event purge",
        std::time::Duration::from_secs(6 * 60 * 60),
        move || {
            let pool = pool.clone();
            async move { purge(&pool).await.map(drop) }
        },
    );
}
