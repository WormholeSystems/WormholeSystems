//! The daily retention sweep: what it removes, and the deliveries it must keep because
//! their occasion can still come round.

mod common;

use common::{SYS_A, world};
use sqlx::PgPool;
use wormholesystems::maps::solar_system::{AddSystem, add_system};
use wormholesystems::retention::{Swept, sweep};

async fn alert(pool: &PgPool, map_id: i64) -> i64 {
    sqlx::query_scalar(
        "insert into map_alerts (map_id, name, kind, delivery, max_jumps)
         values ($1, 'Test', 'killmail', 'webhook', 5)
         returning id",
    )
    .bind(map_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn delivery(pool: &PgPool, alert_id: i64, key: &str, days_old: i32) {
    sqlx::query(
        "insert into map_alert_deliveries (map_alert_id, dedup_key, delivered_at, created_at)
         values ($1, $2, now(), now() - make_interval(days => $3))",
    )
    .bind(alert_id)
    .bind(key)
    .bind(days_old)
    .execute(pool)
    .await
    .unwrap();
}

async fn keys(pool: &PgPool) -> Vec<String> {
    sqlx::query_scalar("select dedup_key from map_alert_deliveries order by dedup_key")
        .fetch_all(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn old_deliveries_go_once_their_occasion_cannot_return(pool: PgPool) {
    let w = world(&pool).await;
    let alert_id = alert(&pool, w.map_id).await;
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
    .unwrap()
    .id;

    delivery(&pool, alert_id, "killmail:1", 40).await;
    delivery(&pool, alert_id, "killmail:2", 5).await;
    delivery(&pool, alert_id, &format!("placement:{placed}"), 40).await;
    delivery(&pool, alert_id, "placement:999999999", 40).await;
    delivery(&pool, alert_id, "placement:999999998", 5).await;
    delivery(&pool, alert_id, "something:else", 400).await;

    let swept = sweep(&pool).await.unwrap();
    assert_eq!(swept.deliveries, 2);
    assert_eq!(
        keys(&pool).await,
        [
            "killmail:2".to_string(),
            format!("placement:{placed}"),
            "placement:999999998".to_string(),
            "something:else".to_string(),
        ]
    );
}

#[sqlx::test]
async fn old_alert_history_and_expired_sessions_go(pool: PgPool) {
    let w = world(&pool).await;
    for days_old in [100, 10] {
        sqlx::query(
            "insert into map_alert_events (map_id, kind, created_at)
             values ($1, 'fired', now() - make_interval(days => $2))",
        )
        .bind(w.map_id)
        .bind(days_old)
        .execute(&pool)
        .await
        .unwrap();
    }
    for (id, expires_in_days) in [("old", -1), ("live", 1)] {
        sqlx::query(
            "insert into sessions (id, user_id, active_character_id, expires_at)
             values ($1, $2, 1001, now() + make_interval(days => $3))",
        )
        .bind(id)
        .bind(w.owner.user_id)
        .bind(expires_in_days)
        .execute(&pool)
        .await
        .unwrap();
    }

    assert_eq!(
        sweep(&pool).await.unwrap(),
        Swept {
            deliveries: 0,
            alert_events: 1,
            sessions: 1,
        }
    );
    let sessions: Vec<String> = sqlx::query_scalar("select id from sessions")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(sessions, ["live"]);
}
