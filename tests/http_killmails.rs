//! The killmails card over HTTP: which kills the viewer's filter lets through.

mod common;

use axum::http::StatusCode;
use common::app::{app, get, request_json, session_cookie};
use common::{SYS_A, SYS_B, SYS_C, world};
use serde_json::json;
use sqlx::PgPool;
use wormholesystems::maps::solar_system::{AddSystem, add_system};

/// Recent kills with a ship, which is what the card needs to show a row. A higher id is a
/// newer kill.
async fn kills(pool: &PgPool, ids: &[i64], solar_system_id: i64) {
    for id in ids {
        sqlx::query(
            "insert into killmails (id, hash, solar_system_id, time, orgs, victim_ship_type_id)
             values ($1, 'h', $2, now() - make_interval(mins => 100 - $1::int), '[]', 587)",
        )
        .bind(id)
        .bind(solar_system_id)
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn listed(pool: &PgPool, cookie: &str, path: &str) -> Vec<i64> {
    let res = get(app(pool), path, Some(cookie)).await;
    assert_eq!(res.status, StatusCode::OK);
    res.body
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k["id"].as_i64().unwrap())
        .collect()
}

/// Following the selection shows that one system, and nothing while no system on the map
/// is selected: neither an empty selection nor a system off the map falls back to all.
#[sqlx::test]
async fn the_selected_system_filter_shows_only_that_system(pool: PgPool) {
    let w = world(&pool).await;
    for system in [SYS_A, SYS_B] {
        add_system(
            &pool,
            w.owner,
            AddSystem {
                map_id: w.map_id,
                solar_system_id: system,
                x: 0.0,
                y: 0.0,
                alias: None,
            },
        )
        .await
        .unwrap();
    }
    kills(&pool, &[1, 2, 3], SYS_A).await;
    kills(&pool, &[4, 5], SYS_B).await;
    kills(&pool, &[6], SYS_C).await;

    let cookie = session_cookie(&pool, w.owner).await;
    let path = format!("/api/maps/{}/killmails", w.map_id);
    assert_eq!(listed(&pool, &cookie, &path).await, vec![5, 4, 3, 2, 1]);

    let saved = request_json(
        app(&pool),
        "POST",
        &format!("/api/maps/{}/settings/user", w.map_id),
        Some(&cookie),
        json!({ "killmail_filter": "selected_system" }),
    )
    .await;
    assert_eq!(saved.status, StatusCode::OK);
    assert_eq!(saved.body["killmail_filter"], "selected_system");

    let selecting = |system: i64| format!("{path}?solar_system_id={system}");
    assert_eq!(
        listed(&pool, &cookie, &selecting(SYS_A)).await,
        vec![3, 2, 1]
    );
    assert_eq!(listed(&pool, &cookie, &selecting(SYS_B)).await, vec![5, 4]);
    assert!(listed(&pool, &cookie, &path).await.is_empty());
    assert!(
        listed(&pool, &cookie, &selecting(SYS_C)).await.is_empty(),
        "a system off the map has nothing to show here"
    );
}
