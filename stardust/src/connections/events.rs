use bevy::prelude::*;
use bytes::Bytes;

/// Send to disconnect a peer.
/// 
/// This should be sent by the dev, not a plugin.
#[derive(Event)]
pub struct DisconnectPeerEvent {
    /// The peer to be disconnected.
    pub target: Entity,
    /// The reason for disconnection.
    pub reason: Option<Bytes>,
    /// Whether or not the peer should be disconnected immediately.
    /// This may cause data loss if set to `true`, and should be used sparingly.
    pub force: bool,
}

/// Sent when a peer is disconnected.
/// 
/// This should be sent by a transport layer.
#[derive(Event)]
pub struct PeerDisconnectedEvent {
    /// The peer that disconnected.
    pub peer: Entity,
    /// The reason for disconnection, if one is available.
    pub reason: Option<Bytes>,
}