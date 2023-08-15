//! Native UDP transport layer for servers.

pub mod policy;

mod listener;
mod receiver;
mod sender;
mod ports;

use std::{net::{UdpSocket, SocketAddr, IpAddr}, ops::RangeInclusive};
use bevy::prelude::*;
use once_cell::sync::Lazy;
use semver::{Version, VersionReq};
use crate::shared::scheduling::{TransportReadPackets, TransportSendPackets};
use self::{receiver::receive_packets_system, sender::send_packets_system, listener::{udp_listener_system, UdpListener}, ports::PortBindings};

pub static STARDUST_UDP_CURRENT_VERSION: Version = Version::new(0, 0, 0);
pub static STARDUST_UDP_VERSION_RANGE: Lazy<VersionReq> = Lazy::new(|| { "=0.0.0".parse::<VersionReq>().unwrap() });

/// A simple transport layer over native UDP sockets.
pub struct ServerUdpTransportPlugin {
    /// The address to use. You can use `IpAddr::UNSPECIFIED` to have the OS assign one for you.
    pub address: IpAddr,

    /// The port that will be used by new clients to join the game.
    pub listen_port: u16,

    /// The range of ports that will be used for communication with connected clients.
    /// Clients will be automatically allocated to use a port once they join.
    /// 
    /// Larger ranges (may) perform better, but use more ports.
    /// If you have a small amount of players, you can keep this to 1.
    pub active_ports: RangeInclusive<u16>,
}

impl Plugin for ServerUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UdpListener::new(self.address, self.listen_port));
        app.insert_resource(PortBindings::new(self.address, self.active_ports.clone()));

        app.add_systems(TransportReadPackets, udp_listener_system);
        
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}

/// A client connected with the `ServerUdpTransportPlugin` transport layer.
/// 
/// Removing this will silently disconnect the peer with no warning.
#[derive(Component)]
pub struct UdpClient {
    address: SocketAddr,
}

/// Maximum packet length that can be sent/received before fragmentation.
const MAX_PACKET_LENGTH: usize = 1500;
/// The amount of bytes that will always be present in all packages.
const PACKET_HEADER_SIZE: usize = 3;