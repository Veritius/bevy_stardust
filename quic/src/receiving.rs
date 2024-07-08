use std::io::ErrorKind;
use bevy::prelude::*;
use crate::{Connection, Endpoint};

pub(crate) fn endpoints_receive_datagrams_system(
    mut endpoints: Query<&mut Endpoint>,
    connections: Query<&mut Connection>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        // Create a new iterator and fill it with zeros
        let mut scratch = Vec::with_capacity(endpoint.recv_size);
        scratch.extend((0..endpoint.recv_size).into_iter().map(|_| 0));

        // Some information about the endpoint we will use frequently
        let local_addr = endpoint.socket().local_addr().unwrap();

        loop {
            match endpoint.socket().recv_from(&mut scratch) {
                // Successful receive on the socket
                Ok((length, address)) => match endpoint.addr_to_ent(address) {
                    Some(entity) => {
                        // SAFETY: Only this endpoint should ever access the connection
                        let mut connection = unsafe { connections.get_unchecked(entity) }.unwrap();
                        if let Err(err) = connection.recv(&mut scratch[..length], address, local_addr) {
                            todo!()
                        }
                    },

                    None => todo!(),
                },

                // If the operation would block, it means there are no further packets to read
                Err(err) if err.kind() == ErrorKind::WouldBlock => { break },

                // An actual I/O error occurred
                Err(err) => todo!(),
            }
        }
    });
}