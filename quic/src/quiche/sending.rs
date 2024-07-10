use bevy::prelude::*;
use crate::{Connection, Endpoint};

pub(super) fn endpoints_transmit_datagrams_system(
    mut endpoints: Query<(Entity, &mut Endpoint)>,
    connections: Query<&mut Connection>,
    commands: ParallelCommands,
) {
    // Iterate over all endpoints in parallel
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        todo!()
    });
}