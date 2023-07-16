use std::net::UdpSocket;
use bevy::prelude::Resource;

#[derive(Resource)]
pub(crate) struct NetworkSocket(pub UdpSocket);

impl NetworkSocket {
    pub fn new(port: u16) -> Self {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).expect("Couldn't create port");
        socket.set_nonblocking(true).expect("Couldn't set socket to nonblocking");
        Self(socket)
    }
}