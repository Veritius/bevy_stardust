use bevy::{ecs::query::QueryData, prelude::*};
use bevy_stardust::{connections::PeerMessages, messages::Incoming};
use crate::{backend::QuicBackend, connection::Connection, ConnectionShared, EndpointShared};
use super::{scoping::{ScopedAccess, ScopedId}, Endpoint};

#[derive(QueryData)]
#[query_data(mutable)]
struct EndpointData<'w, Backend: QuicBackend> {
    shared: &'w mut EndpointShared,
    state: &'w mut Endpoint<Backend::EndpointState>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ConnectionData<'w, Backend: QuicBackend> {
    shared: &'w mut ConnectionShared,
    state: &'w mut Connection<Backend::ConnectionState>,
    messages: &'w mut PeerMessages<Incoming>,
}

fn quic_receiving_system<Backend: QuicBackend>(
    mut endpoints: Query<EndpointData<Backend>>,
    connections: Query<ConnectionData<Backend>>
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