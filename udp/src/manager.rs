use std::net::UdpSocket;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::sockets::SocketManagerEvent;

#[derive(SystemParam)]
pub struct UdpConnectionManager<'w> {
    socket_manager: EventWriter<'w, SocketManagerEvent>,
}

impl<'w> UdpConnectionManager<'w> {
    
}