use std::{sync::Mutex, collections::BTreeMap};
use bevy::{prelude::*, tasks::TaskPool};
use crate::{server::clients::Client, shared::{channels::{components::*, incoming::IncomingNetworkMessages, registry::ChannelRegistry, id::ChannelId}, payload::Payload}};
use super::{PACKET_HEADER_SIZE, MAX_PACKET_LENGTH, UdpClient, ports::PortBindings};

pub(super) fn receive_packets_system(
    mut clients: Query<(Entity, &Client, &UdpClient, &mut IncomingNetworkMessages)>,
    ports: Res<PortBindings>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    registry: Res<ChannelRegistry>,
) {
    // Create task pool
    let pool = TaskPool::new();

    // Place query data into map of mutexes to allow mutation by multiple threads
    let mut query_mutex_map = BTreeMap::new();
    for (id, client, udp, incoming) in clients.iter_mut() {
        query_mutex_map.insert(id, Mutex::new((client, udp, incoming)));
    }

    // Explicit borrows to prevent moves
    let query_mutex_map = &query_mutex_map;
    let _channels = &channels;
    let registry = &registry;

    // Create tasks for all sockets
    pool.scope(|s| {
        for (_, socket, clients) in ports.iter() {
            if clients.len() == 0 { continue }
            s.spawn(async move {
                // Lock mutexes for our port-associated clients
                let mut locks = query_mutex_map.iter()
                    .filter(|(k,_)| clients.contains(k))
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

                // Receive packets from this task's socket
                let mut buffer = [0u8; 1500];
                loop {
                    // Early validity checks
                    let Ok((octets_read, from_address)) = socket.recv_from(&mut buffer) else { break };
                    if !addresses.contains(&from_address) { continue } // Packet isn't from one of the clients associated with this port
                    if octets_read < MAX_PACKET_LENGTH { continue } // Packet is too small to be of any value

                    // Channel info
                    let channel_id = ChannelId::from_bytes(&buffer[..3].try_into().unwrap());
                    if !registry.channel_exists(channel_id) { continue } // Channel doesn't exist

                    // Copy data to vec and make Payload
                    let mut payload = Vec::with_capacity(octets_read - PACKET_HEADER_SIZE);
                    for oct in &buffer[PACKET_HEADER_SIZE..=octets_read] { payload.push(*oct); }
                    let payload = Payload::new(0, 0, payload);

                    // Place payload in incoming component
                    let entity_id = address_map.get(&from_address).unwrap();
                    let incoming = &mut locks.get_mut(entity_id).unwrap().2;
                    incoming.append(channel_id, payload);
                }
            });
        }
    });
}