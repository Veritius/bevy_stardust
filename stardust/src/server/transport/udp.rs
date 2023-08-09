//! Native UDP transport layer for servers.

mod receiver;
mod sender;

use std::net::{UdpSocket, SocketAddr};
use bevy::prelude::*;
use crate::shared::{scheduling::{TransportReadPackets, TransportSendPackets}, hashdiff::UniqueNetworkHash};
use self::{receiver::{receive_packets_system, UdpListener}, sender::send_packets_system};

/// A simple transport layer over native UDP sockets, using TCP for a handshake.
pub struct ServerUdpTransportPlugin {
    pub port: u16,
}

impl Plugin for ServerUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UdpListener::new(self.port));
        
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }

    fn finish(&self, app: &mut App) {
        let hash = app.world
            .get_resource::<UniqueNetworkHash>()
            .expect("Couldn't access UniqueNetworkHash resource, was this plugin added before StardustSharedPlugin?");
    }
}

/// A client connected with the `ServerUdpTransportPlugin` transport layer.
/// 
/// Removing this will silently disconnect the peer with no warning.
#[derive(Component)]
pub struct UdpClient {
    address: SocketAddr,
    socket: UdpSocket,
}

/// Maximum packet length that can be sent/received before fragmentation.
const MAX_PACKET_LENGTH: usize = 1500;
/// The amount of bytes that will always be present in all packages.
const PACKET_HEADER_SIZE: usize = 3;