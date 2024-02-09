use std::net::SocketAddr;

/// An existing connection.
pub(crate) struct Connection {
    pub address: SocketAddr,
}