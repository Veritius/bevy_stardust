use anyhow::Result;
use std::net::{SocketAddr, IpAddr};
use bevy::ecs::system::SystemParam;

/// Manages the UDP connection system.
#[derive(SystemParam)]
pub struct UdpConnectionManager;

impl UdpConnectionManager {
    /// Begins hosting a server.
    /// 
    /// - If `address` is `None` the IP will be chosen by the operating system.
    /// - If using `ProcessingMode::Single`, `active` can be empty as `listen` will be used for communication.
    /// - If using `ProcessingMode::Taskpool`, `active` must have at least `1` element and cannot include `listen`.
    pub fn start_server(&mut self, address: Option<IpAddr>, listen: u16, active: Vec<u16>) -> Result<IpAddr> {
        todo!()
    }

    /// Returns true if in server mode.
    pub fn is_server(&self) -> bool {
        todo!()
    }

    /// Set whether a server will allow new connections.
    /// Only works when a server is running.
    pub fn allow_clients(&mut self, yes: bool) {
        todo!()
    }

    /// Start a client.
    pub fn start_client(&mut self, address: Option<IpAddr>, active: u16) -> Result<IpAddr> {
        todo!()
    }

    /// Returns true if in client mode.
    pub fn is_client(&self) -> bool {
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

    /// Returns true if a fully initialised client or server is open.
    pub fn is_active(&self) -> bool {
        todo!()
    }
}