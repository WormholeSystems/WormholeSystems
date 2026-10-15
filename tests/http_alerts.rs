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

fn bot_alert(name: &str, mention: &str) -> serde_json::Value {
    json!({
        "name": name,
        "kind": "killmail",
        "delivery": "discord_channel",
        "discord_guild_id": "999",
        "discord_channel_id": "998",
        "mention": mention,
        "max_jumps": 5,
        "filters": [],
        "filter_match": "any",
    })
}

async fn channel_of(pool: &PgPool, alert_id: i64) -> (Option<String>, Option<String>) {
    let row = sqlx::query!(
        "select discord_guild_id, discord_channel_id from map_alerts where id = $1",
        alert_id
    )
    .fetch_one(pool)
    .await
    .unwrap();
    (row.discord_guild_id, row.discord_channel_id)
}

#[sqlx::test(migrations = "./migrations")]
async fn a_bot_channel_cannot_be_set_over_http(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let webhook_id = make_webhook(&pool, &cookie, w.map_id, "ops").await;
    let base = format!("/api/maps/{}/alerts", w.map_id);

    // A new channel alert, channel and all.
    let refused = request_json(
        app(&pool),
        "POST",
        &base,
        Some(&cookie),
        bot_alert("anywhere", "everyone"),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert!(
        wormholesystems::maps::alerts::list(&pool, w.map_id)
            .await
            .unwrap()
            .is_empty()
    );

    // A channel smuggled onto a webhook alert is not stored.
    let mut smuggled = killmail_alert("kills", webhook_id);
    smuggled["discord_guild_id"] = json!("999");
    smuggled["discord_channel_id"] = json!("998");
    let created = request_json(app(&pool), "POST", &base, Some(&cookie), smuggled).await;
    assert_eq!(created.status, StatusCode::OK);
    let id = created.body["id"].as_i64().unwrap();
    assert_eq!(channel_of(&pool, id).await, (None, None));

    // Nor can a webhook alert be turned into a channel one.
    let refused = request_json(
        app(&pool),
        "PUT",
        &format!("{base}/{id}"),
        Some(&cookie),
        bot_alert("kills", "none"),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(channel_of(&pool, id).await, (None, None));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_bot_channel_alert_keeps_its_channel_and_ping_when_edited(pool: PgPool) {
    use wormholesystems::alerts::{AlertDelivery, AlertKind, AlertMention, filters};
    use wormholesystems::maps::alerts::{SaveAlert, create, register_role};

    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    // As the Discord command would have made it, in channel 600 of server 500.
    let role = register_role(&pool, w.map_id, "Scouts", "77")
        .await
        .unwrap();
    let made = create(
        &pool,
        w.map_id,
        w.owner.user_id,
        &SaveAlert {
            name: "from discord".into(),
            kind: AlertKind::Killmail,
            delivery: AlertDelivery::DiscordChannel,
            map_webhook_id: None,
            discord_guild_id: Some("500".into()),
            discord_channel_id: Some("600".into()),
            map_webhook_role_id: Some(role),
            mention: AlertMention::Role,
            target_solar_system_id: None,
            origin_solar_system_id: None,
            max_jumps: 5,
            ship_type: None,
            jdc_level: None,
            filters: Vec::new(),
            filter_match: filters::Match::Any,
        },
    )
    .await
    .unwrap();
    let path = format!("/api/maps/{}/alerts/{}", w.map_id, made.id);
    let kept = (Some("500".to_string()), Some("600".to_string()));

    // Renamed, with the same ping and a channel of its own choosing: the channel stays.
    let mut renamed = bot_alert("renamed", "role");
    renamed["map_webhook_role_id"] = json!(role);
    let saved = request_json(app(&pool), "PUT", &path, Some(&cookie), renamed).await;
    assert_eq!(saved.status, StatusCode::OK);
    assert_eq!(saved.body["name"], "renamed");
    assert_eq!(saved.body["discord_channel_id"], "600");
    assert_eq!(channel_of(&pool, made.id).await, kept);

    // Pinging everyone instead is for Discord to allow.
    let refused = request_json(
        app(&pool),
        "PUT",
        &path,
        Some(&cookie),
        bot_alert("renamed", "everyone"),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    // Dropping the ping is fine.
    let quiet = request_json(
        app(&pool),
        "PUT",
        &path,
        Some(&cookie),
        bot_alert("quiet", "none"),
    )
    .await;
    assert_eq!(quiet.status, StatusCode::OK);
    assert_eq!(quiet.body["mention"], "none");
    assert_eq!(channel_of(&pool, made.id).await, kept);
}
