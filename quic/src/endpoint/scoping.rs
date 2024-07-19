use std::marker::PhantomData;
use bevy::{ecs::query::{QueryData, QueryItem, ROQueryItem}, prelude::*};
use crate::{backend::QuicBackend, connection::Connection, ConnectionShared, EndpointShared};
use super::Endpoint;

/// An ID only valid within a scope.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopedId<'a> {
    id: Entity,
    phantom: PhantomData<&'a ()>,
}

impl<'a> ScopedId<'a> {
    pub(super) unsafe fn new(id: Entity) -> Self {
        Self {
            id,
            phantom: PhantomData,
        }
    }

    pub(super) fn inner(&self) -> Entity {
        self.id
    }
}

pub struct ScopedAccess<'a, Data: QueryData> {
    query: &'a Query<'a, 'a, Data>,
}

impl<'a, Data: QueryData> ScopedAccess<'a, Data> {
    pub fn get(&'a self, id: ScopedId<'a>) -> ROQueryItem<'a, Data> {
        // TODO: Safety annotation
        unsafe { self.query.get(id.inner()).unwrap_unchecked() }
    }

    pub fn get_mut(&'a mut self, id: ScopedId<'a>) -> QueryItem<'a, Data> {
        // TODO: Safety annotation
        unsafe { self.query.get_unchecked(id.inner()).unwrap_unchecked() }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct EndpointData<'w, Backend: QuicBackend> {
    shared: &'w mut EndpointShared,
    state: &'w mut Endpoint<Backend::EndpointState>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ConnectionData<'w, Backend: QuicBackend, Additional: QueryData> {
    shared: &'w mut ConnectionShared,
    state: &'w mut Connection<Backend::ConnectionState>,
    additional: Additional,
}

fn scoped_endpoint_process_system<
    Backend: QuicBackend,
    Additional: QueryData,
    Task: Fn(
        &mut EndpointShared,
        &mut Endpoint<Backend::EndpointState>,
    ),
>(
    mut endpoints: Query<EndpointData<Backend>>,
    connections: Query<ConnectionData<Backend, Additional>>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        // TODO: Safety annotations
        let ids: Vec<ScopedId> = endpoint.shared.connections
            .iter()
            .filter(|v| connections.contains(*v))
            .map(|id| unsafe { ScopedId::new(id) })
            .collect::<Vec<_>>();

        let access = ScopedAccess { query: &connections };


    });
}