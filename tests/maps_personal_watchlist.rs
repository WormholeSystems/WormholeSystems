//! A user's own watchlist on a map: theirs alone, kept by viewers too, and apart from the
//! map's shared list and its history.

mod common;

use common::{SYS_B, SYS_C, member_with_role, new_user, world};
use sqlx::PgPool;
use wormholesystems::maps::personal_watchlist::{
    AddPersonalWatchlistEntry, RemovePersonalWatchlistEntry, SetPersonalWatchlistPinned, add, list,
    remove, set_pinned,
};
use wormholesystems::maps::watchlist::list_watchlist;
use wormholesystems::maps::{Actor, MapError, Role};

fn watch(solar_system_id: i64) -> AddPersonalWatchlistEntry {
    AddPersonalWatchlistEntry { solar_system_id }
}

fn systems(entries: &[wormholesystems::maps::watchlist::WatchlistEntry]) -> Vec<i64> {
    entries.iter().map(|e| e.solar_system_id).collect()
}

#[sqlx::test]
async fn each_user_sees_only_their_own(pool: PgPool) {
    let w = world(&pool).await;
    let member = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;

    add(&pool, w.owner, w.map_id, watch(SYS_B)).await.unwrap();
    add(&pool, member, w.map_id, watch(SYS_C)).await.unwrap();
    // Watching the same system twice is one entry.
    add(&pool, member, w.map_id, watch(SYS_C)).await.unwrap();

    assert_eq!(
        systems(&list(&pool, w.owner, w.map_id).await.unwrap()),
        vec![SYS_B]
    );
    assert_eq!(
        systems(&list(&pool, member, w.map_id).await.unwrap()),
        vec![SYS_C]
    );
    // The map's shared list is untouched: still just the seeded hub.
    assert_eq!(
        list_watchlist(&pool, w.owner, w.map_id)
            .await
            .unwrap()
            .len(),
        1
    );
    let history: i64 = sqlx::query_scalar(
        "select count(*) from map_events where map_id = $1 and kind like 'watchlist.%'",
    )
    .bind(w.map_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(history, 0);
}

#[sqlx::test]
async fn nobody_else_can_touch_your_entries(pool: PgPool) {
    let w = world(&pool).await;
    let member = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Member).await;
    let mine = add(&pool, w.owner, w.map_id, watch(SYS_B)).await.unwrap();

    let pin = set_pinned(
        &pool,
        member,
        w.map_id,
        SetPersonalWatchlistPinned {
            entry_id: mine.id,
            value: true,
        },
    )
    .await;
    assert!(matches!(pin, Err(MapError::NotFound)), "{pin:?}");
    let gone = remove(
        &pool,
        member,
        w.map_id,
        RemovePersonalWatchlistEntry { entry_id: mine.id },
    )
    .await;
    assert!(matches!(gone, Err(MapError::NotFound)), "{gone:?}");
    assert_eq!(list(&pool, w.owner, w.map_id).await.unwrap().len(), 1);
}

#[sqlx::test]
async fn a_viewer_keeps_one_and_an_outsider_cannot(pool: PgPool) {
    let w = world(&pool).await;
    let viewer = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Viewer).await;

    let entry = add(&pool, viewer, w.map_id, watch(SYS_B)).await.unwrap();
    let pinned = set_pinned(
        &pool,
        viewer,
        w.map_id,
        SetPersonalWatchlistPinned {
            entry_id: entry.id,
            value: true,
        },
    )
    .await
    .unwrap();
    assert!(pinned.is_pinned);
    remove(
        &pool,
        viewer,
        w.map_id,
        RemovePersonalWatchlistEntry { entry_id: entry.id },
    )
    .await
    .unwrap();
    assert!(list(&pool, viewer, w.map_id).await.unwrap().is_empty());

    let user = new_user(&pool).await;
    common::add_character(&pool, user, 1003, 2003, None).await;
    let outsider = Actor {
        user_id: user,
        character_id: 1003,
    };
    let err = add(&pool, outsider, w.map_id, watch(SYS_B)).await;
    assert!(matches!(err, Err(MapError::NotFound)), "{err:?}");
}
