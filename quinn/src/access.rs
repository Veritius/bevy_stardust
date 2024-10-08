use bevy_ecs::{prelude::*, query::{QueryData, QueryFilter}, system::SystemParam};
use crate::{connection::ConnectionInner, endpoint::{EndpointConnections, EndpointConnectionsIter, EndpointInner}, Connection, Endpoint};

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

pub(crate) struct ParConnections<'a, Ca: QueryData> {
    connections: &'a EndpointConnections,
    query: &'a mut Query<'a, 'a, (&'static mut Connection, Ca)>,
}

pub(crate) struct ParEndpointAccess<'a, Ea: QueryData> {
    pub endpoint: &'a mut EndpointInner,
    pub additional: Ea::Item<'a>,
}

pub(crate) struct ParConnectionAccess<'a, Ca: QueryData> {
    pub connection: &'a mut ConnectionInner,
    pub additional: Ca::Item<'a>,
}

impl<'w, 's, Ea, Ca, Ef, Cf> ParEndpoints<'w, 's, Ea, Ca, Ef, Cf>
where
    Ea: 'static + QueryData, 
    Ca: 'static + QueryData,
    Ef: 'static + QueryFilter,
    Cf: 'static + QueryFilter,
{
    pub fn par_iter_all(
        &mut self,
        func: impl Fn(
            ParEndpointAccess<Ea>,
            ParConnectionIter<Ca, Cf>,
        ) + Send + Sync,
    ) {
        self.endpoints.par_iter_mut().for_each(|(mut endpoint, additional)| {
            let (endpoint, connections) = endpoint.into_inner().split_access();

            let endpoint_access = ParEndpointAccess::<Ea> {
                endpoint,
                additional,
            };

            let connection_iter = ParConnectionIter {
                connections: connections.into_iter(),
                query: &self.connections,
            };

            // Run the access function
            func(endpoint_access, connection_iter);
        });
    }
}

pub(crate) struct ParConnectionIter<'a, Ca: QueryData, Cf: QueryFilter> {
    connections: EndpointConnectionsIter<'a>,
    query: &'a Query<'a, 'a, (&'static mut Connection, Ca), (Without<Endpoint>, Cf)>,
}

impl<'a, Ca: QueryData, Cf: QueryFilter> Iterator for ParConnectionIter<'a, Ca, Cf> {
    type Item = ParConnectionAccess<'a, Ca>;

    fn next(&mut self) -> Option<Self::Item> {
        let (entity, handle) = self.connections.next()?;

        // Only panics if the endpoint-connection relation state is invalid, which is UB anyway so thumbs up
        // SAFETY: The access is exclusive because a connection can only be 'owned' by one endpoint at a time, and we only access owned connections here
        let (connection, additional) = unsafe { self.query.get_unchecked(entity) }.unwrap();

        return Some(ParConnectionAccess {
            connection: &mut connection.into_inner().0,
            additional,
        });
    }
}