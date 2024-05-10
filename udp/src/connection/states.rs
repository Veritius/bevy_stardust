/// The state of the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Handshaking,
    Established,
    Closing,
    Closed,
}

pub(super) enum ConnectionStateInner {
    Handshaking,
    Established,
    Closing,
    Closed,
}

impl ConnectionStateInner {
    pub fn simplify(&self) -> ConnectionState {
        match self {
            ConnectionStateInner::Handshaking => ConnectionState::Handshaking,
            ConnectionStateInner::Established => ConnectionState::Established,
            ConnectionStateInner::Closing => ConnectionState::Closing,
            ConnectionStateInner::Closed => ConnectionState::Closed,
        }
    }
}