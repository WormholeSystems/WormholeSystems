//! Per-user private realtime channel, the server side of the `/ws/user` socket.
//!
//! Like [`MapHub`](crate::realtime::MapHub) but keyed by `user_id`. Most events are routed
//! explicitly, so only the addressed user's connections receive them; a few concern everyone
//! and go out to every live channel.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tokio::sync::broadcast;

const CHANNEL_CAPACITY: usize = 64;

/// An event addressed to a single user.
#[derive(Clone, Debug, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserEvent {
    /// One of the user's characters had its tracked status updated; refetch.
    CharacterStatusChanged { character_id: i64 },
    /// Tranquility went up, went down, or changed version. Sent to everyone.
    ServerStatusChanged,
    /// The user changed their own watchlist on this map, from any of their tabs; refetch.
    PersonalWatchlistChanged { map_id: i64 },
}

/// In-process bus of per-user channels. Cheaply cloneable; hold one in app state.
#[derive(Clone, Default)]
pub struct UserHub {
    channels: Arc<Mutex<HashMap<i64, broadcast::Sender<UserEvent>>>>,
}

impl UserHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to a user's events (one per connected device), creating the channel lazily.
    pub fn subscribe(&self, user_id: i64) -> broadcast::Receiver<UserEvent> {
        let mut channels = self.channels.lock().expect("user hub poisoned");
        channels
            .entry(user_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .subscribe()
    }

    /// Send an event to a user. A no-op if they have no live connection; prunes the channel
    /// once the last one is gone.
    pub fn publish(&self, user_id: i64, event: UserEvent) {
        let mut channels = self.channels.lock().expect("user hub poisoned");
        if let Some(tx) = channels.get(&user_id)
            && tx.send(event).is_err()
        {
            channels.remove(&user_id);
        }
    }

    /// Send an event to every connected user, dropping the channels nobody is listening on.
    pub fn broadcast(&self, event: UserEvent) {
        let mut channels = self.channels.lock().expect("user hub poisoned");
        channels.retain(|_, tx| tx.send(event.clone()).is_ok());
    }
}
