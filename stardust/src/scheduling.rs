//! Schedules used in Stardust.

use bevy::prelude::*;

/// Systems dealing with incoming octet strings. Run in the `PreUpdate` schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkRead {
    /// Transport layers receive packets from the OS.
    Receive,
    /// Game systems process octet strings and mutate the World before [Update].
    Read,
}

/// Systems dealing with outgoing octet strings. Run in the `PostUpdate` schedule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkWrite {
    /// Transport layers send packets written by game systems.
    Send,
    /// Queued messages in `OutgoingNetworkMessages` are cleared.
    Clear,
}