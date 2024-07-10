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

            todo!()
        }
    });
}