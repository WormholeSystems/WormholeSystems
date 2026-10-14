//! The map's ignore list: CRUD, manager gating, undo, and what jump tracking does with it.

mod common;

use common::{SYS_A, SYS_B, SYS_C, member_with_role, world};
use sqlx::PgPool;
use wormholesystems::maps::history::{MapIdBody, undo};
use wormholesystems::maps::ignored::{
    AddIgnoredSystem, ClearIgnoredSystems, RemoveIgnoredSystem, add_ignored_system,
    clear_ignored_systems, ignored_ids, list_ignored_systems, remove_ignored_system,
};
use wormholesystems::maps::map::{GetMap, get_map};
use wormholesystems::maps::solar_system::{AddSystem, add_system};
use wormholesystems::maps::tracking::{TrackJump, track_jump};
use wormholesystems::maps::{Actor, MapError, Role};

async fn ignore(pool: &PgPool, actor: Actor, map_id: i64, solar_system_id: i64) {
    add_ignored_system(
        pool,
        actor,
        AddIgnoredSystem {
            map_id,
            solar_system_id,
        },
    )
    .await
    .unwrap();
}

#[sqlx::test]
async fn crud_and_gating(pool: PgPool) {
    let w = world(&pool).await;
    let member = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;
    let manager = member_with_role(&pool, w.owner, w.map_id, 1003, 2003, Role::Manager).await;

    // Members read the list but do not change it: it moves everyone's routes.
    let err = add_ignored_system(
        &pool,
        member,
        AddIgnoredSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
        },
    )
    .await;
    assert!(matches!(err, Err(MapError::Forbidden)));

    let row = add_ignored_system(
        &pool,
        manager,
        AddIgnoredSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
        },
    )
    .await
    .unwrap();
    let again = add_ignored_system(
        &pool,
        manager,
        AddIgnoredSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
        },
    )
    .await
    .unwrap();
    assert_eq!(again.id, row.id, "adding twice is one row");

    let err = add_ignored_system(
        &pool,
        manager,
        AddIgnoredSystem {
            map_id: w.map_id,
            solar_system_id: 1,
        },
    )
    .await;
    assert!(matches!(err, Err(MapError::Validation(_))));

    ignore(&pool, manager, w.map_id, SYS_C).await;
    let listed = list_ignored_systems(&pool, member, w.map_id).await.unwrap();
    assert_eq!(
        listed.iter().map(|r| r.solar_system_id).collect::<Vec<_>>(),
        vec![SYS_B, SYS_C]
    );

    let err = remove_ignored_system(
        &pool,
        member,
        RemoveIgnoredSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
        },
    )
    .await;
    assert!(matches!(err, Err(MapError::Forbidden)));
    remove_ignored_system(
        &pool,
        manager,
        RemoveIgnoredSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
        },
    )
    .await
    .unwrap();
    let err = remove_ignored_system(
        &pool,
        manager,
        RemoveIgnoredSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
        },
    )
    .await;
    assert!(matches!(err, Err(MapError::NotFound)));
    assert_eq!(ignored_ids(&pool, w.map_id).await.unwrap(), vec![SYS_C]);
}

#[sqlx::test]
async fn clearing_is_one_step_and_undo_brings_the_list_back(pool: PgPool) {
    let w = world(&pool).await;
    ignore(&pool, w.owner, w.map_id, SYS_B).await;
    ignore(&pool, w.owner, w.map_id, SYS_C).await;

    let count = clear_ignored_systems(&pool, w.owner, ClearIgnoredSystems { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(count, 2);
    assert!(ignored_ids(&pool, w.map_id).await.unwrap().is_empty());

    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    let mut back = ignored_ids(&pool, w.map_id).await.unwrap();
    back.sort();
    assert_eq!(back, vec![SYS_B, SYS_C]);

    // One more undo takes the last add away, not the whole list.
    undo(&pool, w.owner, MapIdBody { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(ignored_ids(&pool, w.map_id).await.unwrap(), vec![SYS_B]);
}

#[sqlx::test]
async fn a_tracked_jump_never_places_an_ignored_system(pool: PgPool) {
    let w = world(&pool).await;
    let a = add_system(
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
    .unwrap()
    .id;
    ignore(&pool, w.owner, w.map_id, SYS_B).await;

    let jump = |to: i64| TrackJump {
        map_id: w.map_id,
        from_map_solar_system_id: a,
        to_solar_system_id: to,
        x: 300.0,
        y: 0.0,
        signature_pk: None,
        alias: None,
        size: None,
        mass_status: None,
        time_status: None,
    };
    let err = track_jump(&pool, w.owner, jump(SYS_B)).await;
    assert!(matches!(err, Err(MapError::Validation(_))), "got {err:?}");
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.systems.len(), 1);

    // Placed by hand, an ignored system is somebody's choice, and a jump into it links up.
    add_system(
        &pool,
        w.owner,
        AddSystem {
            map_id: w.map_id,
            solar_system_id: SYS_B,
            x: 300.0,
            y: 0.0,
            alias: None,
        },
    )
    .await
    .unwrap();
    track_jump(&pool, w.owner, jump(SYS_B)).await.unwrap();
    let view = get_map(&pool, w.owner, GetMap { map_id: w.map_id })
        .await
        .unwrap();
    assert_eq!(view.connections.len(), 1);
}
