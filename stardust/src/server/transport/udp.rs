//! Native UDP transport layer for servers.

pub mod policy;

mod listener;
mod receiver;
mod sender;
mod ports;
mod acks;

use std::{net::{SocketAddr, IpAddr}, ops::RangeInclusive};
use bevy::prelude::*;
use once_cell::sync::Lazy;
use semver::{Version, VersionReq};
use crate::shared::scheduling::{TransportReadPackets, TransportSendPackets};
use self::{receiver::receive_packets_system, sender::send_packets_system, listener::{udp_listener_system, UdpListener}, ports::PortBindings};

pub static STARDUST_UDP_CURRENT_VERSION: Version = Version::new(0, 2, 0);
pub static STARDUST_UDP_VERSION_RANGE: Lazy<VersionReq> = Lazy::new(|| { "=0.2.0".parse::<VersionReq>().unwrap() });

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
        // Panic with a more comprehensible message rather than the OS response
        if self.active_ports.contains(&self.listen_port) {
            panic!("Listen port value ({}) is one of the active port values ({} to {} inclusive)",
                self.listen_port, self.active_ports.start(), self.active_ports.end());
        }

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
#[derive(Debug, Component)]
pub struct UdpClient {
    address: SocketAddr,
    hiccups: u16,
}

/// The amount of bytes that will always be present in all packages.
const PACKET_HEADER_SIZE: usize = 3;