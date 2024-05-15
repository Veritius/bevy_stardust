use std::{io, time::Instant};
use bevy::prelude::*;
use bevy_stardust::connections::NetworkPerformanceReduction;
use crate::prelude::*;

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

        let endpoint = &mut *endpoint;
        let socket = &endpoint.udp_socket;
        let connection_map = &endpoint.connections;
        let outgoing_pkts = &mut endpoint.outgoing_pkts;
        let endpoint_statistics = &mut endpoint.statistics;

        // Send all packets that individual connections have queued
        for (_, token) in connection_map {
            // SAFETY: This is safe because ConnectionOwnershipToken ensures that only one endpoint 'owns' a connection.
            let (mut connection, performance) = unsafe { match connections.get_unchecked(token.inner()) {
                Ok(val) => val,
                Err(_) => { continue; },
            } };

            // Basically a dereference
            let connection = connection.inner_mut();
        
            // Check if there's anything to send
            if !connection.shared.any_send() { continue }

            // Send all packets queued in this peer
            while let Some(payload) = connection.shared.pop_send() {
                pkts_sent += 1; bytes_sent += payload.len() as u64;

                // Randomly skip actually sending the packet
                if let Some(performance) = performance {
                    let roll = rng.f32();
                    if performance.packet_drop_chance < roll {
                        // Updating these values simulates a successful packet send
                        // Importantly, we don't actually send anything
                        endpoint_statistics.record_packet_send(payload.len());
                        continue;
                    }
                }

                // Send the packet.
                match socket.send_to(&payload, connection.shared.remote_address()) {
                    Ok(_) => {
                        // Update tracking information
                        endpoint_statistics.record_packet_send(payload.len());
                    },

                    Err(err) => {
                        on_send_failure(endpoint, err);
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
                    on_send_failure(endpoint, err);
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