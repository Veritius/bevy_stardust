use std::net::SocketAddr;
use bevy::{prelude::*, ecs::system::SystemParam};

/// Allows modifying the UDP connection.
#[derive(SystemParam)]
pub struct UdpServerManager<'w, 's> {
    commands: Commands<'w, 's>,
}

impl UdpServerManager<'_, '_> {
    pub fn join(&mut self, address: SocketAddr) {
        self.commands.insert_resource(ConnectToRemoteUdp(address));
    }
}