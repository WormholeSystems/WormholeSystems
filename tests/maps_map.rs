//! Map lifecycle: create / update / delete / list / get.

mod common;

use common::{SYS_A, SYS_B, add_character, member_with_role, new_user, world};
use sqlx::PgPool;
use wormholesystems::allowlist::AffiliationAllowlist;
use wormholesystems::maps::access::effective_role;
use wormholesystems::maps::connection::{AddConnection, add_connection};
use wormholesystems::maps::map::{
    CreateMap, DeleteMap, GetMap, UpdateMap, create_map, delete_map, get_map, list_maps, update_map,
};
use wormholesystems::maps::solar_system::{AddSystem, add_system};
use wormholesystems::maps::{Actor, ConnectionType, MapError, Role, SubjectType};

#[sqlx::test]
async fn create_returns_fields_and_grants_owner(pool: PgPool) {
    common::seed_universe(&pool).await;
    let user = new_user(&pool).await;
    add_character(&pool, user, 1001, 2001, None).await;
    let actor = Actor {
        user_id: user,
        character_id: 1001,
    };

    let map = create_map(
        &pool,
        actor,
        &AffiliationAllowlist::default(),
        CreateMap {
            name: "  Home Chain  ".into(),
            description: Some("J-space".into()),
        },
    )
    .await
    .unwrap();

    // Returned exactly what we asked for (name trimmed), with a real id + timestamp.
    assert_eq!(map.name, "Home Chain");
    assert_eq!(map.description.as_deref(), Some("J-space"));
    assert!(map.id > 0);

    // Exactly one access row: owner, for the acting character.
    let grants: Vec<(SubjectType, i64, Role)> =
        sqlx::query_as("select subject_type, subject_id, role from map_access where map_id = $1")
            .bind(map.id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(grants, vec![(SubjectType::Character, 1001, Role::Owner)]);
    assert_eq!(
        effective_role(&pool, map.id, user).await.unwrap(),
        Some(Role::Owner)
    );
}

#[sqlx::test]
async fn create_rejects_blank_name_and_foreign_character(pool: PgPool) {
    common::seed_universe(&pool).await;
    let user = new_user(&pool).await;
    add_character(&pool, user, 1001, 2001, None).await;
    let actor = Actor {
        user_id: user,
        character_id: 1001,
    };

    assert!(matches!(
        create_map(
            &pool,
            actor,
            &AffiliationAllowlist::default(),
            CreateMap {
                name: "   ".into(),
                description: None,
            }
        )
        .await,
        Err(MapError::Validation(_)),
    ));
    let stranger = Actor {
        user_id: user,
        character_id: 9999,
    };
    assert!(matches!(
        create_map(
            &pool,
            stranger,
            &AffiliationAllowlist::default(),
            CreateMap {
                name: "Chain".into(),
                description: None,
            }
        )
        .await,
        Err(MapError::Forbidden),
    ));
    // Neither attempt created a map.
    let maps: i64 = sqlx::query_scalar("select count(*) from maps")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(maps, 0);
}

#[sqlx::test]
async fn update_is_owner_only_and_patches_fields(pool: PgPool) {
    let w = world(&pool).await;
    let member = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;

    // Member can't modify the map itself.
    assert!(matches!(
        update_map(
            &pool,
            member,
            UpdateMap {
                map_id: w.map_id,
                name: Some("X".into()),
                ..Default::default()
            },
        )
        .await,
        Err(MapError::Forbidden),
    ));

    // Owner renames and sets a description.
    let updated = update_map(
        &pool,
        w.owner,
        UpdateMap {
            map_id: w.map_id,
            name: Some("Renamed".into()),
            description: Some(Some("home".into())),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "Renamed");
    assert_eq!(updated.description.as_deref(), Some("home"));

    // Omitted field unchanged; Some(None) clears.
    let cleared = update_map(
        &pool,
        w.owner,
        UpdateMap {
            map_id: w.map_id,
            description: Some(None),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(cleared.name, "Renamed", "name left untouched");
    assert_eq!(cleared.description, None);

    // Persisted to the row, not just the return value.
    let (name, desc): (String, Option<String>) =
        sqlx::query_as("select name, description from maps where id = $1")
            .bind(w.map_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(name, "Renamed");
    assert_eq!(desc, None);

    // Blank name rejected.
    assert!(matches!(
        update_map(
            &pool,
            w.owner,
            UpdateMap {
                map_id: w.map_id,
                name: Some(" ".into()),
                ..Default::default()
            },
        )
        .await,
        Err(MapError::Validation(_)),
    ));
}

#[sqlx::test]
async fn delete_is_owner_only_and_cascades(pool: PgPool) {
    let w = world(&pool).await;
    let member = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;
    let a = add_system(
        &pool,
        member,
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
    let b = add_system(
        &pool,
        member,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
            x: 1.0,
            y: 1.0,
            alias: None,
        },
    )
    .await
    .unwrap();
    add_connection(
        &pool,
        member,
        AddConnection {
            map_id: w.map_id,
            from_system: a.id,
            to_system: b.id,
            kind: ConnectionType::Wormhole,
            size: None,
        },
    )
    .await
    .unwrap();

    assert!(matches!(
        delete_map(&pool, member, DeleteMap { map_id: w.map_id }).await,
        Err(MapError::Forbidden),
    ));

    delete_map(&pool, w.owner, DeleteMap { map_id: w.map_id })
        .await
        .unwrap();
    for table in ["maps", "map_solar_systems", "map_connections", "map_access"] {
        let col = if table == "maps" { "id" } else { "map_id" };
        // sqlx 0.9 guards runtime query strings against injection; this one is built from a
        // hardcoded table/column list, so assert it's safe.
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "select count(*) from {table} where {col} = $1"
        )))
        .bind(w.map_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(n, 0, "{table} should be empty after delete");
    }
}

#[sqlx::test]
async fn list_returns_accessible_maps_with_highest_role(pool: PgPool) {
    use wormholesystems::maps::SubjectType;
    use wormholesystems::maps::access::{SetAccess, set_access};
    let w = world(&pool).await;

    // A second user, member via their corporation, viewer via their character — member wins.
    let user = new_user(&pool).await;
    add_character(&pool, user, 1002, 2002, None).await;
    set_access(
        &pool,
        w.owner,
        SetAccess {
            map_id: w.map_id,
            subject_type: SubjectType::Corporation,
            subject_id: 2002,
            role: Role::Member,
            expires_at: None,
        },
    )
    .await
    .unwrap();
    set_access(
        &pool,
        w.owner,
        SetAccess {
            map_id: w.map_id,
            subject_type: SubjectType::Character,
            subject_id: 1002,
            role: Role::Viewer,
            expires_at: None,
        },
    )
    .await
    .unwrap();

    let maps = list_maps(&pool, user).await.unwrap();
    assert_eq!(maps.len(), 1);
    assert_eq!(maps[0].0.id, w.map_id);
    assert_eq!(maps[0].1, Role::Member);

    // A user with no grants sees nothing.
    let outsider = new_user(&pool).await;
    add_character(&pool, outsider, 1003, 2003, None).await;
    assert!(list_maps(&pool, outsider).await.unwrap().is_empty());
}

#[sqlx::test]
async fn get_map_returns_exact_graph(pool: PgPool) {
    let w = world(&pool).await;
    let a = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            x: 1.0,
            y: 2.0,
            alias: Some("Home".into()),
        },
    )
    .await
    .unwrap();
    let b = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
            x: 3.0,
            y: 4.0,
            alias: None,
        },
    )
    .await
    .unwrap();
    let conn = add_connection(
        &pool,
        w.owner,
        AddConnection {
            map_id: w.map_id,
            from_system: a.id,
            to_system: b.id,
            kind: ConnectionType::Stargate,
            size: None,
        },
    )
    .await
    .unwrap();

    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.map.id, w.map_id);
    assert_eq!(view.systems.len(), 2);
    assert_eq!(view.systems[0].solar_system_id(), Some(SYS_A));
    assert_eq!(view.systems[0].position(), (1.0, 2.0));
    assert_eq!(view.systems[0].alias(), Some("Home"));
    assert_eq!(view.connections.len(), 1);
    assert_eq!(view.connections[0].id, conn.id);
    assert_eq!(view.connections[0].from_system, a.id);
    assert_eq!(view.connections[0].to_system, b.id);
    assert_eq!(view.connections[0].kind, ConnectionType::Stargate);

    // No access at all → NotFound (existence not leaked), not Forbidden.
    let outsider = new_user(&pool).await;
    add_character(&pool, outsider, 1003, 2003, None).await;
    let stranger = Actor {
        user_id: outsider,
        character_id: 1003,
    };
    assert!(matches!(
        get_map(&pool, stranger, GetMap { map_id: w.map_id }).await,
        Err(MapError::NotFound),
    ));
}
