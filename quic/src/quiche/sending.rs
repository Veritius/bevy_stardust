use bevy::prelude::*;
use crate::{Connection, Endpoint};

pub(super) fn endpoints_transmit_datagrams_system(
    mut endpoints: Query<(Entity, &mut Endpoint)>,
    connections: Query<&mut Connection>,
) {
    // Iterate over all endpoints in parallel
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        // Logging stuff
        let span = trace_span!("Transmitting packets on endpoint", endpoint=?endpoint_id, address=?endpoint.local_addr());
        let entered = span.enter();
        let mut transmits: usize = 0;

        // Create a new iterator and fill it with zeros
        let mut scratch = Vec::with_capacity(endpoint.send_size);
        scratch.extend((0..endpoint.send_size).into_iter().map(|_| 0));
        debug_assert_eq!(endpoint.send_size, scratch.len());

        // Iterate over all associated entities
        for connection in endpoint.iterate_connections_owned() {
            // SAFETY: Only one Endpoint will ever try to access the connection
            let mut connection = match unsafe { connections.get_unchecked(connection) } {
                Ok(connection) => connection,
                Err(_) => {
                    // Disassociate the connection from the endpoint
                    endpoint.remove_connection(connection);
                    trace!(endpoint=?endpoint_id, ?connection, "Endpoint had an entity ID associated with it that didn't appear in a query");
                    continue;
                },
            };

            // If this returns true, quiche::Connection::send will always return Done
            // We check this here to save ourselves some effort
            if connection.quiche.is_draining() { continue }

            'send: loop {
                match connection.quiche.send(&mut scratch[..]) {
                    // The connection wants to send data
                    Ok((written, send_info)) => {
                        // This shouldn't trip but it's worth checking
                        debug_assert_eq!(send_info.from, endpoint.local_addr());

                        // TODO: Handle pacing (the at field in send_info)

                        // Send the data with the socket
                        if let Err(err) = endpoint.socket().send_to(&scratch[..written], send_info.to) {
                            error!("I/O error while sending packets: {err}");
                            todo!()
                        }
                    },

                    // Nothing more to send
                    Err(quiche::Error::Done) => break 'send,

                    // Actual error
                    Err(err) => todo!(),
                }
            }
        }
    });
}