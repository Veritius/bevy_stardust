pub mod statistics;

mod statemachine;
mod handshake;
mod timing;
mod receiving;
mod reliability;
mod ordering;

pub(crate) use receiving::connection_packet_processing_system;

use std::net::SocketAddr;
use bevy_ecs::prelude::*;
use bytes::Bytes;
use tracing::warn;
use crate::packet::PacketQueue;
use statemachine::ConnectionStateMachine;
use statistics::ConnectionStatistics;
use timing::ConnectionTimings;

/// A running UDP connection.
/// 
/// This component exists throughout the entire lifecycle of the connection.
/// However, the `NetworkPeer` component will only be present in the `Established` state.
/// This behavior may change in future.
/// 
/// To close a connection, you should use the `close` method.
/// If you drop the connection without it fully closing, a warning will be logged.
#[derive(Component)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct Connection {
    #[cfg_attr(feature="reflect", reflect(ignore))]
    remote_address: SocketAddr,
    #[cfg_attr(feature="reflect", reflect(ignore))]
    state_machine: ConnectionStateMachine,

    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) packet_queue: PacketQueue,

    pub(crate) owning_endpoint: Entity,
    pub(crate) direction: ConnectionDirection,
    pub(crate) timings: ConnectionTimings,
    pub(crate) statistics: ConnectionStatistics,
}

/// Functions for controlling the connection.
impl Connection {
    pub(crate) fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
        direction: ConnectionDirection,
    ) -> Self {
        Self {
            owning_endpoint,
            remote_address,
            statistics: ConnectionStatistics::default(),
            direction,
            state_machine: match direction {
                ConnectionDirection::Outgoing => ConnectionStateMachine::new_outgoing(),
                ConnectionDirection::Incoming => ConnectionStateMachine::new_incoming(),
            },
            timings: ConnectionTimings::new(None, None, None),
            packet_queue: PacketQueue::new(16, 16),
        }
    }

    /// Queues the connection for closing, informing the peer of why.
    /// 
    /// If `hard` is set to `true`, the connection will be closed immediately.
    /// A packet will be sent to inform the end user of the closure, but it won't be reliable.
    /// This should generally be avoided, as all data that is yet to be received will be lost.
    pub fn close(&mut self, _hard: bool, _reason: Bytes) {
        todo!()
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
        self.state_machine
            .as_simple_repr()
            .recv_hack(self.timings.last_recv.is_none())
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

impl ConnectionState {
    fn recv_hack(self, has_recv: bool) -> Self {
        match self {
            ConnectionState::Handshaking => match has_recv {
                true => ConnectionState::Handshaking,
                false => ConnectionState::Pending,
            },
            _ => self,
        }
    }
}