//! Writes to one map arriving at once: each still lands on the history in turn, so no change
//! falls off the chain and no two undos reverse the same step.

mod common;

use common::world;
use sqlx::PgPool;
use wormholesystems::maps::history::{MapIdBody, undo};

const WRITERS: i64 = 8;
const UNDOS: usize = 3;

/// `WRITERS` more systems in the shared constellation, so each writer places its own.
async fn seed_more_systems(pool: &PgPool) -> Vec<i64> {
    let ids: Vec<i64> = (0..WRITERS).map(|i| 30000200 + i).collect();
    for id in &ids {
        sqlx::query(
            "insert into solar_systems (id, constellation_id, region_id, name, security_status)
             values ($1, 20000001, 10000001, $2, 0.9)",
        )
        .bind(id)
        .bind(format!("System {id}"))
        .execute(pool)
        .await
        .unwrap();
    }
    ids
}

/// The map's undoable steps in the order they were written, as (id, parent).
async fn steps(pool: &PgPool, map_id: i64) -> Vec<(i64, Option<i64>)> {
    sqlx::query_as("select id, parent_id from map_events where map_id = $1 and is_step order by id")
        .bind(map_id)
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn head(pool: &PgPool, map_id: i64) -> Option<i64> {
    sqlx::query_scalar("select head_event_id from maps where id = $1")
        .bind(map_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn placed(pool: &PgPool, map_id: i64) -> i64 {
    sqlx::query_scalar("select count(*) from map_solar_systems where map_id = $1")
        .bind(map_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn concurrent_changes_form_one_chain_and_concurrent_undos_take_distinct_steps(pool: PgPool) {
    let w = world(&pool).await;
    let systems = seed_more_systems(&pool).await;

    let writers: Vec<_> = systems
        .iter()
        .enumerate()
        .map(|(i, &system)| {
            let pool = pool.clone();
            let owner = w.owner;
            let map_id = w.map_id;
            tokio::spawn(async move {
                common::map::place_at(&pool, owner, map_id, system, i as f64 * 200.0).await
            })
        })
        .collect();
    for writer in writers {
        writer.await.unwrap();
    }

    // Every change is a step, each on top of the one before, and the head is the last.
    let chain = steps(&pool, w.map_id).await;
    assert_eq!(chain.len() as i64, WRITERS);
    assert_eq!(chain[0].1, None);
    for pair in chain.windows(2) {
        assert_eq!(pair[1].1, Some(pair[0].0), "{chain:?}");
    }
    assert_eq!(head(&pool, w.map_id).await, Some(chain.last().unwrap().0));
    assert_eq!(placed(&pool, w.map_id).await, WRITERS);

    let undos: Vec<_> = (0..UNDOS)
        .map(|_| {
            let pool = pool.clone();
            let owner = w.owner;
            let map_id = w.map_id;
            tokio::spawn(async move { undo(&pool, owner, MapIdBody { map_id }).await })
        })
        .collect();
    for undo in undos {
        undo.await.unwrap().unwrap();
    }

    // Three different steps undone: three systems gone, and the head three steps back.
    assert_eq!(placed(&pool, w.map_id).await, WRITERS - UNDOS as i64);
    assert_eq!(
        head(&pool, w.map_id).await,
        Some(chain[chain.len() - 1 - UNDOS].0)
    );
}
