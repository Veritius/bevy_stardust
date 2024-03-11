use bevy_ecs::prelude::*;
use crate::{Connection, Endpoint};

pub(crate) fn io_receiving_system(
    mut endpoints: Query<&mut Endpoint>,
    mut connections: Query<&mut Connection>,
) {
    // Iterate all endpoints
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        for connection in &endpoint.connections {
            // SAFETY: This is safe because ConnectionOwnershipToken ensures that only one endpoint 'owns' a connection.
            let mut connection = unsafe { connections.get_unchecked(connection.inner()).unwrap() };

            todo!()
        }
    });
}