use std::{net::UdpSocket, io::ErrorKind, time::Instant};
use bevy::prelude::*;
use crate::shared::hashdiff::UniqueNetworkHash;

use super::policy::BlockingPolicy;

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
pub(super) struct UdpUnregisteredClient {
    socket: UdpSocket,
    since: Instant,
    state: WaitingClientState,
}

pub(super) enum WaitingClientState {
    WaitingForInitial,
}

pub(super) fn udp_listener_system(
    mut commands: Commands,
    waiting: Query<(Entity, &mut UdpUnregisteredClient)>,
    listener: Res<UdpListener>,
    hash: Res<UniqueNetworkHash>,
    policy: Option<Res<BlockingPolicy>>,
) {
    let mut buffer = [0u8; 1500];
    loop {
        // Check if we've run out of packets to read
        let packet = listener.0.recv_from(&mut buffer);
        if packet.as_ref().is_err_and(|e| e.kind() == ErrorKind::WouldBlock) { break; }
        let (octets, addr) = packet.unwrap();

        // Check the sending IP against the blocking policy
        let blocked = policy
            .as_ref()
            .is_some_and(|v| v.addresses.contains(&addr.ip()));

        // Ignore this packet if the IP is blocked
        if blocked { continue }

        // Get relevant information
        let slice = &buffer[0..octets];
    }
}