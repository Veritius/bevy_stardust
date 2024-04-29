use std::{io, sync::Mutex, time::Instant};
use bevy::prelude::*;
use bytes::Bytes;
use crate::{connection::PotentialNewPeer, prelude::*};

// Receives packets from UDP sockets
pub(crate) fn io_receiving_system(
    mut endpoints: Query<(Entity, &mut Endpoint)>,
    connections: Query<&mut Connection>,
    mut new_peers: EventWriter<PotentialNewPeer>,
) {
    // Wrap the new peers eventwriter in a mutex
    // The risk of contention here probably isn't too bad
    let new_peers = Mutex::new(&mut new_peers);

    // Iterate all endpoints
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        // Stuff for logging
        let mut pkts_received: u32 = 0;
        let mut bytes_received: u64 = 0;
        let span = tracing::trace_span!("Receiving packets on endpoint", id=?endpoint_id);
        let _entered_span = span.enter();

        loop {
            let mut scratch = [0u8; 1478];
            match endpoint.udp_socket.recv_from(&mut scratch) {
                // Received a UDP packet
                Ok((bytes, origin)) => {
                    // Track the packet recv
                    pkts_received += 1; bytes_received += bytes as u64;
                    endpoint.statistics.record_packet_recv(bytes);

                    // Store the message in the heap
                    let payload = Bytes::copy_from_slice(&scratch[..bytes]);

                    match endpoint.connections.get(&origin) {
                        // We know this peer
                        Some(token) => {
                            // SAFETY: This is fine because of ConnectionOwnershipToken's guarantees
                            let mut connection = unsafe { connections.get_unchecked(token.inner()).unwrap() };

                            // Set last_recv in timings and update statistics
                            connection.last_recv = Some(Instant::now());

                            // We append it to the queue for later processing
                            // connection.packet_queue.push_incoming(IncomingPacket { payload });
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
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    // Break out of the loop
                    break
                },

                // I/O error reported by the system
                Err(error) => {
                    on_recv_failure(&mut endpoint, error);
                    return;
                }
            }
        }

        // Record relevant information
        if !span.is_disabled() {
            span.record("count", pkts_received);
            span.record("bytes", bytes_received);
        }
    });
}

fn on_recv_failure(endpoint: &mut Endpoint, error: io::Error) {
    match error.kind() {
        _ => {
            // Log this error
            let address = endpoint.address();
            tracing::error!("Socket {address} failed to send packet: {error}");

            // Close the endpoint
            endpoint.state = EndpointState::Closed;
        }
    }
}