//! Raidable skyhooks: what CCP is currently advertising as stealable.
//!
//! ESI publishes the whole raidable set and drops entries once their window has passed, so
//! the local table is a mirror rather than a log. The sync is therefore a full replace, run
//! in one transaction: a partial failure that deleted rows it could not re-insert would
//! empty the card for everyone.

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::time::{MissedTickBehavior, interval};

use crate::esi::EsiClient;
use crate::esi::skyhooks::RaidableSkyhook;
use crate::maps::Sovereignty;
use crate::maps::view::sovereignty_of;
use crate::server_status::ServerWatch;

/// Windows are two hours long and ESI advertises them ahead of time, so five minutes never
/// misses one. `SKYHOOK_POLL_SECS` tightens it for the e2e stack.
fn interval_secs() -> Duration {
    let secs = std::env::var("SKYHOOK_POLL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5 * 60);
    Duration::from_secs(secs.max(1))
}

/// Which reagent a skyhook's planet yields, the only reason anyone sorts them. Derived from
/// the type name rather than a list of type ids, so a new variant classifies itself.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum PlanetKind {
    Lava,
    Ice,
    Other,
}

impl PlanetKind {
    pub fn from_type_name(name: &str) -> PlanetKind {
        let lower = name.to_ascii_lowercase();
        if lower.contains("lava") {
            PlanetKind::Lava
        } else if lower.contains("ice") {
            PlanetKind::Ice
        } else {
            PlanetKind::Other
        }
    }
}

/// A raidable skyhook, enriched with everything a row displays.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Skyhook {
    pub planet_id: i64,
    /// e.g. `4-EP12 VI`, as it reads in the overview.
    pub planet_name: String,
    pub planet_kind: PlanetKind,
    pub solar_system_id: i64,
    pub system_name: String,
    pub region: String,
    /// Carried so a row can offer the system menu without a second round trip to resolve
    /// the system it already names.
    pub region_id: i64,
    pub constellation_id: i64,
    pub security_status: f64,
    /// Who holds the system. Skyhooks only exist in sovereign nullsec, so this is the
    /// alliance whose toes you are stepping on.
    #[ts(optional)]
    pub sovereignty: Option<Sovereignty>,
    pub vulnerable_from: DateTime<Utc>,
    pub vulnerable_until: DateTime<Utc>,
}

/// Spawn the sync loop. Returns immediately; the loop runs for the process lifetime.
pub fn start(pool: PgPool, esi: EsiClient, server: ServerWatch) {
    tokio::spawn(async move {
        let mut ticker = interval(interval_secs());
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            // Nobody is raiding while the server is down, and the call would fail anyway.
            if !server.should_poll() {
                continue;
            }
            if let Err(err) = sync_once(&pool, &esi).await {
                tracing::error!("skyhook sync failed: {err}");
            }
        }
    });
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("esi: {0}")]
    Esi(#[from] crate::esi::EsiError),
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
}

/// Fetch and store in one go. Returns how many rows are now in the table.
pub async fn sync_once(pool: &PgPool, esi: &EsiClient) -> Result<usize, SyncError> {
    let fetched = esi.raidable_skyhooks().await?;
    let stored = store(pool, &fetched).await?;
    if stored < fetched.len() {
        tracing::warn!(
            "skyhook sync: {} of {} planets are not in the SDE, so they were skipped",
            fetched.len() - stored,
            fetched.len()
        );
    }
    Ok(stored)
}

/// Replace the table with `fetched`. The delete and the insert are one transaction, so a
/// failure cannot leave the table emptied of rows it was about to put back.
pub async fn store(pool: &PgPool, fetched: &[RaidableSkyhook]) -> sqlx::Result<usize> {
    let planet_ids: Vec<i64> = fetched.iter().map(|s| s.planet_id).collect();
    let system_ids: Vec<i64> = fetched.iter().map(|s| s.solar_system_id).collect();
    let from: Vec<DateTime<Utc>> = fetched
        .iter()
        .map(|s| s.theft_vulnerability.start)
        .collect();
    let until: Vec<DateTime<Utc>> = fetched.iter().map(|s| s.theft_vulnerability.end).collect();

    let mut tx = pool.begin().await?;
    // Anything ESI no longer lists has stopped being raidable.
    sqlx::query!(
        "delete from raidable_skyhooks where planet_id <> all($1)",
        &planet_ids,
    )
    .execute(&mut *tx)
    .await?;

    // Planets the SDE has never heard of are skipped rather than failing the sync: a content
    // patch should not take the card down until the next seed.
    let stored = sqlx::query_scalar!(
        "with incoming as (
             select * from unnest($1::bigint[], $2::bigint[], $3::timestamptz[], $4::timestamptz[])
                 as t(planet_id, solar_system_id, vulnerable_from, vulnerable_until)
         )
         insert into raidable_skyhooks
             (planet_id, solar_system_id, vulnerable_from, vulnerable_until, updated_at)
         select i.planet_id, i.solar_system_id, i.vulnerable_from, i.vulnerable_until, now()
         from incoming i
         join planets p on p.id = i.planet_id
         on conflict (planet_id) do update set
             solar_system_id = excluded.solar_system_id,
             vulnerable_from = excluded.vulnerable_from,
             vulnerable_until = excluded.vulnerable_until,
             updated_at = now()
         returning planet_id",
        &planet_ids,
        &system_ids,
        &from,
        &until,
    )
    .fetch_all(&mut *tx)
    .await?
    .len();
    tx.commit().await?;
    Ok(stored)
}

/// The in-game name of a planet: its system, then its position in Roman numerals.
///
/// The SDE ships a `name` for only 43 of the 68,000-odd planets, so the label is built the
/// way the client builds it, from the celestial index every planet has.
fn planet_label(system: &str, celestial_index: i32, stored: Option<&str>) -> String {
    if let Some(name) = stored.map(str::trim).filter(|n| !n.is_empty()) {
        return name.to_string();
    }
    format!("{system} {}", roman(celestial_index))
}

fn roman(mut value: i32) -> String {
    // Planets top out in the low tens, but the table runs far enough that a future
    // content patch cannot produce a blank.
    const PARTS: [(i32, &str); 9] = [
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    if value < 1 {
        return String::new();
    }
    let mut out = String::new();
    for (amount, numeral) in PARTS {
        while value >= amount {
            out.push_str(numeral);
            value -= amount;
        }
    }
    out
}

/// Everything still worth showing: open now, or opening later. Ordered by when the window
/// opens, which is the only ordering that is meaningful without knowing where you are.
pub async fn list(pool: &PgPool) -> sqlx::Result<Vec<Skyhook>> {
    let rows = sqlx::query!(
        r#"select s.planet_id, s.solar_system_id, s.vulnerable_from, s.vulnerable_until,
                  p.name as planet_name, p.celestial_index, t.name as planet_type,
                  ss.name as system_name, r.name as region, r.id as region_id,
                  ss.constellation_id, ss.security_status,
                  case
                      when sov.alliance_id is not null then 'alliance'
                      when sov.corporation_id is not null then 'corporation'
                      when sov.faction_id is not null then 'faction'
                  end as "sov_kind?",
                  coalesce(sov.alliance_id, sov.corporation_id, sov.faction_id) as "sov_id?",
                  coalesce(al.name, co.name, f.name) as "sov_name?",
                  coalesce(al.ticker, co.ticker) as "sov_ticker?"
           from raidable_skyhooks s
           join planets p on p.id = s.planet_id
           join types t on t.id = p.type_id
           join solar_systems ss on ss.id = s.solar_system_id
           join constellations c on c.id = ss.constellation_id
           join regions r on r.id = c.region_id
           left join system_sovereignty sov on sov.solar_system_id = s.solar_system_id
           left join alliances al on al.id = sov.alliance_id
           left join corporations co on co.id = sov.corporation_id
           left join factions f on f.id = sov.faction_id
           where s.vulnerable_until > now()
           order by s.vulnerable_from"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| Skyhook {
            planet_id: r.planet_id,
            planet_name: planet_label(&r.system_name, r.celestial_index, r.planet_name.as_deref()),
            planet_kind: PlanetKind::from_type_name(&r.planet_type),
            solar_system_id: r.solar_system_id,
            system_name: r.system_name,
            region: r.region,
            region_id: r.region_id,
            constellation_id: r.constellation_id,
            security_status: r.security_status,
            sovereignty: sovereignty_of(r.sov_kind.as_deref(), r.sov_id, r.sov_name, r.sov_ticker),
            vulnerable_from: r.vulnerable_from,
            vulnerable_until: r.vulnerable_until,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_planet_is_named_by_its_position_when_the_sde_has_no_name() {
        assert_eq!(planet_label("M2GJ-X", 6, None), "M2GJ-X VI");
        assert_eq!(planet_label("Jita", 4, None), "Jita IV");
        assert_eq!(planet_label("X-7OMU", 14, None), "X-7OMU XIV");
        // A name the SDE does supply wins, since it is what shows in the overview.
        assert_eq!(planet_label("Jita", 4, Some("Jita IV")), "Jita IV");
        // Blank counts as absent rather than producing a bare system name.
        assert_eq!(planet_label("Jita", 4, Some("  ")), "Jita IV");
    }

    #[test]
    fn planets_are_classified_by_what_they_yield() {
        assert_eq!(
            PlanetKind::from_type_name("Planet (Lava)"),
            PlanetKind::Lava
        );
        assert_eq!(PlanetKind::from_type_name("Planet (Ice)"), PlanetKind::Ice);
        assert_eq!(
            PlanetKind::from_type_name("Planet (Barren)"),
            PlanetKind::Other
        );
        // Shattered and other variants classify themselves, which a list of type ids
        // would not: legacy had to be taught each new one by hand.
        assert_eq!(
            PlanetKind::from_type_name("Planet (Shattered Lava)"),
            PlanetKind::Lava
        );
    }
}
