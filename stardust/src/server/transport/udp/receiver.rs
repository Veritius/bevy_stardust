use std::collections::BTreeMap;
use bevy::{prelude::*, tasks::TaskPool};
use crate::{shared::{channels::{id::ChannelId, components::*, registry::ChannelRegistry}, messages::receive::IncomingNetworkMessages}, server::clients::Client};
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

    // Receive packets from connected clients
    pool.scope(|s| {
        for (client_id, _, client_udp, client_incoming) in clients.iter_mut() {
            let client_id = client_id.clone();
            s.spawn(async move {
                let mut map = BTreeMap::new();
                let mut buffer = [0u8; MAX_PACKET_LENGTH];

                // Read all packets
                loop {
                    if let Ok(octets) = client_udp.0.recv(&mut buffer) {
                        // Discard packet, too small to be useful.
                        if octets <= 3 { continue; }

                        // Get channel ID and check it exists
                        let channel_id = ChannelId::try_from(&buffer[0..=3]).unwrap();
                        if !channel_registry.channel_exists(channel_id) { break; }

                        // Copy octets from buffer
                        let idx = octets - PACKET_HEADER_SIZE - 1;
                        let mut packet = Vec::with_capacity(idx);
                        for i in (PACKET_HEADER_SIZE + 1)..idx {
                            packet.push(buffer[i]);
                        }

                        map.entry(channel_id).or_insert(Vec::with_capacity(1)).push(packet);
                    } else {
                        // We're done reading packets
                        break;
                    }
                }
            });
        }
    });
}