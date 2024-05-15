pub mod statistics;

mod handshake;
mod machine;
mod ordering;
mod packets;
mod reliability;
mod shared;
mod systems;

pub(crate) use systems::*;

use std::net::SocketAddr;
use bevy::prelude::*;
use bytes::Bytes;
use self::{machine::ConnectionStateMachine, shared::ConnectionShared};

pub const DEFAULT_MTU: usize = 1472;
pub const DEFAULT_BUDGET: usize = 16384;

/// A UDP connection.
#[derive(Component)]
pub struct Connection {
    inner: Box<ConnectionImpl>,
}

impl Connection {
    pub(crate) fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
        direction: ConnectionDirection,
    ) -> Self {
        Self {
            inner: ConnectionImpl::new(
                owning_endpoint,
                remote_address,
                direction
            ),
        }
    }

    #[inline]
    pub(crate) fn inner(&self) -> &ConnectionImpl {
        &self.inner
    }

    #[inline]
    pub(crate) fn inner_mut(&mut self) -> &mut ConnectionImpl {
        &mut self.inner
    }
}

/// Simple information getters for a connection.
impl Connection {
    /// Return the state of the connection.
    #[inline]
    pub fn state(&self) -> ConnectionState {
        self.inner.state()
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
        self.inner.shared.mtu_limit = mtu;
    }

    /// Sets the limit of the number of bytes that will be sent each **second.**
    /// For most cases, the default ([`DEFAULT_BUDGET`]) is good enough.
    /// However, if the connection is running through loopback, it can be
    /// safely set to [`usize::MAX`] for better testing performance.
    /// 
    /// When congestion control is added, this function will be deprecated, and then removed.
    pub fn set_budget(&mut self, budget: usize) {
        self.inner.shared.budget_limit = budget;
    }
}

pub(crate) struct ConnectionImpl {
    pub shared: ConnectionShared,
    machine: ConnectionStateMachine,
}

impl ConnectionImpl {
    fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
        direction: ConnectionDirection,
    ) -> Box<Self> {
        let shared = ConnectionShared::new(
            owning_endpoint,
            remote_address,
            direction,
        );

        Box::new(Self {
            machine: ConnectionStateMachine::new(&shared),
            shared,
        })
    }

    pub fn state(&self) -> ConnectionState {
        todo!()
    }
}

/// The state of the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// The connection is being established.
    Handshaking,

    /// The connection is fully established.
    Established,

    /// The connection is closing.
    Closing,

    /// The connection has closed.
    Closed,
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