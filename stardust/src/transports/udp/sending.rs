use std::collections::BTreeMap;
use std::sync::Mutex;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::prelude::*;
use crate::channels::outgoing::OutgoingOctetStringsAccessor;
use super::peer::UdpPeer;
use super::ports::PortBindings;

/// Sends octet strings using a sequential strategy.
pub(super) fn udp_send_packets_system_single(
    registry: Res<ChannelRegistry>,
    channels: Query<(Entity, &ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    mut clients: Query<(Entity, &UdpPeer), With<NetworkPeer>>,
    outgoing: OutgoingOctetStringsAccessor,
) {

}

/// Sends octet strings using a taskpool strategy.
pub(super) fn udp_send_packets_system_pooled(
    registry: Res<ChannelRegistry>,
    channels: Query<(Entity, &ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    mut clients: Query<(Entity, &UdpPeer), With<NetworkPeer>>,
    ports: Res<PortBindings>,
    outgoing: OutgoingOctetStringsAccessor,
) {
    // Create task pool
    let pool = TaskPoolBuilder::new()
        .thread_name("UdpSendPacketsPool".to_string())
        .build();

    // Place query data into map of mutexes to allow mutation by multiple threads
    let mut query_mutex_map = BTreeMap::new();
    for (id, udp) in clients.iter_mut() {
        query_mutex_map.insert(id, Mutex::new(udp));
    }

    // Intentional borrow to prevent moves
    let registry = &registry;
    let channels = &channels;
    let outgoing = &outgoing;
    let query_mutex_map = &query_mutex_map;
}