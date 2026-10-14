//! In-process realtime event bus for map changes.
//!
//! Mapping runs as a single server, so events go through an in-process [`MapHub`] (one
//! `tokio::broadcast` channel per `map_id`) rather than a cross-instance bus; the WebSocket
//! layer and background producers both use it directly. Events are notify-then-refetch: a
//! payload names what changed (ids), not the new data, and a (re)connect does a full
//! `get_map`, so a missed event self-heals. See
//! [`docs/features/realtime.md`](../../docs/features/realtime.md).

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

/// Per-map channel depth. A receiver that lags past this gets `RecvError::Lagged`; the WS
/// layer treats that as "you're behind" and triggers a full refetch.
const CHANNEL_CAPACITY: usize = 128;

/// A change to a map that its viewers should react to. Carries ids, not data: consumers
/// refetch the named slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MapEvent {
    /// The map's own fields (name / description / image) changed.
    MapUpdated { map_id: i64 },
    SystemAdded {
        map_id: i64,
        map_solar_system_id: i64,
    },
    SystemMoved {
        map_id: i64,
        map_solar_system_id: i64,
    },
    SystemRemoved {
        map_id: i64,
        map_solar_system_id: i64,
    },
    /// A placed system's intel/flags changed (alias, status, occupier, home, pinned): the
    /// position is unchanged, but its details should be refetched.
    SystemDetailsChanged {
        map_id: i64,
        map_solar_system_id: i64,
    },
    /// A connection was added, removed, or had its state changed (incl. trigger-driven sync).
    ConnectionChanged { map_id: i64, connection_id: i64 },
    /// A signature in this system was added, edited, linked, unlinked, or removed.
    SignatureChanged { map_id: i64, solar_system_id: i64 },
    /// An access grant changed (membership/roles).
    AccessChanged { map_id: i64 },
    /// The navigation watchlist changed (entry added/pinned/removed).
    WatchlistChanged { map_id: i64 },
    /// The map's ignore list changed: routes and jump tracking steer differently now.
    IgnoredSystemsChanged { map_id: i64 },
    /// A tracked pilot on this map moved, changed ship, or came on/offline. Sent to the
    /// whole map, unlike the per-user status ping, because everyone's pilot list shows it.
    CharactersChanged { map_id: i64 },
    /// Something died in one of this map's systems.
    KillmailReceived { map_id: i64 },
    /// The command journal grew or an entry was undone, so open clients should refetch
    /// the history (and, after an undo, the map itself).
    HistoryChanged { map_id: i64 },
}

impl MapEvent {
    /// The map every event is routed by.
    pub fn map_id(&self) -> i64 {
        match *self {
            MapEvent::MapUpdated { map_id }
            | MapEvent::SystemAdded { map_id, .. }
            | MapEvent::SystemMoved { map_id, .. }
            | MapEvent::SystemRemoved { map_id, .. }
            | MapEvent::SystemDetailsChanged { map_id, .. }
            | MapEvent::ConnectionChanged { map_id, .. }
            | MapEvent::CharactersChanged { map_id }
            | MapEvent::KillmailReceived { map_id }
            | MapEvent::SignatureChanged { map_id, .. }
            | MapEvent::AccessChanged { map_id }
            | MapEvent::WatchlistChanged { map_id }
            | MapEvent::IgnoredSystemsChanged { map_id }
            | MapEvent::HistoryChanged { map_id } => map_id,
        }
    }
}

/// The in-process event bus. Cheaply cloneable (an `Arc` inside): hold one in app state.
#[derive(Clone)]
pub struct MapHub {
    channels: Arc<Mutex<HashMap<i64, broadcast::Sender<MapEvent>>>>,
    /// Every event on every map, for background work that cannot know which maps to watch.
    all: broadcast::Sender<MapEvent>,
}

impl Default for MapHub {
    fn default() -> Self {
        MapHub {
            channels: Arc::default(),
            all: broadcast::channel(CHANNEL_CAPACITY).0,
        }
    }
}

impl MapHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// Every map's events in one stream. A consumer that falls behind is told it lagged and
    /// carries on from the present; publishing never blocks on a slow subscriber.
    pub fn subscribe_all(&self) -> broadcast::Receiver<MapEvent> {
        self.all.subscribe()
    }

    /// Subscribe to a map's events, creating its channel on first use. Each connected
    /// viewer holds one receiver.
    pub fn subscribe(&self, map_id: i64) -> broadcast::Receiver<MapEvent> {
        let mut channels = self.channels.lock().expect("map hub poisoned");
        channels
            .entry(map_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .subscribe()
    }

    /// Publish an event to its map. A no-op if nobody is watching that map; if the last
    /// subscriber has gone, the channel is pruned so idle maps don't leak.
    pub fn publish(&self, event: MapEvent) {
        let _ = self.all.send(event.clone());
        let map_id = event.map_id();
        let mut channels = self.channels.lock().expect("map hub poisoned");
        if let Some(tx) = channels.get(&map_id)
            && tx.send(event).is_err()
        {
            channels.remove(&map_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn delivers_to_all_subscribers_of_the_map() {
        let hub = MapHub::new();
        let mut a = hub.subscribe(1);
        let mut b = hub.subscribe(1);

        hub.publish(MapEvent::AccessChanged { map_id: 1 });

        assert_eq!(a.try_recv().unwrap(), MapEvent::AccessChanged { map_id: 1 });
        assert_eq!(b.try_recv().unwrap(), MapEvent::AccessChanged { map_id: 1 });
    }

    #[tokio::test]
    async fn events_are_scoped_to_their_map() {
        let hub = MapHub::new();
        let mut other = hub.subscribe(2);

        hub.publish(MapEvent::MapUpdated { map_id: 1 });

        assert!(
            other.try_recv().is_err(),
            "map 2 must not see map 1's events"
        );
    }

    #[tokio::test]
    async fn publishing_with_no_subscribers_is_a_noop() {
        let hub = MapHub::new();
        // No panic, nothing to receive.
        hub.publish(MapEvent::SystemAdded {
            map_id: 99,
            map_solar_system_id: 1,
        });
    }
}
