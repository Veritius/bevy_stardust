use bevy_ecs::{prelude::*, query::{QueryData, QueryFilter}, system::SystemParam};
use crate::{Connection, Endpoint};

#[derive(SystemParam)]
pub(crate) struct InterParallelIterator<
    'w, 's, // lifetimes for bevy
    Ea: 'static + QueryData = (), 
    Ca: 'static + QueryData = (),
    Ef: 'static + QueryFilter = (),
    Cf: 'static + QueryFilter = (),
> {
    endpoints: Query<'w, 's, (&'static mut Endpoint, Ea), (Without<Connection>, Ef)>,
    connections: Query<'w, 's, (&'static mut Connection, Ca), (Without<Endpoint>, Cf)>,
}

pub(crate) fn event_exchange_system(
    parallel_iterator: InterParallelIterator,
) {
    
}