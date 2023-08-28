
use std::net::IpAddr;
use bevy::prelude::*;

pub struct UdpTransportPlugin;

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

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