use std::collections::BTreeMap;
use bevy::{prelude::*, tasks::TaskPool};
use crate::{server::clients::Client, shared::{channels::{components::*, incoming::IncomingNetworkMessages, registry::ChannelRegistry, id::ChannelId}, payload::{Payloads, Payload}}};
use super::{PACKET_HEADER_SIZE, MAX_PACKET_LENGTH, UdpClient};

pub(super) fn receive_packets_system(
    mut clients: Query<(Entity, &Client, &UdpClient, &mut IncomingNetworkMessages)>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    channel_registry: Res<ChannelRegistry>,
) {
    // Create thread pool for processing
    let pool = TaskPool::new();

    // Explicitly borrow to prevent moves
    let channels = &channels;
    let channel_registry = &channel_registry;

    // Receive packets from clients in parallel
    let mut outputs = pool.scope(|s| {
        for (client_id, _, client_udp, client_incoming) in clients.iter_mut() {
            let client_id = client_id.clone();
            s.spawn(async move {
                let mut map = BTreeMap::new();
                let mut buffer = [0u8; MAX_PACKET_LENGTH];

                let addr = client_udp.address.clone();

                // Read all packets
                loop {
                    // Check if we've run out of packets
                    let Ok(octets) = client_udp.socket.recv(&mut buffer) else { break };

                    // Discard packet, too small to be useful.
                    if octets <= 3 { continue; }

                    // Get channel ID and check it exists
                    let channel_id = ChannelId::from_bytes(&buffer[0..=3].try_into().unwrap());
                    if !channel_registry.channel_exists(channel_id) { break; } // Channel doesn't exist

                    // Copy octets from buffer
                    let idx = octets - PACKET_HEADER_SIZE - 1;
                    let mut packet = Vec::with_capacity(idx);
                    for i in (PACKET_HEADER_SIZE + 1)..idx {
                        packet.push(buffer[i]);
                    }

                    println!("Received UDP payload {:?}", &packet);

                    map
                        .entry(channel_id)
                        .or_insert(Vec::with_capacity(1))
                        .push(packet);
                }

                // Process map into Payloads
                let mut nmap = BTreeMap::new();
                while map.len() != 0 {
                    let (cid, mut vec) = map.pop_first().unwrap();
                    let payloads: Payloads = vec
                        .drain(..)
                        .map(|x| {
                            Payload::new(0, 0, x) })
                        .collect::<Vec<Payload>>()
                        .into();
                    nmap.insert(cid, payloads);
                }

                (client_id, nmap)
            });
        }
    });

    // Write client data
    for (client, map) in outputs {
        let (_, _, _, mut data) = clients.get_mut(client).unwrap();
        data.0 = map;
    }
}