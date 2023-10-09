use std::{net::SocketAddr, time::{Instant, Duration}};
use bevy::prelude::*;

/// A connection attempt to a remote peer.
#[derive(Debug, Component)]
pub(super) struct PendingConnection {
    pub address: SocketAddr,
    pub started: Instant,
    pub timeout: Option<Duration>,
}