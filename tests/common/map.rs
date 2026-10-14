//! Putting things on a map, for the tests that need a chain to work on rather than the
//! placing itself.

use sqlx::PgPool;
use wormholesystems::maps::connection::{AddConnection, add_connection};
use wormholesystems::maps::solar_system::{AddSystem, add_system};
use wormholesystems::maps::{Actor, ConnectionType, MapConnection};

/// Place `system` at the origin; returns its `map_solar_systems` id.
pub async fn place(pool: &PgPool, actor: Actor, map_id: i64, system: i64) -> i64 {
    place_at(pool, actor, map_id, system, 0.0).await
}

/// Place `system` `x` along the top row, for the tests that care where it sits.
pub async fn place_at(pool: &PgPool, actor: Actor, map_id: i64, system: i64, x: f64) -> i64 {
    add_system(
        pool,
        actor,
        AddSystem {
            map_id,
            solar_system_id: system,
            x,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// A wormhole between two placements, nothing known about it yet.
pub async fn connect(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    from: i64,
    to: i64,
) -> MapConnection {
    connect_kind(pool, actor, map_id, from, to, ConnectionType::Wormhole).await
}

/// An edge of the given kind between two placements.
pub async fn connect_kind(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    from: i64,
    to: i64,
    kind: ConnectionType,
) -> MapConnection {
    add_connection(
        pool,
        actor,
        AddConnection {
            map_id,
            from_system: from,
            to_system: to,
            kind,
            size: None,
        },
    )
    .await
    .unwrap()
}
