//! Realtime: the in-process channels that tell open WebSockets something changed.
//!
//! [`MapHub`] carries map changes to everyone viewing a map, [`UserHub`] carries one
//! user's private events. Both are notify-then-refetch: a frame names what changed, and the
//! client fetches the new state itself.

mod map_hub;
mod user_channel;

pub use map_hub::{MapEvent, MapHub, Subscription};
pub use user_channel::{UserEvent, UserHub};

/// The process-wide map hub, like [`crate::maps::grid`]: global so a committed command can
/// announce itself without every caller threading the hub through. The only one: the
/// WebSockets, the background loops and the alert subscriber all use it, so a test driving
/// the router hears what its commands publish.
static HUB: std::sync::OnceLock<MapHub> = std::sync::OnceLock::new();

pub fn hub() -> &'static MapHub {
    HUB.get_or_init(MapHub::default)
}
