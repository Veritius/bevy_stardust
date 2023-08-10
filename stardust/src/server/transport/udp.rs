//! Native UDP transport layer for servers.

pub mod policy;

mod listener;
mod receiver;
mod sender;

use std::{net::{UdpSocket, SocketAddr}, sync::OnceLock};
use bevy::prelude::*;
use semver::{Version, VersionReq};
use crate::shared::{scheduling::{TransportReadPackets, TransportSendPackets}, hashdiff::UniqueNetworkHash};
use self::{receiver::receive_packets_system, sender::send_packets_system, listener::{udp_listener_system, UdpListener}};

pub static STARDUST_UDP_CURRENT_VERSION: Version = Version::new(0, 0, 0);
pub static STARDUST_UDP_VERSION_RANGE: OnceLock<VersionReq> = OnceLock::new();
pub static STARDUST_UDP_VERSION_RANGE_STRING: &'static str = "=0.0.0";

/// TODO: This is ugly, remove this.
pub fn get_udp_transport_layer_version_range() -> &'static VersionReq {
    STARDUST_UDP_VERSION_RANGE.get_or_init(|| { STARDUST_UDP_VERSION_RANGE_STRING.parse::<VersionReq>().unwrap() })
}

/// A simple transport layer over native UDP sockets, using TCP for a handshake.
pub struct ServerUdpTransportPlugin {
    pub listen_port: u16,
    pub active_port: u16,
}

impl Plugin for ServerUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UdpListener::new(self.listen_port));
        app.add_systems(TransportReadPackets, udp_listener_system);
        
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