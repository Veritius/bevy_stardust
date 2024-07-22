use std::io::ErrorKind;
use bevy::{prelude::*, utils::HashMap};
use quiche::RecvInfo;
use crate::{connection::{datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams}, streams::{ChannelStreams, IncomingStreams, OutgoingStreams}, Connection}, quiche::QuicheConnection, Endpoint};

pub(super) fn endpoints_receive_datagrams_system(
    mut endpoints: Query<(Entity, &mut Endpoint)>,
    connections: Query<&mut Connection>,
    commands: ParallelCommands,
) {
    // Iterate over all endpoints in parallel
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        // Storage for peers that have connected
        let mut new_peers = HashMap::new();

        // Logging stuff
        let span = trace_span!("Receiving packets on endpoint", endpoint=?endpoint_id, address=?endpoint.local_addr());
        let entered = span.enter();
        let mut receives: usize = 0;

        // Create a new iterator and fill it with zeros
        let mut scratch = Vec::with_capacity(endpoint.recv_size);
        scratch.extend((0..endpoint.recv_size).into_iter().map(|_| 0));
        debug_assert_eq!(endpoint.recv_size, scratch.len());

        // Some information about the endpoint we will use frequently
        let local_addr = endpoint.socket().local_addr().unwrap();

        'recvloop: loop {
            match endpoint.socket().recv_from(&mut scratch) {
                // Successful receive on the socket
                Ok((length, remote_address)) => {
                    // More logging stuff
                    receives += 1;

                    // Stuff that quiche uses for packet receives
                    let recv_data = &mut scratch[..length];
                    let recv_info = RecvInfo { from: remote_address, to: local_addr };

                    // See if the peer exists already
                    match endpoint.connections.get_entity(remote_address) {
                        // Peer exists
                        Some(entity) => {
                            // SAFETY: Only this endpoint should ever access the connection
                            let mut connection = unsafe { connections.get_unchecked(entity) }.unwrap();

                            // Perform the recv with their connection
                            // if let Err(err) = connection.quiche.recv(recv_data, recv_info) {
                            //     todo!()
                            // }
                        },

                        // Peer doesn't exist
                        None => {
                            // Get or create the connection
                            let connection = match new_peers.get_mut(&remote_address) {
                                Some(connection) => connection,
                                None => {
                                    // Try to accept the connection
                                    let connection = match quiche::accept(
                                        &super::issue_connection_id(),
                                        None,
                                        local_addr,
                                        remote_address,
                                        &mut endpoint.quiche_config,
                                    ) {
                                        Ok(connection) => connection,

                                        // error case
                                        Err(err) => {
                                            error!("Accept failed: {err}");
                                            continue 'recvloop;
                                        },
                                    };

                                    // Add it to the map and return a mutable reference
                                    new_peers.insert(remote_address, Box::new(connection));
                                    &mut (*new_peers.get_mut(&remote_address).unwrap())
                                }
                            };

                            // Perform the recv on their connection
                            if let Err(err) = connection.recv(recv_data, recv_info) {
                                todo!()
                            }
                        },
                    }
                },

                // If the operation would block, it means there are no further packets to read
                Err(err) if err.kind() == ErrorKind::WouldBlock => { break },

                // An actual I/O error occurred
                Err(err) => todo!(),
            }
        }

        // Record statistics to the span
        span.record("receives", receives);
        drop(entered); // explicit drop to end the span

        if new_peers.len() != 0 {
            // Start a new span for tracking accepts
            let span = trace_span!("Adding new peers", endpoint=?endpoint_id);
            let _entered = span.enter();

            // Command scope to defer world mutations
            commands.command_scope(|mut commands| {
                // Spawn the new connections
                for (address, connection) in new_peers.drain() {
                    // Spawn a new entity to represent the connection
                    let mut commands = commands.spawn_empty();

                    // Register the connection to this endpoint
                    // SAFETY: Commands gives a unique ID when adding entities
                    let id = commands.id();
                    unsafe { endpoint.connections.register(id, address); }

                    // Queue spawning the entity into the world
                    commands.insert(todo!());

                    // Log the addition for debugging
                    debug!(?address, "Accepted new peer {id}");
                }
            });
        }
    });
}