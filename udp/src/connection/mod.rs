pub mod statistics;

mod closing;
mod congestion;
mod established;
mod handshake;
mod ordering;
mod packets;
mod reliability;
mod timing;

pub(crate) use closing::closing_component_system;
pub(crate) use established::{established_polling_system, established_writing_system};
pub(crate) use handshake::{handshake_polling_system, potential_new_peers_system, OutgoingHandshake};

use std::{collections::VecDeque, net::SocketAddr};
use bevy::prelude::*;
use bytes::Bytes;
use statistics::ConnectionStatistics;
use timing::ConnectionTimings;
use self::congestion::Congestion;

/// An existing UDP connection.
#[derive(Component)]
pub struct Connection {
    remote_address: SocketAddr,
    congestion: Congestion,

    pub(crate) send_queue: VecDeque<Bytes>,
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
            remote_address,
            congestion: Congestion::default(),

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

    /// Returns statistics related to the Connection. See [`ConnectionStatistics`] for more.
    pub fn statistics(&self) -> &ConnectionStatistics {
        &self.statistics
    }
}

/// Advanced configuration for power users.
impl Connection {
    /// Sets the maximum transport units (packet size limit) of the connection.
    /// Currently, MTU is not detected, and this variable lets you change it manually.
    /// For most cases, the default is good enough.
    /// 
    /// MTUs that are too high will cause data loss, and MTUs that are too low will be inefficient.
    /// Note that the average MTU a user will have is probably `1472`, the Ethernet link layer limit.
    /// 
    /// When MTU detection is added, this function will be deprecated, and then removed.
    pub fn set_mtu(&mut self, mtu: usize) {
        self.congestion.set_usr_mtu(mtu);
    }

    /// Sets the limit of the number of bytes that will be sent each **second.**
    /// For most cases, the default is good enough.
    /// However, if the connection is running through loopback, it can be
    /// safely set to [`usize::MAX`] for better testing performance.
    /// 
    /// When congestion control is added, this function will be deprecated, and then removed.
    pub fn set_budget(&mut self, budget: usize) {
        self.congestion.set_usr_budget(budget);
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

#[derive(Event)]
pub(crate) struct PotentialNewPeer {
    pub endpoint: Entity,
    pub address: SocketAddr,
    pub payload: Bytes,
}