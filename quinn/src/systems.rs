use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::SendContext;
use crate::{access::*, connection::ConnectionEvent, Connection};

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
    mut parallel_iterator: ParEndpoints
) {
    parallel_iterator.iter(|
        mut endpoint_access,
        mut connections,
    | {
        for connection_access in connections.iter() {
            while let Some(event) = connection_access.connection.poll_connection_events() {
                match event {
                    ConnectionEvent::Disconnected { reason } => {
                        todo!()
                    },
                }
            }
        }
    });
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