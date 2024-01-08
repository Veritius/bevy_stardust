use std::net::SocketAddr;

/// An existing connection.
#[derive(Debug)]
pub(crate) struct Connection {
    pub address: SocketAddr,
    pub direction: ConnectionDirection,
}

/// The direction of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionDirection {
    /// A connection established from us to a remote peer.
    Outgoing,
    /// A connection established from a remote peer to us.
    Incoming,
}