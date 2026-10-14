//! Shared fixtures for the map action tests. Lives in `tests/common/` so it is compiled
//! as a helper module (not its own test binary); each test file does `mod common;`.
#![allow(dead_code)]

pub mod app;

use sqlx::PgPool;
use wormholesystems::allowlist::AffiliationAllowlist;
use wormholesystems::maps::access::SetAccess;
use wormholesystems::maps::access::set_access;
use wormholesystems::maps::map::{CreateMap, create_map};
use wormholesystems::maps::{Actor, Role, SubjectType};

// Three solar systems are seeded so systems can be placed and connected.
pub const SYS_A: i64 = 30000142;
pub const SYS_B: i64 = 30000144;
pub const SYS_C: i64 = 30000145;

/// A region → constellation → three solar systems.
pub async fn seed_universe(pool: &PgPool) {
    sqlx::query("insert into regions (id, name) values (10000001, 'Test Region')")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "insert into constellations (id, region_id, name) values (20000001, 10000001, 'Test Const')",
    )
    .execute(pool)
    .await
    .unwrap();
    for (id, name) in [
        (SYS_A, "Jita"),
        (SYS_B, "Perimeter"),
        (SYS_C, "New Caldari"),
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
}

pub async fn new_user(pool: &PgPool) -> i64 {
    sqlx::query_scalar("insert into users default values returning id")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Insert a character (and the corp/alliance it references, to satisfy the FKs).
pub async fn add_character(pool: &PgPool, user_id: i64, id: i64, corp: i64, alliance: Option<i64>) {
    sqlx::query("insert into corporations (id, name, ticker) values ($1, $2, 'CORP') on conflict (id) do nothing")
        .bind(corp)
        .bind(format!("Corp {corp}"))
        .execute(pool)
        .await
        .unwrap();
    if let Some(a) = alliance {
        sqlx::query("insert into alliances (id, name, ticker) values ($1, $2, 'ALLY') on conflict (id) do nothing")
            .bind(a)
            .bind(format!("Alliance {a}"))
            .execute(pool)
            .await
            .unwrap();
    }
    sqlx::query(
        "insert into characters (id, user_id, name, owner_hash, corporation_id, alliance_id)
         values ($1, $2, $3, 'hash', $4, $5)",
    )
    .bind(id)
    .bind(user_id)
    .bind(format!("Char {id}"))
    .bind(corp)
    .bind(alliance)
    .execute(pool)
    .await
    .unwrap();
}

/// An owner (character 1001, corp 2001) with a freshly created map, on a seeded universe.
pub struct World {
    pub owner: Actor,
    pub map_id: i64,
}

pub async fn world(pool: &PgPool) -> World {
    seed_universe(pool).await;
    let user = new_user(pool).await;
    add_character(pool, user, 1001, 2001, None).await;
    let owner = Actor {
        user_id: user,
        character_id: 1001,
    };
    let map = create_map(
        pool,
        owner,
        &AffiliationAllowlist::default(),
        CreateMap {
            name: "Chain".into(),
            description: None,
        },
    )
    .await
    .unwrap();
    World {
        owner,
        map_id: map.id,
    }
}

/// Add a fresh user + character and grant them `role` on the map; return their actor.
pub async fn member_with_role(
    pool: &PgPool,
    granter: Actor,
    map_id: i64,
    char_id: i64,
    corp: i64,
    role: Role,
) -> Actor {
    let user = new_user(pool).await;
    add_character(pool, user, char_id, corp, None).await;
    set_access(
        pool,
        granter,
        SetAccess {
            map_id,
            subject_type: SubjectType::Character,
            subject_id: char_id,
            role,
            expires_at: None,
        },
    )
    .await
    .unwrap();
    Actor {
        user_id: user,
        character_id: char_id,
    }
}
