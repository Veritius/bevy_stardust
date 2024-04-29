use bevy::prelude::*;
use bevy_stardust::prelude::*;
use super::{Connection, ConnectionState};

pub(super) fn close_events_system(
    mut events: EventReader<DisconnectPeerEvent>,
    mut connections: Query<(&mut Connection, Option<&mut NetworkPeerLifestage>)>,
) {
    for event in events.read() {
        let (mut connection, mut lifestage) = match connections.get_mut(event.peer) {
            Ok(connection) => connection,
            Err(_) => { continue; },
        };

        connection.close_reason = event.reason.clone();
        connection.local_closed = true;
        connection.state = match event.force {
            true => ConnectionState::Closed,
            false => ConnectionState::Closing,
        };

        if let Some(mut lifestage) = lifestage {
            *lifestage = match event.force {
                true => NetworkPeerLifestage::Closed,
                false => NetworkPeerLifestage::Closing,
            }
        }
    }
}