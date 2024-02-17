use bevy_ecs::prelude::*;
use quinn_proto::{Endpoint, Connection, ConnectionHandle, ConnectionEvent, EndpointEvent};
use crate::{QuicEndpoint, QuicConnection};

pub(super) fn handle_connection_event(
    endpoint: &mut Endpoint,
    connection: &mut Connection,
    event: ConnectionEvent,
) {
    connection.handle_event(event);
}

pub(super) fn handle_connection_event_recurse(
    endpoint: &mut Endpoint,
    connection: &mut Connection,
    conn_handle: ConnectionHandle,
    event: ConnectionEvent,
) {
    handle_connection_event(endpoint, connection, event);
    while let Some(event) = connection.poll_endpoint_events() {
        handle_endpoint_event_recurse(endpoint, connection, conn_handle, event);
    }
}

pub(super) fn handle_endpoint_event_recurse(
    endpoint: &mut Endpoint,
    connection: &mut Connection,
    conn_handle: ConnectionHandle,
    event: EndpointEvent,
) {
    if let Some(event) = endpoint.handle_event(conn_handle, event) {
        handle_connection_event_recurse(endpoint, connection, conn_handle, event);
    }
}

pub(super) fn event_recursing_exchange_system(
    mut endpoints: Query<&mut QuicEndpoint>,
    mut connections: Query<&mut QuicConnection>,
) {
    for mut connection in connections.iter_mut() {
        let conn_handle = connection.handle.clone();
        let mut endpoint = endpoints.get_mut(connection.endpoint()).unwrap();
        let connection = connection.inner.get_mut();
        if let Some(event) = connection.poll_endpoint_events() {
            handle_endpoint_event_recurse(endpoint.inner.get_mut(), connection, conn_handle, event);
        }
    }
}