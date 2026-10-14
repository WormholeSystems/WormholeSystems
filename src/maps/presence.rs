//! Presence: which tracked pilots are online in the map's systems right now, for the node
//! pilot rows and the pilots card.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::Role;
use super::access::require_role;
use super::error::Result;

/// An online, tracked character on the map (presence), for the node pilot rows and the
/// pilots card.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapCharacter {
    pub character_id: i64,
    pub name: String,
    pub corporation_ticker: String,
    pub solar_system_id: Option<i64>,
    pub ship_type_id: Option<i64>,
    /// The name the pilot gave the hull, which is not the hull's type.
    pub ship_name: Option<String>,
    pub ship_type: Option<String>,
    /// The hull's inventory group, so the client can tell a covert-ops scanner from a
    /// combat ship without shipping a table of hull names.
    pub ship_group_id: Option<i64>,
    /// Docked in a station or structure: on the map, but not on grid.
    pub is_docked: bool,
    /// One of the viewer's own characters, so their alts can be told apart at a glance.
    pub is_mine: bool,
}

/// Online characters of users who opted into tracking on this map, holding the location
/// scope, whose user has member-or-better access. Member+ may look: a viewer does not get
/// to see where anyone is.
pub async fn characters_on(pool: &PgPool, map_id: i64, viewer: i64) -> Result<Vec<MapCharacter>> {
    require_role(pool, map_id, viewer, Role::Member).await?;
    let rows = sqlx::query!(
        r#"select c.id as character_id, c.name, co.ticker as corporation_ticker,
                  s.solar_system_id, s.ship_type_id, s.ship_name, t.name as "ship_type?",
                  t.group_id as "ship_group_id?", s.is_docked as "is_docked!",
                  (c.user_id = $2) as "is_mine!"
           from characters c
           join users u on u.id = c.user_id
           join map_user_settings mus
               on mus.map_id = $1 and mus.user_id = u.id and mus.tracking_allowed
           join character_status s on s.character_id = c.id and s.online
           join corporations co on co.id = c.corporation_id
           left join types t on t.id = s.ship_type_id
           where exists (
                     select 1 from esi_tokens et
                     join esi_token_scopes ets on ets.token_id = et.id
                     join esi_scopes sc on sc.id = ets.scope_id
                     where et.character_id = c.id
                       and sc.name = 'esi-location.read_location.v1'
                 )
             and exists (
                     select 1 from map_access_live ma
                     where ma.map_id = $1
                       and ma.role <> 'viewer'
                       and ma.subject_id in (
                           select id from characters where user_id = u.id
                           union all
                           select corporation_id from characters where user_id = u.id
                           union all
                           select alliance_id from characters
                           where user_id = u.id and alliance_id is not null
                       )
                 )
           order by c.name"#,
        map_id,
        viewer,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| MapCharacter {
            character_id: r.character_id,
            name: r.name,
            corporation_ticker: r.corporation_ticker,
            solar_system_id: r.solar_system_id,
            ship_type_id: r.ship_type_id,
            ship_name: r.ship_name,
            ship_type: r.ship_type,
            ship_group_id: r.ship_group_id,
            is_docked: r.is_docked,
            is_mine: r.is_mine,
        })
        .collect())
}
