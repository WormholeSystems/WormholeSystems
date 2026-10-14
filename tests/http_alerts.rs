//! The alerts CRUD over HTTP: the settings-page round trip, and the guards that keep
//! one map's Discord destinations out of another's alerts.

mod common;

use axum::http::StatusCode;
use common::app::{app, get, request_json, session_cookie};
use common::world;
use serde_json::json;
use sqlx::PgPool;
use wormholesystems::allowlist::AffiliationAllowlist;

fn killmail_alert(name: &str, webhook_id: i64) -> serde_json::Value {
    json!({
        "name": name,
        "kind": "killmail",
        "delivery": "webhook",
        "map_webhook_id": webhook_id,
        "mention": "none",
        "max_jumps": 5,
        "filters": [],
        "filter_match": "any",
    })
}

async fn make_webhook(pool: &PgPool, cookie: &str, map_id: i64, name: &str) -> i64 {
    let made = request_json(
        app(pool),
        "POST",
        &format!("/api/maps/{map_id}/webhooks"),
        Some(cookie),
        json!({ "name": name, "url": "https://discord.com/api/webhooks/123456/token" }),
    )
    .await;
    assert_eq!(made.status, StatusCode::OK);
    made.body["id"].as_i64().unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn an_alert_survives_the_full_round_trip(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let webhook_id = make_webhook(&pool, &cookie, w.map_id, "ops").await;
    let base = format!("/api/maps/{}/alerts", w.map_id);

    let created = request_json(
        app(&pool),
        "POST",
        &base,
        Some(&cookie),
        killmail_alert("kills", webhook_id),
    )
    .await;
    assert_eq!(created.status, StatusCode::OK);
    assert_eq!(created.body["webhook_name"], "ops");
    let id = created.body["id"].as_i64().unwrap();

    let renamed = request_json(
        app(&pool),
        "PUT",
        &format!("{base}/{id}"),
        Some(&cookie),
        killmail_alert("chain kills", webhook_id),
    )
    .await;
    assert_eq!(renamed.status, StatusCode::OK);
    assert_eq!(renamed.body["name"], "chain kills");

    let paused = request_json(
        app(&pool),
        "POST",
        &format!("{base}/{id}/active"),
        Some(&cookie),
        json!({ "is_active": false }),
    )
    .await;
    assert_eq!(paused.body["is_active"], false);
    assert_eq!(paused.body["disabled_reason"], "manual");

    let deleted = request_json(
        app(&pool),
        "DELETE",
        &format!("{base}/{id}"),
        Some(&cookie),
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(deleted.status, StatusCode::OK);

    let listed = get(app(&pool), &base, Some(&cookie)).await;
    assert_eq!(listed.body.as_array().map(|a| a.len()), Some(0));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_proximity_alert_keeps_its_starting_point(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let webhook_id = make_webhook(&pool, &cookie, w.map_id, "ops").await;
    let base = format!("/api/maps/{}/alerts", w.map_id);

    let mut alert = killmail_alert("home to jita", webhook_id);
    alert["kind"] = json!("proximity");
    alert["target_solar_system_id"] = json!(common::SYS_A);
    alert["origin_solar_system_id"] = json!(common::SYS_C);
    let created = request_json(app(&pool), "POST", &base, Some(&cookie), alert.clone()).await;
    assert_eq!(created.status, StatusCode::OK);
    assert_eq!(created.body["origin_solar_system_id"], common::SYS_C);
    assert_eq!(created.body["origin_system_name"], "New Caldari");

    alert["origin_solar_system_id"] = json!(999);
    let unknown = request_json(app(&pool), "POST", &base, Some(&cookie), alert.clone()).await;
    assert_eq!(unknown.status, StatusCode::BAD_REQUEST);
    assert_eq!(unknown.body["error"], "that system does not exist");

    alert["kind"] = json!("killmail");
    alert["origin_solar_system_id"] = json!(common::SYS_C);
    let misplaced = request_json(app(&pool), "POST", &base, Some(&cookie), alert).await;
    assert_eq!(misplaced.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        misplaced.body["error"],
        "only a proximity alert has a starting point"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_destination_from_another_map_is_refused(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;

    let other_map = wormholesystems::maps::map::create_map(
        &pool,
        w.owner,
        &AffiliationAllowlist::default(),
        wormholesystems::maps::map::CreateMap {
            name: "other".into(),
            description: None,
        },
    )
    .await
    .unwrap();
    let foreign_webhook = make_webhook(&pool, &cookie, other_map.id, "theirs").await;

    let refused = request_json(
        app(&pool),
        "POST",
        &format!("/api/maps/{}/alerts", w.map_id),
        Some(&cookie),
        killmail_alert("kills", foreign_webhook),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(refused.body["error"], "that destination is not on this map");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_bad_alert_is_refused_before_it_is_stored(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let webhook_id = make_webhook(&pool, &cookie, w.map_id, "ops").await;
    let base = format!("/api/maps/{}/alerts", w.map_id);

    let mut nameless = killmail_alert("   ", webhook_id);
    nameless["name"] = json!("   ");
    let refused = request_json(app(&pool), "POST", &base, Some(&cookie), nameless).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let update_missing = request_json(
        app(&pool),
        "PUT",
        &format!("{base}/999999"),
        Some(&cookie),
        killmail_alert("kills", webhook_id),
    )
    .await;
    assert_eq!(update_missing.status, StatusCode::NOT_FOUND);
}
