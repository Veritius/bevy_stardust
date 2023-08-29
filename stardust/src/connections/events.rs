use bevy::prelude::*;

/// An event sent to disconnect a peer.
#[derive(Event)]
pub struct DisconnectPeerEvent(pub Entity);

/// Raised when a new peer connects.
#[derive(Event)]
pub struct PeerConnectedEvent(pub Entity);

/// Raised when a peer disconnects.
#[derive(Event)]
pub struct PeerDisconnectedEvent(pub Entity);