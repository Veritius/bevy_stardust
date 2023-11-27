//! Schedules used in Stardust.

use bevy::prelude::*;

/// Systems dealing with incoming octet strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkRead {
    /// Transport layers receive packets from the OS.
    Receive,
    /// Game systems process octet strings and mutate the World before [Update].
    Read,
}

/// Systems dealing with outgoing octet strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum NetworkWrite {
    /// Game systems write octet strings for sending.
    Write,
    /// Transport layers send packets written by game systems.
    Send,
}