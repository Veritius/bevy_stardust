//! Connection events.

use std::sync::Arc;
use bevy::prelude::*;

macro_rules! dir_comment {
    (t2a) => { "\n\nThis is sent by transport layers, and read by application systems." };
    (a2t) => { "\n\nThis is sent by application systems, and read by transport layers." };
}

/// Sent when a peer begins to connect.
#[doc = dir_comment!(t2a)]
#[derive(Event)]
pub struct PeerConnectingEvent {
    /// The peer that is connecting.
    pub peer: Entity,
}

/// Sent when a peer finishes connecting.
#[doc = dir_comment!(t2a)]
#[derive(Event)]
pub struct PeerConnectedEvent {
    /// The peer that has connected.
    pub peer: Entity,
}

/// Send to disconnect a peer.
#[doc = dir_comment!(a2t)]
#[derive(Event)]
pub struct DisconnectPeerEvent {
    /// The peer to be disconnected.
    pub peer: Entity,
    /// The reason for disconnection.
    pub reason: Option<Arc<str>>,
    /// Whether or not the peer should be disconnected immediately.
    /// This may cause data loss if set to `true`, and should be used sparingly.
    pub force: bool,
}

/// Sent when a peer starts to disconnect.
#[doc = dir_comment!(t2a)]
#[derive(Event)]
pub struct PeerDisconnectingEvent {
    /// The peer that is disconnecting.
    pub peer: Entity,
    /// The reason for disconnection, if one is available.
    pub reason: Option<Arc<str>>,
}

/// Sent when a peer is disconnected.
#[doc = dir_comment!(t2a)]
#[derive(Event)]
pub struct PeerDisconnectedEvent {
    /// The peer that disconnected.
    pub peer: Entity,
    /// The reason for disconnection, if one is available.
    pub reason: Option<Arc<str>>,
}