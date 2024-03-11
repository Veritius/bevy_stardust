use std::net::SocketAddr;
use bevy_ecs::prelude::*;

/// A UDP connection.
#[derive(Component)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct Connection {
    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) remote_address: SocketAddr,

    pub(crate) connection_dir: ConnectionDirection,
    pub(crate) connection_state: ConnectionState,
}

impl Connection {
    /// Returns the remote address of the connection.
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_address.clone()
    }

    /// Returns the direction of the connection.
    /// See the [`ConnectionDirection`] docs for more information.
    pub fn direction(&self) -> ConnectionDirection {
        self.connection_dir.clone()
    }

    /// Returns the state of the connection.
    /// See the [`ConnectionState`] docs for more information.
    pub fn state(&self) -> ConnectionState {
        self.connection_state.clone()
    }
}

/// The direction of the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub enum ConnectionDirection {
    /// Outgoing connection. We are acting as a client.
    #[doc(alias = "Client")]
    Outgoing,
    /// Incoming connection. We are acting as a server.
    #[doc(alias = "Server")]
    Incoming,
}

/// The state of the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub enum ConnectionState {
    /// The connection attempt has not yet received a response.
    /// This variant only occurs if the direction is [`Outgoing`](ConnectionDirection::Outgoing).
    Pending,
    /// The connection has received a response and is mid-handshake.
    Handshaking,
    /// The connection is fully active and ready to communicate.
    Connected,
    /// The connection is closing and waiting for final data transfer to occur.
    Closing,
    /// The connection is closed and the entity will be despawned soon.
    Closed,
}