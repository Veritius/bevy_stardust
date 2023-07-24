use bevy::{prelude::*, tasks::TaskPool};
use crate::shared::protocol::MAX_PACKET_LENGTH;
use super::clients::Client;

// Receives raw packet information from all UDP sockets associated with clients.
pub(super) fn receive_packets_system(
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
                        let mut packet = buffer.to_vec();
                        packet.resize(bytes, 0);
                        packets.push(packet);
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