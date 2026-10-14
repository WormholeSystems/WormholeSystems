//! The periodic affiliation refresh: a character that changes corporation stops matching
//! the old corporation's grants without logging in again.

mod common;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Json, Path, State};
use axum::routing::{get, post};
use common::{add_character, new_user, world};
use sqlx::PgPool;
use wormholesystems::affiliation::refresh;
use wormholesystems::esi::EsiClient;
use wormholesystems::maps::access::{SetAccess, effective_role, set_access};
use wormholesystems::maps::{Role, SubjectType};

const OLD_CORP: i64 = 5000;
const NEW_CORP: i64 = 6000;
const NEW_ALLIANCE: i64 = 7000;

/// Every batch of ids the stub was asked about.
type Batches = Arc<Mutex<Vec<Vec<i64>>>>;

/// A stand-in ESI where everyone has just joined `NEW_CORP` in `NEW_ALLIANCE`.
async fn stub_esi() -> (EsiClient, Batches) {
    let batches: Batches = Arc::default();
    let app = Router::new()
        .route("/characters/affiliation", post(affiliation))
        .route("/corporations/{id}", get(corporation))
        .route("/alliances/{id}", get(alliance))
        .with_state(batches.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (
        EsiClient::with_config(reqwest::Client::new(), base, "2025-08-26"),
        batches,
    )
}

async fn affiliation(
    State(batches): State<Batches>,
    Json(ids): Json<Vec<i64>>,
) -> Json<serde_json::Value> {
    batches.lock().unwrap().push(ids.clone());
    Json(
        ids.iter()
            .map(|id| {
                serde_json::json!({
                    "character_id": id,
                    "corporation_id": NEW_CORP,
                    "alliance_id": NEW_ALLIANCE,
                })
            })
            .collect(),
    )
}

async fn corporation(Path(id): Path<i64>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": format!("Corp {id}"), "ticker": "NEW", "member_count": 1, "ceo_id": 1,
        "creator_id": 1, "tax_rate": 0.1,
    }))
}

async fn alliance(Path(id): Path<i64>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": format!("Alliance {id}"), "ticker": "ALLY", "creator_corporation_id": NEW_CORP,
        "creator_id": 1, "date_founded": "2020-01-01T00:00:00Z",
    }))
}

#[sqlx::test]
async fn leaving_a_granted_corporation_loses_its_access(pool: PgPool) {
    let w = world(&pool).await;
    set_access(
        &pool,
        w.owner,
        SetAccess {
            map_id: w.map_id,
            subject_type: SubjectType::Corporation,
            subject_id: OLD_CORP,
            role: Role::Member,
            expires_at: None,
        },
    )
    .await
    .unwrap();
    let user = new_user(&pool).await;
    add_character(&pool, user, 1002, OLD_CORP, None).await;
    assert_eq!(
        effective_role(&pool, w.map_id, user).await.unwrap(),
        Some(Role::Member)
    );

    let (esi, _) = stub_esi().await;
    // Both account characters moved: the owner's and this one.
    assert_eq!(refresh(&pool, &esi).await.unwrap(), 2);

    assert_eq!(effective_role(&pool, w.map_id, user).await.unwrap(), None);
    let (corp, alliance): (Option<i64>, Option<i64>) =
        sqlx::query_as("select corporation_id, alliance_id from characters where id = 1002")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((corp, alliance), (Some(NEW_CORP), Some(NEW_ALLIANCE)));

    // The new corporation and alliance are named, so the character still reads as something.
    let named: bool = sqlx::query_scalar(
        "select exists(select 1 from corporations where id = $1)
            and exists(select 1 from alliances where id = $2)",
    )
    .bind(NEW_CORP)
    .bind(NEW_ALLIANCE)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(named);

    // Nothing moved the second time round.
    assert_eq!(refresh(&pool, &esi).await.unwrap(), 0);
}

#[sqlx::test]
async fn account_characters_go_in_batches_of_a_thousand(pool: PgPool) {
    let user = new_user(&pool).await;
    sqlx::query("insert into corporations (id, name, ticker) values ($1, 'Old Corp', 'OLD')")
        .bind(OLD_CORP)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "insert into characters (id, user_id, name, corporation_id)
         select g, $1, 'Char ' || g, $2 from generate_series(1, 1001) g",
    )
    .bind(user)
    .bind(OLD_CORP)
    .execute(&pool)
    .await
    .unwrap();
    // Named from a killmail, never signed in: nobody's access hangs on it.
    sqlx::query("insert into characters (id, name) values (9000001, 'Stranger')")
        .execute(&pool)
        .await
        .unwrap();

    let (esi, batches) = stub_esi().await;
    assert_eq!(refresh(&pool, &esi).await.unwrap(), 1001);

    let batches = batches.lock().unwrap();
    let sizes: Vec<usize> = batches.iter().map(Vec::len).collect();
    assert_eq!(sizes, vec![1000, 1]);
    assert!(!batches.iter().flatten().any(|id| *id == 9000001));
}
