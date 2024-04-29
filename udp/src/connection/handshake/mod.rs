mod codes;
mod impls;
mod packets;
mod system;

pub(crate) use system::{handshake_polling_system, potential_new_peers_system};

use bevy_stardust::connections::NetworkPeer;
use bytes::Bytes;
use std::{net::SocketAddr, time::Instant};
use bevy::prelude::*;
use crate::prelude::*;
use super::reliability::ReliabilityState;
use codes::HandshakeResponseCode;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct Handshaking {
    started: Instant,
    state: HandshakeState,
    reliability: ReliabilityState,
}

#[derive(Debug)]
enum HandshakeState {
    ClientHello,
    ServerHello,
    Finished,
    Failed(HandshakeFailureReason),
}

impl HandshakeState {
    pub fn is_end(&self) -> bool {
        use HandshakeState::*;
        match self {
            Finished | Failed(_) => true,
            _ => false,
        }
    }
}

impl From<HandshakeFailureReason> for HandshakeState {
    fn from(value: HandshakeFailureReason) -> Self {
        Self::Failed(value)
    }
}

#[derive(Debug)]
enum HandshakeFailureReason {
    TimedOut,
    WeRejected {
        code: HandshakeResponseCode,
        message: Option<Bytes>,
    },
    TheyRejected {
        code: HandshakeResponseCode,
        message: Option<Bytes>,
    },
}

impl std::fmt::Display for HandshakeFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HandshakeFailureReason::*;
        match self {
            TimedOut => f.write_str("timed out"),
            WeRejected { code, message } => f.write_fmt(format_args!("we rejected remote peer: {code} ({message:?})")),
            TheyRejected { code, message } => f.write_fmt(format_args!("rejected by remote peer: {code} ({message:?})")),
        }
    }
}

#[derive(Bundle)]
pub(crate) struct OutgoingHandshake {
    pub connection: Connection,
    handshake: Handshaking,
    peercomp: NetworkPeer,
}

impl OutgoingHandshake {
    pub fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
    ) -> Self {
        Self {
            connection: Connection::new(
                owning_endpoint,
                remote_address, 
                ConnectionDirection::Client,
            ),
            handshake: Handshaking {
                started: Instant::now(),
                state: HandshakeState::ClientHello,
                reliability: ReliabilityState::new(),
            },
            peercomp: NetworkPeer::new(),
        }
    }
}