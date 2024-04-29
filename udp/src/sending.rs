use std::{collections::HashMap, io, net::{SocketAddr, UdpSocket}, time::Instant};
use bevy::prelude::*;
use bevy_stardust::connections::NetworkPerformanceReduction;
use bytes::Bytes;
use crate::{endpoint::ConnectionOwnershipToken, prelude::*};

// Sends packets to UDP sockets
pub(crate) fn io_sending_system(
    mut endpoints: Query<(Entity, &mut Endpoint)>,
    connections: Query<(&mut Connection, Option<&NetworkPerformanceReduction>)>,
) {
    // Iterate all endpoints
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        // Get an instance of RNG
        let mut rng = fastrand::Rng::default();

        // Stuff for logging
        let mut pkts_sent: u32 = 0;
        let mut bytes_sent: u64 = 0;
        let span = tracing::trace_span!("Sending packets on endpoint", id=?endpoint_id);
        let _entered_span = span.enter();

        // Split borrow fn to help out the borrow checker
        #[inline]
        fn split_borrow(endpoint: &mut Endpoint) -> (
            &UdpSocket,
            &HashMap<SocketAddr, ConnectionOwnershipToken>,
            &mut Vec<(SocketAddr, Bytes)>,
            &mut EndpointStatistics,
        ) {(
            &endpoint.udp_socket,
            &endpoint.connections,
            &mut endpoint.outgoing_pkts,
            &mut endpoint.statistics,
        )}

        let (
            socket,
            connection_map,
            outgoing_pkts,
            endpoint_statistics,
        ) = split_borrow(&mut endpoint);

        // Send all packets that individual connections have queued
        for (_, token) in connection_map {
            // SAFETY: This is safe because ConnectionOwnershipToken ensures that only one endpoint 'owns' a connection.
            let (mut connection, performance) = unsafe { match connections.get_unchecked(token.inner()) {
                Ok(val) => val,
                Err(_) => { continue; },
            } };
        
            // Check if there's anything to send
            // if connection.packet_queue.outgoing().len() == 0 { continue }

            // Send all packets queued in this peer
            while let Some(packet) = connection.send_packets.pop_front() {
                pkts_sent += 1; bytes_sent += packet.payload.len() as u64;

                // Randomly skip actually sending the packet
                if let Some(performance) = performance {
                    let roll = rng.f32();
                    if performance.packet_drop_chance < roll {
                        // Updating these values simulates a successful packet send
                        connection.last_send = Some(Instant::now());
                        endpoint_statistics.record_packet_send(packet.payload.len());
                        connection.statistics.record_packet_send(packet.messages as usize);

                        continue;
                    }
                }

                // Send the packet.
                match socket.send_to(&packet.payload, connection.remote_address()) {
                    Ok(_) => {
                        // Set last_sent in timings
                        connection.last_send = Some(Instant::now());

                        // Add to statistics counters
                        endpoint_statistics.record_packet_send(packet.payload.len());
                        connection.statistics.record_packet_send(packet.messages as usize);
                    },

                    Err(err) => {
                        on_send_failure(&mut endpoint, err);
                        return;
                    },
                }
            }
        }

        // Send all packets that have just been queued on the endpoint
        while let Some((address, payload)) = outgoing_pkts.pop() {
            pkts_sent += 1; bytes_sent += payload.len() as u64;
            match socket.send_to(&payload, address) {
                Ok(_) => {
                    // Add to statistics tracker
                    endpoint_statistics.record_packet_send(payload.len());
                },

                Err(err) => {
                    on_send_failure(&mut endpoint, err);
                    return;
                },
            }
        }

        // Record relevant information
        if !span.is_disabled() {
            span.record("count", pkts_sent);
            span.record("bytes", bytes_sent);
        }
    });
}

fn on_send_failure(endpoint: &mut Endpoint, error: io::Error) {
    match error.kind() {
        io::ErrorKind::WouldBlock => {},
        _ => {
            // Log this error
            let address = endpoint.address();
            tracing::error!("Socket {address} failed to send packet: {error}");

            // Close the endpoint
            endpoint.state = EndpointState::Closed;
        }
    }
}