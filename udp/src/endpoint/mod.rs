mod receiving;
mod sending;
mod systems;

pub(crate) mod statistics;

use std::{collections::HashMap, net::{SocketAddr, UdpSocket}};
use anyhow::Result;
use bevy::prelude::*;
use bytes::Bytes;
use tracing::warn;
use statistics::EndpointStatistics;
use crate::schedule::*;

pub(crate) fn add_system(app: &mut App) {
    app.add_systems(PreUpdate, receiving::io_receiving_system
        .in_set(PreUpdateSet::PacketRead));

    app.add_systems(PostUpdate, sending::io_sending_system
        .in_set(PostUpdateSet::PacketSend));

    app.add_systems(PostUpdate, systems::close_endpoints_system
        .in_set(PostUpdateSet::CloseEndpoints));
}

/// An endpoint, which is used for I/O.
/// 
/// Removing this component will not inform clients, and they will eventually time out.
/// Any information from the client that hasn't been received will never be received.
/// Instead of removing this component, consider using the [`close`](Self::close) method.
#[derive(Component)]
pub struct Endpoint {
    udp_socket: UdpSocket,
    connections: HashMap<SocketAddr, ConnectionOwnershipToken>,

    state: EndpointState,
    has_ever_had_peer: bool,

    // Outgoing packets that aren't attached to a peer.
    pub(crate) outgoing_pkts: Vec<(SocketAddr, Bytes)>,
    pub(crate) statistics: EndpointStatistics,

    /// Whether or not to accept new incoming connections on this endpoint.
    pub listening: bool,

    /// Close the endpoint when it has no active connections.
    /// This only occurs if the endpoint has a connection in the past.
    pub close_on_empty: bool,
}

/// Functions for controlling the connection.
impl Endpoint {
    pub(crate) fn bind(address: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(address)?;
        socket.set_nonblocking(true)?;

        Ok(Endpoint {
            udp_socket: socket,
            connections: HashMap::with_capacity(8),
            outgoing_pkts: Vec::default(),
            statistics: EndpointStatistics::default(),
            state: EndpointState::Active,
            has_ever_had_peer: false,
            listening: false,
            close_on_empty: false,
        })
    }

    pub(crate) fn add_peer(
        &mut self,
        address: SocketAddr,
        token: ConnectionOwnershipToken
    ) {
        self.connections.insert(address, token);
        self.has_ever_had_peer = true;
    }

    pub(crate) fn remove_peer(
        &mut self,
        peer: Entity,
    ) -> Option<ConnectionOwnershipToken> {
        // Key finding iterator
        let key = self.connections
            .iter()
            .find(|(_,v)| v.inner() == peer)
            .map(|(k,_)| k.clone());

        // Remove item by key
        if let Some(key) = key {
            return self.connections.remove(&key);
        } else {
            return None;
        }
    }

    /// Returns an iterator over all peers currently connected to this endpoint.
    pub fn peers(&self) -> impl Iterator<Item = Entity> + '_ {
        self.connections.values().map(|v| v.inner())
    }

    /// Marks the endpoint for closure.
    /// This will immediately disconnect all peers attached to it with no reason.
    /// If you want to provide a reason, disconnect peers individually.
    pub fn close(&mut self) {
        self.state = EndpointState::Closed;
    }
}

/// Information and statistics about the endpoint.
impl Endpoint {
    /// Returns the local address of the endpoint.
    /// This is the address assigned by the operating system.
    /// It is **not** what other peers use to connect over the Internet.
    pub fn address(&self) -> SocketAddr {
        self.udp_socket.local_addr().unwrap()
    }

    /// Returns an iterator over the entity IDs of all connections attached to this endpoint.
    pub fn connections(&self) -> impl Iterator<Item = Entity> + '_ {
        self.connections.iter().map(|(_,v)| v.inner())
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum EndpointState {
    /// Working as normal.
    Active,
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