use super::handshake::HandshakeStateMachine;

/// The state of the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// The connection is being established.
    Handshaking,

    /// The connection is fully established.
    Established,

    /// The connection is closing.
    Closing,

    /// The connection has closed.
    Closed,
}

pub(super) enum ConnectionStateInner {
    Handshaking { machine: HandshakeStateMachine },
    Established,
    Closing,
    Closed,
}

impl ConnectionStateInner {
    pub fn simplify(&self) -> ConnectionState {
        match self {
            ConnectionStateInner::Handshaking { machine: _ }=> ConnectionState::Handshaking,
            ConnectionStateInner::Established => ConnectionState::Established,
            ConnectionStateInner::Closing => ConnectionState::Closing,
            ConnectionStateInner::Closed => ConnectionState::Closed,
        }
    }
}