use std::{net::SocketAddr, time::{Instant, Duration}};
use bevy::prelude::*;

/// A peer connected over the UDP transport layer.
#[derive(Debug, Component)]
pub struct UdpConnection {
    /// How long before the connection is timed out for inactivity.
    pub timeout: Duration,

    /// Unique identifier for the peer.
    pub(crate) identifier: u32,

    pub(crate) address: SocketAddr,
    pub(crate) last_sent: Option<Instant>,
    pub(crate) last_recv: Option<Instant>,
}

impl UdpConnection {
    /// Returns the address of the peer.
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}