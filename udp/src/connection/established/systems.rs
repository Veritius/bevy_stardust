use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::*;

pub(in crate::connection) fn established_events_system(
    mut events: EventReader<DisconnectPeerEvent>,
    mut connections: Query<(&mut Established, Option<&mut NetworkPeerLifestage>), With<Connection>>,
) {
    for event in events.read() {
        // The error case means that the entity is a network peer we don't control
        if let Ok((mut established, lifestage)) = connections.get_mut(event.peer) {

        }
    }
}