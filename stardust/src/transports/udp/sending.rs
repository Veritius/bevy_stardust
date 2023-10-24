use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use json::object;
use crate::messages::outgoing::TransportOutgoingReader;
use crate::prelude::*;
use crate::protocol::UniqueNetworkHash;
use crate::transports::udp::TRANSPORT_LAYER_VERSION_STR;
use super::connections::{EstablishedUdpPeer, PendingUdpPeer, PendingDirection, PendingOutgoingState};
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
    time: Res<Time>,
    mut last_run: Local<f32>,
    ports: Res<PortBindings>,
    mut pending: Query<&mut PendingUdpPeer>,
    protocol: Res<UniqueNetworkHash>,
) {
    if time.raw_elapsed_seconds() < *last_run + OUTGOING_PACKET_RATE { return }
    *last_run = time.raw_elapsed_seconds();

    for (_, socket, conns) in ports.iter() {
        for conn in conns {
            if !pending.contains(*conn) { continue }
            let mut comp = pending.get_mut(*conn).unwrap();
            if let PendingDirection::Outgoing(state) = &mut comp.direction {
                let request = object! {
                    "msg": "req_join",
                    "transport": TRANSPORT_LAYER_VERSION_STR,
                    "protocol": protocol.hex(),
                }.dump();

                if let Err(error) = socket.send_to(request.as_bytes(), comp.address) {
                    error!("Error while sending attempt packet to {}: {}", comp.address, error);
                }
            }
        }
    }
}