use bevy_ecs::prelude::*;
use crate::access::*;

pub(crate) fn event_exchange_system(
    mut parallel_iterator: ParEndpoints,
) {
    parallel_iterator.par_iter_all(|
        mut endpoint_access,
        mut connection_iterator,
    | {
        for mut connection_access in connection_iterator {

        }
    });
}