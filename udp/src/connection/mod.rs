pub mod statistics;

mod events;
mod handshake;
mod ordering;
mod packets;
mod reliability;
mod systems;

pub(crate) use systems::close_connections_system;

use std::{collections::VecDeque, net::SocketAddr, time::Instant};
use bevy::prelude::*;
use bytes::Bytes;
use tracing::warn;
use statistics::ConnectionStatistics;
use self::packets::{RecvPacket, SendPacket};

/// An existing UDP connection.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false)]
pub struct Connection {
    #[reflect(ignore)]
    remote_address: SocketAddr,

    #[reflect(ignore)]
    pub(crate) recv_packets: VecDeque<RecvPacket>,
    #[reflect(ignore)]
    pub(crate) send_packets: VecDeque<SendPacket>,

    pub(crate) started: Instant,
    pub(crate) last_recv: Option<Instant>,
    pub(crate) last_send: Option<Instant>,

    pub(crate) owning_endpoint: Entity,
    pub(crate) direction: ConnectionDirection,
    pub(crate) statistics: ConnectionStatistics,
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

            recv_packets: VecDeque::with_capacity(128),
            send_packets: VecDeque::with_capacity(16),

            started: Instant::now(),
            last_recv: None,
            last_send: None,

            owning_endpoint,
            direction,
            statistics: ConnectionStatistics::default(),
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
        todo!()
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