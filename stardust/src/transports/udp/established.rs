use std::{net::SocketAddr, time::{Instant, Duration}, fmt::Display};
use bevy::prelude::*;
use super::{reliability::Reliability, ordering::Ordering};

/// If set to `false`, new incoming connections will be ignored.
#[derive(Debug, Resource)]
pub(super) struct AllowNewConnections(pub bool);

/// A UDP connection. May or may not be fully connected.
#[derive(Debug, Component)]
pub(super) struct UdpConnection {
    pub address: SocketAddr,
    pub started: Instant,
    pub last_sent: Option<Instant>,
    pub last_received: Option<Instant>,
    pub timeout: Duration,
    pub reliability: Reliability,
    pub ordering: Ordering,
}

#[derive(Debug, Clone)]
pub(super) enum Disconnected {
    /// A critical packet could not be parsed, had invalid data, or had unexpected data.
    HandshakeMalformedPacket,

    /// The transport layer was not known.
    HandshakeUnknownTransport {
        identifier: u64,
    },

    /// The peer's transport layer was incompatible.
    HandshakeWrongVersion {
        version: u32,
    },

    /// The protocol hash was incorrect.
    HandshakeWrongProtocol {
        protocol: u64,
    },

    /// Too much unacknowledged data was being stored.
    OverMemoryBudget,
}

impl Display for Disconnected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Disconnected::HandshakeMalformedPacket => f.write_str("Peer sent a malformed packet during the handshake"),
            Disconnected::HandshakeUnknownTransport { identifier } => f.write_str(&format!("Peer's transport layer identifier was unknown ({identifier:X})")),
            Disconnected::HandshakeWrongVersion { version } => f.write_str(&format!("Peer's transport layer version ({version}) was out of range")),
            Disconnected::HandshakeWrongProtocol { protocol } => f.write_str(&format!("Peer's protocol hash ({protocol:X})was incompatible")),
            Disconnected::OverMemoryBudget => f.write_str("Peer exceeded maximum memory usage for unacknowledged packets"),
        }
    }
}