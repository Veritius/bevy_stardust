use std::{io::ErrorKind, sync::Mutex};
use bevy::{prelude::*, utils::HashMap};
use quiche::RecvInfo;
use crate::{Connection, Endpoint};

pub(super) fn endpoints_receive_datagrams_system(
    mut endpoints: Query<(Entity, &mut Endpoint)>,
    connections: Query<&mut Connection>,
    mut commands: Commands,
) {
    // Storage for peers that have connected
    let new_peers = Mutex::new(HashMap::new());

    // Iterate over all endpoints in parallel
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        // Logging stuff
        let span = trace_span!("Receiving packets on endpoint", endpoint=?endpoint_id, address=?endpoint.local_addr());
        let _entered = span.enter();
        let mut receives: usize = 0;

        // Create a new iterator and fill it with zeros
        let mut scratch = Vec::with_capacity(endpoint.recv_size);
        scratch.extend((0..endpoint.recv_size).into_iter().map(|_| 0));

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
                    match endpoint.addr_to_ent(remote_address) {
                        // Peer exists
                        Some(entity) => {
                            // SAFETY: Only this endpoint should ever access the connection
                            let mut connection = unsafe { connections.get_unchecked(entity) }.unwrap();

                            // Perform the recv with their connection
                            if let Err(err) = connection.quiche.recv(recv_data, recv_info) {
                                todo!()
                            }
                        },

                        // Peer doesn't exist
                        None => {
                            // Lock new_peers for mutable access
                            let mut new_peers = new_peers.lock().unwrap();

                            // Get or create the connection
                            let connection = match new_peers.get_mut(&remote_address) {
                                Some((_, connection)) => connection,
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
                                    new_peers.insert(remote_address, (endpoint_id, connection));
                                    &mut new_peers.get_mut(&remote_address).unwrap().1
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
    });
}