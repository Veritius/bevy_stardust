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