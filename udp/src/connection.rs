use std::net::SocketAddr;
use bevy_ecs::prelude::*;

/// A UDP connection.
#[derive(Component)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct Connection {
    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) remote_address: SocketAddr,

    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) statistics: ConnectionStatistics,

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

    /// Returns statistics related to the Connection. See [`ConnectionStatistics`] for more.
    pub fn statistics(&self) -> &ConnectionStatistics {
        &self.statistics
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

/// Statistics related to an Endpoint.
#[derive(Debug, Clone)]
pub struct ConnectionStatistics {
    /// How many messages this client has sent, in total.
    pub total_messages_sent: u64,

    /// How many messages this client has received, in total.
    pub total_messages_received: u64,

    /// How many messages this client has sent, this tick.
    pub tick_messages_sent: u32,

    /// How many messages this client has sent, this tick.
    pub tick_messages_received: u32,
}