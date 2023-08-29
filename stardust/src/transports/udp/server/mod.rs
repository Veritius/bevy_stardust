//! Native UDP transport layer for servers.

pub mod policy;

mod listener;
mod receiver;
mod sender;
mod ports;
mod manager;

use std::net::{SocketAddr, IpAddr};
use bevy::prelude::*;
use once_cell::sync::Lazy;
use semver::{Version, VersionReq};
use crate::scheduling::*;
use self::{receiver::receive_packets_system, sender::send_packets_system, listener::{udp_listener_system, UdpListener}, ports::PortBindings};

pub static STARDUST_UDP_CURRENT_VERSION: Version = Version::new(0, 2, 0);
pub static STARDUST_UDP_VERSION_RANGE: Lazy<VersionReq> = Lazy::new(|| { "=0.2.0".parse::<VersionReq>().unwrap() });

/// Config for the UDP server. This resource only needs to be present if in the `NetworkMode::Server` state.
#[derive(Debug, Resource)]
pub struct UdpServerConfig {
    /// The address to use to connect. Use `None` if you want the OS to allocate one for you.
    /// 
    /// Note: This is the local address within your system, and will not be the IP used by clients to connect over the Internet.
    /// That value is usually assigned by your ISP, and you can quickly see it by viewing this website: https://icanhazip.com/
    pub address: Option<IpAddr>,

    /// The port that will be used by new clients to join the game.
    pub listen_port: u16,

    /// The ports that will be used in the dynamic port allocator system.
    /// 
    /// Higher values improve performance with high player counts, to an extent.
    pub active_ports: Vec<u16>,
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