use quinn_proto::{Endpoint, Connection, ConnectionHandle, ConnectionEvent, EndpointEvent};

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