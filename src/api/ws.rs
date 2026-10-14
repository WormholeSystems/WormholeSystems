//! The realtime WebSocket handlers: the per-map event stream and the per-user private
//! channel / activity heartbeat.

use crate::api::extract::Credentials;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade, close_code};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio::sync::broadcast::error::RecvError;

use crate::app::AppState;
use crate::user_channel::UserHub;

/// `GET /ws/map/{map_id}`: the map's events as JSON, gated at the same bar as reading it:
/// a grant, or the share it has been opened with. A watcher following a link gets the
/// stream too; the frames say only that something changed.
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

    match crate::maps::access::reader_for(&state.db, map_id, actor, token.as_deref()).await {
        Ok(_) => ws.on_upgrade(move |socket| stream_map_events(socket, map_id)),
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

async fn stream_map_events(mut socket: WebSocket, map_id: i64) {
    let mut rx = crate::maps::hub().subscribe(map_id);
    loop {
        let received = tokio::select! {
            _ = crate::background::stopped() => {
                going_away(socket).await;
                return;
            }
            received = rx.recv() => received,
        };
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

/// The server is shutting down: 1001 tells the client to reconnect, which lands it on the
/// process that replaces this one, rather than leaving it to notice a dead connection.
async fn going_away(mut socket: WebSocket) {
    let frame = CloseFrame {
        code: close_code::AWAY,
        reason: "server restarting".into(),
    };
    let _ = socket.send(Message::Close(Some(frame))).await;
}
