//! UDP connections.

use bevy::prelude::*;
use std::net::SocketAddr;

pub(crate) mod inner;


/// An active UDP connection.
#[derive(Component)]
pub struct UdpConnection {
    connection: inner::Connection,
}

impl UdpConnection {
    /// Returns the address of the connection.
    pub fn address(&self) -> &SocketAddr {
        &self.connection.address
    }
}