use std::collections::BTreeMap;
use std::net::UdpSocket;
use std::sync::Mutex;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::messages::outgoing::TransportOutgoingReader;
use crate::octets::varints::u24;
use crate::prelude::*;
use crate::transports::udp::{PACKET_HEADER_SIZE, PACKET_MAX_BYTES};
use super::connections::{EstablishedUdpPeer, PendingUdpPeer};
use super::ports::PortBindings;

/// Sends octet strings using a taskpool strategy.
pub(super) fn send_packets_system(
    mut peers: Query<(Entity, &mut EstablishedUdpPeer), With<NetworkPeer>>,
    mut pending: Query<(Entity, &mut PendingUdpPeer)>,
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    ports: Res<PortBindings>,
    outgoing: TransportOutgoingReader,
) {
    // Create task pool
    let taskpool = TaskPoolBuilder::new()
        .thread_name("UDP pkt send".to_string())
        .build();

    taskpool.scope(|s| {
        for (_, socket, peers) in ports.iter() {
            s.spawn(async move {
                todo!()
            });
        }
    });
}