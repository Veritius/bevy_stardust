pub mod statistics;

mod closing;
mod established;
mod handshake;
mod ordering;
mod reliability;
mod systems;
mod timing;

pub(crate) use handshake::{handshake_polling_system, potential_new_peers_system, OutgoingHandshake};
pub(crate) use established::{PackingScratchCells, established_packet_reader_system, established_packet_builder_system, established_timeout_system};
pub(crate) use systems::close_connections_system;

use std::net::SocketAddr;
use bevy::prelude::*;
use bytes::Bytes;
use tracing::warn;
use crate::packet::PacketQueue;
use statistics::ConnectionStatistics;
use timing::ConnectionTimings;

/// An existing UDP connection.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false)]
pub struct Connection {
    #[reflect(ignore)]
    remote_address: SocketAddr,
    #[reflect(ignore)]
    state: ConnectionState,

    #[reflect(ignore)]
    pub(crate) packet_queue: PacketQueue,

    pub(crate) owning_endpoint: Entity,
    pub(crate) direction: ConnectionDirection,
    pub(crate) timings: ConnectionTimings,
    pub(crate) statistics: ConnectionStatistics,

    local_closed: bool,
    remote_closed: bool,
}

/// Functions for controlling the connection.
impl Connection {
    fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
        direction: ConnectionDirection,
    ) -> Self {
        Self {
            remote_address,
            state: ConnectionState::Handshaking,

            packet_queue: PacketQueue::new(16, 16),

            owning_endpoint,
            direction,
            statistics: ConnectionStatistics::default(),
            timings: ConnectionTimings::new(None, None, None),

            local_closed: false,
            remote_closed: false,
        }
    }
}

/// Information and statistics about the connection.
impl Connection {
    /// Returns the remote address of the connection.
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_address.clone()
    }

    /// Returns the direction of the connection.
    /// See the [`ConnectionDirection`] docs for more information.
    pub fn direction(&self) -> ConnectionDirection {
        self.direction.clone()
    }

    /// Returns the [`ConnectionState`] of the connection.
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Returns statistics related to the Connection. See [`ConnectionStatistics`] for more.
    pub fn statistics(&self) -> &ConnectionStatistics {
        &self.statistics
    }
}

// Logs a warning when a non-Closed connection is dropped
// This happens with component removals and drops in scope
impl Drop for Connection {
    fn drop(&mut self) {
        if self.state() != ConnectionState::Closed {
            warn!("Connection dropped while in the {:?} state", self.state());
        }
    }
}

/// The direction of the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ConnectionDirection {
    /// Acting as a client, listening to a server.
    Client,

    /// Acting as a server, talking to a client.
    Server,
}

/// The state of the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ConnectionState {
    /// The connection is in the process of being established.
    Handshaking,

    /// The connection is fully active and ready to communicate.
    Connected,

    /// The connection is closing and waiting for final data transfer to occur.
    Closing,

    /// The connection is closed and the entity will be despawned soon.
    Closed,
}

#[derive(Event)]
pub(crate) struct PotentialNewPeer {
    pub endpoint: Entity,
    pub address: SocketAddr,
    pub payload: Bytes,
}