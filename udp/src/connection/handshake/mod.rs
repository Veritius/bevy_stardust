mod codes;
mod impls;
mod packets;
mod system;

pub(crate) use system::{handshake_polling_system, potential_new_peers_system};

use std::{net::SocketAddr, time::Instant};
use bevy_ecs::prelude::*;
use crate::{Connection, ConnectionDirection};
use super::reliability::ReliabilityState;
use codes::HandshakeResponseCode;

#[derive(Bundle)]
pub(crate) struct OutgoingHandshake {
    pub connection: Connection,
    handshake: Handshaking,
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
                ConnectionDirection::Outgoing,
            ),
            handshake: Handshaking {
                started: Instant::now(),
                state: HandshakeState::ClientHello,
                reliability: ReliabilityState::new(),
            },
        }
    }
}

#[derive(Component)]
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
    WeRejected(HandshakeResponseCode),
    TheyRejected(HandshakeResponseCode),
}

impl std::fmt::Display for HandshakeFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HandshakeFailureReason::*;
        match self {
            TimedOut => f.write_str("timed out"),
            WeRejected(error_code) => f.write_fmt(format_args!("we rejected by remote peer: {error_code}")),
            TheyRejected(error_code) => f.write_fmt(format_args!("rejected by remote peer: {error_code}")),
        }
    }
}