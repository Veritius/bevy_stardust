use std::net::SocketAddr;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::transports::udp::UdpTransportMode;

/// Interface for using the UDP transport layer in client mode.
#[derive(SystemParam)]
pub struct UdpClientManager<'w, 's> {
    commands: Commands<'w, 's>,
    state: Res<'w, State<UdpTransportMode>>,
    next: ResMut<'w, NextState<UdpTransportMode>>,
}

impl UdpClientManager<'_, '_> {
    /// Try to connect to a remote server.
    pub fn connect(&mut self, address: SocketAddr) {
        todo!()
    }

    /// Disconnect from a remote server if connected to one.
    pub fn disconnect(&mut self) {
        todo!()
    }
}