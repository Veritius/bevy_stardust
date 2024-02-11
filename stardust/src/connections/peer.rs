//! "Peers" aka other computers over the network.

use std::time::Instant;
use bevy::{prelude::*, utils::Uuid};

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
#[derive(Debug, Component, Reflect)]
pub struct NetworkPeer {
    /// The point in time this peer was added to the `World`.
    pub joined: Instant,

    /// The connection state of the network peer.
    pub state: NetworkPeerState,

    /// A unique UUID, if it has one.
    /// This can be used to identify a peer across network sessions.
    pub uuid: Option<Uuid>,

    /// The quality of the connection, from `0.0` to `1.0`.
    /// This is subjective and defined by the transport layer.
    pub quality: f32,

    /// Round-trip time, in milliseconds.
    pub ping: u32,

    disconnect_requested: bool,
}

impl NetworkPeer {
    /// Signals to the transport layer to disconnect the peer.
    /// This operation cannot be undone.
    pub fn disconnect(&mut self) {
        self.disconnect_requested = true
    }

    /// Returns `true` if [`disconnect`] has been used.
    /// This is intended for use by transport layers, and you should use [`NetworkPeerState`] instead.
    pub fn disconnect_requested(&self) -> bool {
        self.disconnect_requested
    }
}

/// The connection state of a [`NetworkPeer`]. See variants for more documentation.
#[derive(Debug, Reflect)]
pub enum NetworkPeerState {
    /// The peer is in the process of connecting and/or authenticating.
    Connecting,

    /// The peer is fully connected and ready for data transmission.
    Connected,

    /// The peer is being disconnected by a transport layer.
    Disconnecting,

    /// The peer has been fully disconnected and will be removed from the World soon.
    Disconnected,
}