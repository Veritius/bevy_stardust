//! Schedules used in Stardust.

use bevy::prelude::*;

/// Systems dealing with receiving messages. Run in the [`PreUpdate`] schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkRecv {
    /// Transport layers receive packets or other transmission media.
    Receive,
    /// Game systems process messages and mutate the World before [`Update`].
    /// You can still read messages at any time before [`Receive`](NetworkRecv::Receive).
    Read,
}

/// Systems dealing with outgoing octet strings. Run in the [`PostUpdate`] schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkSend {
    /// Transport layers send messages queued by game systems.
    Transmit,
    /// Records statistics for diagnostic purposes.
    Diagnostics,
    /// Queued messages (both the incoming and outgoing buffers) are cleared.
    Clear,
}

pub(super) fn configure_scheduling(app: &mut App) {
    app.configure_sets(PreUpdate, (
        NetworkRecv::Read.after(NetworkRecv::Receive),
    ));

    app.configure_sets(PostUpdate, (
        NetworkSend::Diagnostics.after(NetworkSend::Transmit),
        NetworkSend::Clear.after(NetworkSend::Diagnostics),
    ));
}