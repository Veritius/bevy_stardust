use bevy::prelude::*;
use bevy_stardust::prelude::*;
use super::{Connection, ConnectionState};

pub(super) fn close_events_system(
    mut events: EventReader<DisconnectPeerEvent>,
    mut connections: Query<&mut Connection>,
) {
    for event in events.read() {
        let mut connection = match connections.get_mut(event.peer) {
            Ok(connection) => connection,
            Err(_) => { continue; },
        };

        connection.close_reason = event.reason.clone();
        connection.local_closed = true;
        connection.state = match event.force {
            true => ConnectionState::Closed,
            false => ConnectionState::Closing,
        };
    }
}