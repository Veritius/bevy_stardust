use anyhow::Result;
use std::net::{SocketAddr, IpAddr};
use bevy::ecs::system::SystemParam;

/// Manages the UDP connection system.
#[derive(SystemParam)]
pub struct UdpConnectionManager;

impl UdpConnectionManager {
    /// Begins hosting a server.
    pub fn start_server(&mut self, address: Option<IpAddr>, listen: u16, active: Vec<u16>) -> Result<IpAddr> {
        todo!()
    }

    /// Set whether a server will allow new connections.
    /// Only works when a server is running.
    pub fn allow_clients(&mut self, yes: bool) {
        todo!()
    }

    /// Start a client.
    pub fn start_client(&mut self, address: Option<IpAddr>, active: Vec<u16>) -> Result<IpAddr> {
        todo!()
    }

    /// Join a server, as a client.
    pub fn join_server(&mut self, remote: SocketAddr) {
        todo!()
    }

    /// Leave a server, as a client.
    pub fn leave_server(&mut self) {
        todo!()
    }

    /// Stop the server or client.
    pub fn stop_multiplayer(&mut self) {
        todo!()
    }
}