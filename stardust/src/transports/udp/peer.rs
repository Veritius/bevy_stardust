use std::net::SocketAddr;
use bevy::prelude::*;

#[derive(Debug, Component)]
pub(super) struct UdpPeer {
    pub address: SocketAddr,
}