mod codes;
mod system;

mod finished;
mod initiatorhello;
mod listenerhello;
mod terminated;

pub(crate) use system::handshake_polling_system;

use bytes::Bytes;
use bevy_stardust::connections::NetworkPeer;
use std::{net::SocketAddr, time::Instant};
use bevy::prelude::*;
use crate::prelude::*;
use self::{finished::Completed, initiatorhello::InitiatorHello, listenerhello::ListenerHello, terminated::Terminated};
use super::reliability::ReliabilityState;

#[derive(Component)]
pub(crate) struct Handshaking {
    started: Instant,
    state: HandshakeState,
    shared: HandshakeShared,
}

struct HandshakeShared {
    reliability: ReliabilityState,
}

enum HandshakeState {
    InitiatorHello(InitiatorHello),
    ListenerHello(ListenerHello),
    Finished(Completed),
    Terminated(Terminated),
}

trait IntermediateState {
    type Next;

    fn recv_packet(&mut self, shared: &mut HandshakeShared, bytes: Bytes) -> bool;
    fn transition(self, shared: &HandshakeShared) -> Result<Self::Next, Terminated>;
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
                ConnectionDirection::Initiator,
            ),
            handshake: Handshaking {
                started: Instant::now(),
                state: HandshakeState::InitiatorHello(InitiatorHello::new()),
                shared: HandshakeShared {
                    reliability: ReliabilityState::new(),
                },
            },
            peercomp: NetworkPeer::new(),
        }
    }
}