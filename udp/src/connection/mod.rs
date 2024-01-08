//! UDP connections.

pub(crate) mod inner;

pub use inner::ConnectionDirection;

use std::net::SocketAddr;

/// An active UDP connection.
pub struct UdpConnection {
    connection: inner::Connection,
}

impl UdpConnection {
    /// Returns the address of the connection.
    pub fn address(&self) -> &SocketAddr {
        &self.connection.address
    }

    /// Returns the direction of the connection.
    pub fn direction(&self) -> ConnectionDirection {
        self.connection.direction
    }
}