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
}