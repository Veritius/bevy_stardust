use bevy_ecs::prelude::*;
use bevy_stardust::connections::peer::NetworkPeer;
use quinn_proto::{Endpoint, Connection, ConnectionHandle, ConnectionEvent, EndpointEvent};
use crate::{connections::ConnectionStage, QuicConnection, QuicEndpoint};

pub(super) fn handle_connection_event(
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
    handle_connection_event(connection, event);
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

pub(super) fn endpoint_connection_comm_system(
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

pub(super) fn poll_application_event_system(
    mut connections: Query<(Entity, &mut QuicConnection)>,
    mut commands: Commands,
) {
    for (entity, mut connection) in connections.iter_mut() {
        // make borrowck happy
        fn split_borrow(conn: &mut QuicConnection) -> (&mut Connection, &mut ConnectionStage) {
            (conn.inner.get_mut(), &mut conn.stage)
        }

        let (connection_inner, connection_stage) = split_borrow(&mut connection);

        while let Some(event) = connection_inner.poll() {
            match event {
                quinn_proto::Event::Connected => {
                    // The QUIC handshake is done, but we run our own checks.
                    *connection_stage = ConnectionStage::GameHandshake {
                        passed_version_check: false,

                        #[cfg(feature="hash_check")]
                        passed_hash_check: false,
                    }
                },

                quinn_proto::Event::ConnectionLost { reason } => {
                    *connection_stage = ConnectionStage::Disconnected;
                    commands.entity(entity).despawn();
                    tracing::info!("Connection {entity:?} lost connection: {reason}");
                },

                _ => {} // we don't care about the other events
            }
        }
    }
}

pub(super) fn remove_drained_connections_system(
    mut endpoints: Query<&mut QuicEndpoint>,
    mut connections: Query<(Entity, &mut QuicConnection)>,
    mut commands: Commands,
) {
    for (entity, mut connection) in connections.iter_mut() {
        if connection.inner.get_mut().is_drained() || connection.force_despawn {
            commands.entity(entity).despawn();
            endpoints.get_mut(connection.endpoint()).unwrap().connections.retain(|_, id| entity == *id);
        }
    }
}