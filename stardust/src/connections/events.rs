//! Connection events, like players leaving or joining.

use bevy_ecs::prelude::*;

/// Raised when a new peer connects.
#[derive(Event)]
pub struct PeerConnectedEvent {
    /// The ID of the peer that has connected.
    pub peer: Entity,
}

/// Raised when a peer disconnects.
#[derive(Event)]
pub struct PeerDisconnectedEvent {
    /// The entity id of the peer.
    pub entity_id: Entity,

    /// The peer's `PeerUuid` value, if it had one.
    #[cfg(feature="uuids")]
    pub uuid: Option<uuid::Uuid>,

    /// The reason the peer was disconnected.
    pub reason: Box<str>,
}

impl PeerDisconnectedEvent {
    /// Returns a new `PeerDisconnectedEvent`.
    pub fn new(peer: Entity, reason: &str) -> Self {
        Self {
            entity_id: peer,
            #[cfg(feature="uuids")]
            uuid: None,
            reason: reason.into(),
        }
    }

    /// Returns a new `PeerDisconnectedEvent` with an associated uuid.
    #[cfg(feature="uuids")]
    pub fn new_with_uuid(peer: Entity, uuid: uuid::Uuid, reason: &str) -> Self {
        Self {
            entity_id: peer,
            uuid: Some(uuid),
            reason: reason.into(),
        }
    }
}