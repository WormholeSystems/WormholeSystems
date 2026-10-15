//! The live killmail feed against a stand-in R2Z2: kills are stored back to back, the
//! cursor follows them, an alert hears about one without the ingest waiting for it, and a
//! kill missing from the feed does not stall it.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use common::{SYS_A, world};
use serde_json::{Value, json};
use sqlx::PgPool;
use wormholesystems::alerts::Runtime;
use wormholesystems::esi::EsiClient;
use wormholesystems::killmails::{Timing, listen};
use wormholesystems::maps::solar_system::{AddSystem, add_system};

const HEAD: i64 = 100;

/// Sequence 101 and 102 are kills in Jita, 103 onwards does not exist yet.
async fn frame(Path(name): Path<String>) -> (StatusCode, String) {
    if name == "sequence.json" {
        return (StatusCode::OK, json!({ "sequence": HEAD }).to_string());
    }
    let seq: i64 = name.trim_end_matches(".json").parse().unwrap_or(0);
    if seq <= HEAD || seq > HEAD + 2 {
        return (StatusCode::NOT_FOUND, String::new());
    }
    (StatusCode::OK, kill(seq))
}

/// Kill `seq` of the feed, as R2Z2 serves it: killmail `5000 + seq`, just now.
fn kill(seq: i64) -> String {
    json!({
        "killmail_id": 5000 + seq,
        "hash": "abc",
        "esi": {
            "killmail_time": chrono::Utc::now().to_rfc3339(),
            "solar_system_id": SYS_A,
            "victim": { "character_id": 1, "ship_type_id": 670 },
            "attackers": [{ "character_id": 2, "final_blow": true }]
        },
        "zkb": { "totalValue": 1000.0, "attackerCount": 1, "solo": true }
    })
    .to_string()
}

async fn receive(State(inbox): State<Arc<Mutex<Vec<Value>>>>, body: String) -> StatusCode {
    inbox
        .lock()
        .unwrap()
        .push(serde_json::from_str(&body).unwrap());
    StatusCode::NO_CONTENT
}

async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    base
}

async fn wait_for(mut done: impl AsyncFnMut() -> bool) -> bool {
    for _ in 0..100 {
        if done().await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

#[sqlx::test]
async fn kills_are_stored_back_to_back_and_alerted_on(pool: PgPool) {
    let w = world(&pool).await;
    add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            x: 0.0,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap();

    let inbox: Arc<Mutex<Vec<Value>>> = Arc::default();
    let discord = serve(
        Router::new()
            .route("/webhook", post(receive))
            .with_state(inbox.clone()),
    )
    .await;
    let webhook = sqlx::query_scalar!(
        "insert into map_webhooks (map_id, name, url) values ($1, 'Kills', $2) returning id",
        w.map_id,
        format!("{discord}/webhook"),
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query!(
        "insert into map_alerts (map_id, name, kind, delivery, map_webhook_id, max_jumps)
         values ($1, 'Kills', 'killmail', 'webhook', $2, 0)",
        w.map_id,
        webhook,
    )
    .execute(&pool)
    .await
    .unwrap();

    let zkb = serve(Router::new().route("/ephemeral/{name}", get(frame))).await;
    let config = wormholesystems::config::KillmailConfig {
        listen: true,
        base_url: zkb,
        backfill_days: 0,
    };
    let runtime = Arc::new(
        Runtime::load(&pool, None, "https://map.example.com".into())
            .await
            .unwrap(),
    );
    // Nothing listens on the ESI address, so naming fails quietly and changes nothing here.
    let esi = EsiClient::with_config(reqwest::Client::new(), "http://127.0.0.1:9", "2025-08-26");
    wormholesystems::killmails::start(pool.clone(), esi, Some(runtime), &config);

    let stored = wait_for(async || {
        sqlx::query_scalar!("select sequence_id from zkb_state")
            .fetch_optional(&pool)
            .await
            .unwrap()
            == Some(HEAD + 2)
    })
    .await;
    assert!(stored, "the cursor follows both kills");
    let ids = sqlx::query_scalar!("select id from killmails order by id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(ids, [5101, 5102]);
    let numbered = sqlx::query_scalar!("select count(ingest_seq) from killmails")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        numbered,
        Some(2),
        "live kills are numbered for the name resolver"
    );

    assert!(
        wait_for(async || inbox.lock().unwrap().len() == 2).await,
        "one message per kill"
    );
}

/// A feed whose head is `head` and which still holds the kills `held` says it does.
#[derive(Clone)]
struct Feed {
    head: i64,
    held: fn(i64) -> bool,
}

async fn feed_frame(State(feed): State<Feed>, Path(name): Path<String>) -> (StatusCode, String) {
    if name == "sequence.json" {
        return (StatusCode::OK, json!({ "sequence": feed.head }).to_string());
    }
    let seq: i64 = name.trim_end_matches(".json").parse().unwrap_or(0);
    if (feed.held)(seq) {
        (StatusCode::OK, kill(seq))
    } else {
        (StatusCode::NOT_FOUND, String::new())
    }
}

/// The feed at test speed.
const FAST: Timing = Timing {
    idle: Duration::from_millis(10),
    pace: Duration::from_millis(1),
    head_check: Duration::ZERO,
    missing_grace: Duration::from_millis(50),
};

/// Follow `feed` from `cursor` on.
async fn follow(pool: &PgPool, cursor: i64, feed: Feed, timing: Timing) {
    world(pool).await;
    sqlx::query!(
        "insert into zkb_state (id, sequence_id) values (true, $1)",
        cursor
    )
    .execute(pool)
    .await
    .unwrap();
    let base = serve(
        Router::new()
            .route("/ephemeral/{name}", get(feed_frame))
            .with_state(feed),
    )
    .await;
    tokio::spawn(listen(pool.clone(), None, base, timing));
}

async fn cursor(pool: &PgPool) -> i64 {
    sqlx::query_scalar!("select sequence_id from zkb_state")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn stored(pool: &PgPool) -> Vec<i64> {
    sqlx::query_scalar!("select id from killmails order by id")
        .fetch_all(pool)
        .await
        .unwrap()
}

/// Down for longer than the feed keeps its files: everything after the cursor 404s. The
/// head says there is more, so the ingest finds where the feed now starts and goes on
/// from there, instead of waiting on the first missing kill forever.
#[sqlx::test]
async fn a_cursor_older_than_the_feed_skips_to_what_it_still_holds(pool: PgPool) {
    let feed = Feed {
        head: 520,
        held: |seq| (500..=520).contains(&seq),
    };
    follow(&pool, 10, feed, FAST).await;

    assert!(wait_for(async || cursor(&pool).await == 520).await);
    assert_eq!(stored(&pool).await, (5500..=5520).collect::<Vec<_>>());
}

/// A kill pulled from the middle of the feed costs that kill and nothing after it.
#[sqlx::test]
async fn a_kill_missing_under_the_head_is_stepped_over(pool: PgPool) {
    let feed = Feed {
        head: 104,
        held: |seq| seq == 101 || (103..=104).contains(&seq),
    };
    follow(&pool, 100, feed, FAST).await;

    assert!(wait_for(async || cursor(&pool).await == 104).await);
    assert_eq!(stored(&pool).await, [5101, 5103, 5104]);
}

/// Past the head a 404 is the kill not having happened yet, and one the head has only just
/// counted gets its grace: neither is skipped.
#[sqlx::test]
async fn a_kill_not_yet_there_is_waited_for(pool: PgPool) {
    let feed = Feed {
        head: 101,
        held: |_| false,
    };
    let patient = Timing {
        missing_grace: Duration::from_secs(60),
        ..FAST
    };
    follow(&pool, 100, feed, patient).await;

    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(cursor(&pool).await, 100);
}
