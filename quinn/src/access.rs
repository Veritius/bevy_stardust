use std::collections::BTreeMap;

use bevy_ecs::{prelude::*, query::{QueryData, QueryFilter}, system::SystemParam};
use crate::{Connection, Endpoint};

#[derive(SystemParam)]
pub(crate) struct EndpointParAccess<
    'w, 's, // lifetimes for bevy
    Ea: 'static + QueryData = (), 
    Ca: 'static + QueryData = (),
    Ef: 'static + QueryFilter = (),
    Cf: 'static + QueryFilter = (),
> {
    endpoints: Query<'w, 's, (&'static mut Endpoint, Ea), (Without<Connection>, Ef)>,
    connections: Query<'w, 's, (&'static mut Connection, Ca), (Without<Endpoint>, Cf)>,
}

impl<'w, 's, Ea, Ca, Ef, Cf> EndpointParAccess<'w, 's, Ea, Ca, Ef, Cf>
where
    Ea: 'static + QueryData, 
    Ca: 'static + QueryData,
    Ef: 'static + QueryFilter,
    Cf: 'static + QueryFilter,
{
    
}

struct ParEndpoint<'a, Ea: QueryData> {
    pub endpoint: &'a mut Endpoint,
    pub additional: Ea::Item<'a>,
}

struct ParEndpointConnections<'a, Ca: QueryData> {
    connections: BTreeMap<Entity, (
        &'a mut Connection,
        Ca::Item<'a>,
    )>,
}