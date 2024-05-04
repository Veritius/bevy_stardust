pub mod statistics;

mod closing;
mod established;
mod handshake;
mod ordering;
mod packets;
mod reliability;
mod systems;
mod timing;

pub(crate) use handshake::{handshake_polling_system, potential_new_peers_system, OutgoingHandshake};
pub(crate) use established::{established_packet_reader_system, established_packet_builder_system, established_timeout_system};
pub(crate) use systems::close_connections_system;

use std::{collections::VecDeque, net::SocketAddr};
use bevy::prelude::*;
use bytes::Bytes;
use tracing::warn;
use statistics::ConnectionStatistics;
use timing::ConnectionTimings;

pub const DEFAULT_MTU: usize = 1472;

/// An existing UDP connection.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false)]
pub struct Connection {
    /// The maximum transport units (packet size limit) of the connection.
    /// Currently, MTU is not detected, and this variable lets you change it manually.
    /// For most cases, the default ([`DEFAULT_MTU`]) is good enough.
    /// 
    /// MTUs that are too high will cause data loss, and MTUs that are too low will be inefficient.
    /// 
    /// When MTU detection is added, this variable will be deprecated, and then removed.
    pub mtu: usize,

    #[reflect(ignore)]
    remote_address: SocketAddr,

    #[reflect(ignore)]
    state: ConnectionState,

    #[reflect(ignore)]
    pub(crate) send_queue: VecDeque<Bytes>,

    #[reflect(ignore)]
    pub(crate) recv_queue: VecDeque<Bytes>,

    pub(crate) owning_endpoint: Entity,
    pub(crate) direction: ConnectionDirection,
    pub(crate) timings: ConnectionTimings,
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
            mtu: DEFAULT_MTU,

            remote_address,

            state: ConnectionState::Handshaking,

            send_queue: VecDeque::with_capacity(16),
            recv_queue: VecDeque::with_capacity(32),

            owning_endpoint,
            direction,
            statistics: ConnectionStatistics::default(),
            timings: ConnectionTimings::new(None, None, None),
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