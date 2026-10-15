//! The realtime WebSocket handlers: the per-map event stream and the per-user private
//! channel / activity heartbeat.

use crate::api::extract::Credentials;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade, close_code};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::time::Duration;

use tokio::sync::broadcast::error::RecvError;
use tokio::time::{MissedTickBehavior, interval};

use crate::app::AppState;
use crate::realtime::UserHub;

/// How often an open map socket asks again whether its reader may still see the map. A
/// change to the map's grants or sharing asks at once; this catches what announces nothing,
/// such as a grant running out or a character leaving the corporation it was granted
/// through.
const ACCESS_RECHECK: Duration = Duration::from_secs(60);

/// The close code for a reader who may no longer see the map. Private-use range, and 4403
/// so it reads as the HTTP status it stands for. A client should not reconnect on it: the
/// upgrade would be refused.
pub const ACCESS_REVOKED: u16 = 4403;

/// `GET /ws/map/{map_id}`: the map's events as JSON, gated at the same bar as reading it:
/// a grant, or the share it has been opened with. A watcher following a link gets the
/// stream too; the frames say only that something changed. Access is asked again while the
/// socket is open, and the socket closed with [`ACCESS_REVOKED`] once it is gone.
pub async fn map_ws(
    Path(map_id): Path<i64>,
    State(state): State<AppState>,
    creds: Credentials,
    ws: WebSocketUpgrade,
) -> Response {
    let actor = crate::api::extract::session_actor(&state.db, &creds)
        .await
        .ok()
        .flatten();
    let token = crate::api::extract::share_cookie(&creds, map_id);

    let reader = Reader {
        db: state.db,
        map_id,
        actor,
        token,
    };
    // Checked before subscribing, so asking after a map that does not exist, or that this
    // reader may not see, leaves nothing behind in the hub. A revocation published between
    // the check and the subscription is caught by the stream asking again as it opens.
    match reader.check().await {
        Ok(_) => {
            let events = crate::realtime::hub().subscribe(map_id);
            ws.on_upgrade(move |socket| stream_map_events(socket, reader, events))
        }
        Err(crate::maps::MapError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// `GET /ws/user`: the signed-in user's private channel, and the activity heartbeat: while
/// connected we ping the client and bump `last_active_at` on each pong, which is what gates
/// the tracking poller.
pub async fn user_ws(
    State(state): State<AppState>,
    creds: Credentials,
    ws: WebSocketUpgrade,
) -> Response {
    let actor = crate::api::extract::session_actor(&state.db, &creds)
        .await
        .ok()
        .flatten();
    match actor {
        Some(actor) => ws.on_upgrade(move |socket| {
            user_heartbeat(socket, state.db, state.user_hub, actor.user_id)
        }),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn user_heartbeat(mut socket: WebSocket, pool: sqlx::PgPool, users: UserHub, user_id: i64) {
    use std::time::{Duration, Instant};

    use tokio::time::{MissedTickBehavior, interval};

    // Active on connect, then again on each heartbeat (throttled) while the socket lives.
    let _ = crate::session::touch_activity(&pool, user_id).await;
    let mut ping = interval(Duration::from_secs(30));
    ping.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_bump = Instant::now();
    let mut events = users.subscribe(user_id);

    loop {
        tokio::select! {
            _ = crate::background::stopped() => {
                going_away(socket).await;
                break;
            }
            _ = ping.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break; // client gone
                }
            }
            frame = socket.recv() => {
                match frame {
                    // Any frame (pong / message) means the client is alive.
                    Some(Ok(_)) => {
                        if last_bump.elapsed() >= Duration::from_secs(50) {
                            let _ = crate::session::touch_activity(&pool, user_id).await;
                            last_bump = Instant::now();
                        }
                    }
                    _ => break, // closed or errored
                }
            }
            event = events.recv() => {
                match event {
                    Ok(event) => {
                        let json = serde_json::to_string(&event).unwrap_or_default();
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(RecvError::Lagged(_)) => {} // client refetches regardless
                    Err(RecvError::Closed) => break,
                }
            }
        }
    }
}

/// Who opened a map socket, kept so the question can be asked again.
struct Reader {
    db: sqlx::PgPool,
    map_id: i64,
    actor: Option<crate::maps::Actor>,
    token: Option<String>,
}

impl Reader {
    async fn check(&self) -> crate::maps::Result<crate::maps::access::Reader> {
        crate::maps::access::reader_for(&self.db, self.map_id, self.actor, self.token.as_deref())
            .await
    }

    /// Whether the socket may stay open. A database hiccup is not a revocation: the socket
    /// stays, and the next check decides.
    async fn still_allowed(&self) -> bool {
        match self.check().await {
            Ok(_) => true,
            Err(crate::maps::MapError::NotFound | crate::maps::MapError::Forbidden) => false,
            Err(err) => {
                tracing::warn!(
                    "map socket: could not recheck access to {}: {err}",
                    self.map_id
                );
                true
            }
        }
    }
}

async fn stream_map_events(
    mut socket: WebSocket,
    reader: Reader,
    mut rx: crate::realtime::Subscription,
) {
    use crate::realtime::MapEvent;

    // The first tick is immediate: the upgrade was allowed before the subscription began,
    // so ask once more now that nothing can slip past.
    let mut recheck = interval(ACCESS_RECHECK);
    recheck.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        let received = tokio::select! {
            _ = crate::background::stopped() => {
                going_away(socket).await;
                return;
            }
            _ = recheck.tick() => {
                if !reader.still_allowed().await {
                    access_revoked(socket).await;
                    return;
                }
                continue;
            }
            // The client sends nothing this needs, but reading is how a closed socket is
            // noticed on a quiet map, and with it goes the map's subscription.
            frame = socket.recv() => match frame {
                Some(Ok(_)) => continue,
                _ => return,
            },
            received = rx.recv() => received,
        };
        // A change to who may see the map, or to how it is shared, is the moment to ask;
        // so is a lag, which may have swallowed one. Asked before the frame goes out, so a
        // reader who just lost the map does not hear about it.
        let ask = matches!(
            received,
            Ok(MapEvent::AccessChanged { .. } | MapEvent::MapUpdated { .. })
                | Err(RecvError::Lagged(_))
        );
        if ask && !reader.still_allowed().await {
            access_revoked(socket).await;
            return;
        }
        let text = match received {
            Ok(event) => serde_json::to_string(&event).unwrap_or_default(),
            Err(RecvError::Lagged(_)) => r#"{"type":"lagged"}"#.to_string(),
            Err(RecvError::Closed) => break,
        };
        if socket.send(Message::Text(text.into())).await.is_err() {
            break;
        }
    }
}

/// The reader may no longer see the map. Said with its own code, so a client can tell it
/// from a restart and stop trying.
async fn access_revoked(mut socket: WebSocket) {
    let frame = CloseFrame {
        code: ACCESS_REVOKED,
        reason: "access to this map was withdrawn".into(),
    };
    let _ = socket.send(Message::Close(Some(frame))).await;
}

/// The server is shutting down: 1001 tells the client to reconnect, which lands it on the
/// process that replaces this one, rather than leaving it to notice a dead connection.
async fn going_away(mut socket: WebSocket) {
    let frame = CloseFrame {
        code: close_code::AWAY,
        reason: "server restarting".into(),
    };
    let _ = socket.send(Message::Close(Some(frame))).await;
}
