use bevy_ecs::prelude::*;
use crate::{Connection, Endpoint};

pub(crate) fn io_receiving_system(
    mut endpoints: Query<&mut Endpoint>,
    mut connections: Query<&mut Connection>,
) {
    // Iterate all endpoints
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        todo!()
    });
}