//! Schedules used in Stardust.

use bevy_ecs::prelude::*;
use bevy_app::prelude::*;

/// Systems dealing with incoming octet strings. Run in the `PreUpdate` schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkRead {
    /// Transport layers receive packets from the OS.
    Receive,
    /// Game systems process octet strings and mutate the World before [Update].
    /// You can still read octet strings at any time, not just in this component.
    Read,
}

/// Systems dealing with outgoing octet strings. Run in the `PostUpdate` schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkWrite {
    /// Transport layers send packets written by game systems.
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