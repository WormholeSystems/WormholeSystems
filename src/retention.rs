//! The daily sweep of rows nothing reads any more, see
//! [processes.md](../docs/processes.md#retention-sweep).
//!
//! Each of these tables grows with use and had no end: every alert that fires adds a
//! delivery and an audit row, and every login a session that outlives its cookie.

use std::time::Duration;

use sqlx::PgPool;
use tokio::time::{MissedTickBehavior, interval};

/// How long a delivery is kept once its occasion cannot come round again.
pub const DELIVERY_RETENTION_DAYS: i32 = 30;
/// How far back an alert's audit trail reaches.
pub const ALERT_EVENT_RETENTION_DAYS: i32 = 90;

/// What one sweep removed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Swept {
    pub deliveries: u64,
    pub alert_events: u64,
    pub sessions: u64,
}

/// Spawn the daily sweep. Returns immediately; the loop runs for the process lifetime.
pub fn start(pool: PgPool) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(24 * 60 * 60));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            if let Err(err) = sweep(&pool).await {
                eprintln!("retention sweep failed: {err}");
            }
        }
    });
}

pub async fn sweep(pool: &PgPool) -> sqlx::Result<Swept> {
    // A delivery is what stops an alert firing twice for one occasion, so it may only go
    // once that occasion is over for good. A killmail is offered once, when it streams in.
    // A placement's delivery stays for as long as the placement does. Once it is gone, an
    // undo could still restore it on its old id and let the alert fire once more; that
    // needs a removal undone within a week of a delivery at least a month old, which is a
    // repeat worth the bounded table. Keys of any other shape are left alone.
    let deliveries = sqlx::query!(
        "delete from map_alert_deliveries d
         where d.created_at < now() - make_interval(days => $1)
           and (d.dedup_key like 'killmail:%'
                or (d.dedup_key like 'placement:%'
                    and not exists (
                        select 1 from map_solar_systems s
                        where s.id = case when d.dedup_key like 'placement:%'
                                          then split_part(d.dedup_key, ':', 2)::bigint
                                     end
                    )))",
        DELIVERY_RETENTION_DAYS,
    )
    .execute(pool)
    .await?
    .rows_affected();

    let alert_events = sqlx::query!(
        "delete from map_alert_events where created_at < now() - make_interval(days => $1)",
        ALERT_EVENT_RETENTION_DAYS,
    )
    .execute(pool)
    .await?
    .rows_affected();

    // Already refused at the door (`expires_at > now()`); this only clears them out.
    let sessions = sqlx::query!("delete from sessions where expires_at < now()")
        .execute(pool)
        .await?
        .rows_affected();

    Ok(Swept {
        deliveries,
        alert_events,
        sessions,
    })
}
