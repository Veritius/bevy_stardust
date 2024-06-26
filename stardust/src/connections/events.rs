use bevy::prelude::*;
use bytes::Bytes;

/// Sent when a peer begins to connect.
/// 
/// This should be sent by a transport layer.
#[derive(Event)]
pub struct PeerConnectingEvent {
    /// The peer that is connecting.
    pub peer: Entity,
}

/// Sent when a peer finishes connecting.
/// 
/// This should be sent by a transport layer.
#[derive(Event)]
pub struct PeerConnectedEvent {
    /// The peer that has connected.
    pub peer: Entity,
}

/// Send to disconnect a peer.
/// 
/// This should be sent by the dev, not a plugin.
#[derive(Event)]
pub struct DisconnectPeerEvent {
    /// The peer to be disconnected.
    pub peer: Entity,
    /// The reason for disconnection.
    pub reason: Option<Bytes>,
    /// Whether or not the peer should be disconnected immediately.
    /// This may cause data loss if set to `true`, and should be used sparingly.
    pub force: bool,
}


/// Sent when a peer starts to disconnect.
/// 
/// This should be sent by a transport layer.
#[derive(Event)]
pub struct PeerDisconnectingEvent {
    /// The peer that is disconnecting.
    pub peer: Entity,
    /// The reason for disconnection, if one is available.
    pub reason: Option<Bytes>,
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