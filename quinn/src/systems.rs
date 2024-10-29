use std::{f32::consts::E, sync::Arc, time::Duration};

use bevy_app::AppExit;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::{DisconnectCode, SendContext};
use quinn_proto::ConnectionError;
use crate::{access::*, connection::ConnectionEvent, Connection, Endpoint};

pub(crate) fn event_exchange_system(
    mut parallel_iterator: ParEndpoints,
) {
    parallel_iterator.iter(|
        mut endpoint_access,
        mut connections,
    | {
        for connection_access in connections.iter() {
            while let Some(event) = connection_access.connection.poll_endpoint_events() {
                let event = endpoint_access.inner.handle_event(connection_access.connection.handle(), event);

                if let Some(event) = event {
                    connection_access.connection.handle_connection_event(event);
                }
            }
        }
    });
}

pub(crate) fn event_polling_system(
    mut commands: Commands,
    mut endpoints: Query<&mut Endpoint>,
    mut connections: Query<&mut Connection>,

    mut connect_events: EventWriter<PeerConnectedEvent>,
    mut disconnect_events: EventWriter<PeerDisconnectedEvent>,
) {
    for mut endpoint in endpoints.iter_mut() {
        let mut disconnections = Vec::new();

        let (endpoint_inner, conn_map) = endpoint.split_access();
        for (entity, handle) in conn_map {
            // If this panics it means the hierarchy is invalid, which is UB anyway
            let connection = &mut *(connections.get_mut(entity).unwrap()).0;

            while let Some(event) = connection.poll_connection_events() {
                match event {
                    ConnectionEvent::Connected => {
                        // Add peer components
                        commands.entity(entity).insert((
                            Peer::new(),
                            PeerLifestage::Established,
                        ));

                        // Send Stardust event to inform systems
                        connect_events.send(PeerConnectedEvent {
                            peer: entity
                        });

                        #[cfg(feature="log")]
                        bevy_log::debug!("Connection {entity} established");
                    }

                    ConnectionEvent::Disconnected { reason } => {
                        let (reason, comment) = match reason {
                            ConnectionError::VersionMismatch => (
                                DisconnectReason::Unspecified,
                                Some(Arc::from("unimplemented QUIC version")),
                            ),

                            ConnectionError::TransportError(error) => (
                                DisconnectReason::ProtocolViolation,
                                Some(Arc::from(error.reason)),
                            ),

                            ConnectionError::ConnectionClosed(connection_close) => (
                                DisconnectReason::Finished,
                                Some(match std::str::from_utf8(&connection_close.reason) {
                                    Ok(str) => Arc::from(str),
                                    Err(_) => Arc::from(format!("code: {}", connection_close.error_code)),
                                }),
                            ),

                            ConnectionError::ApplicationClosed(application_close) => (
                                DisconnectReason::Finished,
                                Some(match std::str::from_utf8(&application_close.reason) {
                                    Ok(str) => Arc::from(str),
                                    Err(_) => Arc::from(format!("code: {}", application_close.error_code)),
                                }),
                            ),

                            ConnectionError::Reset => (
                                DisconnectReason::Finished,
                                Some(Arc::from("connection reset")),
                            ),

                            ConnectionError::TimedOut => (
                                // TODO: figure out if after can be set to something meaningful
                                DisconnectReason::TimedOut { after: Duration::ZERO },
                                None,
                            ),

                            _ => (
                                DisconnectReason::Unspecified,
                                None,
                            ),
                        };

                        // Send Stardust event to inform systems
                        disconnect_events.send(PeerDisconnectedEvent {
                            peer: entity,
                            reason,
                            comment,
                        });

                        disconnections.push(entity);
                    },
                }
            }
        }

        for entity in disconnections.drain(..) {
            unsafe { endpoint.inform_connection_close(entity) };
            commands.entity(entity).remove::<Connection>();

            #[cfg(feature="log")]
            bevy_log::debug!("Connection {entity} closed");
        }
    }
}

pub(crate) fn poll_incoming_messages_system(
    mut query: Query<(&mut Connection, Option<&mut PeerMessages<Incoming>>)>,
) {
    query.par_iter_mut().for_each(|(mut connection, incoming)| {
        if incoming.is_none() {
            todo!()
        }

        let mut incoming = incoming.unwrap();
        while let Some(message) = connection.0.poll_messages() {
            incoming.push_one(message);
        }
    });
}

pub(crate) fn put_outgoing_messages_system(
    channels: Channels,
    mut query: Query<(&mut Connection, &mut PeerMessages<Outgoing>)>,
) {
    let send_context = SendContext {
        registry: &channels,
        dgram_max_size: 1472, // TODO
    };

    query.par_iter_mut().for_each(|(mut connection, outgoing)| {
        connection.0.handle_outgoing(send_context, &outgoing);
    });
}

pub(crate) fn disconnect_event_system(
    mut events: EventReader<DisconnectPeerEvent>,
    mut connections: Query<&mut Connection>,
) {
    for event in events.read() {
        if let Ok(mut c) = connections.get_mut(event.peer) {
            c.0.close(
                quinn_proto::VarInt::from_u32(0), // TODO
                match &event.comment {
                    Some(comment) => todo!(),
                    None => Bytes::new(),
                },
            );
        }
    }
}

pub(crate) fn application_exit_system(
    mut event: EventReader<AppExit>,

    mut connections: Query<(Entity, &mut Connection)>,
) {
    if event.is_empty() { return }
    event.clear();

    // Close all connections
    connections.iter_mut().for_each(|(peer, mut conn)| {
        // Signal the connection state machine to close
        // This makes it send a final packet informing the other side
        conn.0.close(
            quinn_proto::VarInt::default(),
            Bytes::new(),
        );
    });
}