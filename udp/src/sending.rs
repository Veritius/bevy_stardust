use std::{collections::HashMap, net::{SocketAddr, UdpSocket}};
use bevy_ecs::prelude::*;
use bytes::Bytes;
use crate::{endpoint::ConnectionOwnershipToken, Connection, Endpoint, EndpointStatistics};

// Sends packets to UDP sockets
pub(crate) fn io_sending_system(
    mut endpoints: Query<(Entity, &mut Endpoint)>,
    connections: Query<&mut Connection>,
) {
    // Iterate all endpoints
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
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
            let mut connection = unsafe { match connections.get_unchecked(token.inner()) {
                Ok(val) => val,
                Err(_) => { continue; },
            } };
        
            // Check if there's anything to send
            if connection.packet_queue.outgoing().len() == 0 { continue }

            // Send all packets queued in this peer
            while let Some(packet) = connection.packet_queue.pop_outgoing() {
                pkts_sent += 1; bytes_sent += packet.payload.len() as u64;
                match socket.send_to(&packet.payload, connection.remote_address()) {
                    Ok(_) => {
                        // Set last_sent in timings
                        connection.timings.set_last_sent_now();

                        // Add to statistics counters
                        endpoint_statistics.track_send_packet(packet.payload.len());
                        connection.statistics.track_send_packet(packet.messages as usize);
                    },

                    Err(_) => todo!(),
                }
            }
        }

        // Send all packets that have just been queued on the endpoint
        while let Some((address, payload)) = outgoing_pkts.pop() {
            pkts_sent += 1; bytes_sent += payload.len() as u64;
            match socket.send_to(&payload, address) {
                Ok(_) => {
                    // Add to statistics tracker
                    endpoint_statistics.track_send_packet(payload.len());
                },

                Err(_) => todo!(),
            }
        }

        // Record relevant information
        if !span.is_disabled() {
            span.record("count", pkts_sent);
            span.record("bytes", bytes_sent);
        }
    });
}