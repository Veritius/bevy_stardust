use std::{net::SocketAddr, time::Instant};
use bevy::prelude::*;

/// A component for entities that represent a virtual connection.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
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

/// The lifestage of a connection.
/// 
/// This exists to model the average lifecycle of a connection, from an initial handshake to being disconnected.
/// An `Ord` implementation is provided, with variants being 'greater' if they're later in the model lifecycle.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[reflect(Debug, Component, PartialEq)]
#[non_exhaustive]
pub enum PeerLifestage {
    /// Midway through a [handshake].
    /// Messages sent to peers in this stage will likely be ignored.
    /// 
    /// [handshake]: https://en.wikipedia.org/wiki/Handshake_(computing)
    Handshaking,

    /// Fully connected and communicating normally.
    Established,

    /// In the process of closing the connection.
    /// 
    /// This step may be skipped and peers will jump directly to the `Closed` stage from any other variant.
    Closing,

    /// The connection is closed.
    Closed,
}

/// The IP address of a network peer, if it has one.
#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerAddress(pub SocketAddr);

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