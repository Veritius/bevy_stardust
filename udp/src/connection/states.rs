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