use std::{net::UdpSocket, collections::BTreeMap};

use bevy::prelude::*;
use super::socket::Socket;

#[derive(Resource)]
pub(crate) struct SocketManager {
    peers: BTreeMap<Entity, usize>,
    sockets: Vec<Socket>,
}

impl SocketManager {
    pub fn new() -> Self {
        Self {
            peers: BTreeMap::new(),
            sockets: vec![],
        }
    }

    pub fn push_socket(&mut self, socket: UdpSocket) {
        self.sockets.push(Socket::new(socket));
    }

    pub fn clear_sockets(&mut self, send_disconnect: bool) {
        if send_disconnect { todo!() }

        self.peers.clear();
        self.sockets.clear();
    }
}