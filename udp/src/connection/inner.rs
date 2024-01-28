use std::net::SocketAddr;

/// An existing connection.
#[derive(Debug)]
pub(crate) struct Connection {
    pub address: SocketAddr,
}