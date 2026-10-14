//! "Tell me when we can reach somewhere with capitals."
//!
//! Fires when a k-space system is placed on the map within jump range of the target. This
//! is the one alert that is not about gate jumps: a capital does not use gates, so the
//! question is light years from a system you can actually jump *from*, which means k-space.
//! A wormhole placed next to the target is no use to a dreadnought.

use sqlx::PgPool;

use super::delivery::{Embed, Field, security_color};
use super::ships::{self, JumpShip};
use super::{Alert, Outgoing};
use crate::util::security::ccp_round_security;

/// A system with the coordinates jump range is measured from.
struct Located {
    name: String,
    security: f64,
    position: (f64, f64, f64),
    /// Wormhole space cannot be jumped from or to.
    is_wormhole: bool,
}

async fn locate(pool: &PgPool, id: i64) -> Option<Located> {
    let row = sqlx::query!(
        "select id, name, security_status, wormhole_class_id, pos_x, pos_y, pos_z
         from solar_systems where id = $1",
        id,
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    Some(Located {
        name: row.name,
        security: row.security_status,
        position: (row.pos_x?, row.pos_y?, row.pos_z?),
        // 7/8/9 are the k-space bands WormholeSystems assigns; anything else is a hole of some kind.
        is_wormhole: !matches!(row.wormhole_class_id, Some(7) | Some(8) | Some(9) | None),
    })
}

/// Evaluate a map's jump-range alerts after a system landed on it, and claim the ones that
/// fire.
pub async fn evaluate(pool: &PgPool, alerts: &[Alert], map_solar_system_id: i64) -> Vec<Outgoing> {
    // A ghost placement has no system to measure a jump range from.
    let Ok(Some(Some(placed_id))) = sqlx::query_scalar!(
        "select solar_system_id from map_solar_systems where id = $1",
        map_solar_system_id,
    )
    .fetch_optional(pool)
    .await
    else {
        return Vec::new();
    };
    let Some(exit) = locate(pool, placed_id).await else {
        return Vec::new();
    };
    if exit.is_wormhole {
        return Vec::new();
    }

    let mut out = Vec::new();
    for alert in alerts {
        let (Some(target_id), Some(ship), Some(jdc)) = (
            alert.target_solar_system_id,
            alert.ship_type,
            alert.jdc_level,
        ) else {
            continue;
        };
        let Some(target) = locate(pool, target_id).await else {
            continue;
        };
        let distance = ships::distance_ly(exit.position, target.position);
        if distance > ship.max_range_ly(jdc) {
            continue;
        }
        let key = format!("placement:{map_solar_system_id}");
        if !super::claim(pool, alert.id, &key).await {
            continue;
        }
        out.push(Outgoing {
            alert: alert.clone(),
            key,
            embed: build(&exit, &target, ship, jdc, distance),
        });
    }
    out
}

fn build(exit: &Located, target: &Located, ship: JumpShip, jdc: i32, distance: f64) -> Embed {
    let dotlan = format!(
        "https://evemaps.dotlan.net/range/{},{jdc}/{}",
        ship.dotlan_hull(),
        target.name.replace(' ', "_")
    );
    Embed {
        title: format!("New exit {distance:.2} ly from {}", target.name),
        url: Some(dotlan.clone()),
        description: Some(format!(
            "**{}** was just added, **{distance:.2} ly** from **{}**, within {} range.",
            exit.name,
            target.name,
            ship.label()
        )),
        color: security_color(exit.security),
        fields: vec![
            Field::new(
                "Exit",
                format!("{} ({:.1})", exit.name, ccp_round_security(exit.security)),
                true,
            ),
            Field::new("Distance", format!("{distance:.2} ly"), true),
            Field::new(
                "Ship",
                format!(
                    "{} (JDC {jdc}): {:.1} ly max",
                    ship.label(),
                    ship.max_range_ly(jdc)
                ),
                true,
            ),
            Field::new("Range map", format!("[Dotlan]({dotlan})"), false),
        ],
        thumbnail: None,
        footer: None,
        timestamp: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(name: &str, x: f64) -> Located {
        Located {
            name: name.into(),
            security: 0.5,
            position: (x * ships::METRES_PER_LIGHTYEAR, 0.0, 0.0),
            is_wormhole: false,
        }
    }

    #[test]
    fn the_message_states_the_distance_and_what_can_fly_it() {
        let embed = build(
            &at("Amamake", 0.0),
            &at("Rens", 4.0),
            JumpShip::Dreadnought,
            5,
            4.0,
        );
        assert_eq!(embed.title, "New exit 4.00 ly from Rens");
        assert!(embed.fields.iter().any(|f| f.value.contains("7.0 ly max")));
        assert!(embed.fields.iter().any(|f| f.value.contains("dotlan.net")));
    }

    /// Dotlan wants the system name with underscores, and the link is the whole point of
    /// the field.
    #[test]
    fn the_range_link_survives_a_two_word_system_name() {
        let embed = build(
            &at("Jita", 0.0),
            &at("New Caldari", 1.0),
            JumpShip::BlackOps,
            4,
            1.0,
        );
        assert!(embed.url.as_deref().unwrap().ends_with("New_Caldari"));
    }
}
