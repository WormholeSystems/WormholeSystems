//! Threat analysis: the legacy rules over locally-ingested killmails. Names resolve from
//! the local corporation/alliance tables here, so no ESI call is made.

mod common;

use sqlx::PgPool;
use wormholesystems::killmails::{Org, analyze};
use wormholesystems::maps::ThreatLevel;

const WH_SYSTEM: i64 = 31000099;

async fn seed_wormhole_system(pool: &PgPool) {
    common::seed_universe(pool).await;
    common::catalog::seed_wormhole_system(pool, WH_SYSTEM, 5).await;
    sqlx::query(
        "insert into alliances (id, name, ticker) values (99000001, 'Threat Alliance', 'THRT')",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_kills(pool: &PgPool, count: i64, days: i64, org: Org, start_id: i64) {
    for i in 0..count {
        let day_offset = i % days;
        sqlx::query(
            "insert into killmails (id, hash, solar_system_id, time, orgs)
             values ($1, 'h', $2, now() - make_interval(days => $3::int), $4)",
        )
        .bind(start_id + i)
        .bind(WH_SYSTEM)
        .bind(day_offset as i32 + 1)
        .bind(serde_json::to_value(vec![org.clone()]).unwrap())
        .execute(pool)
        .await
        .unwrap();
    }
}

#[sqlx::test]
async fn threat_levels_follow_the_kill_thresholds(pool: PgPool) {
    seed_wormhole_system(&pool).await;
    let org = Org {
        id: 99000001,
        kind: "alliance".into(),
    };

    // 60 kills spread over 6 distinct days → critical, with the org in the top list.
    insert_kills(&pool, 60, 6, org.clone(), 1).await;
    analyze(&pool, &wormholesystems::esi::EsiClient::new())
        .await
        .unwrap();

    let level: ThreatLevel =
        sqlx::query_scalar("select threat_level from wormhole_systems where solar_system_id = $1")
            .bind(WH_SYSTEM)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(level, ThreatLevel::Critical);

    let (name, kills): (String, i32) = sqlx::query_as(
        "select name, kills from wormhole_system_threats where solar_system_id = $1",
    )
    .bind(WH_SYSTEM)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(name, "Threat Alliance");
    assert_eq!(kills, 60);
}

#[sqlx::test]
async fn orgs_below_five_active_days_are_ignored(pool: PgPool) {
    seed_wormhole_system(&pool).await;
    let org = Org {
        id: 99000001,
        kind: "alliance".into(),
    };

    // 60 kills but concentrated on 3 days → filtered out entirely → unknown.
    insert_kills(&pool, 60, 3, org, 1).await;
    analyze(&pool, &wormholesystems::esi::EsiClient::new())
        .await
        .unwrap();

    let level: ThreatLevel =
        sqlx::query_scalar("select threat_level from wormhole_systems where solar_system_id = $1")
            .bind(WH_SYSTEM)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(level, ThreatLevel::Unknown);
    let count: i64 = sqlx::query_scalar(
        "select count(*) from wormhole_system_threats where solar_system_id = $1",
    )
    .bind(WH_SYSTEM)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test]
async fn twenty_kills_across_a_week_is_high(pool: PgPool) {
    seed_wormhole_system(&pool).await;
    insert_kills(
        &pool,
        20,
        7,
        Org {
            id: 99000001,
            kind: "alliance".into(),
        },
        1,
    )
    .await;
    analyze(&pool, &wormholesystems::esi::EsiClient::new())
        .await
        .unwrap();
    let level: ThreatLevel =
        sqlx::query_scalar("select threat_level from wormhole_systems where solar_system_id = $1")
            .bind(WH_SYSTEM)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(level, ThreatLevel::High);
}
