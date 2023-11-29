//! Connection events, like players leaving or joining.

use bevy::{prelude::*, utils::Uuid};

/// Raise to try and disconnect a peer, with an optional reason.
#[derive(Event)]
pub struct DisconnectPeerEvent {
    /// The peer to disconnect.
    pub target: Entity,
    /// The reason for disconnection.
    pub reason: Option<Box<str>>,
}

/// Raised when a peer tries to connect, but fails.
#[derive(Event)]
pub struct FailedConnectionEvent {
    /// The origin of the connection, such as a `SocketAddr`.
    pub origin: Box<str>,
    /// The reason the connection failed.
    pub reason: Box<str>,
}

/// Raised when a new peer connects.
#[derive(Event)]
pub struct PeerConnectedEvent(pub Entity);

/// Raised when a peer disconnects.
#[derive(Event)]
pub struct PeerDisconnectedEvent {
    /// The entity id of the peer.
    pub entity_id: Entity,
    /// The peer's `PeerUuid` value, if it had one.
    pub uuid: Option<Uuid>,
    /// The reason the peer was disconnected.
    pub reason: Box<str>,
}