//! Limiting map creation to named affiliations (`MAP_CREATOR_AFFILIATION_IDS`): who gets
//! through, that it is the active character that counts, and that every way of creating a
//! map and the maps page's button agree.

mod common;

use axum::http::StatusCode;
use common::app::{get, request_json, session_cookie, state};
use common::{add_character, new_user, seed_universe, world};
use serde_json::json;
use sqlx::PgPool;
use wormholesystems::allowlist::AffiliationAllowlist;
use wormholesystems::maps::map::{CreateMap, GetMap, create_map, get_map};
use wormholesystems::maps::transfer::{SectionSet, export_map, import_map_as_new, parse_export};
use wormholesystems::maps::{Actor, MapError};

const CORP: i64 = 2001;
const ALLIANCE: i64 = 3001;

fn only(ids: &[i64]) -> AffiliationAllowlist {
    AffiliationAllowlist::new(ids.iter().copied())
}

/// A user flying one character in [`CORP`], which is in [`ALLIANCE`].
async fn pilot(pool: &PgPool) -> Actor {
    seed_universe(pool).await;
    let user = new_user(pool).await;
    add_character(pool, user, 1001, CORP, Some(ALLIANCE)).await;
    Actor {
        user_id: user,
        character_id: 1001,
    }
}

async fn create(
    pool: &PgPool,
    actor: Actor,
    creators: &AffiliationAllowlist,
    name: &str,
) -> Result<i64, MapError> {
    create_map(
        pool,
        actor,
        creators,
        CreateMap {
            name: name.into(),
            description: None,
        },
    )
    .await
    .map(|map| map.id)
}

async fn map_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("select count(*) from maps")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn anyone_may_create_a_map_when_nothing_is_configured(pool: PgPool) {
    let actor = pilot(&pool).await;
    assert!(create(&pool, actor, &only(&[]), "Home").await.is_ok());
}

#[sqlx::test]
async fn a_listed_character_corporation_or_alliance_may_create_a_map(pool: PgPool) {
    let actor = pilot(&pool).await;
    for id in [1001, CORP, ALLIANCE] {
        assert!(
            create(&pool, actor, &only(&[id]), "Home").await.is_ok(),
            "{id} was refused"
        );
    }
    assert_eq!(map_count(&pool).await, 3);
}

#[sqlx::test]
async fn anyone_else_is_refused(pool: PgPool) {
    let actor = pilot(&pool).await;
    assert!(matches!(
        create(&pool, actor, &only(&[99_999_999]), "Home").await,
        Err(MapError::Forbidden)
    ));
    assert_eq!(map_count(&pool).await, 0);
}

// An account can hold a director and an alt in an NPC corporation. Flying the alt should
// not inherit the director's right to create maps.
#[sqlx::test]
async fn it_is_the_active_character_that_counts(pool: PgPool) {
    let director = pilot(&pool).await;
    add_character(&pool, director.user_id, 1002, 1_000_125, None).await;
    let alt = Actor {
        character_id: 1002,
        ..director
    };
    let creators = only(&[CORP]);

    assert!(matches!(
        create(&pool, alt, &creators, "From the alt").await,
        Err(MapError::Forbidden)
    ));
    assert!(
        create(&pool, director, &creators, "From the director")
            .await
            .is_ok()
    );
    assert_eq!(map_count(&pool).await, 1);
}

// The list governs creation only: a user who may not create a map still works on the ones
// they were given.
#[sqlx::test]
async fn it_does_not_touch_maps_that_already_exist(pool: PgPool) {
    let w = world(&pool).await;
    assert!(
        get_map(&pool, w.owner, GetMap { map_id: w.map_id })
            .await
            .is_ok()
    );
}

#[sqlx::test]
async fn importing_a_file_as_a_new_map_is_limited_the_same_way(pool: PgPool) {
    let w = world(&pool).await;
    let sections = || SectionSet::from_names(&["solarsystems".into()]).unwrap();
    let file = export_map(&pool, w.owner, w.map_id, sections())
        .await
        .unwrap();
    let content = serde_json::to_string(&file).unwrap();

    let refused = import_map_as_new(
        &pool,
        w.owner,
        &only(&[99_999_999]),
        parse_export(&content, sections(), true).unwrap(),
        None,
    )
    .await;
    assert!(matches!(refused, Err(MapError::Forbidden)));
    assert_eq!(map_count(&pool).await, 1);

    import_map_as_new(
        &pool,
        w.owner,
        &only(&[w.owner.character_id]),
        parse_export(&content, sections(), true).unwrap(),
        None,
    )
    .await
    .unwrap();
    assert_eq!(map_count(&pool).await, 2);
}

/// The router with map creation limited to `creators`.
fn app_with(pool: &PgPool, creators: &[i64]) -> axum::Router {
    let mut state = state(pool);
    state.map_creators = only(creators);
    wormholesystems::api::router().with_state(state)
}

// The maps page hides its create buttons from whoever would only get a 403 for pressing them.
#[sqlx::test]
async fn the_signed_in_character_says_whether_it_may_create_a_map(pool: PgPool) {
    let actor = pilot(&pool).await;
    let cookie = session_cookie(&pool, actor).await;

    let open = get(app_with(&pool, &[]), "/api/me", Some(&cookie)).await;
    assert_eq!(open.body["can_create_map"], true);

    let listed = get(app_with(&pool, &[CORP]), "/api/me", Some(&cookie)).await;
    assert_eq!(listed.body["can_create_map"], true);

    let closed = get(app_with(&pool, &[99_999_999]), "/api/me", Some(&cookie)).await;
    assert_eq!(closed.body["can_create_map"], false);
}

#[sqlx::test]
async fn creating_over_http_is_forbidden_off_the_list(pool: PgPool) {
    let actor = pilot(&pool).await;
    let cookie = session_cookie(&pool, actor).await;

    let res = request_json(
        app_with(&pool, &[99_999_999]),
        "POST",
        "/api/maps",
        Some(&cookie),
        json!({ "name": "Home" }),
    )
    .await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);

    let res = request_json(
        app_with(&pool, &[ALLIANCE]),
        "POST",
        "/api/maps",
        Some(&cookie),
        json!({ "name": "Home" }),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
}
