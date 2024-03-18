//! "Peers" aka other computers over the network.

use std::time::Instant;
use bevy_ecs::prelude::*;

/// Another peer that this peer is aware of, representing someone else over the Internet.
/// 
/// - If you're writing server-side code, you can think of this as a client.
/// - If you're writing client-side code, you can think of this as the server.
/// - If you're writing peer-to-peer code, you can think of this as another peer in the mesh.
/// 
/// `NetworkPeer`s don't have any associated transport layer information by themselves.
/// However, they are always treated as an entity that stores information related to the network.
/// You shouldn't create, mutate, or delete this component unless you know what you're doing.
/// Managing these entities should be (and usually is) done by the transport layer.
/// 
/// Entities with `NetworkPeer` have their entity IDs used in the writing and reading APIs.
/// They are used as the 'target' of messages, and the transport layer will handle the actual sending and receiving.
#[derive(Debug, Component)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect))]
pub struct NetworkPeer {
    /// The point in time this peer was added to the `World`.
    pub joined: Instant,

    /// The quality of the connection, from `0.0` to `1.0`.
    /// This is subjective and defined by the transport layer.
    /// `None` means a value is not provided.
    pub quality: Option<f32>,

    /// Round-trip time estimate, in milliseconds.
    pub ping: u32,

    disconnect_requested: bool,
}

impl NetworkPeer {
    /// Creates the component in the `Handshaking` state.
    pub fn new() -> Self {
        Self {
            joined: Instant::now(),
            quality: None,
            ping: 0,
            disconnect_requested: false,
        }
    }

    /// Signals to the transport layer to disconnect the peer.
    /// This operation cannot be undone.
    pub fn disconnect(&mut self) {
        self.disconnect_requested = true
    }

    /// Returns `true` if [`disconnect`] has been used.
    /// This is intended for use by transport layers, and you should use [`NetworkPeerLifestage`] instead.
    pub fn disconnect_requested(&self) -> bool {
        self.disconnect_requested
    }
}

/// The lifestage of a connection.
/// 
/// This exists to model the average lifecycle of a connection, from an initial handshake to being disconnected.
/// An `Ord` implementation is provided, with variants being 'greater' if they're later in the model lifecycle.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect))]
#[non_exhaustive]
pub enum NetworkPeerLifestage {
    /// Midway through a [handshake].
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

/// A unique identifier for a [`NetworkPeer`], to store persistent data across multiple connections.
/// This component should only be constructed by the app developer, but can be read by any plugins.
/// 
/// This value is intended only for use within memory and local databases, like savegames.
/// If you need to share a unique player identifier, use UUIDs.
/// 
/// If you're working with another ID namespace, like UUIDs and Steam IDs, you should
/// map the ids from that space into a unique value here through some kind of associative array.
/// 
/// The `Display` implementation will display the internal integer in hexadecimal.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect))]
pub struct NetworkPeerUid(pub u64);

impl std::fmt::Display for NetworkPeerUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:X}", self.0))
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