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

    /// A unique UUID, if it has one.
    /// This can be used to identify a peer across network sessions.
    pub uuid: Option<Uuid>,

    /// The quality of the connection, from `0.0` to `1.0`.
    pub quality: f32,

    /// Round-trip time, in milliseconds.
    pub ping: u32,
}

impl NetworkPeer {
    /// Creates a new [NetworkPeer] component with `connected` set to now.
    pub fn new() -> Self {
        Self {
            joined: Instant::now(),
            uuid: None,
            quality: 1.0,
            ping: 0,
        }
    }
}