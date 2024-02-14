//! Connection events, like players leaving or joining.

use bevy_ecs::prelude::*;

/// Raise to try and disconnect a peer, with an optional reason.
#[derive(Event)]
pub struct DisconnectPeerEvent {
    /// The peer to disconnect.
    pub target: Entity,
    /// The reason for disconnection.
    pub reason: Option<Box<str>>,
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
    pub uuid: Option<uuid::Uuid>,
    /// The reason the peer was disconnected.
    pub reason: Box<str>,
}