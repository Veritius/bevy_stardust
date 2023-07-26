use std::collections::BTreeMap;
use bevy::{prelude::*, tasks::TaskPool, ecs::system::SystemState};
use crate::shared::{protocol::{MAX_PACKET_LENGTH, Protocol}, channel::{ChannelId, ChannelDirection, ChannelOrdering}};
use super::{clients::Client, receive::Payload};

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
                        let channel_config = protocol.channel_config(channel_id).unwrap();
                        if channel_config.direction == ChannelDirection::ServerToClient { continue; } // Messages cannot be sent this way

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
}