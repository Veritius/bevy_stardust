pub mod statistics;

mod closing;
mod established;
mod events;
mod handshake;
mod ordering;
mod packets;
mod reliability;

pub(crate) use packets::frames::{Frame, FrameHeader};
pub(crate) use packets::{RecvPacket, SendPacket};

use bevy::prelude::*;
use closing::Closing;
use established::Established;
use handshake::Handshake;
use statistics::ConnectionStatistics;
use std::{collections::VecDeque, net::SocketAddr, time::Instant};
use tracing::warn;

/// An existing UDP connection.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false)]
pub struct Connection {
    #[reflect(ignore)]
    remote_address: SocketAddr,

    #[reflect(ignore)]
    stage: ConnectionStage,

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

impl Connection {
    fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
        direction: ConnectionDirection,
    ) -> Self {
        Self {
            remote_address,

            stage: ConnectionStage::Closed,

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

    /// Returns statistics related to the Connection. See [`ConnectionStatistics`] for more.
    pub fn statistics(&self) -> &ConnectionStatistics {
        &self.statistics
    }
}

// Logs a warning when a non-Closed connection is dropped
// This happens with component removals and drops in scope
impl Drop for Connection {
    fn drop(&mut self) {
        match self.stage {
            ConnectionStage::Closed => {},
            _ => { warn!("An open connection was dropped. Connections should be closed before they're removed.") },
        }
    }
}

/// The direction of the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ConnectionDirection {
    /// The peer that sent the connection attempt,
    Initiator,

    /// The peer that received the connection attempt.
    Listener,
}

/// The state of the connection.
enum ConnectionStage {
    Handshaking(Handshake),
    Established(Established),
    Closing(Closing),
    Closed,
}

#[derive(Event)]
pub(crate) struct PotentialNewPeer {
    pub endpoint: Entity,
    pub address: SocketAddr,
    pub packet: RecvPacket,
}