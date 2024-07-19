use bevy::ecs::query::QueryData;
use crate::{backend::QuicBackend, connection::Connection, ConnectionShared, EndpointShared};
use super::Endpoint;

#[derive(QueryData)]
#[query_data(mutable)]
pub(in crate::endpoint) struct EndpointData<'w, Backend: QuicBackend> {
    pub shared: &'w mut EndpointShared,
    pub state: &'w mut Endpoint<Backend::EndpointState>,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(in crate::endpoint) struct ConnectionData<'w, Backend: QuicBackend, Additional: QueryData> {
    pub shared: &'w mut ConnectionShared,
    pub state: &'w mut Connection<Backend::ConnectionState>,
    pub additional: Additional,
}