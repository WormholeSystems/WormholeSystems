//! Storing ESI's sovereignty map: holders written, unclaimed systems cleared, claims we
//! cannot place skipped, and a holder that did not change left untouched.

mod common;

use chrono::{DateTime, Utc};
use common::{SYS_A, SYS_B, SYS_C, seed_universe};
use sqlx::PgPool;
use wormholesystems::esi::SovereigntySystem;
use wormholesystems::sovereignty::store;

const ALLIANCE: i64 = 99000001;
const CORP: i64 = 98000001;

fn systems(json: serde_json::Value) -> Vec<SovereigntySystem> {
    serde_json::from_value(json).unwrap()
}

async fn holder(pool: &PgPool, system: i64) -> Option<(Option<i64>, DateTime<Utc>)> {
    sqlx::query_as(
        "select alliance_id, updated_at from system_sovereignty where solar_system_id = $1",
    )
    .bind(system)
    .fetch_optional(pool)
    .await
    .unwrap()
}

#[sqlx::test]
async fn the_map_is_stored_and_unchanged_rows_are_left_alone(pool: PgPool) {
    seed_universe(&pool).await;
    sqlx::query("insert into alliances (id, name, ticker) values ($1, 'Holders', 'HOLD')")
        .bind(ALLIANCE)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("insert into corporations (id, name, ticker) values ($1, 'Holding', 'HLD')")
        .bind(CORP)
        .execute(&pool)
        .await
        .unwrap();
    // C was held last time and is unclaimed now.
    sqlx::query("insert into system_sovereignty (solar_system_id, alliance_id) values ($1, $2)")
        .bind(SYS_C)
        .bind(ALLIANCE)
        .execute(&pool)
        .await
        .unwrap();

    let map = systems(serde_json::json!([
        {"solar_system_id": SYS_A, "claim": {"alliance": {
            "alliance_id": ALLIANCE, "corporation_id": CORP,
            "claimed_since": "2026-01-01T00:00:00Z", "is_capital_system": false}}},
        // An alliance we could not resolve, and a system we do not know.
        {"solar_system_id": SYS_B, "claim": {"alliance": {
            "alliance_id": 99000002, "corporation_id": CORP}}},
        {"solar_system_id": 30009999, "claim": {"alliance": {
            "alliance_id": ALLIANCE, "corporation_id": CORP}}},
        {"solar_system_id": SYS_C, "claim": {"unclaimed": true}},
    ]));

    assert_eq!(store(&pool, &map).await.unwrap(), 2);
    let (alliance, first_write) = holder(&pool, SYS_A).await.unwrap();
    assert_eq!(alliance, Some(ALLIANCE));
    assert!(holder(&pool, SYS_B).await.is_none());
    assert!(holder(&pool, SYS_C).await.is_none());

    // The same map again: nothing to rewrite, so the row keeps its stamp.
    store(&pool, &map).await.unwrap();
    assert_eq!(holder(&pool, SYS_A).await.unwrap().1, first_write);
}
