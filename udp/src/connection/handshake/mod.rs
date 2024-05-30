mod codes;
mod parse;
mod system;

mod finished;
mod initiatorhello;
mod listenerhello;
mod terminated;

pub(crate) use system::handshake_polling_system;

use bytes::Bytes;
use bevy_stardust::connections::NetworkPeer;
use unbytes::Reader;
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
    Completed(Completed),
    Terminated(Terminated),

    // Used for various ownership-related stuff
    Swapping,
}

trait Transition: Sized {
    type Next;

    #[must_use]
    fn recv_packet(self, shared: &mut HandshakeShared, reader: &mut Reader) -> TransitionOutcome<Self>;

    #[must_use]
    fn poll_send(&mut self, shared: &mut HandshakeShared) -> Option<Bytes>;
}

enum TransitionOutcome<T: Transition> {
    None(T),
    Next(T::Next),
    Fail(Terminated),
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