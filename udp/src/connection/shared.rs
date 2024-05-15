use std::{collections::{BTreeMap, VecDeque}, net::SocketAddr, time::Instant};
use bevy::prelude::*;
use bytes::Bytes;
use crate::sequences::SequenceId;
use super::{ordering::OrderingManager, packets::{builder::PacketBuilder, reader::PacketReader}, reliability::{ReliabilityState, UnackedPacket}, statistics::ConnectionStatistics, ConnectionDirection, DEFAULT_BUDGET, DEFAULT_MTU};

/// Lifecycle-agnostic metadata for connections.
pub(crate) struct ConnectionShared {
    owning_endpoint: Entity,
    remote_address: SocketAddr,
    direction: ConnectionDirection,
    statistics: ConnectionStatistics,

    opened: Instant,
    last_sent: Option<Instant>,
    last_recv: Option<Instant>,

    pub(super) send_queue: VecDeque<Bytes>,
    pub(super) recv_queue: VecDeque<Bytes>,

    pub(super) orderings: OrderingManager,
    pub(super) reliability: ReliabilityState,
    pub(super) unacked_pkts: BTreeMap<SequenceId, UnackedPacket>,

    pub(super) frame_builder: PacketBuilder,
    pub(super) frame_parser: PacketReader,

    pub ice_thickness: u16,

    pub mtu_limit: usize,
    pub budget_limit: usize,
    budget_count: usize,
    budget_ltime: Instant,
}

/// Functions for controlling the connection.
impl ConnectionShared {
    pub(super) fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
        direction: ConnectionDirection,
    ) -> Self {
        Self {
            owning_endpoint,
            remote_address,
            direction,
            statistics: ConnectionStatistics::default(),

            opened: Instant::now(),
            last_recv: None,
            last_sent: None,

            send_queue: VecDeque::with_capacity(16),
            recv_queue: VecDeque::with_capacity(32),

            orderings: OrderingManager::new(),
            reliability: ReliabilityState::new(),
            unacked_pkts: BTreeMap::new(),

            frame_builder: PacketBuilder::default(),
            frame_parser: PacketReader::default(),

            ice_thickness: u16::MAX,

            mtu_limit: DEFAULT_MTU,
            budget_limit: DEFAULT_BUDGET,
            budget_count: DEFAULT_BUDGET,
            budget_ltime: Instant::now(),
        }
    }

    #[inline]
    pub fn owning_endpoint(&self) -> Entity {
        self.owning_endpoint
    }

    #[inline]
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_address
    }

    #[inline]
    pub fn direction(&self) -> ConnectionDirection {
        self.direction
    }

    pub fn pop_send(&mut self) -> Option<Bytes> {
        self.last_sent = Some(Instant::now());
        self.send_queue.pop_front()
    }

    pub fn any_send(&self) -> bool {
        self.send_queue.len() > 0
    }

    pub fn push_recv(&mut self, packet: Bytes) {
        self.last_recv = Some(Instant::now());
        self.recv_queue.push_back(packet)
    }
}

// Logs a warning when a non-Closed connection is dropped
// This happens with component removals and drops in scope
impl Drop for ConnectionShared {
    fn drop(&mut self) {
        if true { // TODO
            // warn!("Connection dropped while in the {:?} state", self.state());
        }
    }
}