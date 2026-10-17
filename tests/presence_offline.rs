//! A pilot is on the map only while their user has the app open: once the poller stops
//! asking about a character (user idle, token refused), it is marked offline and leaves
//! the map instead of lingering where it last was.

mod common;

use std::time::{Duration, SystemTime};

use common::{SYS_A, add_character, member_with_role, world};
use sqlx::PgPool;
use wormholesystems::db::PgTokenStore;
use wormholesystems::esi::{Token, TokenStore};
use wormholesystems::maps::presence::characters_on;
use wormholesystems::maps::{Actor, Role};
use wormholesystems::pilots::poller::{Due, mark_unpolled_offline};

/// The character is online in SYS_A with a working token, tracked on the map.
async fn flying(pool: &PgPool, map_id: i64, actor: Actor) {
    PgTokenStore::new(pool.clone())
        .save(
            actor.character_id,
            &Token {
                access_token: "access".into(),
                refresh_token: "refresh".into(),
                expires_at: SystemTime::now() + Duration::from_secs(1200),
                scopes: vec![
                    "esi-location.read_online.v1".into(),
                    "esi-location.read_location.v1".into(),
                ],
            },
        )
        .await
        .unwrap();
    sqlx::query(
        "insert into character_status (character_id, online, solar_system_id) values ($1, true, $2)",
    )
    .bind(actor.character_id)
    .bind(SYS_A)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into map_user_settings (map_id, user_id, tracking_allowed) values ($1, $2, true)
         on conflict (map_id, user_id) do update set tracking_allowed = true",
    )
    .bind(map_id)
    .bind(actor.user_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn last_active(pool: &PgPool, user_id: i64, minutes_ago: i32) {
    sqlx::query(
        "update users set last_active_at = now() - make_interval(mins => $2) where id = $1",
    )
    .bind(user_id)
    .bind(minutes_ago)
    .execute(pool)
    .await
    .unwrap();
}

async fn on_map(pool: &PgPool, map_id: i64, viewer: Actor) -> Vec<i64> {
    characters_on(pool, map_id, viewer.user_id)
        .await
        .unwrap()
        .into_iter()
        .map(|c| c.character_id)
        .collect()
}

async fn online(pool: &PgPool, character_id: i64) -> bool {
    sqlx::query_scalar("select online from character_status where character_id = $1")
        .bind(character_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn an_idle_users_pilots_leave_the_map(pool: PgPool) {
    let w = world(&pool).await;
    let idle = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;
    flying(&pool, w.map_id, w.owner).await;
    flying(&pool, w.map_id, idle).await;
    last_active(&pool, w.owner.user_id, 1).await;
    last_active(&pool, idle.user_id, 6).await;
    assert_eq!(on_map(&pool, w.map_id, w.owner).await, vec![1001, 1002]);

    let gone = mark_unpolled_offline(&pool).await.unwrap();

    assert_eq!(
        gone,
        vec![Due {
            character_id: 1002,
            user_id: idle.user_id
        }]
    );
    assert_eq!(on_map(&pool, w.map_id, w.owner).await, vec![1001]);
    assert!(!online(&pool, 1002).await);
    // Said once: the next sweep has nothing left to flip.
    assert!(mark_unpolled_offline(&pool).await.unwrap().is_empty());
}

#[sqlx::test]
async fn a_user_never_seen_active_is_not_online(pool: PgPool) {
    let w = world(&pool).await;
    flying(&pool, w.map_id, w.owner).await;

    assert_eq!(mark_unpolled_offline(&pool).await.unwrap().len(), 1);
    assert!(!online(&pool, w.owner.character_id).await);
}

#[sqlx::test]
async fn a_refused_token_takes_the_pilot_off_the_map(pool: PgPool) {
    let w = world(&pool).await;
    flying(&pool, w.map_id, w.owner).await;
    last_active(&pool, w.owner.user_id, 0).await;
    // An alt of the same user, still polled.
    add_character(&pool, w.owner.user_id, 1003, 2001, None).await;
    let alt = Actor {
        user_id: w.owner.user_id,
        character_id: 1003,
    };
    flying(&pool, w.map_id, alt).await;
    assert!(
        PgTokenStore::new(pool.clone())
            .reject(w.owner.character_id)
            .await
            .unwrap()
    );

    let gone = mark_unpolled_offline(&pool).await.unwrap();

    assert_eq!(
        gone,
        vec![Due {
            character_id: 1001,
            user_id: w.owner.user_id
        }]
    );
    assert_eq!(on_map(&pool, w.map_id, w.owner).await, vec![1003]);
}
