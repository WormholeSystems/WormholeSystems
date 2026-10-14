//! Naming the pilots on killmails as they arrive: only what is new since the last pass,
//! in bulk, and without one dead id costing the rest their names.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{Value, json};
use sqlx::PgPool;
use wormholesystems::esi::EsiClient;
use wormholesystems::killmails::NameResolver;

const DEAD: i64 = 2_100_000_666;
const CORP: i64 = 98_000_001;

/// A stand-in for the bulk name endpoint: every id is a pilot, except [`DEAD`], which
/// fails the whole call the way ESI does. Calls are counted.
async fn stub_esi() -> (EsiClient, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/universe/names", post(names))
        .with_state(calls.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (
        EsiClient::with_config(reqwest::Client::new(), base, "2025-08-26"),
        calls,
    )
}

async fn names(
    State(calls): State<Arc<AtomicUsize>>,
    Json(ids): Json<Vec<i64>>,
) -> (StatusCode, Json<Value>) {
    calls.fetch_add(1, Ordering::SeqCst);
    if ids.contains(&DEAD) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Ensure all IDs are valid before resolving." })),
        );
    }
    let named: Vec<Value> = ids
        .iter()
        .map(|id| json!({ "id": id, "name": format!("Pilot {id}"), "category": "character" }))
        .collect();
    (StatusCode::OK, Json(Value::Array(named)))
}

async fn setup(pool: &PgPool) {
    sqlx::query("insert into zkb_state (id, sequence_id) values (true, 1)")
        .execute(pool)
        .await
        .unwrap();
    // Already named, so the organisations never need ESI here.
    sqlx::query("insert into corporations (id, name, ticker) values ($1, 'Corp', 'CORP')")
        .bind(CORP)
        .execute(pool)
        .await
        .unwrap();
}

/// A kill as the live stream (`live`) or the archive import would have stored it.
async fn kill(pool: &PgPool, id: i64, victim: i64, attacker: i64, live: bool) {
    sqlx::query(
        "insert into killmails (id, hash, solar_system_id, time, orgs, victim_character_id,
                                victim_corporation_id, final_blow_character_id, ingest_seq)
         values ($1, 'h', 30000142, now(), '[]', $2, $3, $4,
                 case when $5 then nextval('killmails_ingest_seq') end)",
    )
    .bind(id)
    .bind(victim)
    .bind(CORP)
    .bind(attacker)
    .bind(live)
    .execute(pool)
    .await
    .unwrap();
}

async fn named(pool: &PgPool) -> Vec<i64> {
    sqlx::query_scalar("select id from characters order by id")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn mark(pool: &PgPool) -> i64 {
    sqlx::query_scalar("select named_seq from zkb_state")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn new_kills_are_named_once_and_old_ones_left_alone(pool: PgPool) {
    setup(&pool).await;
    let (esi, calls) = stub_esi().await;
    // A pilot ESI once said was gone, long enough ago to be asked about again.
    sqlx::query(
        "insert into unresolvable_entities (kind, id, noted_at)
         values ('character', 11, now() - interval '31 days')",
    )
    .execute(&pool)
    .await
    .unwrap();
    kill(&pool, 1, 11, 12, true).await;
    kill(&pool, 2, 13, 12, true).await;
    kill(&pool, 3, 99, 98, false).await;

    let mut resolver = NameResolver::default();
    resolver.pass(&pool, &esi).await;
    assert_eq!(
        named(&pool).await,
        [11, 12, 13],
        "the archive kill is its own business"
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "one bulk call for all three"
    );
    let noted: i64 = sqlx::query_scalar("select count(*) from unresolvable_entities")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(noted, 0, "the pilot is back, so the note goes");

    resolver.pass(&pool, &esi).await;
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "nothing new, nothing asked"
    );

    // A fresh resolver, as after a restart, carries on from the stored mark.
    kill(&pool, 4, 14, 12, true).await;
    NameResolver::default().pass(&pool, &esi).await;
    assert_eq!(named(&pool).await, [11, 12, 13, 14]);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[sqlx::test]
async fn one_dead_pilot_costs_only_its_own_name(pool: PgPool) {
    setup(&pool).await;
    let (esi, _) = stub_esi().await;
    for id in 1..=8 {
        kill(&pool, id, 100 + id, DEAD, true).await;
    }

    NameResolver::default().pass(&pool, &esi).await;
    assert_eq!(named(&pool).await, (101..=108).collect::<Vec<i64>>());
    let dead: bool = sqlx::query_scalar(
        "select exists(select 1 from unresolvable_entities where kind = 'character' and id = $1)",
    )
    .bind(DEAD)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(dead);
}

#[sqlx::test]
async fn an_unreachable_esi_holds_the_mark_for_a_few_passes(pool: PgPool) {
    setup(&pool).await;
    kill(&pool, 1, 11, 12, true).await;
    // Nothing listens here, so every call fails without ESI saying anything about the ids.
    let esi = EsiClient::with_config(reqwest::Client::new(), "http://127.0.0.1:9", "2025-08-26");

    let mut resolver = NameResolver::default();
    for _ in 0..3 {
        resolver.pass(&pool, &esi).await;
        assert_eq!(mark(&pool).await, 0, "tried again next pass");
    }
    resolver.pass(&pool, &esi).await;
    assert!(mark(&pool).await > 0, "and given up on after that");
}
