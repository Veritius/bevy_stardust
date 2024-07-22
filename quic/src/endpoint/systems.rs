use bevy::ecs::query::QueryData;
use crate::{backend::QuicBackend, connection::ConnectionStateData, Connection, Endpoint};
use super::EndpointStateData;

#[derive(QueryData)]
#[query_data(mutable)]
pub(in crate::endpoint) struct EndpointData<'w, Backend: QuicBackend> {
    pub shared: &'w mut Endpoint,
    pub state: &'w mut EndpointStateData<Backend::EndpointState>,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(in crate::endpoint) struct ConnectionData<'w, Backend: QuicBackend, Additional: QueryData> {
    pub shared: &'w mut Connection,
    pub state: &'w mut ConnectionStateData<Backend::ConnectionState>,
    pub additional: Additional,
}