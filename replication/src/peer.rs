use bevy::prelude::*;

/// Stardust channel for negotiating replication peer data.
#[derive(Default)]
pub(crate) struct PeerNegotiation;

/// Enables replication for a peer. Must be added manually.
/// 
/// ## Sidedness
/// You must ensure that for a given connection, one side is a [`Client`](Side::Client) and the other is a [`Server`](Side::Server).
/// Failure to do so will result in the connection immediately being terminated and an error being logged.
/// 
/// The side must not change after being created, especially not through reflection interfaces.
/// If this occurs, it'll caused unexpected (but memory safe) behavior leading to connection termination.
#[derive(Debug, Component, Reflect)]
#[reflect(Debug, Component)]
pub struct ReplicationPeer {
    side: Side,
}

impl ReplicationPeer {
    /// Creates a new `ReplicationPeer` for a [`Side`], with reasonable defaults for configuration.
    pub fn new(side: Side) -> Self {
        Self { side }
    }

    /// Returns the [`Side`] of this [`ReplicationPeer`].
    pub fn side(&self) -> Side {
        self.side
    }
}

/// Defines whether a peer is a client or a server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum Side {
    /// A **non-authority** over the connection.
    /// There can be unlimited clients out of all connections.
    Client,

    /// The **authority** over the connection.
    /// There can only be one server out of all connections.
    Server,
}