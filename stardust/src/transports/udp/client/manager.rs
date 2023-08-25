use std::net::SocketAddr;

use bevy::{prelude::*, ecs::system::SystemParam};

use super::attempt::ConnectToRemoteUdp;

/// Allows modifying the UDP connection.
#[derive(SystemParam)]
pub struct UdpConnectionManager<'w, 's> {
    commands: Commands<'w, 's>,
}

impl UdpConnectionManager<'_, '_> {
    pub fn join(&mut self, address: SocketAddr) {
        self.commands.insert_resource(ConnectToRemoteUdp(address));
    }
}