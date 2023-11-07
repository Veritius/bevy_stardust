use std::{net::SocketAddr, time::{Instant, Duration}};
use bevy::prelude::*;
use super::reliability::Reliability;

/// If set to `false`, new incoming connections will be ignored.
#[derive(Debug, Resource)]
pub(super) struct AllowNewConnections(pub bool);

/// A UDP connection. May or may not be fully connected.
#[derive(Debug, Component)]
pub struct UdpConnection {
    pub address: SocketAddr,
    pub started: Instant,
    pub timeout: Duration,
    pub reliability: Reliability,
    pub status: ConnectionStatus,
}

impl UdpConnection {
    /// Create an outgoing connection attempt to a new peer.
    pub fn new_outgoing(address: SocketAddr, timeout: Duration) -> Self {
        Self {
            address,
            started: Instant::now(),
            timeout,
            reliability: Reliability::default(),
            status: ConnectionStatus::PendingOutgoing(PendingOutgoing::WaitingForResponse),
        }
    }

    pub fn new_incoming(address: SocketAddr, timeout: Duration) -> Self {
        Self {
            address,
            started: Instant::now(),
            timeout,
            reliability: Reliability::default(),
            status: ConnectionStatus::PendingIncoming(PendingIncoming::JustRegistered),
        }
    }
}

#[derive(Debug)]
pub enum ConnectionStatus {
    /// A connection attempt from a hitherto-unknown remote peer.
    PendingIncoming(PendingIncoming),
    /// A connection attempt to a known remote peer, emanating from this peer.
    PendingOutgoing(PendingOutgoing),
    /// A fully established connection.
    Established,
    /// A previously established connection that is closed.
    Disconnected,
}

#[derive(Debug)]
pub enum PendingIncoming {
    JustRegistered,
    Accepted,
}

#[derive(Debug)]
pub enum PendingOutgoing {
    WaitingForResponse,
    Accepted,
}