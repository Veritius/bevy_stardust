use bevy::{prelude::*, tasks::TaskPool};
use crate::shared::{protocol::{MAX_PACKET_LENGTH, Protocol}, channel::ChannelId};
use super::clients::Client;

/// Minimum packet length in bytes. Packets with less information than this will be discarded.
const MIN_PACKET_BYTES: usize = 7;

// Receives raw packet information from all UDP sockets associated with clients.
pub(super) fn receive_packets_system(
    protocol: Res<Protocol>,
    clients: Query<&Client>,
) {
    let mut pool = TaskPool::new();

    // Read packets from all clients
    let packets = pool.scope(|s| {
        for client in clients.iter() {
            s.spawn(async {
                let mut packets = vec![];
                let mut buffer = [0u8; MAX_PACKET_LENGTH];
                loop {
                    if let Ok(bytes) = client.socket.recv(&mut buffer) {
                        // Some early checks
                        if bytes < MIN_PACKET_BYTES { continue; } // Not enough data to be a valid packet, discard
                        let protocol_id = u32::from_be_bytes(buffer[0..=3].try_into().unwrap());
                        if protocol_id != protocol.id() { continue; } // Wrong protocol ID, discard packet
                        
                        // Get channel ID
                        let channel_id = ChannelId::from_bytes(buffer[4..=7].try_into().unwrap());

                        // Copy relevant buffer data into vec
                        let mut packet = Vec::with_capacity(bytes - MIN_PACKET_BYTES);
                        for i in (MIN_PACKET_BYTES + 1)..bytes {
                            let byte = buffer.get(i);
                            if byte.is_none() { continue; }
                            packet.push(byte.unwrap().clone());
                        }
                        
                        // Push packet payload and channel ID
                        packets.push((channel_id, packet));
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