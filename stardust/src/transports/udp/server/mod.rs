//! Native UDP transport layer for servers.

pub mod policy;

mod listener;
mod receiver;
mod sender;
mod ports;

use std::net::{SocketAddr, IpAddr};
use bevy::prelude::*;
use once_cell::sync::Lazy;
use semver::{Version, VersionReq};
use crate::scheduling::*;
use self::{receiver::receive_packets_system, sender::send_packets_system, listener::{udp_listener_system, UdpListener}, ports::PortBindings};

pub static STARDUST_UDP_CURRENT_VERSION: Version = Version::new(0, 2, 0);
pub static STARDUST_UDP_VERSION_RANGE: Lazy<VersionReq> = Lazy::new(|| { "=0.2.0".parse::<VersionReq>().unwrap() });

/// A simple transport layer over native UDP sockets.
pub struct ServerUdpTransportPlugin {
    /// The address to use to connect. Use `None` if you want the OS to allocate one for you.
    /// 
    /// *Note: This is the local address within your system, and will not be the IP used by clients to connect over the Internet.*
    pub address: Option<IpAddr>,

    /// The port that will be used by new clients to join the game.
    pub listen_port: u16,

    /// The ports that will be used in the dynamic port allocator system.
    /// 
    /// Higher values improve performance with high player counts, to an extent.
    pub active_ports: Vec<u16>,
}

impl Plugin for ServerUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        // Clone vec for mutability, sort it and remove duplicate values
        let mut active_cloned = self.active_ports.clone();
        active_cloned.sort_unstable();
        active_cloned.dedup();

        // Panic with a more comprehensible message rather than the OS response
        if self.active_ports.contains(&self.listen_port) {
            panic!("Listen port value ({}) is one of the active port values ({:?})",
                self.listen_port, self.active_ports);
        }
        
        // Add resources
        let address = if self.address.is_some() { self.address.unwrap() }
            else { IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED) };
        app.insert_resource(UdpListener::new(address, self.listen_port));
        app.insert_resource(PortBindings::new(address, &self.active_ports));

        // Add systems
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
const PACKET_HEADER_SIZE: usize = 5;