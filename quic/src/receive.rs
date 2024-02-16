use std::{collections::HashMap, io::ErrorKind, time::Instant};
use bevy_ecs::prelude::*;
use bytes::BytesMut;
use crate::{connections::ConnectionHandleMap, QuicConnection, QuicEndpoint};

pub(super) fn quic_receive_packets_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    handle_map: Res<ConnectionHandleMap>,
    connections: Query<&QuicConnection>,
    commands: ParallelCommands,
) {
    // Receive as many packets as we can
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        let mut pending_local: HashMap<quinn_proto::ConnectionHandle, quinn_proto::Connection> = HashMap::default();
        let mut scratch = [0u8; 1472]; // TODO: make this configurable

        loop {
            match endpoint.udp_socket.recv_from(&mut scratch) {
                // Packet received, forward it to the endpoint
                Ok((bytes, address)) => {
                    tracing::trace!("Received a packet of length {bytes} from {address}");
                    if let Some((handle, event)) = endpoint.inner.get_mut().handle(
                        Instant::now(),
                        address,
                        None,
                        None,
                        BytesMut::from(&scratch[..bytes]),
                    ) {
                        match event {
                            // Relay connection events
                            quinn_proto::DatagramEvent::ConnectionEvent(event) => {
                                if let Some(id) = handle_map.0.get(&handle) {
                                    // Connection exists as an entity, just push to its queue
                                    let connection = connections.get(*id).unwrap();
                                    connection.events.lock().unwrap().push(event);
                                } else {
                                    if let Some(conn) = pending_local.get_mut(&handle) {
                                        // Connection exists in thread-local storage, handle event directly
                                        crate::polling::handle_connection_event(
                                            handle,
                                            conn,
                                            endpoint.inner.get_mut(),
                                            event,
                                        )
                                    } else {
                                        // Shouldn't happen
                                        todo!();
                                    }
                                }
                            },

                            quinn_proto::DatagramEvent::NewConnection(connection) => {
                                // Add connection to thread-local storage
                                pending_local.insert(handle, connection);
                            },
                        }
                    }
                },

                // We've run out of packets to read
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    break
                },

                // Actual IO error
                Err(e) => {
                    tracing::error!("IO error while reading packets: {e}");
                    break
                }
            }
        }

        // Spawn connection entities
        commands.command_scope(|mut commands| {
            for (handle, connection) in pending_local.drain() {
                commands.spawn(QuicConnection::new(endpoint_id, handle, connection));
            }
        });
    });
}