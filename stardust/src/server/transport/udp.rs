//! Native UDP transport layer for servers.

use std::{net::UdpSocket, collections::{BTreeMap, HashMap}};
use bevy::{prelude::*, tasks::TaskPool};
use crate::{shared::{scheduling::{TransportReadPackets, TransportSendPackets}, channels::{components::{OrderedChannel, ReliableChannel, FragmentedChannel, ChannelData}, id::ChannelId, registry::ChannelRegistry}, receive::{Payload, Payloads}}, server::{clients::Client, receive::{AllClientMessages, AllChannelData}}};

/// A simple transport layer over native UDP sockets.
pub struct ServerUdpTransportPlugin;
impl Plugin for ServerUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}

/// A client connected with the `ServerUdpTransportPlugin` transport layer.
#[derive(Component)]
pub struct UdpClient(UdpSocket);

/// Maximum packet length that can be sent/received before fragmentation.
const MAX_PACKET_LENGTH: usize = 1500;
/// The amount of bytes that will always be present in all packages.
const PACKET_HEADER_SIZE: usize = 3;

fn receive_packets_system(
    mut commands: Commands,
    clients: Query<(Entity, &Client, &UdpClient)>,
    channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    channel_registry: Res<ChannelRegistry>,
) {
    // Create thread pool for processing
    let pool = TaskPool::new();

    // Explicitly borrow to prevent moves
    let channels = &channels;
    let channel_registry = &channel_registry;

    // Receive packets from connected clients
    let mut client_packets = pool.scope(|s| {
        for (client_id, _, client_udp) in clients.iter() {
            let client_id = client_id.clone();
            s.spawn(async move {
                let mut packets = vec![];
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

                        // Insert packet data
                        packets.push((client_id, channel_id, packet.into_boxed_slice()));
                    } else {
                        // We're done reading packets
                        break;
                    }
                }

                // Return packets
                packets
            });
        }
    });

    let mut sorted: BTreeMap<ChannelId, Vec<(Entity, Box<[u8]>)>> = BTreeMap::new();

    // Sort into channels for processing
    while client_packets.len() != 0 {
        let mut pg = client_packets.pop().unwrap();
        while pg.len() != 0 {
            let (client, channel, payload) = pg.pop().unwrap();
            let v = sorted.entry(channel).or_insert(Vec::with_capacity(1));
            v.push((client, payload));
        }
    }

    // Process all packets by channel
    let mut processed = pool.scope(|s| {
        while sorted.len() != 0 {
            // Channel config
            let (channel_id, payloads) = sorted.pop_first().unwrap();
            let channel_ent = channel_registry.get_from_id(channel_id).unwrap();
            let (c_data, c_ord, c_rel, c_fra) = channels.get(channel_ent).unwrap();

            // Process packets
            s.spawn(async move {
                let mut intermediate_map = HashMap::new();

                // Process individual packets
                for (client, data) in payloads {
                    let payload = Payload {
                        ignore_head: 0,
                        ignore_tail: 0,
                        data,
                    };

                    let keyref = intermediate_map.entry(client).or_insert(Vec::with_capacity(1));
                    keyref.push(payload);
                }

                // Put into channel map
                let names: Vec<Entity> = intermediate_map.keys().cloned().collect();
                let mut channel_data = AllClientMessages(HashMap::new());
                for name in names {
                    let f = intermediate_map.remove(&name).unwrap();
                    channel_data.0.insert(name, Payloads(f.into_boxed_slice()));
                }

                (channel_id, channel_data)
            });
        }
    });

    let mut cdata = BTreeMap::new();
    while processed.len() != 0 {
        let (cid, data) = processed.pop().unwrap();
        cdata.insert(cid, data);
    }

    commands.insert_resource(AllChannelData(cdata));
}

fn send_packets_system(
    
) {

}