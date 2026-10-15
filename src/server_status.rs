//! Is Tranquility up?
//!
//! Feeds the header indicator, and gates the other ESI pollers: during downtime every
//! authenticated call fails, and hammering a rebooting server burns the error limit for
//! nothing. Pollers ask [`ServerWatch::should_poll`] before each tick, and this loop is the
//! only one that keeps running, because it is what notices the recovery.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::watch;

use crate::esi::{EsiClient, EsiError};
use crate::realtime::{UserEvent, UserHub};

/// What the server is doing, as one word for the client to render.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum ServerState {
    /// Nothing polled yet; this process just started.
    Unknown,
    Online,
    /// Up, but only CCP can log in.
    Vip,
    /// ESI answered and said nobody is playing: downtime, or a crash.
    Offline,
    /// ESI itself could not be reached, so Tranquility's state is anyone's guess.
    Unreachable,
}

impl ServerState {
    /// Whether it is worth making an ESI call right now. `Unknown` counts as yes (a fresh
    /// process should not sit idle waiting for its first poll), and so does `Vip` (the few
    /// characters that are online are still worth tracking).
    pub fn worth_polling(self) -> bool {
        !matches!(self, ServerState::Offline | ServerState::Unreachable)
    }

    /// What a successful `/status` reply means. ESI keeps answering through downtime with
    /// the last figures it had, so zero players is the only reliable sign the server has
    /// gone away; VIP is reported separately and outranks the count.
    pub fn classify(vip: bool, players: i64) -> ServerState {
        if vip {
            ServerState::Vip
        } else if players > 0 {
            ServerState::Online
        } else {
            ServerState::Offline
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ServerStatus {
    pub state: ServerState,
    pub players: i64,
    #[ts(optional)]
    pub server_version: Option<String>,
    /// When this run of the server started, for the uptime readout.
    #[ts(optional)]
    pub start_time: Option<DateTime<Utc>>,
    /// When we last asked. A stale value here means *our* poller is stuck, not the server.
    #[ts(optional)]
    pub checked_at: Option<DateTime<Utc>>,
}

impl ServerStatus {
    /// The starting point: we have not asked yet, and assume the best until we have.
    pub fn unknown() -> Self {
        ServerStatus {
            state: ServerState::Unknown,
            players: 0,
            server_version: None,
            start_time: None,
            checked_at: None,
        }
    }
}

/// A cheap handle on the latest status. Clone it into anything that needs to check.
#[derive(Clone)]
pub struct ServerWatch(watch::Receiver<ServerStatus>);

impl ServerWatch {
    pub fn current(&self) -> ServerStatus {
        self.0.borrow().clone()
    }

    /// A fixed status, for tests that need an `AppState` but no poller. The last value
    /// stays readable after the sender is dropped.
    pub fn fixed(status: ServerStatus) -> ServerWatch {
        let (_tx, rx) = watch::channel(status);
        ServerWatch(rx)
    }

    /// Whether an ESI-backed poller should do its work this tick.
    pub fn should_poll(&self) -> bool {
        self.0.borrow().state.worth_polling()
    }
}

/// Spawn the status loop. Returns the handle the other pollers gate on. A minute
/// (`SERVER_STATUS_POLL_SECS`) is well inside the downtime window and nowhere near ESI's
/// rate limit.
pub fn start(pool: PgPool, esi: EsiClient, users: UserHub, every: Duration) -> ServerWatch {
    let (tx, rx) = watch::channel(ServerStatus::unknown());
    let tx = Arc::new(tx);
    tokio::spawn(async move {
        // Start from the last known value so a restart does not blank the header, then
        // correct it on the first poll a moment later.
        // Guarded, as the poll below is: nothing here may stop the poll from starting.
        let stored = crate::background::guarded("server status", || {
            let pool = pool.clone();
            async move { load(&pool).await }
        })
        .await;
        if let Some(Ok(Some(stored))) = stored {
            let _ = tx.send(stored);
        }
        crate::background::spawn_every("server status poll", every, move || {
            let (pool, esi, users, tx) = (pool.clone(), esi.clone(), users.clone(), tx.clone());
            async move {
                // Held back by our own ESI backoff, which says nothing about Tranquility:
                // keep what we last knew rather than calling it unreachable.
                let Some(status) = poll(&esi).await else {
                    return Ok(());
                };
                let saved = save(&pool, &status).await;
                // Only tell the clients when something actually changed; the players count
                // moves constantly and nobody needs a push every minute for it.
                let changed = {
                    let previous = tx.borrow();
                    previous.state != status.state
                        || previous.server_version != status.server_version
                };
                let _ = tx.send(status);
                if changed {
                    users.broadcast(UserEvent::ServerStatusChanged);
                }
                // Reported last: a status that could not be stored is still the status.
                saved.map_err(|err| format!("could not store the server status: {err}"))
            }
        });
    });
    ServerWatch(rx)
}

async fn poll(esi: &EsiClient) -> Option<ServerStatus> {
    let now = Utc::now();
    Some(match esi.server_status().await {
        Ok(tq) => ServerStatus {
            state: ServerState::classify(tq.vip, tq.players),
            players: tq.players,
            server_version: Some(tq.server_version),
            start_time: Some(tq.start_time),
            checked_at: Some(now),
        },
        Err(EsiError::Backoff(_)) => return None,
        Err(_) => ServerStatus {
            state: ServerState::Unreachable,
            players: 0,
            server_version: None,
            start_time: None,
            checked_at: Some(now),
        },
    })
}

pub async fn load(pool: &PgPool) -> sqlx::Result<Option<ServerStatus>> {
    let row = sqlx::query!(
        "select reachable, players, server_version, start_time, vip, checked_at
         from server_status where id"
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| ServerStatus {
        state: if r.reachable {
            ServerState::classify(r.vip, r.players)
        } else {
            ServerState::Unreachable
        },
        players: r.players,
        server_version: r.server_version,
        start_time: r.start_time,
        checked_at: Some(r.checked_at),
    }))
}

async fn save(pool: &PgPool, status: &ServerStatus) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into server_status (id, reachable, players, server_version, start_time, vip,
                                    checked_at)
         values (true, $1, $2, $3, $4, $5, now())
         on conflict (id) do update set
             reachable = excluded.reachable,
             players = excluded.players,
             server_version = excluded.server_version,
             start_time = excluded.start_time,
             vip = excluded.vip,
             checked_at = excluded.checked_at",
        status.state != ServerState::Unreachable,
        status.players,
        status.server_version.as_deref(),
        status.start_time,
        status.state == ServerState::Vip,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reply_is_classified_by_vip_then_by_headcount() {
        assert_eq!(ServerState::classify(false, 24_000), ServerState::Online);
        assert_eq!(ServerState::classify(false, 0), ServerState::Offline);
        // VIP wins even with players on: they are CCP, and nobody else can get in.
        assert_eq!(ServerState::classify(true, 12), ServerState::Vip);
        assert_eq!(ServerState::classify(true, 0), ServerState::Vip);
    }

    #[test]
    fn only_a_server_known_to_be_gone_stops_the_pollers() {
        // Not yet asked, and VIP, both still worth a call: the first is ignorance, and the
        // second is a server that is genuinely up.
        assert!(ServerState::Unknown.worth_polling());
        assert!(ServerState::Online.worth_polling());
        assert!(ServerState::Vip.worth_polling());
        assert!(!ServerState::Offline.worth_polling());
        assert!(!ServerState::Unreachable.worth_polling());
    }
}
