//! Writes to `map_solar_systems`: a system (or a ghost) placed on a map.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::maps::MapSolarSystem;
use crate::maps::command::Tx;
use crate::maps::error::Result;

/// A whole placement as it stands, keys included: what a removal snapshots and an undo puts
/// back under the same id. Serialized into the undo history, so the field names are a
/// stored format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementRow {
    pub id: i64,
    pub solar_system_id: Option<i64>,
    pub position_x: f64,
    pub position_y: f64,
    pub alias: Option<String>,
    pub is_home: bool,
    /// Only read from snapshots taken before rally points carried a timestamp; restoring
    /// one of those makes the system the newest rally point.
    ///
    /// Can go once no stored history predates migration 0030 on any install. The purge
    /// drops events after `history::RETENTION_DAYS`, but never a map's head, so a map
    /// nobody has touched since keeps an old snapshot indefinitely: check
    /// `select count(*) from map_events e, _sqlx_migrations m where m.version = 30 and
    /// e.created_at < m.installed_on` is zero first, or rewrite those rows.
    #[serde(default)]
    pub is_rally: bool,
    /// When the system became a rally point, kept so a restored one is as old as it was.
    #[serde(default)]
    pub rallied_at: Option<DateTime<Utc>>,
    pub is_pinned: bool,
    /// Set on a ghost, which is not allowed back on the map without naming its scan and
    /// the system it hung off. `None` on a system somebody placed.
    #[serde(default)]
    pub raised_by_signature_id: Option<i64>,
    #[serde(default)]
    pub hangs_off_id: Option<i64>,
    /// Comes back with the signatures, so the panel still says how old they are.
    #[serde(default)]
    pub signatures_pasted_at: Option<DateTime<Utc>>,
    /// When it was first placed, so an undo does not make it new. Absent from snapshots
    /// taken before it was kept, which come back as placed now.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
}

/// A placement that is not on the map yet. Home, rally and the paste time are earned on the
/// map, never brought along, so a new row starts without them.
#[derive(Debug, Clone, Copy)]
pub struct NewPlacement<'a> {
    pub solar_system_id: Option<i64>,
    pub position_x: f64,
    pub position_y: f64,
    pub alias: Option<&'a str>,
    pub is_pinned: bool,
    pub raised_by_signature_id: Option<i64>,
    pub hangs_off_id: Option<i64>,
}

impl<'a> NewPlacement<'a> {
    /// A system somebody placed.
    pub fn system(solar_system_id: i64, x: f64, y: f64, alias: Option<&'a str>) -> Self {
        NewPlacement {
            solar_system_id: Some(solar_system_id),
            position_x: x,
            position_y: y,
            alias,
            is_pinned: false,
            raised_by_signature_id: None,
            hangs_off_id: None,
        }
    }

    /// The far side of a scanned hole, naming the scan and the placement it hangs off.
    pub fn ghost(
        signature_pk: i64,
        hangs_off_id: i64,
        x: f64,
        y: f64,
        alias: Option<&'a str>,
    ) -> Self {
        NewPlacement {
            solar_system_id: None,
            position_x: x,
            position_y: y,
            alias,
            is_pinned: false,
            raised_by_signature_id: Some(signature_pk),
            hangs_off_id: Some(hangs_off_id),
        }
    }
}

/// Place a new row and hand it back.
pub(crate) async fn insert(
    tx: &mut Tx<'_>,
    map_id: i64,
    row: &NewPlacement<'_>,
) -> Result<MapSolarSystem> {
    Ok(sqlx::query_as!(
        MapSolarSystem,
        "insert into map_solar_systems
             (map_id, solar_system_id, position_x, position_y, alias, is_pinned,
              raised_by_signature_id, hangs_off_id)
         values ($1, $2, $3, $4, $5, $6, $7, $8)
         returning id, map_id, solar_system_id, position_x, position_y, alias, created_at",
        map_id,
        row.solar_system_id,
        row.position_x,
        row.position_y,
        row.alias,
        row.is_pinned,
        row.raised_by_signature_id,
        row.hangs_off_id,
    )
    .fetch_one(&mut **tx)
    .await?)
}

/// Place a row, or when the system is on the map already, move it and take this one's
/// alias and pin. An import's way in: the file is the newer word on where things are.
pub(crate) async fn upsert_by_system(
    tx: &mut Tx<'_>,
    map_id: i64,
    row: &NewPlacement<'_>,
) -> Result<()> {
    sqlx::query!(
        "insert into map_solar_systems
             (map_id, solar_system_id, position_x, position_y, alias, is_pinned,
              raised_by_signature_id, hangs_off_id)
         values ($1, $2, $3, $4, $5, $6, $7, $8)
         on conflict (map_id, solar_system_id)
         do update set position_x = excluded.position_x, position_y = excluded.position_y,
             alias = excluded.alias, is_pinned = excluded.is_pinned",
        map_id,
        row.solar_system_id,
        row.position_x,
        row.position_y,
        row.alias,
        row.is_pinned,
        row.raised_by_signature_id,
        row.hangs_off_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// These placements as they stand, for a snapshot.
pub(crate) async fn snapshot(
    tx: &mut Tx<'_>,
    map_id: i64,
    ids: &[i64],
) -> Result<Vec<PlacementRow>> {
    Ok(sqlx::query_as!(
        PlacementRow,
        r#"select id, solar_system_id, position_x, position_y, alias, is_home, is_rally,
                  rallied_at, is_pinned, raised_by_signature_id, hangs_off_id,
                  signatures_pasted_at, created_at as "created_at?"
           from map_solar_systems where map_id = $1 and id = any($2)"#,
        map_id,
        ids,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// Put a snapshotted placement back under its own id, so the connections and signatures
/// restored with it still line up. A row that is somehow back already is left alone.
/// Put a placement back under its own id. Returns whether it had to be: a snapshot can
/// carry a placement that never went, and the caller undoing this restore must take back
/// only what it actually put.
pub(crate) async fn reinsert(tx: &mut Tx<'_>, map_id: i64, row: &PlacementRow) -> Result<bool> {
    let inserted = sqlx::query!(
        "insert into map_solar_systems
             (id, map_id, solar_system_id, position_x, position_y, alias,
              is_home, rallied_at, is_pinned, raised_by_signature_id, hangs_off_id,
              signatures_pasted_at, created_at)
         overriding system value
         values ($1, $2, $3, $4, $5, $6, $7,
                 coalesce($8, case when $9 then now() end), $10, $11, $12, $13,
                 coalesce($14, now()))
         on conflict (id) do nothing",
        row.id,
        map_id,
        row.solar_system_id,
        row.position_x,
        row.position_y,
        row.alias.as_deref(),
        row.is_home,
        row.rallied_at,
        row.is_rally,
        row.is_pinned,
        row.raised_by_signature_id,
        row.hangs_off_id,
        row.signatures_pasted_at,
        row.created_at,
    )
    .execute(&mut **tx)
    .await?;
    Ok(inserted.rows_affected() > 0)
}

/// Move a placement, unless it is pinned. The server holds the drag lock rather than
/// trusting the client.
pub(crate) async fn move_unpinned(
    tx: &mut Tx<'_>,
    map_id: i64,
    id: i64,
    x: f64,
    y: f64,
) -> Result<()> {
    sqlx::query!(
        "update map_solar_systems set position_x = $1, position_y = $2
         where id = $3 and map_id = $4 and not is_pinned",
        x,
        y,
        id,
        map_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Take these placements off the map, whatever they are. Their connections and signatures
/// go with them by the foreign keys.
pub(crate) async fn delete(tx: &mut Tx<'_>, map_id: i64, ids: &[i64]) -> Result<()> {
    sqlx::query!(
        "delete from map_solar_systems where map_id = $1 and id = any($2)",
        map_id,
        ids,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Take these placements off the map, passing over home and pinned ones. The guard is
/// repeated on the delete itself, so the rule travels with the query whatever the caller
/// checked beforehand. Returns how many went.
pub(crate) async fn delete_unprotected(tx: &mut Tx<'_>, map_id: i64, ids: &[i64]) -> Result<u64> {
    Ok(sqlx::query!(
        "delete from map_solar_systems
         where map_id = $1 and id = any($2) and not is_home and not is_pinned",
        map_id,
        ids,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected())
}

/// Which of these placements are orphans: unpinned, unmarked, and without a connection
/// left. Asked by a sweep that snapshots them before they go.
pub(crate) async fn orphans(tx: &mut Tx<'_>, map_id: i64, ids: &[i64]) -> Result<Vec<i64>> {
    Ok(sqlx::query_scalar!(
        r#"select id as "id!" from map_solar_systems mss
           where mss.map_id = $1 and mss.id = any($2)
             and not mss.is_pinned and not mss.is_home and not mss.is_rally
             and not exists (
                 select 1 from map_connections c
                 where c.map_id = $1 and (c.from_system = mss.id or c.to_system = mss.id)
             )"#,
        map_id,
        ids,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// Take a placement off the map if it is an orphan by the same rule as [`orphans`]. Returns
/// whether it went.
pub(crate) async fn delete_if_orphan(tx: &mut Tx<'_>, map_id: i64, id: i64) -> Result<bool> {
    let deleted = sqlx::query!(
        "delete from map_solar_systems mss
         where mss.id = $1 and mss.map_id = $2
           and not mss.is_pinned and not mss.is_home and not mss.is_rally
           and not exists (
               select 1 from map_connections c
               where c.map_id = $2 and (c.from_system = $1 or c.to_system = $1)
           )",
        id,
        map_id,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected();
    Ok(deleted > 0)
}

/// Turn a ghost into the system it was found to be. A real system names no scan and hangs
/// off nothing, so both go in the same write.
pub(crate) async fn assign_system(
    tx: &mut Tx<'_>,
    map_id: i64,
    id: i64,
    solar_system_id: i64,
) -> Result<MapSolarSystem> {
    Ok(sqlx::query_as!(
        MapSolarSystem,
        "update map_solar_systems
         set solar_system_id = $1, raised_by_signature_id = null, hangs_off_id = null
         where id = $2 and map_id = $3
         returning id, map_id, solar_system_id, position_x, position_y, alias, created_at",
        solar_system_id,
        id,
        map_id,
    )
    .fetch_one(&mut **tx)
    .await?)
}

/// Turn a placement back into a ghost of this scan. A ghost may not be home, a rally point
/// or pinned, so those marks are dropped in the same write; a new mark a ghost cannot carry
/// has to be cleared here too.
pub(crate) async fn back_to_ghost(
    tx: &mut Tx<'_>,
    map_id: i64,
    id: i64,
    signature_pk: i64,
    hangs_off_id: i64,
) -> Result<MapSolarSystem> {
    Ok(sqlx::query_as!(
        MapSolarSystem,
        "update map_solar_systems
         set solar_system_id = null, is_home = false, rallied_at = null, is_pinned = false,
             raised_by_signature_id = $3, hangs_off_id = $4
         where id = $1 and map_id = $2
         returning id, map_id, solar_system_id, position_x, position_y, alias, created_at",
        id,
        map_id,
        signature_pk,
        hangs_off_id,
    )
    .fetch_one(&mut **tx)
    .await?)
}
