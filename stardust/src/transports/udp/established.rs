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
    pub last_sent: Option<Instant>,
    pub last_received: Option<Instant>,
    pub timeout: Duration,
    pub reliability: Reliability,
    pub ordering: Ordering,
}

#[derive(Debug, Clone)]
pub(super) enum Disconnected {
    /// The reason could not be determined, as the input bytes were unknown.
    DeserializationError,

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
            Disconnected::DeserializationError => f.write_str("Unknown response type"),
            Disconnected::HandshakeMalformedPacket => f.write_str("Peer sent a malformed packet during the handshake"),
            Disconnected::HandshakeUnknownTransport { identifier } => f.write_str(&format!("Peer's transport layer identifier was unknown ({identifier:X})")),
            Disconnected::HandshakeWrongVersion { version } => f.write_str(&format!("Peer's transport layer version ({version}) was out of range")),
            Disconnected::HandshakeWrongProtocol { protocol } => f.write_str(&format!("Peer's protocol hash ({protocol:X})was incompatible")),
            Disconnected::OverMemoryBudget => f.write_str("Peer exceeded maximum memory usage for unacknowledged packets"),
        }
    }
}

impl From<&[u8]> for Disconnected {
    fn from(value: &[u8]) -> Self {
        if value.len() == 0 { return Self::DeserializationError }
        match value[0] {
            0 => Self::DeserializationError,
            1 => Self::HandshakeMalformedPacket,
            2 => {
                if value.len() < 9 { return Self::DeserializationError }
                Self::HandshakeUnknownTransport {
                    identifier: u64::from_be_bytes(value[1..9].try_into().unwrap())
                }
            },
            3 => {
                if value.len() < 5 { return Self::DeserializationError }
                Self::HandshakeWrongVersion {
                    version: u32::from_be_bytes(value[1..5].try_into().unwrap())
                }
            },
            4 => {
                if value.len() < 9 { return Self::DeserializationError }
                Self::HandshakeWrongProtocol {
                    protocol: u64::from_be_bytes(value[1..9].try_into().unwrap())
                }
            },
            5 => Self::OverMemoryBudget,
            _ => Self::DeserializationError,
        }
    }
}

impl Into<Box<[u8]>> for Disconnected {
    fn into(self) -> Box<[u8]> {
        let mut buffer: Vec<u8> = Vec::with_capacity(1);
        match self {
            Disconnected::DeserializationError => buffer.push(0),
            Disconnected::HandshakeMalformedPacket => buffer.push(1),
            Disconnected::HandshakeUnknownTransport { identifier } => {
                buffer.push(2);
                buffer.clone_from_slice(&u64::to_be_bytes(identifier));
            },
            Disconnected::HandshakeWrongVersion { version } => {
                buffer.push(3);
                buffer.clone_from_slice(&u32::to_be_bytes(version));
            },
            Disconnected::HandshakeWrongProtocol { protocol } => {
                buffer.push(4);
                buffer.clone_from_slice(&u64::to_be_bytes(protocol));
            },
            Disconnected::OverMemoryBudget => {
                buffer.push(5);
            },
        }
        buffer.into()
    }
}