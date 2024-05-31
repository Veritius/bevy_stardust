mod codes;
mod messages;
mod system;

use bytes::Bytes;
use bevy_stardust::connections::NetworkPeer;
use std::{net::SocketAddr, time::{Duration, Instant}};
use bevy::prelude::*;
use crate::prelude::*;
use self::codes::HandshakeResponseCode;
use super::reliability::ReliabilityState;

pub(super) use system::{
    handshake_polling_system,
    handshake_sending_system,
};

const RESEND_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Component)]
pub(crate) struct Handshaking {
    state: HandshakeState,
    started: Instant,
    last_sent: Option<Instant>,
    scflag: bool,
    direction: Direction,
    reliability: ReliabilityState,
}

impl Handshaking {
    fn change_state(
        &mut self,
        state: HandshakeState,
    ) {
        self.state = state;
        self.scflag = true;
    }

    fn record_send(&mut self) {
        self.last_sent = Some(Instant::now());
        self.scflag = false;
    }

    fn terminate(
        &mut self,
        code: HandshakeResponseCode,
        reason: Option<Bytes>
    ) {
        self.change_state(HandshakeState::Terminated(Termination { code, reason }));
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
                scflag: false,
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