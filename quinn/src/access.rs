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

pub(crate) struct ParEndpointAccess<'a, Ea: QueryData> {
    pub endpoint: &'a mut EndpointInner,
    pub additional: Ea::Item<'a>,
}

impl<'w, 's, Ea, Ca, Ef, Cf> ParEndpoints<'w, 's, Ea, Ca, Ef, Cf>
where
    Ea: 'static + QueryData, 
    Ca: 'static + QueryData,
    Ef: 'static + QueryFilter,
    Cf: 'static + QueryFilter,
{
    pub fn iter(
        &mut self,
        func: impl Fn(
            ParEndpointAccess<Ea>,
            ParConnections<Ca, Cf>,
        ) + Send + Sync,
    ) {
        self.endpoints.par_iter_mut().for_each(|(mut endpoint, additional)| {
            let (endpoint, connections) = endpoint.into_inner().split_access();

            let endpoint_access = ParEndpointAccess::<Ea> {
                endpoint,
                additional,
            };

            let connection_access = ParConnections {
                connections,
                query: &self.connections,
            };

            // Run the access function
            func(endpoint_access, connection_access);
        });
    }
}

pub(crate) struct ParConnections<'a, Ca: QueryData, Cf: QueryFilter> {
    connections: &'a EndpointConnections,
    query: &'a Query<'a, 'a, (&'static mut Connection, Ca), (Without<Endpoint>, Cf)>,
}

pub(crate) struct ParConnectionAccess<'a, Ca: QueryData> {
    pub connection: &'a mut ConnectionInner,
    pub additional: Ca::Item<'a>,
}

impl<'a, Ca: QueryData, Cf: QueryFilter> ParConnections<'a, Ca, Cf> {
    pub fn iter(&mut self) -> ParConnectionIter<Ca, Cf> {
        ParConnectionIter {
            connections: self.connections.into_iter(),
            query: self.query,
        }
    }

    pub fn get(
        &mut self,
        id: Entity,
        func: impl FnOnce(ParConnectionAccess<'_, Ca>),
    ) -> Result<(), ConnectionAccessError> {
        if self.connections.get_handle(id).is_none() {
            return Err(ConnectionAccessError::NotOwned);
        }

        let (mut connection, additional) = match unsafe { self.query.get_unchecked(id) } {
            Ok(res) => res,

            Err(_) => return Err(ConnectionAccessError::NoSuchEntity),
        };

        let access = ParConnectionAccess {
            connection: &mut (*connection).0,
            additional,
        };

        func(access);

        return Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectionAccessError {
    NoSuchEntity,
    NotOwned,
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