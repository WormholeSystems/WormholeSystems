//! Alert plumbing: the delivery ledger that stops a message being sent twice, the filter
//! matching that decides whether a kill is worth a message at all, and the proximity
//! evaluation that decides when the chain has come close enough.

mod common;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::State;
use axum::routing::post;
use common::{SYS_A, SYS_B, SYS_C, world};
use serde_json::Value;
use sqlx::PgPool;
use wormholesystems::alerts::filters::{Candidates, Match, Mode, Rule, Side, Subject};
use wormholesystems::alerts::{self, AlertKind, Runtime};
use wormholesystems::maps::Actor;

async fn make_webhook(pool: &PgPool, map_id: i64, url: &str) -> i64 {
    sqlx::query_scalar!(
        "insert into map_webhooks (map_id, name, url)
         values ($1, 'Test channel', $2)
         on conflict (map_id, name) do update set url = excluded.url
         returning id",
        map_id,
        url,
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn make_alert(pool: &PgPool, map_id: i64, kind: &str) -> i64 {
    let webhook = make_webhook(pool, map_id, "https://discord.com/api/webhooks/1/x").await;
    sqlx::query_scalar!(
        "insert into map_alerts (map_id, name, kind, delivery, map_webhook_id, max_jumps)
         values ($1, 'Test', $2, 'webhook', $3, 5)
         returning id",
        map_id,
        kind,
        webhook,
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

/// A stand-in for a Discord channel: a webhook URL on localhost that keeps what it is sent.
struct Inbox {
    url: String,
    messages: Arc<Mutex<Vec<Value>>>,
}

impl Inbox {
    async fn start() -> Inbox {
        let messages: Arc<Mutex<Vec<Value>>> = Arc::default();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/webhook", listener.local_addr().unwrap());
        let app = Router::new()
            .route("/webhook", post(receive))
            .with_state(messages.clone());
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Inbox { url, messages }
    }

    fn descriptions(&self) -> Vec<String> {
        self.messages
            .lock()
            .unwrap()
            .iter()
            .map(|m| {
                m["embeds"][0]["description"]
                    .as_str()
                    .unwrap_or("")
                    .to_string()
            })
            .collect()
    }
}

async fn receive(
    State(messages): State<Arc<Mutex<Vec<Value>>>>,
    body: String,
) -> axum::http::StatusCode {
    let message: Value = serde_json::from_str(&body).unwrap();
    messages.lock().unwrap().push(message);
    axum::http::StatusCode::NO_CONTENT
}

// Three more k-space systems, so the seeded universe can hold a gate route longer than
// any sensible `max_jumps`.
const SYS_D: i64 = 30000146;
const SYS_E: i64 = 30000147;
const SYS_F: i64 = 30000148;

/// Gates in a line: A - B - C - D - E - F.
async fn seed_gates(pool: &PgPool) {
    for (id, name) in [
        (SYS_D, "Niyabainen"),
        (SYS_E, "Tunttaras"),
        (SYS_F, "Isanamo"),
    ] {
        sqlx::query(
            "insert into solar_systems (id, constellation_id, region_id, name, security_status)
             values ($1, 20000001, 10000001, $2, 0.9)",
        )
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
    }
    sqlx::query("insert into categories (id, name) values (1, 'Celestial')")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("insert into groups (id, category_id, name) values (10, 1, 'Stargate')")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("insert into types (id, group_id, name) values (100, 10, 'Stargate')")
        .execute(pool)
        .await
        .unwrap();
    let line = [SYS_A, SYS_B, SYS_C, SYS_D, SYS_E, SYS_F];
    for (index, pair) in line.windows(2).enumerate() {
        for (gate, (from, to)) in [(pair[0], pair[1]), (pair[1], pair[0])]
            .into_iter()
            .enumerate()
        {
            sqlx::query(
                "insert into stargates (id, solar_system_id, destination_system_id, destination_stargate_id, type_id)
                 values ($1, $2, $3, 0, 100)",
            )
            .bind((index * 2 + gate + 1) as i64)
            .bind(from)
            .bind(to)
            .execute(pool)
            .await
            .unwrap();
        }
    }
}

async fn proximity_alert(
    pool: &PgPool,
    map_id: i64,
    inbox: &Inbox,
    target: i64,
    origin: Option<i64>,
    max_jumps: i32,
) -> i64 {
    let webhook = make_webhook(pool, map_id, &inbox.url).await;
    sqlx::query_scalar!(
        "insert into map_alerts
             (map_id, name, kind, delivery, map_webhook_id, target_solar_system_id,
              origin_solar_system_id, max_jumps)
         values ($1, 'Near', 'proximity', 'webhook', $2, $3, $4, $5)
         returning id",
        map_id,
        webhook,
        target,
        origin,
        max_jumps,
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn place(pool: &PgPool, runtime: &Runtime, actor: Actor, map_id: i64, sys: i64) -> i64 {
    let placed = common::map::place(pool, actor, map_id, sys).await;
    for out in runtime.placed(pool, map_id, placed).await {
        runtime.send(pool, out).await;
    }
    placed
}

async fn connect(
    pool: &PgPool,
    runtime: &Runtime,
    actor: Actor,
    map_id: i64,
    from: i64,
    to: i64,
) -> i64 {
    let connection = common::map::connect(pool, actor, map_id, from, to).await;
    for out in runtime.connected(pool, map_id, connection.id).await {
        runtime.send(pool, out).await;
    }
    connection.id
}

/// The claim is the whole reason a retry cannot double-post.
#[sqlx::test]
async fn an_occasion_can_only_be_claimed_once(pool: PgPool) {
    let w = world(&pool).await;
    let alert = make_alert(&pool, w.map_id, "killmail").await;

    assert!(alerts::claim(&pool, alert, "killmail:1").await);
    assert!(!alerts::claim(&pool, alert, "killmail:1").await);
    // A different occasion is free to claim.
    assert!(alerts::claim(&pool, alert, "killmail:2").await);
}

/// Giving up must put the occasion back, or one Discord hiccup silences that kill forever.
#[sqlx::test]
async fn releasing_an_unsent_claim_allows_another_attempt(pool: PgPool) {
    let w = world(&pool).await;
    let alert = make_alert(&pool, w.map_id, "killmail").await;

    assert!(alerts::claim(&pool, alert, "killmail:1").await);
    alerts::unclaim(&pool, alert, "killmail:1").await;
    assert!(alerts::claim(&pool, alert, "killmail:1").await);
}

/// A delivered one stays claimed, whatever else happens.
#[sqlx::test]
async fn a_sent_claim_is_never_released(pool: PgPool) {
    let w = world(&pool).await;
    let alert = make_alert(&pool, w.map_id, "killmail").await;

    alerts::claim(&pool, alert, "killmail:1").await;
    alerts::sent(&pool, alert, "killmail:1").await;
    alerts::unclaim(&pool, alert, "killmail:1").await;
    assert!(!alerts::claim(&pool, alert, "killmail:1").await);

    let fired: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar!("select last_fired_at from map_alerts where id = $1", alert)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(fired.is_some());
}

#[sqlx::test]
async fn only_active_alerts_of_the_asked_kind_are_loaded(pool: PgPool) {
    let w = world(&pool).await;
    let killmail = make_alert(&pool, w.map_id, "killmail").await;
    let proximity = make_alert(&pool, w.map_id, "proximity").await;
    sqlx::query!(
        "update map_alerts set is_active = false where id = $1",
        proximity,
    )
    .execute(&pool)
    .await
    .unwrap();

    let loaded = alerts::active(&pool, AlertKind::Killmail).await.unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, killmail);
    assert!(
        alerts::active(&pool, AlertKind::Proximity)
            .await
            .unwrap()
            .is_empty()
    );
}

/// Disabling records why, so the settings page can say more than "off".
#[sqlx::test]
async fn disabling_records_a_reason_and_an_event(pool: PgPool) {
    let w = world(&pool).await;
    let id = make_alert(&pool, w.map_id, "killmail").await;
    let alert = alerts::active(&pool, AlertKind::Killmail)
        .await
        .unwrap()
        .remove(0);

    alerts::disable(&pool, &alert, alerts::DisabledReason::DestinationGone, None).await;

    let row = sqlx::query!(
        "select is_active, disabled_reason from map_alerts where id = $1",
        id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!row.is_active);
    assert_eq!(row.disabled_reason.as_deref(), Some("destination_gone"));

    let events = sqlx::query_scalar!(
        "select kind from map_alert_events where map_alert_id = $1",
        id,
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(events, vec!["disabled".to_string()]);
}

/// The filter vocabulary, end to end against a realistic kill.
#[test]
fn filters_decide_what_is_worth_a_message() {
    let kill = Candidates {
        victim_alliance: Some(99000001),
        victim_ship_type: Some(29990),
        attacker_alliance: Some(99000002),
        ..Default::default()
    };
    let watch_them = Rule {
        subject: Subject::Alliance,
        side: Side::Either,
        mode: Mode::Include,
        ids: vec![99000002],
    };
    assert!(wormholesystems::alerts::filters::matches(
        std::slice::from_ref(&watch_them),
        Match::Any,
        &kill
    ));

    // "Anything involving them, except when it is us dying" is the shape people actually
    // want, and it only works if an exclusion outranks a match.
    let not_our_losses = Rule {
        subject: Subject::Alliance,
        side: Side::Victim,
        mode: Mode::Exclude,
        ids: vec![99000001],
    };
    assert!(!wormholesystems::alerts::filters::matches(
        &[watch_them, not_our_losses],
        Match::Any,
        &kill
    ));
}

/// From a starting point the route runs through the chain, and only a system on that route
/// is worth a message: B sits between A and C, F is nowhere near.
#[sqlx::test]
async fn a_starting_point_fires_for_a_placement_on_its_route(pool: PgPool) {
    let w = world(&pool).await;
    seed_gates(&pool).await;
    let inbox = Inbox::start().await;
    proximity_alert(&pool, w.map_id, &inbox, SYS_C, Some(SYS_A), 2).await;
    let runtime = Runtime::load(&pool, None).await.unwrap();

    place(&pool, &runtime, w.owner, w.map_id, SYS_F).await;
    assert!(
        inbox.descriptions().is_empty(),
        "F is not on the way from A to C"
    );

    place(&pool, &runtime, w.owner, w.map_id, SYS_B).await;
    let sent = inbox.descriptions();
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0].contains("**Perimeter** was just added"),
        "{}",
        sent[0]
    );
    assert!(sent[0].contains("2 jumps away"), "{}", sent[0]);
}

#[sqlx::test]
async fn a_starting_point_stays_quiet_beyond_max_jumps(pool: PgPool) {
    let w = world(&pool).await;
    seed_gates(&pool).await;
    let inbox = Inbox::start().await;
    proximity_alert(&pool, w.map_id, &inbox, SYS_D, Some(SYS_A), 2).await;
    let runtime = Runtime::load(&pool, None).await.unwrap();

    place(&pool, &runtime, w.owner, w.map_id, SYS_B).await;
    place(&pool, &runtime, w.owner, w.map_id, SYS_C).await;
    assert!(inbox.descriptions().is_empty(), "D is three gates from A");
}

/// A wormhole from A to E puts F one jump from A. Neither placement could say so, the
/// connection can, and it says so once however many times it is reported.
#[sqlx::test]
async fn a_connection_that_completes_the_route_fires_once(pool: PgPool) {
    let w = world(&pool).await;
    seed_gates(&pool).await;
    let inbox = Inbox::start().await;
    proximity_alert(&pool, w.map_id, &inbox, SYS_F, Some(SYS_A), 2).await;
    let runtime = Runtime::load(&pool, None).await.unwrap();

    let a = place(&pool, &runtime, w.owner, w.map_id, SYS_A).await;
    let e = place(&pool, &runtime, w.owner, w.map_id, SYS_E).await;
    assert!(inbox.descriptions().is_empty(), "F is five gates from A");

    let connection = connect(&pool, &runtime, w.owner, w.map_id, a, e).await;
    let sent = inbox.descriptions();
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0].contains("**Jita** and **Tunttaras** were just connected"),
        "{}",
        sent[0]
    );
    assert!(sent[0].contains("1 jump away"), "{}", sent[0]);

    // A status change re-reports the connection; the route is the same, so nothing new.
    for out in runtime.connected(&pool, w.map_id, connection).await {
        runtime.send(&pool, out).await;
    }
    assert_eq!(inbox.descriptions().len(), 1);
}

/// Without a starting point the chain is measured from wherever it is nearest, and a
/// wormhole between two mapped systems changes nothing about that.
#[sqlx::test]
async fn without_a_starting_point_the_nearest_mapped_system_counts(pool: PgPool) {
    let w = world(&pool).await;
    seed_gates(&pool).await;
    let inbox = Inbox::start().await;
    proximity_alert(&pool, w.map_id, &inbox, SYS_F, None, 1).await;
    let runtime = Runtime::load(&pool, None).await.unwrap();

    let a = place(&pool, &runtime, w.owner, w.map_id, SYS_A).await;
    assert!(inbox.descriptions().is_empty(), "A is five gates from F");

    let e = place(&pool, &runtime, w.owner, w.map_id, SYS_E).await;
    let sent = inbox.descriptions();
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0].contains("**Tunttaras** was just added"),
        "{}",
        sent[0]
    );

    connect(&pool, &runtime, w.owner, w.map_id, a, e).await;
    assert_eq!(
        inbox.descriptions().len(),
        1,
        "the connection is not a new occasion"
    );
}

/// The runtime keeps the active alerts in memory, but a change made through the store is
/// seen on the very next evaluation rather than once the copy expires.
#[sqlx::test]
async fn a_switched_off_alert_stops_firing_at_once(pool: PgPool) {
    let w = world(&pool).await;
    seed_gates(&pool).await;
    let inbox = Inbox::start().await;
    let alert = proximity_alert(&pool, w.map_id, &inbox, SYS_C, None, 1).await;
    let runtime = Runtime::load(&pool, None).await.unwrap();

    place(&pool, &runtime, w.owner, w.map_id, SYS_F).await;
    assert!(inbox.descriptions().is_empty(), "F is three gates from C");

    wormholesystems::maps::alerts::set_active(&pool, w.map_id, alert, w.owner.user_id, false, None)
        .await
        .unwrap();
    place(&pool, &runtime, w.owner, w.map_id, SYS_B).await;
    assert!(inbox.descriptions().is_empty(), "the alert is off");
}

fn kill_in(id: i64, solar_system_id: i64) -> alerts::killmail::Kill {
    alerts::killmail::Kill {
        id,
        solar_system_id,
        candidates: Candidates::default(),
        victim_name: Some("Pilot".into()),
        victim_ship: Some("a Loki".into()),
        victim_ship_type_id: None,
        attacker_name: None,
        total_value: None,
        attacker_count: Some(1),
        is_solo: true,
        is_npc: false,
    }
}

/// A batch of kills is weighed against each map's chain, and a kill already claimed is
/// not claimed again however often it comes round.
#[sqlx::test]
async fn kills_near_the_chain_are_claimed_once(pool: PgPool) {
    let w = world(&pool).await;
    seed_gates(&pool).await;
    let inbox = Inbox::start().await;
    let webhook = make_webhook(&pool, w.map_id, &inbox.url).await;
    sqlx::query!(
        "insert into map_alerts (map_id, name, kind, delivery, map_webhook_id, max_jumps)
         values ($1, 'Kills', 'killmail', 'webhook', $2, 2)",
        w.map_id,
        webhook,
    )
    .execute(&pool)
    .await
    .unwrap();
    let runtime = Runtime::load(&pool, None).await.unwrap();
    place(&pool, &runtime, w.owner, w.map_id, SYS_A).await;

    let batch = [kill_in(1, SYS_C), kill_in(2, SYS_F), kill_in(3, SYS_A)];
    let out = runtime.killmails(&pool, &batch).await;
    let keys: Vec<&str> = out.iter().map(|o| o.key.as_str()).collect();
    assert_eq!(keys, ["killmail:1", "killmail:3"], "F is five gates out");
    for message in out {
        runtime.send(&pool, message).await;
    }
    let sent = inbox.descriptions();
    assert_eq!(sent, ["in New Caldari, 2 jumps from Jita", "in Jita"]);

    assert!(runtime.killmails(&pool, &batch).await.is_empty());
}

/// Placing through the map's own event stream reaches Discord by way of the delivery
/// worker, without anyone waiting on it.
#[sqlx::test]
async fn the_subscriber_hands_messages_to_the_delivery_worker(pool: PgPool) {
    let w = world(&pool).await;
    seed_gates(&pool).await;
    let inbox = Inbox::start().await;
    proximity_alert(&pool, w.map_id, &inbox, SYS_C, None, 1).await;
    let runtime = Arc::new(Runtime::load(&pool, None).await.unwrap());
    // A hub of its own: the process-wide one carries other tests' maps.
    let hub = wormholesystems::realtime::MapHub::new();
    alerts::start_on(pool.clone(), hub.clone(), runtime);
    // Give the subscriber a moment to subscribe before anything is published.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let placed = common::map::place(&pool, w.owner, w.map_id, SYS_B).await;
    hub.publish(wormholesystems::realtime::MapEvent::SystemAdded {
        map_id: w.map_id,
        map_solar_system_id: placed,
    });

    for _ in 0..100 {
        if !inbox.descriptions().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let sent = inbox.descriptions();
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0].contains("**Perimeter** was just added"),
        "{}",
        sent[0]
    );
}
