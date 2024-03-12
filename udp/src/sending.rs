use std::{collections::HashMap, net::{SocketAddr, UdpSocket}};
use bevy_ecs::prelude::*;
use crate::{endpoint::ConnectionOwnershipToken, Connection, Endpoint, EndpointStatistics};

// Sends packets to UDP sockets
pub(crate) fn io_sending_system(
    mut endpoints: Query<&mut Endpoint>,
    connections: Query<&mut Connection>,
) {
    // Iterate all endpoints
    endpoints.par_iter_mut().for_each(|mut endpoint| {

        // Split borrow fn to help out the borrow checker
        #[inline]
        fn split_borrow(endpoint: &mut Endpoint) -> (
            &UdpSocket,
            &HashMap<SocketAddr, ConnectionOwnershipToken>,
            &mut EndpointStatistics,
        ) {(
            &endpoint.socket,
            &endpoint.connections,
            &mut endpoint.statistics,
        )}

        let (
            socket,
            connection_map,
            endpoint_statistics
        ) = split_borrow(&mut endpoint);

        for (_, token) in connection_map {
            // SAFETY: This is safe because ConnectionOwnershipToken ensures that only one endpoint 'owns' a connection.
            let mut connection = unsafe { connections.get_unchecked(token.inner()).unwrap() };

            // Send all packets queued in this peer
            while let Some(packet) = connection.packet_queue.pop_outgoing() {
                match socket.send_to(&packet.payload, connection.remote_address()) {
                    Ok(_) => {
                        // Add to statistics counters
                        endpoint_statistics.track_send_packet(packet.payload.len());
                        connection.statistics.track_send_packet(packet.messages as usize);
                    },
                    Err(_) => todo!(),
                }
            }
        }
    });
}