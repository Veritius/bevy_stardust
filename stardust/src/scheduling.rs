//! Schedules used in Stardust.

use bevy::prelude::*;

/// Systems dealing with receiving messages. Run in the [`PreUpdate`] schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkRead {
    /// Transport layers receive packets or other transmission media.
    Receive,
    /// Game systems process messages and mutate the World before [`Update`].
    /// You can still read messages at any time before [`Receive`](NetworkRead::Receive).
    Read,
}

/// Systems dealing with outgoing octet strings. Run in the [`PostUpdate`] schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkWrite {
    /// Transport layers send messages queued by game systems.
    Send,
    /// Queued messages (both the incoming and outgoing buffers) are cleared.
    Clear,
}

pub(super) fn configure_scheduling(app: &mut App) {
    app.configure_sets(PreUpdate, (
        NetworkRead::Read.after(NetworkRead::Receive),
    ));

    app.configure_sets(PostUpdate, (
       NetworkWrite::Clear.after(NetworkWrite::Send),
    ));
}