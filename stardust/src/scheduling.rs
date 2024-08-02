//! Schedules used in Stardust.

use bevy::prelude::*;

/// Systems dealing with receiving messages. Run in the [`PreUpdate`] schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkRecv {
    /// Transport layers insert received messages into [`PeerMessages<Incoming>`](crate::connections::PeerMessages) components.
    Receive,

    /// Systems update game state and deal with after-effects of received messages,
    /// before the main game systems in [`Update`] are run.
    /// 
    /// You can still read messages at any time after [`Receive`](NetworkRecv::Receive).
    Synchronise,
}

/// Systems dealing with sending messages. Run in the [`PostUpdate`] schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkSend {
    /// Transport layers send messages queued in [`PeerMessages<Outgoing>`](crate::connections::PeerMessages) components.
    Transmit,

    /// Network statistics and diagnostics are recorded.
    Diagnostics,

    /// Queued messages (both the incoming and outgoing `PeerMessages` buffers) are cleared.
    Clear,
}

pub(super) fn configure_scheduling(app: &mut App) {
    app.configure_sets(PreUpdate, (
        NetworkRecv::Synchronise.after(NetworkRecv::Receive),
    ));

    app.configure_sets(PostUpdate, (
        NetworkSend::Diagnostics.after(NetworkSend::Transmit),
        NetworkSend::Clear.after(NetworkSend::Diagnostics),
    ));
}