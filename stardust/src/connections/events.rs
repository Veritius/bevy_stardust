//! Connection events, like players leaving or joining.

use std::fmt::Display;
use bevy::{prelude::*, utils::Uuid};

/// An event sent to disconnect a peer.
#[derive(Event)]
pub struct DisconnectPeerEvent(pub Entity);

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
    pub reason: Box<dyn Display + Send + Sync>,
}