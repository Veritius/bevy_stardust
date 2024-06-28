//! Connection events.

use std::{fmt::Display, sync::Arc, time::Duration};
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
    /// such as a ban reason or detailed error code.
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
    /// such as a ban reason or detailed error code.
    pub comment: Option<Arc<str>>,
}

impl Display for PeerDisconnectedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("peer {:?} disconnected: {}", self.peer, self.reason))?;

        // If a comment is present, show it
        if let Some(comment) = &self.comment {
            f.write_fmt(format_args!(" ({comment})"))?;
        }

        return Ok(())
    }
}

/// A reason for disconnection.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum DisconnectReason {
    /// No reason given.
    #[default]
    Unspecified,

    /// The connection was finished gracefully,
    /// and both sides terminated with no data loss.
    Finished,

    /// The peer failed some kind of verification check for protocol,
    /// such as checking game versions, or game modifications.
    /// This most often will occur during a handshake.
    FailedVerification,

    /// The peer failed some kind of authentication check for their identity,
    /// such as an account ID, key exchange, or a TLS certificate.
    /// This most often will occur during a handshake.
    FailedAuthentication,

    /// The connection was refused by the remote peer,
    /// as their acceptance would exceed the limit for some resource.
    /// 
    /// This reason is returned for instances such as a
    /// server at capacity, or a full lobby in a party game.
    ResourceCapacity,

    /// The peer stopped responding.
    TimedOut {
        /// The duration between the last data received from the peer, and the time of disconnection.
        after: Duration,
    },

    /// The transport layer identified the peer as violating
    /// its protocol, and was subsequently disconnected.
    ProtocolViolation,

    /// The peer behaved unexpectedly, and was disconnected by the application.
    /// This is useful for instances such as kicking buggy or hacked clients.
    Misbehaving,
}

impl Display for DisconnectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DisconnectReason::*;

        match self {
            Unspecified => f.write_str("no reason given"),
            Finished => f.write_str("finished"),
            FailedVerification => f.write_str("failed verification"),
            FailedAuthentication => f.write_str("failed authentication"),
            ResourceCapacity => f.write_str("at capacity"),
            ProtocolViolation => f.write_str("transport protocol violation"),
            Misbehaving => f.write_str("peer misbehaving"),

            TimedOut { after } => {
                let (secs, millis) = (after.as_secs(), after.subsec_millis());
                if (secs, millis) == (0, 0) { return f.write_str("timed out immediately"); }
    
                f.write_str("timed out after ")?;
                if secs != 0 { f.write_fmt(format_args!("{secs} seconds "))?; }
                if secs != 0 && millis != 0 { f.write_str("and ")?; }
                if millis != 0 { f.write_fmt(format_args!("{millis} milliseconds"))?; }

                return Ok(())
            },
        }
    }
}