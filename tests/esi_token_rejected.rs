//! A refresh token EVE refuses (`invalid_grant`) is dead for good: it is marked once, never
//! sent to the SSO again, shown to its pilot, and a fresh login clears it.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

use axum::Router;
use axum::http::StatusCode;
use axum::routing::post;
use sqlx::PgPool;
use wormholesystems::db::PgTokenStore;
use wormholesystems::esi::jwt::Claims;
use wormholesystems::esi::sso::{Sso, SsoConfig};
use wormholesystems::esi::{EsiError, Token, TokenStore};
use wormholesystems::maps::Actor;
use wormholesystems::session::{Entity, persist_identity};

const CHARACTER: i64 = 100;

/// An SSO whose token endpoint refuses every refresh, and how often it was asked.
async fn refusing_sso() -> (Sso, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    let app = Router::new().route(
        "/token",
        post(move || {
            counter.fetch_add(1, Ordering::SeqCst);
            async {
                (
                    StatusCode::BAD_REQUEST,
                    r#"{"error":"invalid_grant","error_description":"Invalid refresh token."}"#,
                )
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let config = SsoConfig {
        client_id: "test".into(),
        client_secret: "test".into(),
        redirect_uri: "http://localhost/auth/callback".into(),
        scopes: Vec::new(),
    };
    (Sso::stub_at(reqwest::Client::new(), config, &base), hits)
}

fn token(expires_at: SystemTime) -> Token {
    Token {
        access_token: "access".into(),
        refresh_token: "refresh".into(),
        expires_at,
        scopes: vec!["esi-location.read_online.v1".into()],
    }
}

async fn signed_in(pool: &PgPool) -> Actor {
    let claims = Claims {
        character_id: CHARACTER,
        name: "Pilot".into(),
        owner_hash: "h1".into(),
        scopes: vec![],
    };
    let corp = Entity {
        id: 2001,
        name: "Corp".into(),
        ticker: "CRP".into(),
    };
    let user_id = persist_identity(pool, &claims, corp, None, None)
        .await
        .unwrap();
    Actor {
        user_id,
        character_id: CHARACTER,
    }
}

async fn needs_login(pool: &PgPool, actor: Actor) -> bool {
    wormholesystems::identity::characters::list(pool, actor)
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.character_id == CHARACTER)
        .unwrap()
        .needs_login
}

#[sqlx::test]
async fn a_refused_refresh_is_marked_once_and_not_retried(pool: PgPool) {
    let actor = signed_in(&pool).await;
    let store = PgTokenStore::new(pool.clone());
    store
        .save(CHARACTER, &token(SystemTime::UNIX_EPOCH))
        .await
        .unwrap();
    let (sso, hits) = refusing_sso().await;
    assert!(!needs_login(&pool, actor).await);

    let first = sso.stored_token(&store, CHARACTER).await;
    assert!(matches!(first, Err(EsiError::TokenRejected)), "{first:?}");
    assert!(needs_login(&pool, actor).await);

    // The next ask is answered from the store, without bothering EVE.
    let again = sso.stored_token(&store, CHARACTER).await;
    assert!(matches!(again, Err(EsiError::TokenRejected)), "{again:?}");
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    assert!(!store.reject(CHARACTER).await.unwrap(), "already marked");
}

#[sqlx::test]
async fn a_fresh_login_clears_the_mark(pool: PgPool) {
    let actor = signed_in(&pool).await;
    let store = PgTokenStore::new(pool.clone());
    store
        .save(CHARACTER, &token(SystemTime::UNIX_EPOCH))
        .await
        .unwrap();
    assert!(store.reject(CHARACTER).await.unwrap());
    assert!(needs_login(&pool, actor).await);

    // What the SSO callback does with the new token.
    store
        .save(
            CHARACTER,
            &token(SystemTime::now() + Duration::from_secs(1200)),
        )
        .await
        .unwrap();
    assert!(!needs_login(&pool, actor).await);
    let (sso, hits) = refusing_sso().await;
    let fresh = sso.stored_token(&store, CHARACTER).await.unwrap();
    assert_eq!(fresh.access_token, "access");
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}
