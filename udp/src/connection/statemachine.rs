use crate::ConnectionState;
use super::handshake::ConnectionHandshake;

#[derive(Debug)]
pub(super) enum ConnectionStateMachine {
    Handshaking(ConnectionHandshake),
    Established,
    Closing,
    Closed,
}

impl ConnectionStateMachine {
    pub fn new_incoming() -> Self {
        todo!()
    }

    pub fn new_outgoing() -> Self {
        todo!()
    }

    pub fn as_simple_repr(&self) -> ConnectionState {
        match self {
            Self::Handshaking(_) => ConnectionState::Handshaking,
            Self::Established => ConnectionState::Connected,
            Self::Closing => ConnectionState::Closing,
            Self::Closed => ConnectionState::Closed,
        }
    }
}

pub(super) enum PotentialStateTransition<R, T> {
    Nothing(R),
    Transition(T),
    Failure,
}