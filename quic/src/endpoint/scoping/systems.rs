use std::ops::DerefMut;
use bevy::{prelude::*, ecs::query::QueryData};
use scoping::context::EndpointScopeContext;
use scoping::id::Connections;
use crate::connection::*;
use crate::endpoint::*;
use crate::backend::*;

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
    Task: for<'a> Fn(
        EndpointScopeContext<'a, Backend>,
    ),
>(
    backend: BackendInstance<Backend>,
    mut endpoints: Query<EndpointData<Backend>>,
    connections: Query<ConnectionData<Backend, Additional>>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        // SAFETY: This Connections set uses the endpoint's own connection set, so it should never overlap with other sets
        let connections = unsafe { Connections::new(endpoint.shared.connections.expose()) };

        // Context object for the task we're about to run
        let context = EndpointScopeContext::<Backend> {
            backend: backend.as_ref(),
            state: endpoint.state.deref_mut().inner_mut(),
            connections,
        };

        // let access = ScopedAccess { query: &connections };
    });
}