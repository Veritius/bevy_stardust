use std::{net::SocketAddr, time::{Instant, Duration}};
use bevy::prelude::*;

#[derive(Debug, Resource)]
pub(super) struct AllowNewConnections(pub bool);

/// A UDP peer that is fully connected.
#[derive(Debug, Component)]
pub(super) struct EstablishedUdpPeer {
    pub address: SocketAddr,
    /// The 'oopsies' meter. Increments when they do something weird and disconnects them if it gets too high.
    pub hiccups: u32,
}

/// A connection attempt to a remote peer.
#[derive(Debug, Component)]
pub(super) struct PendingUdpPeer {
    pub address: SocketAddr,
    pub started: Instant,
    pub timeout: Duration,
}