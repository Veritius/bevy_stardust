use std::{net::IpAddr, time::Instant};
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;

/// A component for entities that represent a virtual connection.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
#[non_exhaustive]
pub struct Peer {
    /// The point in time this peer was added to the `World`.
    pub joined: Instant,
}

impl Peer {
    /// Creates the component in the `Handshaking` state.
    pub fn new() -> Self {
        Self {
            joined: Instant::now(),
        }
    }
}

/// The IP address of a network peer, if it has one.
#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerAddress(pub IpAddr);

/// A unique identifier for a [`Peer`], to store persistent data across multiple connections.
/// This component should only be constructed by the app developer, but can be read by any plugins.
/// 
/// If you're working with another ID namespace, like UUIDs and Steam IDs, you should
/// map the ids from that space into a unique value here through some kind of associative array.
#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
#[reflect(Debug, Component, PartialEq, Hash)]
pub struct PeerUid(pub u64);

impl std::fmt::Debug for PeerUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:X}", self.0))
    }
}

impl std::fmt::Display for PeerUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl From<u64> for PeerUid {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<PeerUid> for u64 {
    #[inline]
    fn from(value: PeerUid) -> Self {
        value.0
    }
}