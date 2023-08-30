use std::collections::BTreeMap;
use std::net::UdpSocket;
use std::sync::Mutex;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::octets::varints::u24;
use crate::prelude::*;
use crate::channels::outgoing::OutgoingOctetStringsAccessor;
use crate::transports::udp::PACKET_HEADER_SIZE;
use super::peer::UdpPeer;
use super::ports::PortBindings;

/// Sends octet strings using a sequential strategy.
pub(super) fn udp_send_packets_system_single(
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    mut peers: Query<(Entity, &mut UdpPeer), With<NetworkPeer>>,
    ports: Res<PortBindings>,
    outgoing: OutgoingOctetStringsAccessor,
) {
    // Create buffer
    let mut buffer = [0u8; 1500];

    // Map of channels to speed up accesses
    let channel_map = (0..registry.channel_count())
        .map(|v| ChannelId::try_from(v).unwrap())
        .map(|v| {
            let ent = registry.get_from_id(v).unwrap();
            let q = channels.get(ent).unwrap();
            (v, (q.0, q.1.is_some(), q.2.is_some(), q.3.is_some()))
        })
        .collect::<BTreeMap<_, _>>();
}

/// Sends octet strings using a taskpool strategy.
pub(super) fn udp_send_packets_system_pooled(
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    mut peers: Query<(Entity, &mut UdpPeer), With<NetworkPeer>>,
    ports: Res<PortBindings>,
    outgoing: OutgoingOctetStringsAccessor,
) {
    // Create task pool
    let pool = TaskPoolBuilder::new()
        .thread_name("UdpSendPacketsPool".to_string())
        .build();

    // Place query data into map of mutexes to allow mutation by multiple threads
    let mut query_mutex_map = BTreeMap::new();
    for (id, udp) in peers.iter_mut() {
        query_mutex_map.insert(id, Mutex::new(udp));
    }

    // Map of channels to speed up accesses
    let channels = (0..registry.channel_count())
        .map(|v| ChannelId::try_from(v).unwrap())
        .map(|v| {
            let ent = registry.get_from_id(v).unwrap();
            let q = channels.get(ent).unwrap();
            (v, (q.0, q.1.is_some(), q.2.is_some(), q.3.is_some()))
        })
        .collect::<BTreeMap<_, _>>();

    // Intentional borrow to prevent moves
    let channels = &channels;
    let outgoing = &outgoing;
    let query_mutex_map = &query_mutex_map;

    // Add tasks to pool
    pool.scope(|s| {
        for (port, socket, socket_peers) in ports.iter() {
            // Check the bound port is worth processing
            if socket_peers.len() == 0 { continue; }

            // Spawn task
            s.spawn(async move {
                // Create buffer
                let mut buffer = [0u8; 1500];

                // Take locks for our clients
                let mut locks = query_mutex_map.iter()
                    .filter(|(k,_)| socket_peers.contains(k))
                    .map(|(k,v)| (k, v.lock().unwrap()))
                    .collect::<BTreeMap<_, _>>();

                // Iterate over all clients
                for peer in socket_peers {
                    let peer_data = locks.get_mut(peer).unwrap();

                    for channel in outgoing.by_channel() {
                        let channel_id = channel.id();
                        let (channel_type_path, direction, ordered, reliable) = channels.get(&channel_id).unwrap();

                        let channel_id = (channel_id.0 + 1.into()).unwrap();
                    }
                }
            });
        }
    });
}

fn send_octets(
    socket: &UdpSocket,
    buffer: &mut [u8; 1500],
    channel: u24,
    string: &OctetString,
    client: &mut UdpPeer,
) {
    // Store the current buffer element we're at
    let mut index: usize = 0;

    // Helpful function
    fn write_octet(
        buffer: &mut [u8; 1500],
        index: &mut usize,
        octet: Octet,
    ) {
        buffer[*index] = octet;
        *index += 1;
    }

    // Write channel id
    for octet in channel.bytes() { write_octet(buffer, &mut index, octet); }

    // Write octet string
    if string.as_slice().len() > (1500 - PACKET_HEADER_SIZE) {
        panic!("Packet was too big. Fragmenting is not currently supported, try sending your data in multiple pieces.");
    }
    for octet in string.as_slice() { write_octet(buffer, &mut index, *octet); }

    // Send data
    let _ = socket.send_to(buffer.as_slice(), client.address);
}