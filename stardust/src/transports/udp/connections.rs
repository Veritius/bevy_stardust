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
    pub status: ConnectionStatus,
}

impl UdpConnection {
    pub fn new_incoming(address: SocketAddr, timeout: Duration) -> Self {
        Self {
            address,
            started: Instant::now(),
            last_sent: None,
            last_received: None,
            timeout,
            reliability: Reliability::default(),
            ordering: Ordering::default(),
            status: ConnectionStatus::PendingIncoming(PendingIncoming::default()),
        }
    }

    /// Create an outgoing connection attempt to a new peer.
    pub fn new_outgoing(address: SocketAddr, timeout: Duration) -> Self {
        Self {
            address,
            started: Instant::now(),
            last_sent: None,
            last_received: None,
            timeout,
            reliability: Reliability::default(),
            ordering: Ordering::default(),
            status: ConnectionStatus::PendingOutgoing(PendingOutgoing::default()),
        }
    }
}

#[derive(Debug)]
pub(super) enum ConnectionStatus {
    /// A connection attempt from a hitherto-unknown remote peer.
    PendingIncoming(PendingIncoming),
    /// A connection attempt to a known remote peer, emanating from this peer.
    PendingOutgoing(PendingOutgoing),
    /// A fully established connection.
    Established(Established),
    /// A previously established connection that is closed.
    Disconnected(Disconnected),
}

#[derive(Debug)]
pub(super) struct PendingIncoming {
    pub state: PendingIncomingState,
}

impl Default for PendingIncoming {
    fn default() -> Self {
        Self {
            state: PendingIncomingState::JustRegistered,
        }
    }
}

#[derive(Debug)]
pub(super) enum PendingIncomingState {
    /// The peer has just been noticed, and hasn't even had one of their messages processed.
    JustRegistered,
    /// The peer has finished the handshake and will soon become `Established`.
    Accepted,
    /// The peer has failed the handshake and will move to the `Disconnected` state with the enclosed reason.
    Rejected(Disconnected),
}

#[derive(Debug)]
pub(super) struct PendingOutgoing {
    pub state: PendingOutgoingState,
}

impl Default for PendingOutgoing {
    fn default() -> Self {
        Self {
            state: PendingOutgoingState::WaitingForResponse,
        }
    }
}

#[derive(Debug)]
pub(super) enum PendingOutgoingState {
    WaitingForResponse,
    Accepted,
}

#[derive(Debug)]
pub(super) struct Established;

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