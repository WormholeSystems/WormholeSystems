//! Solar systems: placing / moving / removing / aliasing systems on a map.

mod common;

use common::{SYS_A, SYS_B, SYS_C, member_with_role, world};
use sqlx::PgPool;
use wormholesystems::maps::events_log::{MapIdBody, redo, undo};
use wormholesystems::maps::solar_system::{
    AddSystem, MoveSystem, MoveSystems, RemoveSystem, RemoveSystems, SetAlias, SetHome, SetPinned,
    SetRally, SystemMove, add_system, move_system, move_systems, remove_system, remove_systems,
    set_alias, set_home, set_pinned, set_rally,
};
use wormholesystems::maps::{MapError, Role};

#[sqlx::test]
async fn add_system_returns_fields_and_persists(pool: PgPool) {
    let w = world(&pool).await;
    let viewer = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Viewer).await;

    // A viewer can't edit the graph.
    assert!(matches!(
        add_system(
            &pool,
            viewer,
            AddSystem {
                map_id: w.map_id,
                solar_system_id: SYS_A,
                x: 0.0,
                y: 0.0,
                alias: None
            }
        )
        .await,
        Err(MapError::Forbidden),
    ));

    let placed = add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_A,
            x: 1.5,
            y: -2.5,
            alias: Some("Staging".into()),
        },
    )
    .await
    .unwrap();
    // Return value reflects the inputs exactly.
    assert_eq!(placed.map_id, w.map_id);
    assert_eq!(placed.solar_system_id, Some(SYS_A));
    assert_eq!((placed.position_x, placed.position_y), (1.5, -2.5));
    assert_eq!(placed.alias.as_deref(), Some("Staging"));
    // And the row matches.
    let (sid, x, y, alias): (i64, f64, f64, Option<String>) = sqlx::query_as(
        "select solar_system_id, position_x, position_y, alias from map_solar_systems where id = $1",
    )
    .bind(placed.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (sid, x, y, alias.as_deref()),
        (SYS_A, 1.5, -2.5, Some("Staging"))
    );

    // Duplicate placement → Conflict.
    assert!(matches!(
        add_system(
            &pool,
            w.owner,
            AddSystem {
                map_id: w.map_id,
                solar_system_id: SYS_A,
                x: 5.0,
                y: 5.0,
                alias: None
            }
        )
        .await,
        Err(MapError::Conflict(_)),
    ));
    // Unknown system id → Validation.
    assert!(matches!(
        add_system(
            &pool,
            w.owner,
            AddSystem {
                map_id: w.map_id,
                solar_system_id: 88888888,
                x: 0.0,
                y: 0.0,
                alias: None
            }
        )
        .await,
        Err(MapError::Validation(_)),
    ));
}

#[sqlx::test]
async fn remove_system_keeps_details(pool: PgPool) {
    let w = world(&pool).await;
    let placed = add_system(
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
    sqlx::query(
        "insert into map_solar_system_details (map_id, solar_system_id, status) values ($1, $2, 'hostile')",
    )
    .bind(w.map_id)
    .bind(SYS_A)
    .execute(&pool)
    .await
    .unwrap();

    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: placed.id,
        },
    )
    .await
    .unwrap();

    let placements: i64 =
        sqlx::query_scalar("select count(*) from map_solar_systems where map_id = $1")
            .bind(w.map_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(placements, 0);
    let details: i64 = sqlx::query_scalar(
        "select count(*) from map_solar_system_details where map_id = $1 and solar_system_id = $2",
    )
    .bind(w.map_id)
    .bind(SYS_A)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details, 1, "details should survive system removal");

    // A placement not on this map → NotFound.
    assert!(matches!(
        remove_system(
            &pool,
            w.owner,
            RemoveSystem {
                map_id: w.map_id,
                map_solar_system_id: 4242
            }
        )
        .await,
        Err(MapError::NotFound),
    ));
}

#[sqlx::test]
async fn move_and_alias_update_the_row(pool: PgPool) {
    let w = world(&pool).await;
    let placed = add_system(
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

    move_system(
        &pool,
        w.owner,
        MoveSystem {
            map_id: w.map_id,
            map_solar_system_id: placed.id,
            x: 3.0,
            y: 4.0,
        },
    )
    .await
    .unwrap();
    set_alias(
        &pool,
        w.owner,
        SetAlias {
            map_id: w.map_id,
            map_solar_system_id: placed.id,
            alias: Some("Home".into()),
        },
    )
    .await
    .unwrap();

    let (x, y, alias): (f64, f64, Option<String>) =
        sqlx::query_as("select position_x, position_y, alias from map_solar_systems where id = $1")
            .bind(placed.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((x, y), (3.0, 4.0));
    assert_eq!(alias.as_deref(), Some("Home"));

    // Clearing the alias.
    set_alias(
        &pool,
        w.owner,
        SetAlias {
            map_id: w.map_id,
            map_solar_system_id: placed.id,
            alias: None,
        },
    )
    .await
    .unwrap();
    let alias: Option<String> =
        sqlx::query_scalar("select alias from map_solar_systems where id = $1")
            .bind(placed.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(alias, None);

    // Unknown placement → NotFound.
    assert!(matches!(
        move_system(
            &pool,
            w.owner,
            MoveSystem {
                map_id: w.map_id,
                map_solar_system_id: 4242,
                x: 1.0,
                y: 1.0
            }
        )
        .await,
        Err(MapError::NotFound),
    ));
}

/// What "pinned" and "home" protect, exactly.
///
/// Pinned means the system does not move and does not get deleted. Home means it does not
/// get deleted, but it can still be dragged around. Neither is an error to try: a marquee
/// across the chain is the same gesture as selecting one system, and refusing the whole
/// delete because the box happened to contain the home system would punish the gesture.
#[sqlx::test]
async fn pinned_and_home_systems_are_passed_over_rather_than_refused(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let pinned = place(&pool, w.owner, w.map_id, SYS_B, 200.0).await;
    let ordinary = place(&pool, w.owner, w.map_id, SYS_C, 400.0).await;
    mark_home(&pool, w.owner, w.map_id, home).await;
    pin(&pool, w.owner, w.map_id, pinned, true).await;

    // A selection spanning all three takes only the one that is not protected, and says so
    // by succeeding rather than by erroring.
    let removed = remove_systems(
        &pool,
        w.owner,
        RemoveSystems {
            map_id: w.map_id,
            map_solar_system_ids: vec![home, pinned, ordinary],
        },
    )
    .await
    .unwrap();
    assert_eq!(removed, 1);
    assert_eq!(placed_ids(&pool, w.owner, w.map_id).await.len(), 2);

    // Selecting only protected systems is a quiet no-op, not a failure.
    let removed = remove_systems(
        &pool,
        w.owner,
        RemoveSystems {
            map_id: w.map_id,
            map_solar_system_ids: vec![home, pinned],
        },
    )
    .await
    .unwrap();
    assert_eq!(removed, 0);
    assert_eq!(placed_ids(&pool, w.owner, w.map_id).await.len(), 2);

    // Asking for one of them on its own is the same: nothing happens, nothing complains.
    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: home,
        },
    )
    .await
    .unwrap();
    assert_eq!(placed_ids(&pool, w.owner, w.map_id).await.len(), 2);
}

#[sqlx::test]
async fn unpinning_gives_the_system_back(pool: PgPool) {
    let w = world(&pool).await;
    let pinned = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    pin(&pool, w.owner, w.map_id, pinned, true).await;

    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: pinned,
        },
    )
    .await
    .unwrap();
    assert_eq!(placed_ids(&pool, w.owner, w.map_id).await.len(), 1);

    // A guard, not a life sentence.
    pin(&pool, w.owner, w.map_id, pinned, false).await;
    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: pinned,
        },
    )
    .await
    .unwrap();
    assert!(placed_ids(&pool, w.owner, w.map_id).await.is_empty());
}

#[sqlx::test]
async fn a_pinned_system_will_not_move_but_the_home_system_will(pool: PgPool) {
    let w = world(&pool).await;
    let home = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let pinned = place(&pool, w.owner, w.map_id, SYS_B, 200.0).await;
    mark_home(&pool, w.owner, w.map_id, home).await;
    pin(&pool, w.owner, w.map_id, pinned, true).await;

    // Being home says nothing about moving.
    move_system(
        &pool,
        w.owner,
        MoveSystem {
            map_id: w.map_id,
            map_solar_system_id: home,
            x: 640.0,
            y: 80.0,
        },
    )
    .await
    .unwrap();
    assert_eq!(
        position(&pool, w.owner, w.map_id, home).await,
        (640.0, 80.0)
    );

    // Pinning does, and the server holds it rather than trusting the client's drag lock.
    move_system(
        &pool,
        w.owner,
        MoveSystem {
            map_id: w.map_id,
            map_solar_system_id: pinned,
            x: 900.0,
            y: 900.0,
        },
    )
    .await
    .unwrap();
    assert_eq!(
        position(&pool, w.owner, w.map_id, pinned).await,
        (200.0, 0.0)
    );

    // Dragging a multi-selection moves everything in it except the pinned one.
    move_systems(
        &pool,
        w.owner,
        MoveSystems {
            map_id: w.map_id,
            moves: vec![
                SystemMove {
                    map_solar_system_id: home,
                    x: 20.0,
                    y: 20.0,
                },
                SystemMove {
                    map_solar_system_id: pinned,
                    x: 20.0,
                    y: 20.0,
                },
            ],
        },
    )
    .await
    .unwrap();
    assert_eq!(position(&pool, w.owner, w.map_id, home).await, (20.0, 20.0));
    assert_eq!(
        position(&pool, w.owner, w.map_id, pinned).await,
        (200.0, 0.0)
    );
}

async fn mark_home(pool: &PgPool, actor: wormholesystems::maps::Actor, map_id: i64, id: i64) {
    set_home(
        pool,
        actor,
        SetHome {
            map_id,
            map_solar_system_id: id,
            value: true,
        },
    )
    .await
    .unwrap();
}

async fn pin(
    pool: &PgPool,
    actor: wormholesystems::maps::Actor,
    map_id: i64,
    id: i64,
    value: bool,
) {
    set_pinned(
        pool,
        actor,
        SetPinned {
            map_id,
            map_solar_system_id: id,
            value,
        },
    )
    .await
    .unwrap();
}

async fn position(
    pool: &PgPool,
    actor: wormholesystems::maps::Actor,
    map_id: i64,
    id: i64,
) -> (f64, f64) {
    let view = wormholesystems::maps::map::get_map(
        pool,
        actor,
        wormholesystems::maps::map::GetMap { map_id },
    )
    .await
    .unwrap();
    let s = view.systems.iter().find(|s| s.id() == id).expect("placed");
    s.position()
}

async fn place(
    pool: &PgPool,
    actor: wormholesystems::maps::Actor,
    map_id: i64,
    sys: i64,
    x: f64,
) -> i64 {
    add_system(
        pool,
        actor,
        AddSystem {
            map_id,
            solar_system_id: sys,
            x,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn placed_ids(pool: &PgPool, actor: wormholesystems::maps::Actor, map_id: i64) -> Vec<i64> {
    wormholesystems::maps::map::get_map(pool, actor, wormholesystems::maps::map::GetMap { map_id })
        .await
        .unwrap()
        .systems
        .iter()
        .map(|s| s.id())
        .collect()
}

/// Home is one-per-map, and undo puts the previous holder back rather than leaving the map
/// with none.
#[sqlx::test]
async fn a_map_has_one_home_and_undo_returns_the_last_one(pool: PgPool) {
    let w = world(&pool).await;
    let first = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let second = place(&pool, w.owner, w.map_id, SYS_B, 200.0).await;

    mark_home(&pool, w.owner, w.map_id, first).await;
    mark_home(&pool, w.owner, w.map_id, second).await;
    assert_eq!(home_of(&pool, w.map_id).await, Some(second));

    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(home_of(&pool, w.map_id).await, Some(first));

    // A rally point is its own mark: setting one leaves the home system alone.
    rally(&pool, w.owner, w.map_id, second, true).await.unwrap();
    assert_eq!(home_of(&pool, w.map_id).await, Some(first));
}

/// A map holds two rally points, oldest first. A third replaces the oldest, and undo
/// brings that one back in its old place, so the next eviction still picks it.
#[sqlx::test]
async fn a_third_rally_point_replaces_the_oldest_and_undo_brings_it_back(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 200.0).await;
    let c = place(&pool, w.owner, w.map_id, SYS_C, 400.0).await;

    rally(&pool, w.owner, w.map_id, a, true).await.unwrap();
    rally(&pool, w.owner, w.map_id, b, true).await.unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![a, b]);

    rally(&pool, w.owner, w.map_id, c, true).await.unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![b, c]);

    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![a, b]);
    redo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![b, c]);

    // Setting one that already is a rally point makes it the newest instead of adding it
    // twice.
    rally(&pool, w.owner, w.map_id, b, true).await.unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![c, b]);
}

/// Clearing a rally point clears only that one, and clearing a system that is not one
/// changes nothing.
#[sqlx::test]
async fn a_rally_point_clears_on_its_own(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 200.0).await;
    let c = place(&pool, w.owner, w.map_id, SYS_C, 400.0).await;
    rally(&pool, w.owner, w.map_id, a, true).await.unwrap();
    rally(&pool, w.owner, w.map_id, b, true).await.unwrap();

    rally(&pool, w.owner, w.map_id, a, false).await.unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![b]);
    rally(&pool, w.owner, w.map_id, c, false).await.unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![b]);

    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![a, b]);

    let viewer = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Viewer).await;
    assert!(matches!(
        rally(&pool, viewer, w.map_id, c, true).await,
        Err(MapError::Forbidden),
    ));
    assert!(matches!(
        rally(&pool, viewer, w.map_id, a, false).await,
        Err(MapError::Forbidden),
    ));
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![a, b]);
}

/// Removing a rally system and undoing it puts it back as old as it was.
#[sqlx::test]
async fn a_restored_rally_point_keeps_its_place(pool: PgPool) {
    let w = world(&pool).await;
    let a = place(&pool, w.owner, w.map_id, SYS_A, 0.0).await;
    let b = place(&pool, w.owner, w.map_id, SYS_B, 200.0).await;
    let c = place(&pool, w.owner, w.map_id, SYS_C, 400.0).await;
    rally(&pool, w.owner, w.map_id, a, true).await.unwrap();
    rally(&pool, w.owner, w.map_id, b, true).await.unwrap();

    remove_system(
        &pool,
        w.owner,
        RemoveSystem {
            map_id: w.map_id,
            map_solar_system_id: a,
        },
    )
    .await
    .unwrap();
    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![a, b]);

    rally(&pool, w.owner, w.map_id, c, true).await.unwrap();
    assert_eq!(rallies_of(&pool, w.map_id).await, vec![b, c]);
}

async fn rally(
    pool: &PgPool,
    actor: wormholesystems::maps::Actor,
    map_id: i64,
    id: i64,
    value: bool,
) -> Result<(), MapError> {
    set_rally(
        pool,
        actor,
        SetRally {
            map_id,
            map_solar_system_id: id,
            value,
        },
    )
    .await
}

async fn home_of(pool: &PgPool, map_id: i64) -> Option<i64> {
    sqlx::query_scalar!(
        "select id from map_solar_systems where map_id = $1 and is_home",
        map_id
    )
    .fetch_optional(pool)
    .await
    .unwrap()
}

/// The map's rally points, oldest first.
async fn rallies_of(pool: &PgPool, map_id: i64) -> Vec<i64> {
    sqlx::query_scalar!(
        "select id from map_solar_systems where map_id = $1 and is_rally
         order by rallied_at, id",
        map_id
    )
    .fetch_all(pool)
    .await
    .unwrap()
}
