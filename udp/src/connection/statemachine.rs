use crate::ConnectionState;
use super::handshake::ConnectionHandshake;

/// Inner state machine controlling data exchange.
pub(crate) struct ConnectionStateMachine(StateMachineInner);

impl ConnectionStateMachine {
    pub fn new_incoming() -> Self {
        todo!()
    }

    pub fn new_outgoing() -> Self {
        todo!()
    }

    pub fn as_simple_repr(&self) -> ConnectionState {
        match self.0 {
            StateMachineInner::Handshaking(_) => ConnectionState::Handshaking,
            StateMachineInner::Established => ConnectionState::Connected,
            StateMachineInner::Closing => ConnectionState::Closing,
            StateMachineInner::Closed => ConnectionState::Closed,
        }
    }
}

#[derive(Debug)]
enum StateMachineInner {
    Handshaking(ConnectionHandshake),
    Established,
    Closing,
    Closed,
}