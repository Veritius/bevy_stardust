//! Connection events.

use std::{sync::Arc, time::Duration};
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

/// Sent by the application to disconnect a peer.
#[doc = dir_comment!(a2t)]
#[derive(Debug, Clone, Event)]
pub struct DisconnectPeerEvent {
    /// The peer to be disconnected.
    pub peer: Entity,

    /// The reason for disconnection.
    pub reason: DisconnectReason,

    /// A human-readable string associated with the disconnection.
    /// This is useful to communicate information that isn't in the reason enum,
    /// such as a player being banned for flyhacking.
    pub comment: Option<Arc<str>>,

    /// Whether or not the peer should be disconnected immediately.
    /// This may cause data loss if set to `true`, and should be used sparingly.
    pub force: bool,
}

/// Sent when a peer starts to disconnect.
/// 
/// Doesn't contain any information about why the disconnect is occurring.
/// For that, wait for the [`PeerDisconnectedEvent`] event.
#[doc = dir_comment!(t2a)]
#[derive(Debug, Clone, Event)]
pub struct PeerDisconnectingEvent {
    /// The peer that is disconnecting.
    pub peer: Entity,
}

/// Sent when a peer is disconnected.
#[doc = dir_comment!(t2a)]
#[derive(Debug, Clone, Event)]
pub struct PeerDisconnectedEvent {
    /// The peer that disconnected.
    pub peer: Entity,

    /// The reason for disconnection, if one is available.
    pub reason: DisconnectReason,

    /// A human-readable string associated with the disconnection.
    /// This is useful to communicate information that isn't in the reason enum,
    /// such as a player being banned for flyhacking.
    pub comment: Option<Arc<str>>,
}

/// A reason for disconnection.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum DisconnectReason {
    /// No reason given.
    #[default]
    Unspecified,

    /// The connection was refused.
    Refused,

    /// The peer stopped responding for too long.
    TimedOut {
        /// The duration between the last data received from the peer, and the time of disconnection.
        after: Duration,
    },

    /// The transport layer identified the peer as violating its protocol.
    Protocol,

    /// The peer behaved unexpectedly, and was disconnected by the application.
    Misbehaving,
}