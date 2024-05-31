mod codes;
mod messages;
mod system;

use bytes::Bytes;
pub(crate) use system::handshake_polling_system;

use bevy_stardust::connections::NetworkPeer;
use std::{net::SocketAddr, time::Instant};
use bevy::prelude::*;
use crate::prelude::*;
use self::codes::HandshakeResponseCode;
use super::reliability::ReliabilityState;

#[derive(Component)]
pub(crate) struct Handshaking {
    state: HandshakeState,
    started: Instant,
    last_sent: Option<Instant>,
    direction: Direction,
    reliability: ReliabilityState,
}

impl Handshaking {
    fn terminate(
        &mut self,
        code: HandshakeResponseCode,
        reason: Option<Bytes>
    ) {
        self.state = HandshakeState::Terminated(Termination { code, reason });
    }
}

#[derive(Clone)]
enum HandshakeState {
    Hello,
    Completed,
    Terminated(Termination),
}

#[derive(Clone)]
struct Termination {
    pub code: HandshakeResponseCode,
    pub reason: Option<Bytes>,
}

#[derive(Bundle)]
pub(crate) struct OutgoingHandshakeBundle {
    pub connection: Connection,
    handshake: Handshaking,
    peercomp: NetworkPeer,
}

impl OutgoingHandshakeBundle {
    pub fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
    ) -> Self {
        Self {
            connection: Connection::new(
                owning_endpoint,
                remote_address, 
            ),
            handshake: Handshaking {
                state: HandshakeState::Hello,
                started: Instant::now(),
                last_sent: None,
                direction: Direction::Listener,
                reliability: ReliabilityState::new(),
            },
            peercomp: NetworkPeer::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Initiator,
    Listener,
}