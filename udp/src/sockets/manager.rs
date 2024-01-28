use std::{net::UdpSocket, collections::BTreeMap};
use bevy::prelude::*;
use super::socket::Socket;

#[derive(Resource)]
pub(crate) struct SocketManager {
    sockets: Vec<Socket>,
}

impl SocketManager {
    pub fn new() -> Self {
        Self {
            sockets: vec![],
        }
    }

    pub fn push_socket(&mut self, socket: UdpSocket) {
        self.sockets.push(Socket::new(socket));
    }

    pub fn clear_sockets(&mut self, send_disconnect: bool) {
        if send_disconnect { todo!() }
        self.sockets.clear();
    }

    pub fn iter_sockets(&self) -> impl Iterator<Item = &Socket> {
        self.sockets.iter()
    }
}

#[derive(Event)]
pub(crate) enum SocketManagerEvent {
    PushSocket {
        socket: UdpSocket
    },
    ClearSockets {
        disconnect: bool,
    },
}

pub(crate) fn socket_manager_system(
    mut reader: EventReader<SocketManagerEvent>,
    mut manager: ResMut<SocketManager>,
) {
    for item in reader.read() {
        match item {
            // Adds a socket to the manager
            SocketManagerEvent::PushSocket { socket } => {
                // Clone the socket
                let cloned = socket.try_clone();
                if let Err(ref error) = cloned {
                    error!("Failed to clone UdpSocket for address {:?}: {error}", socket.local_addr());
                }

                // Pushes the socket
                manager.push_socket(cloned.unwrap());
            },

            // Clears the bound sockets in the manager
            SocketManagerEvent::ClearSockets { disconnect } => {
                manager.clear_sockets(*disconnect);
            },
        }
    }
}