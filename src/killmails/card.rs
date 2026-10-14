//! The killmail card's read model: recent kills in the chain, as rows to render.

use sqlx::PgPool;

/// How far back the killmails card looks, and therefore how far back names are resolved.
pub const CARD_WINDOW_DAYS: i32 = 7;
/// How many rows the card asks for. A recent feed, not an archive.
pub const CARD_LIMIT: i64 = 60;

/// One entity as a killmail row names it: a portrait, and something to call them.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct KillParty {
    #[ts(optional)]
    pub character_id: Option<i64>,
    #[ts(optional)]
    pub character_name: Option<String>,
    #[ts(optional)]
    pub corporation_id: Option<i64>,
    #[ts(optional)]
    pub corporation_ticker: Option<String>,
    /// Spelled out for the tooltip; the row has room for a ticker at most.
    #[ts(optional)]
    pub corporation_name: Option<String>,
    #[ts(optional)]
    pub alliance_id: Option<i64>,
    #[ts(optional)]
    pub alliance_ticker: Option<String>,
    #[ts(optional)]
    pub alliance_name: Option<String>,
    #[ts(optional)]
    pub ship_type_id: Option<i64>,
    #[ts(optional)]
    pub ship_name: Option<String>,
}

/// A killmail as the card shows it: what a row renders, not the raw payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapKillmail {
    pub id: i64,
    pub solar_system_id: i64,
    pub system_name: String,
    pub region: String,
    pub security_status: f64,
    #[ts(optional)]
    pub wormhole_class_id: Option<i64>,
    pub time: chrono::DateTime<chrono::Utc>,
    pub victim: KillParty,
    pub final_blow: KillParty,
    #[ts(optional)]
    pub total_value: Option<f64>,
    pub attacker_count: i32,
    pub is_npc: bool,
    pub is_solo: bool,
}

/// Which part of the chain a map's killmail card is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillmailFilter {
    All,
    Wormhole,
    KnownSpace,
    /// One solar system. Still only while it is on the map, like every other filter.
    System(i64),
}

impl KillmailFilter {
    /// The stored preference, as the query builder wants it, with the solar system the
    /// viewer has selected. Two names for one idea, because the setting is a column and
    /// this is what the query does about it; the match is total, so a scope that grew a
    /// variant would stop compiling here. `None` when the card follows the selection and
    /// nothing is selected: that shows nothing, not everything.
    pub fn new(scope: crate::maps::KillmailScope, selected: Option<i64>) -> Option<Self> {
        Some(match scope {
            crate::maps::KillmailScope::All => KillmailFilter::All,
            crate::maps::KillmailScope::Jspace => KillmailFilter::Wormhole,
            crate::maps::KillmailScope::Kspace => KillmailFilter::KnownSpace,
            crate::maps::KillmailScope::SelectedSystem => KillmailFilter::System(selected?),
        })
    }
}

/// Recent kills in the systems currently on a map, newest first. Bounded by time as well
/// as count: a row cap alone shows a quiet chain kills from a year ago as though new.
pub async fn list_for_map(
    pool: &PgPool,
    map_id: i64,
    filter: KillmailFilter,
    limit: i64,
) -> sqlx::Result<Vec<MapKillmail>> {
    let wormholes_only = matches!(filter, KillmailFilter::Wormhole);
    let kspace_only = matches!(filter, KillmailFilter::KnownSpace);
    let only_system = match filter {
        KillmailFilter::System(id) => Some(id),
        _ => None,
    };
    let rows = sqlx::query!(
        r#"select k.id, k.solar_system_id, k.time, k.total_value,
                  coalesce(k.attacker_count, 0) as "attacker_count!",
                  k.is_npc, k.is_solo,
                  ss.name as system_name, r.name as region, ss.security_status,
                  ws.wormhole_class_id as "wormhole_class_id?",
                  k.victim_character_id, vc.name as "victim_character_name?",
                  k.victim_corporation_id, vco.ticker as "victim_corporation_ticker?",
                  vco.name as "victim_corporation_name?",
                  k.victim_alliance_id, va.ticker as "victim_alliance_ticker?",
                  va.name as "victim_alliance_name?",
                  k.victim_ship_type_id, vt.name as "victim_ship_name?",
                  k.final_blow_character_id, fc.name as "final_blow_character_name?",
                  k.final_blow_corporation_id, fco.ticker as "final_blow_corporation_ticker?",
                  fco.name as "final_blow_corporation_name?",
                  k.final_blow_alliance_id, fa.ticker as "final_blow_alliance_ticker?",
                  fa.name as "final_blow_alliance_name?",
                  k.final_blow_ship_type_id, ft.name as "final_blow_ship_name?"
           from killmails k
           join map_solar_systems mss
               on mss.map_id = $1 and mss.solar_system_id = k.solar_system_id
           join solar_systems ss on ss.id = k.solar_system_id
           join constellations c on c.id = ss.constellation_id
           join regions r on r.id = c.region_id
           left join wormhole_systems ws on ws.solar_system_id = ss.id
           left join characters vc on vc.id = k.victim_character_id
           left join corporations vco on vco.id = k.victim_corporation_id
           left join alliances va on va.id = k.victim_alliance_id
           left join types vt on vt.id = k.victim_ship_type_id
           left join characters fc on fc.id = k.final_blow_character_id
           left join corporations fco on fco.id = k.final_blow_corporation_id
           left join alliances fa on fa.id = k.final_blow_alliance_id
           left join types ft on ft.id = k.final_blow_ship_type_id
           where k.time >= now() - make_interval(days => $2)
             -- Rows from before the ingest kept any detail would render as blank lines.
             and k.victim_ship_type_id is not null
             and (not $3 or ws.solar_system_id is not null)
             and (not $4 or ws.solar_system_id is null)
             -- The join above already holds this to the map, so a system that is not on
             -- it matches nothing.
             and ($5::bigint is null or k.solar_system_id = $5)
           order by k.time desc
           limit $6"#,
        map_id,
        CARD_WINDOW_DAYS,
        wormholes_only,
        kspace_only,
        only_system,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MapKillmail {
            id: r.id,
            solar_system_id: r.solar_system_id,
            system_name: r.system_name,
            region: r.region,
            security_status: r.security_status,
            wormhole_class_id: r.wormhole_class_id.map(i64::from),
            time: r.time,
            victim: KillParty {
                character_id: r.victim_character_id,
                character_name: r.victim_character_name,
                corporation_id: r.victim_corporation_id,
                corporation_ticker: r.victim_corporation_ticker,
                corporation_name: r.victim_corporation_name,
                alliance_id: r.victim_alliance_id,
                alliance_ticker: r.victim_alliance_ticker,
                alliance_name: r.victim_alliance_name,
                ship_type_id: r.victim_ship_type_id,
                ship_name: r.victim_ship_name,
            },
            final_blow: KillParty {
                character_id: r.final_blow_character_id,
                character_name: r.final_blow_character_name,
                corporation_id: r.final_blow_corporation_id,
                corporation_ticker: r.final_blow_corporation_ticker,
                corporation_name: r.final_blow_corporation_name,
                alliance_id: r.final_blow_alliance_id,
                alliance_ticker: r.final_blow_alliance_ticker,
                alliance_name: r.final_blow_alliance_name,
                ship_type_id: r.final_blow_ship_type_id,
                ship_name: r.final_blow_ship_name,
            },
            total_value: r.total_value,
            attacker_count: r.attacker_count,
            is_npc: r.is_npc,
            is_solo: r.is_solo,
        })
        .collect())
}
