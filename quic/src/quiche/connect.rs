use bevy::prelude::*;
use crate::{datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams}, streams::{ChannelStreams, IncomingStreams, OutgoingStreams}, Connection, Endpoint, TryConnectEvent};
use super::QuicheConnection;

pub(super) fn connection_attempt_events_system(
    mut endpoints: Query<&mut Endpoint>,
    mut events: EventReader<TryConnectEvent>,
    mut commands: Commands,
) {
    // Iterate over all connection attempt events
    for event in events.read() {
        // Try to get the endpoint that will be associated with the connection
        let mut endpoint = match endpoints.get_mut(event.endpoint) {
            Ok(endpoint) => endpoint,
            Err(_) => todo!(),
        };

        // Create the inner quiche connection
        let quiche = match quiche::connect(
            event.server_name.as_deref(),
            &super::issue_connection_id(),
            endpoint.local_addr(),
            event.address,
            &mut endpoint.quiche_config,
        ) {
            Ok(connection) => QuicheConnection::new(connection),
            Err(_) => todo!(),
        };

        // Construct the connection component
        let connection = Connection {
            quiche,

            endpoint: event.endpoint,
            incoming_streams: IncomingStreams::new(),
            outgoing_streams: OutgoingStreams::new(),
            channel_streams: ChannelStreams::new(),
            incoming_datagrams: IncomingDatagrams::new(),
            outgoing_datagrams: OutgoingDatagrams::new(),
            channel_datagrams: ChannelDatagrams::new(),
        };

        // Spawn the connection entity
        let id = commands.spawn(connection).id();

        // Register it to the endpoint
        // SAFETY: Since we just spawned the entity, we know its ID is unique
        unsafe { endpoint.insert_connection(id, event.address); }
    }
}