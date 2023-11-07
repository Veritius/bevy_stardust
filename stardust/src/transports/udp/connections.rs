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
    pub last_sent: Option<Instant>,
    pub last_received: Option<Instant>,
    pub timeout: Duration,
    pub reliability: Reliability,
    pub status: ConnectionStatus,
}

impl UdpConnection {
    pub fn new_incoming(address: SocketAddr, timeout: Duration) -> Self {
        Self {
            address,
            started: Instant::now(),
            last_sent: None,
            last_received: None,
            timeout,
            reliability: Reliability::default(),
            status: ConnectionStatus::PendingIncoming(PendingIncoming::default()),
        }
    }

    /// Create an outgoing connection attempt to a new peer.
    pub fn new_outgoing(address: SocketAddr, timeout: Duration) -> Self {
        Self {
            address,
            started: Instant::now(),
            last_sent: None,
            last_received: None,
            timeout,
            reliability: Reliability::default(),
            status: ConnectionStatus::PendingOutgoing(PendingOutgoing::default()),
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
    Established(Established),
    /// A previously established connection that is closed.
    Disconnected,
}

#[derive(Debug)]
pub struct PendingIncoming {
    pub state: PendingIncomingState,
}

impl Default for PendingIncoming {
    fn default() -> Self {
        Self {
            state: PendingIncomingState::JustRegistered,
        }
    }
}

#[derive(Debug)]
pub enum PendingIncomingState {
    JustRegistered,
    Accepted,
}

#[derive(Debug)]
pub struct PendingOutgoing {
    pub state: PendingOutgoingState,
}

impl Default for PendingOutgoing {
    fn default() -> Self {
        Self {
            state: PendingOutgoingState::WaitingForResponse,
        }
    }
}

#[derive(Debug)]
pub enum PendingOutgoingState {
    WaitingForResponse,
    Accepted,
}

#[derive(Debug)]
pub struct Established;