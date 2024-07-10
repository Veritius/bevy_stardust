use std::{io::ErrorKind, sync::Mutex};
use bevy::{prelude::*, utils::HashMap};
use bytes::Bytes;
use quiche::RecvInfo;
use crate::{Connection, Endpoint};

pub(super) fn endpoints_receive_datagrams_system(
    mut endpoints: Query<(Entity, &mut Endpoint)>,
    connections: Query<&mut Connection>,
) {
    // Storage for peers that have connected
    let new_peers = Mutex::new(HashMap::new());

    // Iterate over all endpoints in parallel
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        // Logging stuff
        let span = trace_span!("Receiving packets on endpoint", endpoint=?endpoint_id, address=?endpoint.local_addr());
        let _entered = span.enter();
        let mut receives = 0;

        // Create a new iterator and fill it with zeros
        let mut scratch = Vec::with_capacity(endpoint.recv_size);
        scratch.extend((0..endpoint.recv_size).into_iter().map(|_| 0));

        // Some information about the endpoint we will use frequently
        let local_addr = endpoint.socket().local_addr().unwrap();

        loop {
            match endpoint.socket().recv_from(&mut scratch) {
                // Successful receive on the socket
                Ok((length, address)) => {
                    // More logging stuff
                    receives += 1;

                    // Extra information that quiche uses for packet receives
                    let recv_info = RecvInfo { from: address, to: local_addr };

                    // See if the peer exists already
                    match endpoint.addr_to_ent(address) {
                        // Peer exists
                        Some(entity) => {
                            // SAFETY: Only this endpoint should ever access the connection
                            let mut connection = unsafe { connections.get_unchecked(entity) }.unwrap();

                            // Perform the recv with their connection
                            if let Err(err) = connection.quiche.recv(&mut scratch[..length], recv_info) {
                                todo!()
                            }
                        },

                        // Peer doesn't exist
                        None => {
                            let mut lock = new_peers.lock().unwrap();
                            lock.entry(address).and_replace_entry_with(|_, _| Some(Bytes::copy_from_slice(&scratch[..length])));
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