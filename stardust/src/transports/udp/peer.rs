use std::net::SocketAddr;
use bevy::prelude::*;

#[derive(Debug, Component)]
pub(super) struct EstablishedUdpPeer {
    pub address: SocketAddr,
    /// The 'oopsies' meter. Increments when they do something weird and disconnects them if it gets too high.
    pub hiccups: u32,
}