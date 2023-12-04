use std::{net::SocketAddr, time::{Instant, Duration}};
use bevy::prelude::*;
use crate::{ordering::OrderingData, reliability::ReliabilityData};

/// A peer connected over the UDP transport layer.
#[derive(Component)]
pub struct UdpConnection {
    /// How long before the connection is timed out for inactivity.
    pub timeout: Duration,

    pub(crate) address: SocketAddr,
    pub(crate) last_sent: Option<Instant>,
    pub(crate) last_recv: Option<Instant>,

    pub(crate) reliability: ReliabilityData,
    pub(crate) ordering: OrderingData,
}

impl UdpConnection {
    /// Returns the address of the peer.
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

impl std::fmt::Debug for UdpConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UdpConnection")
        .field("timeout", &self.timeout)
        .field("address", &self.address)
        .field("last_sent", &self.last_sent)
        .field("last_recv", &self.last_recv)
        .finish()
    }
}