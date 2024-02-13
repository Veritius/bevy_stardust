use std::{sync::Mutex, time::{Duration, Instant}};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use quinn_proto::{Connection, ConnectionEvent};
use crate::{QuicConnection, QuicEndpoint};

pub(super) fn event_exchange_polling_system(
    mut connections: Query<&mut QuicConnection>,
    mut endpoints: Query<&mut QuicEndpoint>,
) {
    // This can't be in parallel because we access the endpoints query
    for mut connection_comp in connections.iter_mut() {
        let connection_handle = connection_comp.handle.clone();
        let target_endpoint = connection_comp.endpoint.clone();
        let connection = connection_comp.inner.get_mut();

        // Handle timeouts
        if let Some(timeout) = connection.poll_timeout() {
            if timeout.duration_since(Instant::now()) == Duration::ZERO {
                connection.handle_timeout(Instant::now());
            }
        }

        // Handle endpoint events and subsequent connection events
        let mut endpoint = endpoints.get_mut(target_endpoint).unwrap();
        while let Some(event) = connection.poll_endpoint_events() {
            if let Some(event) = endpoint.inner.get_mut().handle_event(connection_handle, event) {
                connection.handle_event(event);
            }
        }

        // We have to do this to access the connection mutably and connection immutably simultaneously
        fn get_lock_and_connection(
            connection_comp: &mut QuicConnection
        ) -> (&mut Connection, &Mutex<Vec<ConnectionEvent>>) {
            (connection_comp.inner.get_mut(), &connection_comp.events)
        }

        // Handle connection events stored in the component's queue
        let (connection, mutex) = get_lock_and_connection(&mut connection_comp);
        let mut queue_lock = mutex.lock().unwrap();
        for event in queue_lock.drain(..) {
            connection.handle_event(event);
        }
    }
}

pub(super) fn connection_events_polling_system(
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