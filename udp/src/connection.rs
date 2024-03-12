use std::{collections::VecDeque, net::SocketAddr, time::Instant};
use bevy_ecs::prelude::*;
use bytes::Bytes;
use tracing::warn;

use crate::packet::{IncomingPacket, OutgoingPacket};

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
    pub(crate) owning_endpoint: Entity,

    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) remote_address: SocketAddr,

    pub(crate) statistics: ConnectionStatistics,
    pub(crate) connection_dir: ConnectionDirection,
    pub(crate) connection_state: ConnectionState,

    pub(crate) last_recv: Option<Instant>,
    pub(crate) last_send: Option<Instant>,

    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) outgoing_packets: VecDeque<OutgoingPacket>,

    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) incoming_packets: VecDeque<IncomingPacket>,
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
            connection_dir: direction,
            connection_state: match direction {
                ConnectionDirection::Outgoing => ConnectionState::Pending,
                ConnectionDirection::Incoming => ConnectionState::Handshaking,
            },
            last_recv: match direction {
                ConnectionDirection::Outgoing => None,
                ConnectionDirection::Incoming => Some(Instant::now()),
            },
            last_send: match direction {
                ConnectionDirection::Outgoing => Some(Instant::now()),
                ConnectionDirection::Incoming => None,
            },
            outgoing_packets: VecDeque::default(),
            incoming_packets: VecDeque::default()
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

// Logs a warning when a non-Closed connection is dropped
// This happens with component removals and drops in scope
impl Drop for Connection {
    fn drop(&mut self) {
        if self.connection_state != ConnectionState::Closed {
            warn!("Connection dropped while in the {:?} state", self.connection_state);
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

/// Statistics related to a [`Connection`].
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct ConnectionStatistics {
    /// How many packets this client has sent, in total.
    pub total_packets_sent: u64,

    /// How many packets this client has received, in total.
    pub total_packets_received: u64,

    /// How many packets this client has dropped, in total.
    pub total_packets_dropped: u64,

    /// How many messages this client has sent, in total.
    pub total_messages_sent: u64,

    /// How many messages this client has received, in total.
    pub total_messages_received: u64,

    /// How many packets this client has sent, this tick.
    pub tick_packets_sent: u32,

    /// How many packets this client has sent, this tick.
    pub tick_packets_received: u32,

    /// How many messages this client has sent, this tick.
    pub tick_messages_sent: u32,

    /// How many messages this client has sent, this tick.
    pub tick_messages_received: u32,
}

impl ConnectionStatistics {
    pub(crate) fn track_send_packet(&mut self, messages: usize) {
        self.total_packets_sent += 1;
        self.total_messages_sent += messages as u64;
        self.tick_packets_sent += 1;
        self.tick_messages_sent += messages as u32;
    }

    pub(crate) fn track_recv_packet(&mut self, messages: usize) {
        self.total_packets_received += 1;
        self.total_messages_received += messages as u64;
        self.tick_packets_received += 1;
        self.tick_messages_received += messages as u32;
    }
}

pub(crate) fn reset_connection_statistics_system(
    mut connections: Query<&mut Connection>,
) {
    for mut connection in connections.iter_mut() {
        let statistics = &mut connection.statistics;
        statistics.tick_packets_sent = 0;
        statistics.tick_packets_received = 0;
        statistics.tick_messages_sent = 0;
        statistics.tick_messages_received;
    }
}