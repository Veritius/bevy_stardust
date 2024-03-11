use std::net::UdpSocket;
use bevy_ecs::prelude::*;
use crate::{endpoint::ConnectionOwnershipToken, Connection, Endpoint, EndpointStatistics};

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
            &[ConnectionOwnershipToken],
            &mut EndpointStatistics,
        ) {(
            &endpoint.socket,
            &endpoint.connections,
            &mut endpoint.statistics,
        )}

        let (
            socket,
            owned_connections,
            endpoint_statistics
        ) = split_borrow(&mut endpoint);

        for connection in owned_connections {
            // SAFETY: This is safe because ConnectionOwnershipToken ensures that only one endpoint 'owns' a connection.
            let mut connection = unsafe { connections.get_unchecked(connection.inner()).unwrap() };

            // Send all packets queued in this peer
            while let Some(packet) = connection.outgoing_packets.pop_front() {
                match socket.send_to(&packet, connection.remote_address()) {
                    Ok(_) => {
                        // Add to statistics counters
                        endpoint_statistics.track_send_packet(packet.len());
                        connection.statistics.track_send_packet(todo!());
                    },
                    Err(_) => todo!(),
                }
            }
        }
    });
}