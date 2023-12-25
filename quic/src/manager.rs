use std::net::SocketAddr;

use bevy::ecs::system::SystemParam;

#[derive(SystemParam)]
pub struct QuicManager {

}

impl QuicManager {
    pub fn start_endpoint(&mut self, address: SocketAddr) {
        todo!()
    }

    pub fn close_endpoint(&mut self) {
        todo!()
    }

    pub fn allow_incoming(&mut self, value: bool) {
        todo!()
    }

    pub fn try_connect(&mut self, address: SocketAddr) {
        todo!()
    }
}