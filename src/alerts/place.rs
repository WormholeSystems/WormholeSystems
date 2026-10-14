//! "Tell me when the chain gets near somewhere."
//!
//! Fires when a system is placed on the map: the chain just changed shape, so anything it
//! was too far from a moment ago might not be now. An alert with a starting point also
//! fires when a wormhole is mapped, since that edge can complete the route from there to
//! the target without anything being placed.
//!
//! The claim is per placement, so a system added, removed and added again is a new occasion
//! worth a message, while a hundred other edits are not. With a starting point the claim is
//! per route instead: however many placements and connections report the same way to the
//! target, it is said once, and a new way is a new occasion.

use sqlx::PgPool;

use super::delivery::{Embed, Field, security_color};
use super::killmail::chain_of;
use super::proximity::{self, Universe};
use super::{Alert, Outgoing};
use crate::util::security::ccp_round_security;

/// What just changed on the map, by `map_solar_systems` id.
#[derive(Debug, Clone, Copy)]
pub enum Occasion {
    Placed(i64),
    /// A wormhole now joins these two mapped systems.
    Connected(i64, i64),
}

impl Occasion {
    fn systems(self) -> Vec<i64> {
        match self {
            Occasion::Placed(id) => vec![id],
            Occasion::Connected(a, b) => vec![a, b],
        }
    }
}

/// Evaluate a map's proximity alerts after its shape changed, and claim the ones that fire.
pub async fn evaluate(
    pool: &PgPool,
    universe: &Universe,
    alerts: &[Alert],
    map_id: i64,
    occasion: Occasion,
) -> Vec<Outgoing> {
    let Some(chain) = chain_of(pool, map_id).await else {
        return Vec::new();
    };
    let changed_ids = occasion.systems();
    // What just changed, so the message can say what happened rather than only what is
    // now reachable.
    let changed = sqlx::query!(
        "select mss.id, ss.name from map_solar_systems mss
         join solar_systems ss on ss.id = mss.solar_system_id
         where mss.id = any($1)",
        &changed_ids,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let changed_names: Vec<&str> = changed.iter().map(|row| row.name.as_str()).collect();

    let mut out = Vec::new();
    for alert in alerts {
        let Some(target_id) = alert.target_solar_system_id else {
            continue;
        };
        let (found, key) = match (alert.origin_solar_system_id, occasion) {
            (Some(origin), _) => {
                let Some(found) = proximity::nearest(
                    universe,
                    &[origin],
                    &chain.edges,
                    &chain.ignored,
                    target_id,
                    alert.max_jumps,
                ) else {
                    continue;
                };
                // A fixed starting point measures the same pair on every change; only a
                // change that is part of the route may fire, so an in-range pair does not
                // re-alert for every unrelated system added later.
                let Ok(mapped) = mapped_along(pool, map_id, &found.route).await else {
                    continue;
                };
                if !mapped.iter().any(|id| changed_ids.contains(id)) {
                    continue;
                }
                (found, route_key(&mapped))
            }
            (None, Occasion::Placed(placed)) => {
                let Some(found) = proximity::nearest(
                    universe,
                    &chain.systems,
                    &chain.edges,
                    &chain.ignored,
                    target_id,
                    alert.max_jumps,
                ) else {
                    continue;
                };
                (found, format!("placement:{placed}"))
            }
            // Every mapped system is already zero jumps from the chain, so a wormhole
            // between two of them cannot bring the target any closer.
            (None, Occasion::Connected(..)) => continue,
        };
        if !super::claim(pool, alert.id, &key).await {
            continue;
        }
        let Ok(names) = names(pool, &found.route).await else {
            super::unclaim(pool, alert.id, &key).await;
            continue;
        };
        out.push(Outgoing {
            alert: alert.clone(),
            key,
            embed: build(&names, &found, &changed_names, map_id),
        });
    }
    out
}

/// The `map_solar_systems` ids of the route's systems that are on the map, in route order.
async fn mapped_along(pool: &PgPool, map_id: i64, route: &[i64]) -> sqlx::Result<Vec<i64>> {
    let rows = sqlx::query!(
        "select id, solar_system_id from map_solar_systems
         where map_id = $1 and solar_system_id = any($2)",
        map_id,
        route,
    )
    .fetch_all(pool)
    .await?;
    Ok(route
        .iter()
        .filter_map(|system| {
            rows.iter()
                .find(|row| row.solar_system_id == Some(*system))
                .map(|row| row.id)
        })
        .collect())
}

/// A route is the mapped systems it runs through: re-placing any of them, or finding a
/// different way, is a new occasion; a second event reporting the same way is not.
fn route_key(mapped: &[i64]) -> String {
    let ids: Vec<String> = mapped.iter().map(i64::to_string).collect();
    format!("route:{}", ids.join("-"))
}

/// One system's display data, in route order.
pub struct Named {
    pub id: i64,
    pub name: String,
    pub security: f64,
}

async fn names(pool: &PgPool, route: &[i64]) -> sqlx::Result<Vec<Named>> {
    let rows = sqlx::query!(
        "select id, name, security_status from solar_systems where id = any($1)",
        route,
    )
    .fetch_all(pool)
    .await?;
    // Back into route order: the query returns them however the index felt like.
    Ok(route
        .iter()
        .filter_map(|id| {
            rows.iter().find(|r| r.id == *id).map(|r| Named {
                id: r.id,
                name: r.name.clone(),
                security: r.security_status,
            })
        })
        .collect())
}

fn build(route: &[Named], found: &proximity::Proximity, changed: &[&str], map_id: i64) -> Embed {
    let target = route.last();
    let target_name = target.map(|s| s.name.as_str()).unwrap_or("Somewhere");
    let from = route
        .first()
        .map(|s| s.name.as_str())
        .unwrap_or("the chain");
    let jumps = found.jumps;
    let away = format!(
        "putting **{target_name}** {jumps} {} away",
        if jumps == 1 { "jump" } else { "jumps" }
    );

    let description = match changed {
        [added] if jumps == 0 => format!("**{added}** is **{target_name}**."),
        [added] => format!("**{added}** was just added, {away}."),
        [a, b] => format!("**{a}** and **{b}** were just connected, {away}."),
        _ => format!("The chain now reaches **{target_name}**."),
    };

    let mut fields = vec![
        Field::new(
            "Target",
            target
                .map(|s| format!("{} ({:.1})", s.name, ccp_round_security(s.security)))
                .unwrap_or_else(|| target_name.to_string()),
            true,
        ),
        Field::new("Gate jumps", jumps.to_string(), true),
        Field::new("From", from, true),
    ];
    if route.len() > 1 {
        let names: Vec<&str> = route.iter().map(|s| s.name.as_str()).collect();
        fields.push(Field::new("Route", names.join(" → "), false));
    }

    Embed {
        title: format!(
            "{target_name} is {jumps} {} out",
            if jumps == 1 { "jump" } else { "jumps" }
        ),
        url: Some(format!("/maps/{map_id}")),
        description: Some(description),
        color: security_color(target.map(|s| s.security).unwrap_or(-1.0)),
        fields,
        thumbnail: None,
        footer: None,
        timestamp: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(id: i64, name: &str, security: f64) -> Named {
        Named {
            id,
            name: name.into(),
            security,
        }
    }

    #[test]
    fn the_message_names_what_changed_and_the_way_there() {
        let route = vec![
            named(1, "J122515", -0.99),
            named(2, "Amarr", 1.0),
            named(3, "Ashab", 1.0),
        ];
        let found = proximity::Proximity {
            jumps: 2,
            from: 1,
            route: vec![1, 2, 3],
        };
        let embed = build(&route, &found, &["J122515"], 7);
        assert_eq!(embed.title, "Ashab is 2 jumps out");
        assert!(embed.description.as_deref().unwrap().contains("J122515"));
        assert!(
            embed
                .fields
                .iter()
                .any(|f| f.value == "J122515 → Amarr → Ashab")
        );
    }

    #[test]
    fn one_jump_reads_as_one_jump() {
        let route = vec![named(1, "Perimeter", 1.0), named(2, "Jita", 0.9)];
        let found = proximity::Proximity {
            jumps: 1,
            from: 1,
            route: vec![1, 2],
        };
        let embed = build(&route, &found, &["Perimeter"], 1);
        assert_eq!(embed.title, "Jita is 1 jump out");
        assert!(
            embed
                .description
                .as_deref()
                .unwrap()
                .contains("1 jump away")
        );
    }

    /// Placing the target itself is worth saying plainly rather than "0 jumps away".
    #[test]
    fn placing_the_target_says_so() {
        let route = vec![named(1, "Jita", 0.9)];
        let found = proximity::Proximity {
            jumps: 0,
            from: 1,
            route: vec![1],
        };
        let embed = build(&route, &found, &["Jita"], 1);
        assert_eq!(embed.description.as_deref(), Some("**Jita** is **Jita**."));
    }

    /// A connection names both ends, since neither of them is new on its own.
    #[test]
    fn a_connection_names_both_ends() {
        let route = vec![
            named(1, "J122515", -0.99),
            named(2, "J100001", -0.99),
            named(3, "Amarr", 1.0),
        ];
        let found = proximity::Proximity {
            jumps: 1,
            from: 1,
            route: vec![1, 2, 3],
        };
        let embed = build(&route, &found, &["J122515", "J100001"], 7);
        assert_eq!(
            embed.description.as_deref(),
            Some("**J122515** and **J100001** were just connected, putting **Amarr** 1 jump away.")
        );
    }

    #[test]
    fn a_route_is_named_by_the_mapped_systems_along_it() {
        assert_eq!(route_key(&[4, 9]), "route:4-9");
        assert_eq!(route_key(&[]), "route:");
    }
}
