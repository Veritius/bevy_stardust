use std::net::{SocketAddr, UdpSocket};
use std::time::Instant;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use json::object;
use crate::messages::outgoing::TransportOutgoingReader;
use crate::prelude::*;
use crate::protocol::UniqueNetworkHash;
use crate::transports::udp::TRANSPORT_LAYER_VERSION_STR;
use super::connections::{EstablishedUdpPeer, PendingUdpPeer};
use super::ports::PortBindings;

/// Sends octet strings using a taskpool strategy.
pub(super) fn send_packets_system(
    mut established: Query<(Entity, &mut EstablishedUdpPeer), With<NetworkPeer>>,
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    ports: Res<PortBindings>,
    outgoing: TransportOutgoingReader,
) {
    // Create task pool
    let taskpool = TaskPoolBuilder::new()
        .thread_name("UDP pkt send".to_string())
        .build();
    
    // Iterate over all messages
    taskpool.scope(|s| {
        for (_, socket, peers) in ports.iter() {
            s.spawn(async move {
                
            });
        }
    });
}

/// Delay between sending attempt packets. Currently 5hz.
const OUTGOING_PACKET_RATE: f32 = 0.2;

/// Sends connection attempt packets every now and then.
pub(super) fn attempt_connection_system(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut last_run: Local<f32>,
    ports: Res<PortBindings>,
    mut pending: Query<&mut PendingUdpPeer>,
    protocol: Res<UniqueNetworkHash>,
) {
    if time.elapsed_seconds() < *last_run + OUTGOING_PACKET_RATE { return }
    *last_run = time.elapsed_seconds();

    for (_, socket, conns) in ports.iter() {
        for conn in conns {
            if !pending.contains(*conn) { continue }
            let comp = pending.get_mut(*conn).unwrap();
            
            if comp.started.duration_since(Instant::now()) > comp.timeout {
                info!("Connection attempt to {} timed out", comp.address);
                commands.entity(*conn).despawn();
            }

            let request = object! {
                "msg": "req_join",
                "transport": TRANSPORT_LAYER_VERSION_STR,
                "protocol": protocol.hex(),
            }.dump();

            send_zero_packet(socket, comp.address, request.as_bytes());
        }
    }
}

pub(super) fn send_zero_packet(socket: &UdpSocket, address: SocketAddr, bytes: &[u8]) {
    let mut buffer = [0u8; 256]; // The limit can be increased if needed
    let mut idx: usize = 3; // skip first f32 bytes
    for byte in bytes {
        buffer[idx] = *byte;
        idx += 1;
    }
    if let Err(error) = socket.send_to(&buffer[0..idx], address) {
        error!("Error while sending packet to {}: {}", address, error);
    }
}