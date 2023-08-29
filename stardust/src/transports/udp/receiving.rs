use std::net::SocketAddr;
use std::{collections::BTreeMap, sync::Mutex};
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::channels::incoming::IncomingNetworkMessages;
use crate::prelude::*;
use super::PACKET_HEADER_SIZE;
use super::peer::UdpPeer;
use super::ports::PortBindings;

/// Receives octet strings using a sequential strategy.
pub(super) fn udp_receive_packets_system_single(
    mut peers: Query<(Entity, &mut UdpPeer, &mut IncomingNetworkMessages), With<NetworkPeer>>,
    ports: Res<PortBindings>,
    channels: Query<(Option<&DirectionalChannel>, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    registry: Res<ChannelRegistry>,
) {
    // Map of channels to speed up accesses
    let channel_map = (0..registry.channel_count())
        .map(|v| ChannelId::try_from(v).unwrap())
        .map(|v| {
            let ent = registry.get_from_id(v).unwrap();
            let q = channels.get(ent).unwrap();
            (v, (q.0, q.1.is_some(), q.2.is_some(), q.3.is_some()))
        })
        .collect::<BTreeMap<_, _>>();

    // Map addresses to peer entity's components
    let peer_addresses = peers.iter_mut()
        .map(|(id, udp, incoming)| (udp.address.clone(), (udp, id, incoming)))
        .collect::<BTreeMap<SocketAddr, _>>();
}

/// Receives octet strings using a taskpool strategy.
pub(super) fn udp_receive_packets_system_pooled(
    mut peers: Query<(Entity, &NetworkPeer, &mut UdpPeer, &mut IncomingNetworkMessages)>,
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
    for (id, client, udp, incoming) in peers.iter_mut() {
        query_mutex_map.insert(id, Mutex::new((client, udp, incoming)));
    }

    // Map of channels to speed up accesses
    let channel_map = (0..registry.channel_count())
        .map(|v| ChannelId::try_from(v).unwrap())
        .map(|v| {
            let ent = registry.get_from_id(v).unwrap();
            let q = channels.get(ent).unwrap();
            (v, (q.0, q.1.is_some(), q.2.is_some(), q.3.is_some()))
        })
        .collect::<BTreeMap<ChannelId, _>>();

    // Explicit borrows to prevent moves
    let query_mutex_map = &query_mutex_map;
    let channel_map = &channel_map;
    let registry = &registry;

    // Add task to pool
    pool.scope(|s| {
        for (_, socket, socket_peers) in ports.iter() {
            // Check if this socket is worth processing
            if socket_peers.len() == 0 { continue; }

            // Spawn task
            s.spawn(async move {
                // Allocate a buffer for storing incoming data
                let mut buffer = [0u8; 1500];

                // Lock mutexes for our port-associated clients
                let mut locks = query_mutex_map.iter()
                    .filter(|(k,_)| socket_peers.contains(k))
                    .map(|(k,v)| (k, v.lock().unwrap()))
                    .collect::<BTreeMap<_, _>>();

                // Create vec of addresses for rapid filtering
                let addresses = locks.iter()
                    .map(|(_, guard)| guard.1.address)
                    .collect::<Vec<_>>();

                // Map addresses to entity IDs
                let address_map = locks.iter()
                    .map(|(entity, guard)| (guard.1.address, **entity))
                    .collect::<BTreeMap<_, _>>();

                // Read packets until we run out
                loop {
                    // Read packet
                    let (octets_read, from_address) = match socket.recv_from(&mut buffer) {
                        Ok(n) => n,
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // No more data to read
                            break
                        }
                        Err(e) => {
                            // Something went wrong
                            error!("IO error while reading UDP socket {:?}: {}", socket.local_addr().unwrap(), e);
                            continue
                        },
                    };

                    // Simple validity checks
                    if !addresses.contains(&from_address) { continue } // Not from a client associated with this socket
                    if octets_read < PACKET_HEADER_SIZE { continue } // Packet is too small to be of any value

                    // Check channel id
                    let channel_id = ChannelId::from(TryInto::<[u8; 3]>::try_into(&buffer[..3]).unwrap());
                    if channel_id.0 == 0.into() {
                        // This is a special packet
                        todo!()
                    }

                    // Get channel config
                    let channel_id = ChannelId(channel_id.0 - 1.into()); // Shift the channel ID back
                    if !registry.channel_exists(channel_id) { continue } // Channel doesn't exist
                    let (direction, ordered, reliable, fragmented) = channel_map.get(&channel_id).unwrap();

                    // Check channel direction
                    if direction.is_some_and(|v| *v == DirectionalChannel::ServerToClient) {
                        // Packet went in the wrong direction
                        let entity_id = address_map.get(&from_address).unwrap();
                        let peer = &mut locks.get_mut(entity_id).unwrap().1;
                        peer.hiccups += 1;
                        continue
                    }

                    // Get client lock
                    let entity_id: &Entity = address_map.get(&from_address).unwrap();
                    let guard = locks.get_mut(entity_id).unwrap();

                    // Any bytes that don't need to be cloned
                    let cutoff = PACKET_HEADER_SIZE;

                    // Copy data to vec and make Payload
                    let mut payload = Vec::with_capacity(octets_read - cutoff);
                    for oct in &buffer[cutoff..=octets_read] { payload.push(*oct); }
                    let payload = Payload::new(0, 0, payload);

                    // Place payload in incoming component
                    guard.2.append(channel_id, payload);
                }
            });
        }
    });
}