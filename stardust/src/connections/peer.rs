//! "Peers" aka other computers over the network.

use std::{net::SocketAddr, time::Instant};
use bevy::prelude::*;

/// An active connection to a remote peer.
/// 
/// The term 'peer' is used interchangeably for any kind of connection to another instance of the application.
/// If you're writing a star-topology system, you can treat these as servers and clients.
/// If you're writing a mesh-topology system, you can treat these as another peer in the mesh.
/// 
/// The `NetworkPeer` component is intended to be queried freely, but not created by the developer.
/// Instead, it should be managed by transport layer plugins.
/// 
/// Entities with this component may also have the following components:
/// - [`NetworkMessages`](crate::messages::NetworkMessages), relating to messages.
/// - [`NetworkPeerAddress`], relating to IP address data.
/// - [`NetworkPeerUid`], relating to persistent data.
/// - [`NetworkPeerLifestage`], relating to connection state.
/// - [`NetworkSecurity`](super::security::NetworkSecurity), relating to encryption.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
pub struct NetworkPeer {
    /// The point in time this peer was added to the `World`.
    pub joined: Instant,

    /// The quality of the connection, from `0.0` to `1.0`.
    /// This is subjective and defined by the transport layer.
    /// `None` means a value is not provided.
    pub quality: Option<f32>,

    /// Round-trip time estimate, in milliseconds.
    /// `None`  means an estimate is not available.
    pub ping: Option<u32>,
}

impl NetworkPeer {
    /// Creates the component in the `Handshaking` state.
    pub fn new() -> Self {
        Self {
            joined: Instant::now(),
            quality: None,
            ping: None,
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
pub enum NetworkPeerLifestage {
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

    /// The connection is closed, and the entity will soon be despawned automatically.
    Closed,
}

/// The IP address of a network peer, if it has one.
#[derive(Component, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NetworkPeerAddress(pub SocketAddr);

/// A unique identifier for a [`NetworkPeer`], to store persistent data across multiple connections.
/// This component should only be constructed by the app developer, but can be read by any plugins.
/// 
/// If you're working with another ID namespace, like UUIDs and Steam IDs, you should
/// map the ids from that space into a unique value here through some kind of associative array.
#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
#[reflect(Debug, Component, PartialEq, Hash)]
pub struct NetworkPeerUid(pub u64);

impl std::fmt::Debug for NetworkPeerUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:X}", self.0))
    }
}

impl std::fmt::Display for NetworkPeerUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl From<u64> for NetworkPeerUid {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<NetworkPeerUid> for u64 {
    #[inline]
    fn from(value: NetworkPeerUid) -> Self {
        value.0
    }
}