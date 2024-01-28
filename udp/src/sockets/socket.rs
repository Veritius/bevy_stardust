use std::net::UdpSocket;
use bevy::prelude::*;

pub(crate) struct Socket {
    socket: UdpSocket,
    peers: Vec<Entity>,
}

impl Socket {
    pub(super) fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            peers: vec![],
        }
    }
}