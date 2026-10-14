//! How the ESI client treats the polled character routes and ESI's limits: a reply is
//! reused inside its cache window, revalidated with its `ETag` after it, and a 420 or 429
//! holds requests back instead of piling more errors onto the limit.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use wormholesystems::esi::{EsiClient, EsiError};

const CHARACTER: i64 = 91234567;

#[derive(Clone, Default)]
struct Stub {
    hits: Arc<AtomicUsize>,
    /// Requests that carried our `If-None-Match`.
    revalidations: Arc<AtomicUsize>,
    max_age: u64,
}

async fn stub(app: Router) -> EsiClient {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    EsiClient::with_config(reqwest::Client::new(), base, "2026-06-09")
}

const LOCATION: &str = r#"{"solar_system_id":30000142}"#;

/// Location with an ETag, cached for as long as the stub's `max_age` says.
async fn location(State(stub): State<Stub>, headers: HeaderMap) -> Response {
    stub.hits.fetch_add(1, Ordering::SeqCst);
    let reply = [
        (header::ETAG, "\"v1\"".to_string()),
        (header::CACHE_CONTROL, format!("max-age={}", stub.max_age)),
    ];
    if headers
        .get(header::IF_NONE_MATCH)
        .is_some_and(|v| v == "\"v1\"")
    {
        stub.revalidations.fetch_add(1, Ordering::SeqCst);
        (StatusCode::NOT_MODIFIED, reply).into_response()
    } else {
        (StatusCode::OK, reply, LOCATION).into_response()
    }
}

#[tokio::test]
async fn a_reply_is_reused_inside_its_cache_window() {
    let state = Stub {
        max_age: 60,
        ..Stub::default()
    };
    let esi = stub(
        Router::new()
            .route("/characters/{id}/location", get(location))
            .with_state(state.clone()),
    )
    .await;

    for _ in 0..3 {
        let loc = esi.character_location("token", CHARACTER).await.unwrap();
        assert_eq!(loc.solar_system_id, 30000142);
    }
    assert_eq!(state.hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn an_expired_reply_is_revalidated_with_its_etag() {
    let state = Stub::default();
    let esi = stub(
        Router::new()
            .route("/characters/{id}/location", get(location))
            .with_state(state.clone()),
    )
    .await;

    for _ in 0..3 {
        let loc = esi.character_location("token", CHARACTER).await.unwrap();
        assert_eq!(
            loc.solar_system_id, 30000142,
            "a 304 reuses the stored body"
        );
    }
    assert_eq!(state.hits.load(Ordering::SeqCst), 3);
    assert_eq!(state.revalidations.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn the_error_limit_holds_every_request_back_until_it_resets() {
    let state = Stub::default();
    let esi = stub(
        Router::new()
            .route(
                "/characters/{id}/location",
                get(|State(stub): State<Stub>| async move {
                    stub.hits.fetch_add(1, Ordering::SeqCst);
                    (
                        StatusCode::from_u16(420).unwrap(),
                        [
                            ("x-esi-error-limit-remain", "0"),
                            ("x-esi-error-limit-reset", "30"),
                        ],
                        r#"{"error":"error limited"}"#,
                    )
                }),
            )
            .with_state(state.clone()),
    )
    .await;

    assert!(matches!(
        esi.character_location("token", CHARACTER).await,
        Err(EsiError::Api { status: 420, .. })
    ));
    // Every route, public ones included, waits out the window without asking.
    assert!(matches!(
        esi.character_location("token", CHARACTER).await,
        Err(EsiError::Backoff(secs)) if secs > 0 && secs <= 30
    ));
    assert!(matches!(
        esi.character_public(CHARACTER).await,
        Err(EsiError::Backoff(_))
    ));
    assert_eq!(state.hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_rate_limited_token_waits_alone() {
    let state = Stub::default();
    let esi = stub(
        Router::new()
            .route(
                "/characters/{id}/location",
                get(|State(stub): State<Stub>, headers: HeaderMap| async move {
                    stub.hits.fetch_add(1, Ordering::SeqCst);
                    if headers
                        .get(header::AUTHORIZATION)
                        .is_some_and(|v| v == "Bearer busy")
                    {
                        (StatusCode::TOO_MANY_REQUESTS, [("retry-after", "20")], "").into_response()
                    } else {
                        (StatusCode::OK, LOCATION).into_response()
                    }
                }),
            )
            .with_state(state.clone()),
    )
    .await;

    assert!(matches!(
        esi.character_location("busy", CHARACTER).await,
        Err(EsiError::Api { status: 429, .. })
    ));
    assert!(matches!(
        esi.character_location("busy", CHARACTER).await,
        Err(EsiError::Backoff(_))
    ));
    // Another character's bucket is its own.
    assert!(esi.character_location("idle", CHARACTER + 1).await.is_ok());
    assert_eq!(state.hits.load(Ordering::SeqCst), 2);
}
