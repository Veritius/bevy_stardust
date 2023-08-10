use std::net::UdpSocket;
use bevy::prelude::*;

/// Unfiltered socket for listening to UDP packets from unregistered peers.
#[derive(Resource)]
pub(super) struct UdpListener(pub UdpSocket);

impl UdpListener {
    pub fn new(port: u16) -> Self {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))
            .expect("Couldn't bind to port");
        
        socket.set_nonblocking(true).unwrap();

        Self(socket)
    }
}

#[derive(Component)]
struct UdpUnregisteredClient;

pub(super) fn udp_listener_system(
    listener: Res<UdpListener>,
) {

}