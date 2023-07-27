use std::collections::{BTreeMap, HashMap};
use bevy::{prelude::*, tasks::TaskPool, ecs::system::SystemState};
use crate::shared::{protocol::{MAX_PACKET_LENGTH, Protocol}, channel::{ChannelId, ChannelDirection}, receive::{Payload, Payloads}};
use super::{clients::Client, receive::{AllChannelData, ChannelData}};

// TODO: All of this can be done better, but I just want it to work.
// Focus on optimisation, especially with the ordering/fragmentation code involving sorting into channels.

/// Minimum packet length in bytes. Packets with less information than this will be discarded.
const MIN_PACKET_BYTES: usize = 7;

// Receives raw packet information from all UDP sockets associated with clients.
pub(super) fn receive_packets_system(
    world: &mut World,
) {
    let pool = TaskPool::new();

    let mut state: SystemState<(
        Res<Protocol>,
        Query<(Entity, &Client)>,
    )> = SystemState::new(world);
    let state = state.get_mut(world);
    
    let protocol = state.0.as_ref();
    let clients = state.1;

    // Read packets from all clients
    let mut packet_groups = pool.scope(|s| {
        for (client_id, client_comp) in clients.iter() {
            let client_id = client_id.clone();
            s.spawn(async move {
                let mut packets = vec![];
                let mut buffer = [0u8; MAX_PACKET_LENGTH];
                loop {
                    if let Ok(bytes) = client_comp.socket.recv(&mut buffer) {
                        // Some early checks
                        // Source address is already filtered by UdpSocket::connect()
                        if bytes < MIN_PACKET_BYTES { continue; } // Not enough data to be a valid packet, discard
                        let protocol_id = u32::from_be_bytes(buffer[0..=3].try_into().unwrap());
                        if protocol_id != protocol.id() { continue; } // Wrong protocol ID, discard packet

                        // Get channel ID and check some things
                        let channel_id = ChannelId::from_bytes(buffer[4..=7].try_into().unwrap());
                        if !protocol.channel_exists(channel_id) { continue; } // Channel does not exist
                        if protocol.channel_config(channel_id).unwrap().direction == ChannelDirection::ServerToClient { continue; } // Messages cannot be sent this way

                        // Copy relevant buffer data into vec
                        let midx = bytes - MIN_PACKET_BYTES - 1;
                        let mut packet = Vec::with_capacity(midx);
                        for i in (MIN_PACKET_BYTES + 1)..(midx) {
                            packet.push(buffer[i]);
                        }

                        // Add message information to list
                        packets.push((client_id, channel_id, packet.into_boxed_slice()));
                    } else {
                        // No more data to read
                        break;
                    }
                }

                packets
            });
        }
    });

    let mut sorted: BTreeMap<ChannelId, Vec<(Entity, Box<[u8]>)>> = BTreeMap::new();

    // Sort into map of messages by channel for processing
    let pg_i = packet_groups.len();
    for _ in 0..(pg_i-1) {
        let mut vs = packet_groups.pop().unwrap();
        let vs_i = vs.len();
        for _ in 0..(vs_i-1) {
            let (client, cid, payload) = vs.pop().unwrap();

            let v = sorted.entry(cid).or_insert(Vec::with_capacity(1));
            v.push((client, payload));
        }
    }

    // Process all packets by channel for transmission data (ordering, fragmentation)
    let mut processed = pool.scope(|s| {
        loop {
            if sorted.len() == 0 { break; } // out of channels to deal with
            let (channel_id, payloads) = sorted.pop_first().unwrap();

            s.spawn(async move {
                let mut intermediate_map = HashMap::new();

                // Invalid channels have been removed while reading packets, so this is fine.
                // let channel_cfg = protocol.channel_config(channel_id).unwrap();

                // Process individual packets
                for (client, data) in payloads {
                    let payload = Payload {
                        ignore_head: 0,
                        ignore_tail: 0,
                        data,
                    };

                    let keyref = intermediate_map.entry(client).or_insert(vec![]);
                    keyref.push(payload);
                }

                // Put into channel map
                let names: Vec<Entity> = intermediate_map.keys().cloned().collect();
                let mut channel_data = ChannelData(HashMap::new());
                for name in names {
                    let f = intermediate_map.remove(&name).unwrap();
                    channel_data.0.insert(name, Payloads(f.into_boxed_slice()));
                }

                (channel_id, channel_data)
            });
        }
    });
    
    // Access resource. Data is cleared at the end of NetworkPreUpdate, so this will be clean for writing.
    let mut cdata = world.resource_mut::<AllChannelData>();
    
    // Write to 
    while processed.len() != 0 {
        let (cid, data) = processed.pop().unwrap();
        cdata.0.insert(cid, data);
    }
}