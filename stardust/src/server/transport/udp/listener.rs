use std::{net::UdpSocket, io::ErrorKind};
use bevy::prelude::*;
use super::lists::BlockingPolicy;

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
    policy: Res<BlockingPolicy>,
) {
    let mut buffer = [0u8; 1500];
    loop  {
        let packet = listener.0.recv_from(&mut buffer);
        if packet.as_ref().is_err_and(|e| e.kind() == ErrorKind::WouldBlock) { break; }
        let (octets, addr) = packet.unwrap();
    }
}