use std::{sync::Mutex, time::{Duration, Instant}};
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use quinn_proto::{Connection, ConnectionEvent, ConnectionHandle, Endpoint, EndpointEvent};
use crate::{QuicConnection, QuicEndpoint};

pub(super) fn event_exchange_polling_system(
    mut connections: Query<&mut QuicConnection>,
    mut endpoints: Query<&mut QuicEndpoint>,
) {
    // TODO: This might be able to run in parallel too
    for mut connection_comp in connections.iter_mut() {
        let connection_handle = connection_comp.handle.clone();
        let target_endpoint = connection_comp.endpoint.clone();
        let connection = connection_comp.inner.get_mut();

        // Handle timeouts
        if let Some(timeout) = connection.poll_timeout() {
            if Instant::now().saturating_duration_since(timeout) > Duration::ZERO {
                connection.handle_timeout(Instant::now());
            }
        }

        /*
            TODO: See if a limit needs to be imposed on the amount of ping-ponging back and forth that occurs
            The quinn-proto documentation doesn't say anything about if constant blocking exchange will ever halt
            There could potentially be a situation where this process enters an infinite loop, which is obviously bad
            Though this probably isn't a real concern since if that were the case it'd happen in the async Quinn too
        */

        fn access_dual(comp: &mut QuicConnection) -> (&mut Connection, &Mutex<Vec<ConnectionEvent>>) {
            (comp.inner.get_mut(), &comp.events)
        }

        let (connection, queue) = access_dual(&mut connection_comp);
        let mut endpoint = endpoints.get_mut(target_endpoint).unwrap();
        let endpoint = endpoint.inner.get_mut();

        // Iterate over all queued messages
        let mut queue_lock = queue.lock().unwrap();
        for event in queue_lock.drain(..) {
            handle_connection_event(connection_handle, connection, endpoint, event);
        }

        // Connection inners might want to talk
        while let Some(event) = connection.poll_endpoint_events() {
            handle_endpoint_event(connection_handle, connection, endpoint, event);
        }
    }
}

fn handle_endpoint_event(
    connection_handle: ConnectionHandle,
    connection: &mut Connection,
    endpoint: &mut Endpoint,
    event: EndpointEvent,
) {
    if let Some(event) = endpoint.handle_event(connection_handle, event) {
        handle_connection_event(connection_handle, connection, endpoint, event);
    }
}

fn handle_connection_event(
    connection_handle: ConnectionHandle,
    connection: &mut Connection,
    endpoint: &mut Endpoint,
    event: ConnectionEvent,
) {
    connection.handle_event(event);
    while let Some(event) = connection.poll_endpoint_events() {
        handle_endpoint_event(connection_handle, connection, endpoint, event);
    }
}

/// Gets application-facing Event objects from connections
pub(super) fn application_events_polling_system(
    mut connections: Query<(Entity, &NetworkPeer, &mut QuicConnection)>,
    mut disconnect_events: EventWriter<PeerDisconnectedEvent>,
    commands: ParallelCommands,
) {
    // Put the event writer in a mutex so it can be accessed in parallel
    // Disconnection events are very infrequent so this is fine to do
    let events = Mutex::new(&mut disconnect_events);

    connections.par_iter_mut().for_each(|(entity_id, peer_data, mut connection_comp)| {
        let disconnect_logged = connection_comp.disconnect_logged;

        // Poll events from inner Quinn connection
        let connection = connection_comp.inner.get_mut();
        while let Some(event) = connection.poll() {
            match event {
                quinn_proto::Event::ConnectionLost { reason } => {
                    // Queue the entity's deletion
                    commands.command_scope(|mut commands| {
                        commands.entity(entity_id).despawn();
                    });

                    // Check if we need to do anything here
                    if disconnect_logged { continue }

                    // Send the disconnection event to alert game systems
                    events.lock().unwrap().send(PeerDisconnectedEvent {
                        entity_id,
                        uuid: peer_data.uuid,
                        reason: format!("{reason}").into(),
                    });
                },
                quinn_proto::Event::Stream(_) => todo!(),
                quinn_proto::Event::DatagramReceived => todo!(),
                _ => {},
            }
        }
    });
}