use std::net::{SocketAddr, UdpSocket};
use anyhow::Result;
use bevy_ecs::prelude::*;
use bytes::Bytes;
use smallvec::SmallVec;
use tracing::warn;

/// An endpoint, which is used for I/O.
/// 
/// Removing this component will not inform clients, and they will eventually time out.
/// Any information from the client that hasn't been received will never be received.
/// Instead of removing this component, consider using the [`close`](Self::close) method.
#[derive(Component)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct Endpoint {
    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) socket: UdpSocket,

    #[cfg_attr(feature="reflect", reflect(ignore))]
    pub(crate) connections: SmallVec::<[ConnectionOwnershipToken; 8]>,

    pub(crate) statistics: EndpointStatistics,
    pub(crate) state: EndpointState,

    /// Whether or not to accept new incoming connections on this endpoint.
    pub listening: bool,
}

/// Functions for controlling the connection.
impl Endpoint {
    pub(crate) fn bind(address: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(address)?;
        socket.set_nonblocking(true)?;

        Ok(Endpoint {
            socket,
            connections: SmallVec::new(),
            statistics: EndpointStatistics::default(),
            state: EndpointState::Active,
            listening: false,
        })
    }

    /// Marks the endpoint for closure.
    /// This will inform all clients of the disconnection along with the `reason` if present,
    /// and waits for data exchange to stop. This is the best solution for most use cases.
    /// 
    /// If `hard` is set to `true`, the endpoint will be closed as soon as possible.
    /// A message will be sent to inform clients but nothing will be done to ensure its arrival.
    /// Messages from the client that haven't been received will never be received.
    pub fn close(&mut self, hard: bool, reason: Option<Bytes>) {
        todo!()
    }
}

/// Information and statistics about the endpoint.
impl Endpoint {
    /// Returns the local address of the endpoint.
    /// This is the address assigned by the operating system.
    /// It is **not** what other peers use to connect over the Internet.
    pub fn address(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }

    /// Returns an iterator over the entity IDs of all connections attached to this endpoint.
    pub fn connections(&self) -> impl Iterator<Item = Entity> + '_ {
        self.connections.iter().map(|f| f.inner())
    }

    /// Returns statistics related to the Endpoint. See [`EndpointStatistics`] for more.
    pub fn statistics(&self) -> &EndpointStatistics {
        &self.statistics
    }

    /// Returns the current state of the endpoint.
    pub fn state(&self) -> &EndpointState {
        &self.state
    }
}

// Logs a warning when a non-Closed endpoint is dropped
// This happens with component removals and drops in scope
impl Drop for Endpoint {
    fn drop(&mut self) {
        if self.state != EndpointState::Closed {
            warn!("Endpoint dropped while in the {:?} state", self.state);
        }
    }
}

/// The state of the endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub enum EndpointState {
    /// Working as normal.
    Active,
    /// The endpoint is closing, and all connections are performing the disconnect handshake.
    Closing,
    /// The endpoint is closed and will be despawned soon.
    Closed,
}

/// A wrapper around an entity ID that guarantees that a Connection is only 'owned' by one [`Endpoint`] at a time.
/// 
/// This is done by making it that only one ConnectionOwnershipToken exists for a given entity ID in the same World.
/// Because of this, all constructor functions (currently only `new`) are marked as unsafe.
/// 
/// If a token ever ends up attached to more than one [`Endpoint`] at a time, it will lead to undefined behavior.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct ConnectionOwnershipToken(Entity);

impl ConnectionOwnershipToken {
    /// Creates a new `ConnectionOwnershipToken`.
    pub unsafe fn new(entity: Entity) -> Self {
        Self(entity)
    }

    /// Returns the inner [`Entity`] id.
    pub fn inner(&self) -> Entity {
        self.0
    }
}

impl std::ops::Deref for ConnectionOwnershipToken {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Statistics related to an [`Endpoint`].
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct EndpointStatistics {
    /// How many packets have been sent, in total.
    pub total_packets_sent: u64,

    /// How many packets have been received, in total.
    pub total_packets_received: u64,

    /// How many packets have been detected to be dropped, in total.
    pub total_packets_dropped: u64,

    /// How many bytes have been sent, in total.
    pub total_bytes_sent: u64,

    /// How many bytes have been received, in total.
    pub total_bytes_received: u64,

    /// How many packets have been sent, this tick.
    pub tick_packets_sent: u32,

    /// How many packets have been received, this tick.
    pub tick_packets_received: u32,

    /// How many bytes have been sent, this tick.
    pub tick_bytes_sent: u32,

    /// How many bytes have been received, this tick.
    pub tick_bytes_received: u32,
}

impl EndpointStatistics {
    pub(crate) fn track_send_packet(&mut self, bytes: usize) {
        self.total_packets_sent += 1;
        self.total_bytes_sent += bytes as u64;
        self.tick_packets_sent += 1;
        self.tick_bytes_sent += bytes as u32;
    }

    pub(crate) fn track_recv_packet(&mut self, bytes: usize) {
        self.total_packets_received += 1;
        self.total_bytes_received += bytes as u64;
        self.tick_packets_received += 1;
        self.tick_bytes_received += bytes as u32;
    }
}