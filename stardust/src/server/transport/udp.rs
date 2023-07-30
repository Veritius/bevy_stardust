//! Native UDP transport layer for servers.

mod listener;
mod receiver;
mod sender;

use std::net::UdpSocket;
use bevy::prelude::*;
use crate::shared::scheduling::{TransportReadPackets, TransportSendPackets};
use self::{receiver::receive_packets_system, sender::send_packets_system, listener::TcpListenerServer};

/// A simple transport layer over native UDP sockets.
pub struct ServerUdpTransportPlugin {
    pub udp_port: u16,
    pub tcp_port: u16,
}

impl Plugin for ServerUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TcpListenerServer::new(self.tcp_port));

        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}

/// A client connected with the `ServerUdpTransportPlugin` transport layer.
#[derive(Component)]
pub struct UdpClient(UdpSocket);

/// Maximum packet length that can be sent/received before fragmentation.
const MAX_PACKET_LENGTH: usize = 1500;
/// The amount of bytes that will always be present in all packages.
const PACKET_HEADER_SIZE: usize = 3;