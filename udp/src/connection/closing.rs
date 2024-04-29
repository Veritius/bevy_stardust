use bevy::prelude::*;
use bevy_stardust::prelude::*;
use super::{Connection, ConnectionState};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(super) struct Closing {
    pub reason: Option<Bytes>,
    this_side_closed: bool,
    other_side_closed: bool,
}

pub(super) fn close_events_system(
    mut events: EventReader<DisconnectPeerEvent>,
    mut connections: Query<(&mut Connection, Option<&mut NetworkPeerLifestage>)>,
) {
    for event in events.read() {
        let (mut connection, lifestage) = match connections.get_mut(event.peer) {
            Ok(connection) => connection,
            Err(_) => { continue; },
        };

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