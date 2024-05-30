pub mod statistics;

mod closing;
mod congestion;
mod established;
mod handshake;
mod ordering;
mod reliability;
mod timing;

use std::{collections::VecDeque, net::SocketAddr, ops::Neg};
use bevy::prelude::*;
use bytes::Bytes;
use statistics::ConnectionStatistics;
use timing::ConnectionTimings;
use self::congestion::Congestion;

pub(crate) use handshake::OutgoingHandshakeBundle;

pub(crate) fn add_systems(app: &mut App) {

}

/// An existing UDP connection.
#[derive(Component)]
pub struct Connection {
    remote_address: SocketAddr,
    congestion: Congestion,

    pub(crate) send_queue: VecDeque<Bytes>,
    pub(crate) recv_queue: VecDeque<Bytes>,

    pub(crate) owning_endpoint: Entity,
    pub(crate) timings: ConnectionTimings,
    pub(crate) statistics: ConnectionStatistics,
}

/// Functions for controlling the connection.
impl Connection {
    fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
    ) -> Self {
        Self {
            remote_address,
            congestion: Congestion::default(),

            send_queue: VecDeque::with_capacity(16),
            recv_queue: VecDeque::with_capacity(32),

            owning_endpoint,
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

#[derive(Event)]
pub(crate) struct PotentialNewPeer {
    pub endpoint: Entity,
    pub address: SocketAddr,
    pub payload: Bytes,
}