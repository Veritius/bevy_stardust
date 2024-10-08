use std::collections::BTreeMap;

use bevy_ecs::{prelude::*, query::{QueryData, QueryFilter}, system::SystemParam};
use crate::{Connection, Endpoint};

#[derive(SystemParam)]
pub(crate) struct ParEndpoints<
    'w, 's, // lifetimes for bevy
    Ea: 'static + QueryData = (), 
    Ca: 'static + QueryData = (),
    Ef: 'static + QueryFilter = (),
    Cf: 'static + QueryFilter = (),
> {
    endpoints: Query<'w, 's, (&'static mut Endpoint, Ea), (Without<Connection>, Ef)>,
    connections: Query<'w, 's, (&'static mut Connection, Ca), (Without<Endpoint>, Cf)>,
}

impl<'w, 's, Ea, Ca, Ef, Cf> ParEndpoints<'w, 's, Ea, Ca, Ef, Cf>
where
    Ea: 'static + QueryData, 
    Ca: 'static + QueryData,
    Ef: 'static + QueryFilter,
    Cf: 'static + QueryFilter,
{
    
}

struct ParEndpointAccess<'a, Ea: QueryData> {
    pub endpoint: &'a mut Endpoint,
    pub additional: Ea::Item<'a>,
}

struct ParConnections<'a, Ca: QueryData> {
    connections: BTreeMap<Entity, ParConnectionAccess<'a, Ca>>,
}

struct ParConnectionAccess<'a, Ca: QueryData> {
    pub connection: &'a mut Connection,
    pub additional: Ca::Item<'a>,
}