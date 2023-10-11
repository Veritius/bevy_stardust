use std::collections::BTreeMap;
use std::net::UdpSocket;
use std::sync::Mutex;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::octets::varints::u24;
use crate::prelude::*;
use crate::channels::outgoing::OutgoingOctetStringsAccessor;
use crate::transports::udp::{PACKET_HEADER_SIZE, PACKET_MAX_BYTES};
use super::peer::{EstablishedUdpPeer, PendingUdpPeer};
use super::ports::PortBindings;

/// Sends octet strings using a taskpool strategy.
pub(super) fn send_packets_system(
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    mut peers: Query<(Entity, &mut EstablishedUdpPeer), With<NetworkPeer>>,
    pending: Query<(Entity, &PendingUdpPeer)>,
    ports: Option<Res<PortBindings>>,
    outgoing: OutgoingOctetStringsAccessor,
) {
    // Create task pool
    let pool = TaskPoolBuilder::new()
        .thread_name("UDP pkt send".to_string())
        .build();
}