use std::{collections::BTreeMap, sync::Mutex};
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::channels::incoming::IncomingNetworkMessages;
use crate::prelude::*;
use super::peer::UdpPeer;
use super::ports::PortBindings;

/// Receives octet strings using a sequential strategy.
pub(super) fn udp_receive_packets_system_single(
    mut clients: Query<(Entity, &NetworkPeer, &mut UdpPeer, &mut IncomingNetworkMessages)>,
    channels: Query<(Option<&DirectionalChannel>, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    registry: Res<ChannelRegistry>,
) {

}

/// Receives octet strings using a taskpool strategy.
pub(super) fn udp_receive_packets_system_pooled(
    mut clients: Query<(Entity, &NetworkPeer, &mut UdpPeer, &mut IncomingNetworkMessages)>,
    ports: Res<PortBindings>,
    channels: Query<(Option<&DirectionalChannel>, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    registry: Res<ChannelRegistry>,
) {
    // Create task pool
    let pool = TaskPoolBuilder::new()
        .thread_name("UdpReadPacketsPool".to_string())
        .build();

    // Place query data into map of mutexes to allow mutation by multiple threads
    let mut query_mutex_map = BTreeMap::new();
    for (id, client, udp, incoming) in clients.iter_mut() {
        query_mutex_map.insert(id, Mutex::new((client, udp, incoming)));
    }

    // Explicit borrows to prevent moves
    let query_mutex_map = &query_mutex_map;
    let channels = &channels;
    let registry = &registry;
}