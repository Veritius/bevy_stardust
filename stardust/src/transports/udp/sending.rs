use std::collections::BTreeMap;
use std::net::UdpSocket;
use std::sync::Mutex;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::octets::varints::u24;
use crate::prelude::*;
use crate::channels::outgoing::OutgoingOctetStringsAccessor;
use crate::transports::udp::{PACKET_HEADER_SIZE, PACKET_MAX_BYTES};
use super::peer::UdpPeer;
use super::ports::PortBindings;

// TODO: A lot of code is repeated here.

/// Sends octet strings using a sequential strategy.
pub(super) fn udp_send_packets_system_single(
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&DirectionalChannel>, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    mut peers: Query<(Entity, &mut UdpPeer), With<NetworkPeer>>,
    ports: Res<PortBindings>,
    outgoing: OutgoingOctetStringsAccessor,
) {
    // Create buffer
    let mut buffer = [0u8; PACKET_MAX_BYTES];

    // Map of channels to speed up accesses
    let channels = (0..registry.channel_count())
        .map(|v| ChannelId::try_from(v).unwrap())
        .map(|v| {
            let ent = registry.get_from_id(v).unwrap();
            let q = channels.get(ent).unwrap();
            (v, (q.0, q.1, q.2.is_some(), q.3.is_some(), q.4.is_some()))
        })
        .collect::<BTreeMap<_, _>>();

    // Iterate all sockets
    for (_, socket, socket_peers) in ports.iter() {
        // Check the bound port is worth processing
        if socket_peers.len() == 0 { continue; }
        
        // Iterate over clients
        for peer in socket_peers {
            let mut udp_peer = peers.get_mut(*peer).unwrap().1;

            for channel in outgoing.by_channel() {
                let channel_id = channel.id();
                let (channel_data, direction, _ordered, _reliable, _fragmented) = channels.get(&channel_id).unwrap();

                // Shift the channel ID by 1
                let channel_shf_id = (channel_id.0 + 1.into()).unwrap();

                // Check channel direction
                if direction.is_some_and(|v| *v == DirectionalChannel::ClientToServer) {
                    panic!("Tried to send a message on client to server channel: {}", channel_data.type_path());
                    // TODO: Maybe write an error to the console?
                }

                // Iterate all octet strings
                for (target, octets) in channel.strings().read() {
                    // Check message is for client
                    if target.excludes(*peer) { continue; }

                    // Send packet
                    send_octets(socket, &mut buffer, channel_shf_id, octets, &mut udp_peer);
                }
            }
        }
    }
}

/// Sends octet strings using a taskpool strategy.
pub(super) fn udp_send_packets_system_pooled(
    registry: Res<ChannelRegistry>,
    channels: Query<(&ChannelData, Option<&DirectionalChannel>, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
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
            (v, (q.0, q.1, q.2.is_some(), q.3.is_some(), q.4.is_some()))
        })
        .collect::<BTreeMap<_, _>>();

    // Intentional borrow to prevent moves
    let channels = &channels;
    let outgoing = &outgoing;
    let query_mutex_map = &query_mutex_map;

    // Add tasks to pool
    pool.scope(|s| {
        for (_, socket, socket_peers) in ports.iter() {
            // Check the bound port is worth processing
            if socket_peers.len() == 0 { continue; }

            // Spawn task
            s.spawn(async move {
                // Create buffer
                let mut buffer = [0u8; PACKET_MAX_BYTES];

                // Take locks for our clients
                let mut locks = query_mutex_map.iter()
                    .filter(|(k,_)| socket_peers.contains(k))
                    .map(|(k,v)| (k, v.lock().unwrap()))
                    .collect::<BTreeMap<_, _>>();

                // Iterate over all clients
                for peer in socket_peers {
                    let mut udp_peer = locks.get_mut(peer).unwrap();

                    for channel in outgoing.by_channel() {
                        let channel_id = channel.id();
                        let (channel_data, direction, _ordered, _reliable, _fragmented) = channels.get(&channel_id).unwrap();

                        // Shift the channel ID by 1
                        let channel_shf_id = (channel_id.0 + 1.into()).unwrap();

                        // Check channel direction
                        if direction.is_some_and(|v| *v == DirectionalChannel::ClientToServer) {
                            panic!("Tried to send a message on client to server channel: {}", channel_data.type_path());
                            // TODO: Maybe write an error to the console?
                        }

                        // Iterate all octet strings
                        for (target, octets) in channel.strings().read() {
                            // Check message is for client
                            if target.excludes(*peer) { continue; }

                            // Send packet
                            send_octets(socket, &mut buffer, channel_shf_id, octets, &mut udp_peer);
                        }
                    }
                }
            });
        }
    });
}

fn send_octets(
    socket: &UdpSocket,
    buffer: &mut [u8; PACKET_MAX_BYTES],
    channel: u24,
    string: &OctetString,
    udp_peer: &mut UdpPeer,
) {
    // Store the current buffer element we're at
    let mut index: usize = 0;

    // Helpful function
    fn write_octet(
        buffer: &mut [u8; PACKET_MAX_BYTES],
        index: &mut usize,
        octet: Octet,
    ) {
        buffer[*index] = octet;
        *index += 1;
    }

    // Write channel id
    for octet in channel.bytes() { write_octet(buffer, &mut index, octet); }

    // Write octet string
    if string.as_slice().len() > (PACKET_MAX_BYTES - PACKET_HEADER_SIZE) {
        panic!("Packet was too big. Fragmenting is not currently supported, try sending your data in multiple pieces.");
    }
    for octet in string.as_slice() { write_octet(buffer, &mut index, *octet); }

    // Send data
    let _ = socket.send_to(buffer.as_slice(), udp_peer.address);
}