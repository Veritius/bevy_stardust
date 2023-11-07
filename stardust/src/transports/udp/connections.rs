use std::{net::SocketAddr, time::{Instant, Duration}};
use bevy::prelude::*;

use super::reliability::Reliability;

#[derive(Debug, Resource)]
pub(super) struct AllowNewConnections(pub bool);

/// A UDP peer that is fully connected.
#[derive(Debug, Component)]
pub(super) struct EstablishedUdpPeer {
    /// The peer's address.
    pub address: SocketAddr,
    /// Reliability information, like packet sequence IDs both from and to this peer.
    pub reliability: Reliability,
}

/// A connection attempt to a remote peer.
#[derive(Debug, Component)]
pub(super) struct PendingUdpPeer {
    pub address: SocketAddr,
    pub started: Instant,
    pub timeout: Duration,
}