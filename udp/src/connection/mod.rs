pub mod statistics;

mod established;
mod handshake;
mod ordering;
mod packets;
mod reliability;
mod systems;
mod timing;

pub(crate) use handshake::{handshake_polling_system, potential_new_peers_system, OutgoingHandshake};
pub(crate) use established::{established_polling_system, established_writing_system, established_timeout_system, established_closing_system};
pub(crate) use systems::close_connections_system;

use std::{collections::{BTreeMap, VecDeque}, net::SocketAddr, time::Instant};
use bevy::prelude::*;
use bytes::Bytes;
use tracing::warn;
use statistics::ConnectionStatistics;
use timing::ConnectionTimings;
use self::{ordering::OrderingManager, packets::{builder::PacketBuilder, reader::PacketReader}, reliability::{ReliabilityState, UnackedPacket}};

pub const DEFAULT_MTU: usize = 1472;
pub const DEFAULT_BUDGET: usize = 16384;

/// An existing UDP connection.
#[derive(Component)]
pub struct Connection {
    mtu_limit: usize,
    budget_limit: usize,
    budget_count: usize,
    budget_ltime: Instant,

    remote_address: SocketAddr,
    connection_state: ConnectionState,
    ice_thickness: u16,

    pub(crate) owning_endpoint: Entity,
    pub(crate) direction: ConnectionDirection,
    pub(crate) timings: ConnectionTimings,
    pub(crate) statistics: ConnectionStatistics,

    pub(crate) send_queue: VecDeque<Bytes>,
    pub(crate) recv_queue: VecDeque<Bytes>,

    orderings: OrderingManager,
    reliability: ReliabilityState,
    unacked_packets: BTreeMap<u16, UnackedPacket>,

    frame_builder: PacketBuilder,
    frame_parser: PacketReader,
}

/// Functions for controlling the connection.
impl Connection {
    fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
        direction: ConnectionDirection,
    ) -> Self {
        Self {
            mtu_limit: DEFAULT_MTU,
            budget_limit: DEFAULT_BUDGET,
            budget_count: DEFAULT_BUDGET,
            budget_ltime: Instant::now(),

            remote_address,
            connection_state: ConnectionState::Handshaking,
            ice_thickness: u16::MAX,

            owning_endpoint,
            direction,
            statistics: ConnectionStatistics::default(),
            timings: ConnectionTimings::new(None, None, None),

            send_queue: VecDeque::with_capacity(16),
            recv_queue: VecDeque::with_capacity(32),

            orderings: OrderingManager::new(),
            reliability: ReliabilityState::new(),
            unacked_packets: BTreeMap::default(),

            frame_builder: PacketBuilder::default(),
            frame_parser: PacketReader::default(),
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
        self.connection_state
    }

    /// Returns statistics related to the Connection. See [`ConnectionStatistics`] for more.
    pub fn statistics(&self) -> &ConnectionStatistics {
        &self.statistics
    }
}

/// Advanced configuration for power users.
impl Connection {
    /// Sets the maximum transport units (packet size limit) of the connection.
    /// Currently, MTU is not detected, and this variable lets you change it manually.
    /// For most cases, the default ([`DEFAULT_MTU`]) is good enough.
    /// 
    /// MTUs that are too high will cause data loss, and MTUs that are too low will be inefficient.
    /// Note that the average MTU a user will have is probably `1472`, the Ethernet link layer limit.
    /// 
    /// When MTU detection is added, this function will be deprecated, and then removed.
    pub fn set_mtu(&mut self, mtu: usize) {
        self.mtu_limit = mtu;
    }

    /// Sets the limit of the number of bytes that will be sent each **second.**
    /// For most cases, the default ([`DEFAULT_BUDGET`]) is good enough.
    /// However, if the connection is running through loopback, it can be
    /// safely set to [`usize::MAX`] for better testing performance.
    /// 
    /// When congestion control is added, this function will be deprecated, and then removed.
    pub fn set_budget(&mut self, budget: usize) {
        self.budget_limit = budget;
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