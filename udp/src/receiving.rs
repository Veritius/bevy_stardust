use std::{io::ErrorKind, sync::Mutex};
use bevy_ecs::prelude::*;
use bytes::Bytes;
use crate::{appdata::AppNetVersionWrapper, connection::PotentialNewPeer, packet::IncomingPacket, Connection, Endpoint};

// Receives packets from UDP sockets
pub(crate) fn io_receiving_system(
    commands: ParallelCommands,
    appdata: Res<AppNetVersionWrapper>,
    mut endpoints: Query<(Entity, &mut Endpoint)>,
    connections: Query<&mut Connection>,
    mut new_peers: EventWriter<PotentialNewPeer>,
) {
    // Wrap the new peers eventwriter in a mutex
    // The risk of contention here probably isn't too bad
    let new_peers = Mutex::new(&mut new_peers);

    // Iterate all endpoints
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        loop {
            let mut scratch = [0u8; 1478];
            match endpoint.udp_socket.recv_from(&mut scratch) {
                // Received a UDP packet
                Ok((bytes, origin)) => {
                    let payload = Bytes::copy_from_slice(&scratch[..bytes]);

                    match endpoint.connections.get(&origin) {
                        // We know this peer
                        Some(token) => {
                            // SAFETY: This is fine because of ConnectionOwnershipToken's guarantees
                            let mut connection = unsafe { connections.get_unchecked(token.inner()).unwrap() };

                            // Set last_recv in timings
                            connection.timings.set_last_recv_now();

                            // We append it to the queue for later processing
                            connection.packet_queue.push_incoming(IncomingPacket { payload });
                        },

                        // We don't know this peer
                        None => {
                            new_peers.lock().unwrap().send(PotentialNewPeer {
                                endpoint: endpoint_id,
                                address: origin,
                                payload,
                            });
                        },
                    }
                },

                // No more packets to read
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    // Break out of the loop
                    break
                },

                // I/O error reported by the system
                Err(err) => {
                    // TODO: Close endpoints based on certain errors
                    todo!();
                }
            }
        }
    });
}