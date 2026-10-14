//! The live killmail feed against a stand-in R2Z2: kills are stored back to back, the
//! cursor follows them, and an alert hears about one without the ingest waiting for it.

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
    let body = json!({
        "killmail_id": 5000 + seq,
        "hash": "abc",
        "esi": {
            "killmail_time": chrono::Utc::now().to_rfc3339(),
            "solar_system_id": SYS_A,
            "victim": { "character_id": 1, "ship_type_id": 670 },
            "attackers": [{ "character_id": 2, "final_blow": true }]
        },
        "zkb": { "totalValue": 1000.0, "attackerCount": 1, "solo": true }
    });
    (StatusCode::OK, body.to_string())
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
    let runtime = Arc::new(Runtime::load(&pool, None).await.unwrap());
    // Nothing listens on the ESI address, so naming fails quietly and changes nothing here.
    let esi = EsiClient::with_config(reqwest::Client::new(), "http://127.0.0.1:9", "2025-08-26");
    wormholesystems::killmails::start(
        pool.clone(),
        esi,
        wormholesystems::maps::MapHub::new(),
        Some(runtime),
        &config,
    );

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
