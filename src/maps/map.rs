//! The map entity: create, update, delete, list, and read the graph.
//!
//! Each mutating action takes a dedicated command struct (the HTTP request body); `actor`
//! stays a separate argument, injected from the session, never read from the payload.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use sqlx::PgPool;

use crate::allowlist::AffiliationAllowlist;

use super::access::{owns_character, require_role};
use super::command::{CommandOutput, Effect, EventActor, MapTx};
use super::error::{MapError, Result};

use super::view::{MapSystemView, sovereignty_of};
use super::{Actor, AliasScheme, MapLayout, MapView, Role, SubjectType};
use crate::realtime::MapEvent;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Map {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub naming: MapNaming,
    /// Whether pasting a wormhole signature puts its far side on the map as a ghost.
    /// Map-wide: a ghost is a node everyone on the chain sees.
    pub ghost_unlinked_wormholes: bool,
    /// How the chain is placed: `manual` (dragged into shape) or `tree` (drawn from the
    /// connections). Map-wide, so everyone on a chain reads the same picture.
    pub layout: MapLayout,
    /// Whether a viewer may pick their own placement instead of the map's.
    pub allow_layout_override: bool,
    /// Whether anyone with the address may watch it, account or no account.
    pub is_public: bool,
    /// The secret that lets somebody watch a private map. Only ever sent to a manager:
    /// handing it to a viewer would be handing them the map to pass on.
    #[ts(optional)]
    pub share_token: Option<String>,
}

/// How a map names its chain. Map-wide rather than per-user: an alias is written on the
/// map for everyone to read.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapNaming {
    /// `numeric` (`1`, `11`, `12`) or `alphabetical` (`A`, `AB`, with `H/L/N/P` reserved
    /// for k-space exits).
    pub alias_scheme: AliasScheme,
    /// The alias that sits outside the chain, e.g. `HOME`.
    pub ignored_alias: String,
    pub bookmark_wormhole: String,
    pub bookmark_kspace: String,
    pub bookmark_return: String,
}

impl MapNaming {
    pub fn validate(&self) -> Result<()> {
        // A blank format renders as an empty bookmark name, which the game silently
        // replaces with a generic one and the whole convention is lost.
        for (label, format) in [
            ("wormhole", &self.bookmark_wormhole),
            ("k-space", &self.bookmark_kspace),
            ("return", &self.bookmark_return),
        ] {
            if format.trim().is_empty() {
                return Err(MapError::Validation(format!(
                    "the {label} bookmark format must not be blank"
                )));
            }
        }
        Ok(())
    }
}

/// A row selected with the map columns, assembled into the nested shape the client sees.
macro_rules! map_from_row {
    ($row:expr) => {{
        let row = $row;
        Map {
            id: row.id,
            name: row.name,
            description: row.description,
            image_url: row.image_url,
            created_at: row.created_at,
            ghost_unlinked_wormholes: row.ghost_unlinked_wormholes,
            layout: row.layout,
            allow_layout_override: row.allow_layout_override,
            is_public: row.is_public,
            share_token: row.share_token,
            naming: MapNaming {
                alias_scheme: row.alias_scheme,
                ignored_alias: row.ignored_alias,
                bookmark_wormhole: row.bookmark_wormhole,
                bookmark_kspace: row.bookmark_kspace,
                bookmark_return: row.bookmark_return,
            },
        }
    }};
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CreateMap {
    pub name: String,
    pub description: Option<String>,
}

impl CreateMap {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(MapError::Validation("name must not be blank".into()));
        }
        Ok(())
    }
}

/// Whether the acting character may create a map. Anyone may, unless the instance names
/// who can in `creators`.
///
/// Checked against the active character rather than every character on the account: an
/// account holding a director and an alt in an NPC corporation should not let the alt
/// create maps, and the active character is the one the map is created as (its owner).
pub async fn may_create_map(
    pool: &PgPool,
    actor: Actor,
    creators: &AffiliationAllowlist,
) -> Result<bool> {
    // Nothing configured means nothing changes, without a query for it.
    if !creators.is_enforced() {
        return Ok(true);
    }
    let Some(character) = sqlx::query!(
        "select corporation_id, alliance_id from characters where id = $1 and user_id = $2",
        actor.character_id,
        actor.user_id,
    )
    .fetch_optional(pool)
    .await?
    else {
        return Ok(false);
    };
    Ok(creators.allows(
        actor.character_id,
        character.corporation_id,
        character.alliance_id,
    ))
}

/// Create a map and grant ownership to the acting character. Any authenticated user may
/// create a map, unless the instance limits it to `creators`.
pub async fn create_map(
    pool: &PgPool,
    actor: Actor,
    creators: &AffiliationAllowlist,
    cmd: CreateMap,
) -> Result<Map> {
    cmd.validate()?;
    if !owns_character(pool, actor.user_id, actor.character_id).await? {
        return Err(MapError::Forbidden);
    }
    if !may_create_map(pool, actor, creators).await? {
        return Err(MapError::Forbidden);
    }

    let mut tx = pool.begin().await?;
    let map = insert_map(
        &mut tx,
        cmd.name.trim(),
        cmd.description.as_deref(),
        actor.character_id,
    )
    .await?;
    tx.commit().await?;
    Ok(map)
}

/// The inserts a new map is made of: the row, the owner grant, and the seeded watchlist.
/// Shared with the import path, which creates the map inside its own transaction.
pub(super) async fn insert_map(
    tx: &mut sqlx::PgConnection,
    name: &str,
    description: Option<&str>,
    owner_character_id: i64,
) -> Result<Map> {
    let map = map_from_row!(
        sqlx::query!(
            r#"insert into maps (name, description) values ($1, $2)
             returning id, name, description, image_url, created_at, alias_scheme, ignored_alias,
                 ghost_unlinked_wormholes, layout, allow_layout_override, is_public, share_token,
                 bookmark_wormhole, bookmark_kspace, bookmark_return"#,
            name,
            description,
        )
        .fetch_one(&mut *tx)
        .await?
    );
    sqlx::query!(
        "insert into map_access (map_id, subject_type, subject_id, role)
         values ($1, $2, $3, $4)",
        map.id,
        SubjectType::Character,
        owner_character_id,
        Role::Owner,
    )
    .execute(&mut *tx)
    .await?;
    // Resolved by name rather than by id so an unseeded database quietly starts with an
    // empty watchlist instead of failing the whole creation on a foreign key.
    sqlx::query!(
        "insert into map_watchlist (map_id, solar_system_id, is_pinned)
         select $1, id, true from solar_systems where name = any($2)",
        map.id,
        &TRADE_HUBS.map(String::from)[..],
    )
    .execute(&mut *tx)
    .await?;
    Ok(map)
}

/// The map row as the client sees it, off whatever executor the caller is inside.
pub(super) async fn fetch_map(exec: impl sqlx::PgExecutor<'_>, map_id: i64) -> Result<Map> {
    Ok(map_from_row!(
        sqlx::query!(
            r#"select id, name, description, image_url, created_at, alias_scheme, ignored_alias,
                 bookmark_wormhole, bookmark_kspace, bookmark_return, ghost_unlinked_wormholes,
                 layout, allow_layout_override, is_public, share_token
             from maps where id = $1"#,
            map_id,
        )
        .fetch_optional(exec)
        .await?
        .ok_or(MapError::NotFound)?
    ))
}

/// Seeded onto every new map's watchlist, pinned, as legacy's `CreateMapAction` does.
const TRADE_HUBS: [&str; 5] = ["Jita", "Amarr", "Dodixie", "Rens", "Hek"];

/// A partial update of a map's fields. `None` leaves a field unchanged; `Some(None)`
/// explicitly clears a nullable field.
#[derive(Debug, Default, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UpdateMap {
    pub map_id: i64,
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "super::double_option")]
    #[ts(optional)]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "super::double_option")]
    #[ts(optional)]
    pub image_url: Option<Option<String>>,
    /// All-or-nothing: the naming block is edited as one form.
    #[serde(default)]
    #[ts(optional)]
    pub naming: Option<MapNaming>,
    #[serde(default)]
    #[ts(optional)]
    pub ghost_unlinked_wormholes: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub layout: Option<MapLayout>,
    #[serde(default)]
    #[ts(optional)]
    pub allow_layout_override: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub is_public: Option<bool>,
}

impl UpdateMap {
    pub fn validate(&self) -> Result<()> {
        if let Some(name) = &self.name
            && name.trim().is_empty()
        {
            return Err(MapError::Validation("name must not be blank".into()));
        }
        if let Some(naming) = &self.naming {
            naming.validate()?;
        }
        Ok(())
    }
}

/// Rename / re-describe / re-icon a map. Owner only.
pub async fn update_map(pool: &PgPool, actor: Actor, cmd: UpdateMap) -> Result<Map> {
    cmd.validate()?;
    let mut tx = MapTx::begin(pool, cmd.map_id, EventActor::Character(actor), Role::Owner).await?;
    let current = map_from_row!(
        sqlx::query!(
            r#"select id, name, description, image_url, created_at, alias_scheme, ignored_alias,
                 bookmark_wormhole, bookmark_kspace, bookmark_return, ghost_unlinked_wormholes,
                 layout, allow_layout_override, is_public, share_token
             from maps where id = $1"#,
            cmd.map_id,
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(MapError::NotFound)?
    );

    let name = match cmd.name {
        Some(name) => name.trim().to_string(),
        None => current.name,
    };
    let description = cmd.description.unwrap_or(current.description);
    let image_url = cmd.image_url.unwrap_or(current.image_url);
    let naming = cmd.naming.unwrap_or(current.naming);
    let ghost_unlinked_wormholes = cmd
        .ghost_unlinked_wormholes
        .unwrap_or(current.ghost_unlinked_wormholes);
    let layout = cmd.layout.unwrap_or(current.layout);
    let allow_layout_override = cmd
        .allow_layout_override
        .unwrap_or(current.allow_layout_override);
    let is_public = cmd.is_public.unwrap_or(current.is_public);

    let map = map_from_row!(
        sqlx::query!(
            r#"update maps set name = $1, description = $2, image_url = $3,
                    alias_scheme = $5, ignored_alias = $6, bookmark_wormhole = $7,
                    bookmark_kspace = $8, bookmark_return = $9, ghost_unlinked_wormholes = $10,
                    layout = $11, allow_layout_override = $12, is_public = $13
             where id = $4
             returning id, name, description, image_url, created_at, alias_scheme, ignored_alias,
                 bookmark_wormhole, bookmark_kspace, bookmark_return, ghost_unlinked_wormholes,
                 layout, allow_layout_override, is_public, share_token"#,
            name,
            description,
            image_url,
            cmd.map_id,
            naming.alias_scheme,
            naming.ignored_alias,
            naming.bookmark_wormhole,
            naming.bookmark_kspace,
            naming.bookmark_return,
            ghost_unlinked_wormholes,
            layout,
            allow_layout_override,
            is_public,
        )
        .fetch_one(&mut **tx)
        .await?
    );
    tx.emit(MapEvent::MapUpdated { map_id: cmd.map_id });
    // Drawing unmapped holes is a map setting, so turning it on draws the holes already
    // scanned and turning it off takes those nodes away, rather than waiting for the next
    // scan to notice. The nodes are a change to the chain like any other, so they go in the
    // history, but only for the record: undo moves through commands, and a settings change
    // is not one, so stepping back over it would bring the nodes back against the setting.
    if !tx.reconcile_ghosts().await?.is_empty() {
        let label = if map.ghost_unlinked_wormholes {
            "drew the unmapped holes"
        } else {
            "took the unmapped holes off the map"
        };
        tx.record(Effect::new("map.ghosts", label, CommandOutput::None))
            .await?;
    }
    tx.commit().await?;
    Ok(map)
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct DeleteMap {
    pub map_id: i64,
}

/// Delete a map. Owner only. The database cascades placements, details, connections,
/// signatures, and access grants.
pub async fn delete_map(pool: &PgPool, actor: Actor, cmd: DeleteMap) -> Result<()> {
    require_role(pool, cmd.map_id, actor.user_id, Role::Owner).await?;
    sqlx::query!("delete from maps where id = $1", cmd.map_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Every map the user can access, paired with their effective role on it.
pub async fn list_maps(pool: &PgPool, user_id: i64) -> Result<Vec<(Map, Role)>> {
    let rows = sqlx::query!(
        r#"select m.id, m.name, m.description, m.image_url, m.created_at, m.alias_scheme,
                  m.ignored_alias, m.bookmark_wormhole, m.bookmark_kspace, m.bookmark_return,
                  m.ghost_unlinked_wormholes, m.layout, m.allow_layout_override,
                  m.is_public, m.share_token,
                  -- The view cannot promise not-null the way the table does.
                  ma.role as "role!: Role"
           from maps m
           join map_access_live ma on ma.map_id = m.id
           where ma.subject_id in (
                 select id from characters where user_id = $1
                 union all
                 select corporation_id from characters where user_id = $1
                 union all
                 select alliance_id from characters where user_id = $1 and alliance_id is not null
           )
           order by m.created_at"#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    // A map can match more than once (e.g. via a character and its corp); keep the
    // highest role per map.
    let mut out: Vec<(Map, Role)> = Vec::new();
    for r in rows {
        let role = r.role;
        let map = map_from_row!(r);
        match out.iter_mut().find(|(m, _)| m.id == map.id) {
            Some((_, existing)) => *existing = (*existing).max(role),
            None => out.push((map, role)),
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct GetMap {
    pub map_id: i64,
}

/// Read a map's graph: the map, its placed systems, and its connections. Viewer+. Live
/// pilot locations are not included (that's the member-gated tracking path).
pub async fn get_map(pool: &PgPool, actor: Actor, cmd: GetMap) -> Result<MapView> {
    let role = require_role(pool, cmd.map_id, actor.user_id, Role::Viewer).await?;
    read_map(
        pool,
        super::access::Reader {
            actor: Some(actor),
            role,
        },
        cmd,
    )
    .await
}

/// The map as a resolved reader sees it. Shared with the guest paths, where there is no
/// user behind the request at all.
pub async fn read_map(
    pool: &PgPool,
    reader: super::access::Reader,
    cmd: GetMap,
) -> Result<MapView> {
    let role = reader.role;
    // The share token is a key to the map, so only the people who can mint one ever see it.
    let hide_token = role < Role::Manager;

    let map = map_from_row!(
        sqlx::query!(
            r#"select id, name, description, image_url, created_at, alias_scheme, ignored_alias,
                 bookmark_wormhole, bookmark_kspace, bookmark_return, ghost_unlinked_wormholes,
                 layout, allow_layout_override, is_public, share_token
             from maps where id = $1"#,
            cmd.map_id,
        )
        .fetch_optional(pool)
        .await?
        .ok_or(MapError::NotFound)?
    );

    let rows = sqlx::query!(
        r#"select
               mss.id, mss.map_id, mss.solar_system_id, mss.position_x, mss.position_y,
               mss.alias, mss.is_home, mss.is_rally, mss.is_pinned, mss.signatures_pasted_at,
               coalesce(d.status, 'unknown') as "status!: super::SystemStatus",
               d.occupying_group,
               ss.name as "name?", ss.security_status as "security_status?",
               ss.wormhole_class_id,
               r.name as "region?", ss.region_id as "region_id?",
               ss.constellation_id as "constellation_id?", c.name as "constellation?",
               ws.effect_name,
               coalesce(ws.is_shattered, false) as "is_shattered!",
               ws.threat_level as "threat_level?: super::ThreatLevel",
               sov.kind as "sov_kind?", sov.entity_id as "sov_id?",
               sov.name as "sov_name?", sov.ticker as "sov_ticker?"
           from map_solar_systems mss
           -- Left joins throughout: a ghost placement has no system to join to.
           left join solar_systems ss on ss.id = mss.solar_system_id
           left join regions r on r.id = ss.region_id
           left join constellations c on c.id = ss.constellation_id
           left join map_solar_system_details d
               on d.map_id = mss.map_id and d.solar_system_id = mss.solar_system_id
           left join wormhole_systems ws on ws.solar_system_id = ss.id
           left join system_sovereignty_resolved sov on sov.solar_system_id = ss.id
           where mss.map_id = $1
           order by mss.id"#,
        cmd.map_id,
    )
    .fetch_all(pool)
    .await?;

    let system_ids: Vec<i64> = rows.iter().filter_map(|r| r.solar_system_id).collect();
    let mut statics_by_system = super::view::statics_for(pool, &system_ids).await?;

    let systems = rows
        .into_iter()
        .map(|row| {
            let sovereignty = sovereignty_of(
                row.sov_kind.as_deref(),
                row.sov_id,
                row.sov_name,
                row.sov_ticker,
            );
            // The reference columns come off left joins, so they are all optional to sqlx.
            // They arrive together or not at all: the placement's system id is a foreign
            // key, and every column below is `not null` on the row it reaches.
            let Some(solar_system_id) = row.solar_system_id else {
                return MapSystemView::Ghost {
                    id: row.id,
                    map_id: row.map_id,
                    position_x: row.position_x,
                    position_y: row.position_y,
                    alias: row.alias,
                    is_home: row.is_home,
                    is_rally: row.is_rally,
                    is_pinned: row.is_pinned,
                    status: row.status,
                };
            };
            MapSystemView::System {
                id: row.id,
                map_id: row.map_id,
                solar_system_id,
                position_x: row.position_x,
                position_y: row.position_y,
                alias: row.alias,
                is_home: row.is_home,
                is_rally: row.is_rally,
                is_pinned: row.is_pinned,
                signatures_pasted_at: row.signatures_pasted_at,
                status: row.status,
                occupying_group: row.occupying_group,
                name: row.name.unwrap_or_default(),
                security_status: row.security_status.unwrap_or_default(),
                wormhole_class_id: row.wormhole_class_id,
                region: row.region.unwrap_or_default(),
                region_id: row.region_id.unwrap_or_default(),
                constellation_id: row.constellation_id.unwrap_or_default(),
                constellation: row.constellation.unwrap_or_default(),
                effect_name: row.effect_name,
                is_shattered: row.is_shattered,
                threat_level: row.threat_level,
                statics: statics_by_system
                    .remove(&solar_system_id)
                    .unwrap_or_default(),
                sovereignty,
            }
        })
        .collect();

    let connections = super::connection::connections_with_stats(pool, cmd.map_id, None).await?;

    // Whether the acting character is covered, a different question from whether the user
    // is. A guest has no character, and nothing to warn about.
    let character_has_access = match reader.actor {
        Some(actor) => {
            sqlx::query_scalar!(
                r#"select exists(
                   select 1 from map_access_live a
                   join characters c on c.id = $2
                   where a.map_id = $1
                     and a.subject_id in (c.id, c.corporation_id, c.alliance_id)
               ) as "has!""#,
                cmd.map_id,
                actor.character_id,
            )
            .fetch_one(pool)
            .await?
        }
        None => true,
    };

    let map = if hide_token {
        Map {
            share_token: None,
            ..map
        }
    } else {
        map
    };

    Ok(MapView {
        map,
        role,
        character_has_access,
        systems,
        connections,
    })
}

/// Mint a share link for the map, replacing any link already out there. Manager+. One link
/// at a time, so regenerating is how you lock out whoever holds the old one.
pub async fn rotate_share_token(pool: &PgPool, actor: Actor, map_id: i64) -> Result<String> {
    let mut tx = MapTx::begin(pool, map_id, EventActor::Character(actor), Role::Manager).await?;
    let token = uuid::Uuid::new_v4().simple().to_string();
    sqlx::query!(
        "update maps set share_token = $2 where id = $1",
        map_id,
        token,
    )
    .execute(&mut **tx)
    .await?;
    tx.emit(MapEvent::MapUpdated { map_id });
    tx.commit().await?;
    Ok(token)
}

/// Withdraw the share link. Whoever has it is out; the map itself is untouched. Manager+.
pub async fn revoke_share_token(pool: &PgPool, actor: Actor, map_id: i64) -> Result<()> {
    let mut tx = MapTx::begin(pool, map_id, EventActor::Character(actor), Role::Manager).await?;
    sqlx::query!("update maps set share_token = null where id = $1", map_id)
        .execute(&mut **tx)
        .await?;
    tx.emit(MapEvent::MapUpdated { map_id });
    tx.commit().await
}

/// A map in the user's list, with their role on it.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapEntry {
    pub id: i64,
    pub name: String,
    #[ts(optional)]
    pub description: Option<String>,
    pub role: String,
    pub system_count: i64,
    pub connection_count: i64,
    /// How many people can see it, counting every grant however it was made.
    pub member_count: i64,
    /// Tracked pilots currently online in one of its systems.
    pub pilots_online: i64,
    /// Whether this user keeps it in the top bar.
    pub is_pinned: bool,
    /// When the chain last changed. `None` for a map nobody has touched yet.
    #[ts(optional)]
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    /// Hidden from this user's list. Per-user, so archiving does not touch anyone else.
    pub is_archived: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// The signed-in user's maps, each with the counts its card shows.
pub async fn list_entries(pool: &PgPool, user_id: i64) -> Result<Vec<MapEntry>> {
    let maps = list_maps(pool, user_id).await?;
    let ids: Vec<i64> = maps.iter().map(|(m, _)| m.id).collect();

    // The counts in one pass rather than a query per card. Each is a scalar subquery so a
    // map with nothing on it still comes back as a zero rather than falling out of a join.
    let stats = sqlx::query!(
        r#"select m.id,
                  (select count(*) from map_solar_systems s where s.map_id = m.id)
                      as "systems!",
                  (select count(*) from map_connections c where c.map_id = m.id)
                      as "connections!",
                  (select count(*) from map_access a where a.map_id = m.id) as "members!",
                  coalesce((select mus.is_archived from map_user_settings mus
                             where mus.map_id = m.id and mus.user_id = $2), false)
                      as "archived!",
                  coalesce((select mus.is_pinned from map_user_settings mus
                             where mus.map_id = m.id and mus.user_id = $2), false)
                      as "pinned!",
                  (select max(e.created_at) from map_events e where e.map_id = m.id)
                      as "last_activity?",
                  (select count(*)
                     from map_user_settings mus
                     join characters c on c.user_id = mus.user_id
                     join character_status cs on cs.character_id = c.id and cs.online
                     join map_solar_systems ms
                          on ms.map_id = m.id and ms.solar_system_id = cs.solar_system_id
                    where mus.map_id = m.id and mus.tracking_allowed) as "pilots!"
           from maps m
           where m.id = any($1)"#,
        &ids,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    let by_id: std::collections::HashMap<i64, _> = stats.into_iter().map(|r| (r.id, r)).collect();
    Ok(maps
        .into_iter()
        .map(|(m, role)| {
            let s = by_id.get(&m.id);
            MapEntry {
                id: m.id,
                name: m.name,
                description: m.description,
                role: role.as_str().to_string(),
                system_count: s.map_or(0, |s| s.systems),
                connection_count: s.map_or(0, |s| s.connections),
                member_count: s.map_or(0, |s| s.members),
                pilots_online: s.map_or(0, |s| s.pilots),
                last_activity: s.and_then(|s| s.last_activity),
                is_archived: s.is_some_and(|s| s.archived),
                is_pinned: s.is_some_and(|s| s.pinned),
                created_at: m.created_at,
            }
        })
        .collect())
}

/// The map a share link leads to, for whoever holds it, signed in or not. A token that
/// matches nothing, a withdrawn one and a never-shared map all answer `NotFound`, so a
/// guesser learns nothing from the reply.
pub async fn read_shared_map(pool: &PgPool, token: &str, actor: Option<Actor>) -> Result<MapView> {
    if token.is_empty() {
        return Err(MapError::NotFound);
    }
    let map_id = sqlx::query_scalar!("select id from maps where share_token = $1", token)
        .fetch_optional(pool)
        .await?
        .ok_or(MapError::NotFound)?;
    let reader = super::access::reader_for(pool, map_id, actor, Some(token)).await?;
    read_map(pool, reader, GetMap { map_id }).await
}
